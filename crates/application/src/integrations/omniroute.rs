//! OmniRoute gateway adapter (local AI gateway).
//! Docs: https://github.com/diegosouzapw/OmniRoute
//! Default: http://127.0.0.1:20128/v1 — OpenAI-compatible.
//! Connect = run OmniRoute + set base_url / model / optional key in Settings.

use base64::Engine;
use serde::Deserialize;
use serde_json::json;

use crate::error::AppError;
use crate::integrations::config::IntegrationConfig;
use crate::integrations::image_gen::GeneratedImage;

const DEFAULT_BASE: &str = "http://127.0.0.1:20128/v1";

fn normalize_base(url: &str) -> String {
    let mut u = url.trim().trim_end_matches('/').to_string();
    if u.is_empty() {
        return DEFAULT_BASE.into();
    }
    // Accept host without /v1
    if !u.ends_with("/v1") && !u.contains("/v1/") {
        u = format!("{u}/v1");
    }
    u
}

/// POST OpenAI-compatible image generation through OmniRoute.
pub fn generate_image_via_omniroute(
    prompt: &str,
    cfg: &IntegrationConfig,
) -> Result<GeneratedImage, AppError> {
    let base = normalize_base(&cfg.omniroute_base_url);
    let model = if cfg.omniroute_image_model.trim().is_empty() {
        // Prefer free auto routing when user left model blank
        if cfg.omniroute_prefer_free {
            "auto".to_string()
        } else {
            "auto".to_string()
        }
    } else {
        cfg.omniroute_image_model.trim().to_string()
    };

    let url = format!("{base}/images/generations");
    let body = json!({
        "model": model,
        "prompt": prompt,
        "n": 1,
        "size": "1024x1024",
        "response_format": "b64_json",
    });

    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(120))
        .build()
        .map_err(|e| AppError::Internal(format!("http client: {e}")))?;

    let mut req = client
        .post(&url)
        .header("Content-Type", "application/json");
    let key = cfg.omniroute_api_key.trim();
    if !key.is_empty() {
        req = req.bearer_auth(key);
    } else {
        // Some gateways accept a dummy key
        req = req.bearer_auth("omniroute");
    }

    let resp = req
        .json(&body)
        .send()
        .map_err(|e| {
            AppError::Validation(format!(
                "OmniRoute no responde en {url}: {e}. \
                 Arranca OmniRoute (npm i -g omniroute / docker) y revisa base_url en Settings."
            ))
        })?;

    let status = resp.status();
    let text = resp
        .text()
        .map_err(|e| AppError::Storage(format!("leer body OmniRoute: {e}")))?;

    if !status.is_success() {
        // Fallback: some builds only expose chat; surface clear message
        return Err(AppError::Validation(format!(
            "OmniRoute images HTTP {status}: {}. \
             Comprueba que el gateway expone /v1/images/generations y el model `{}`. \
             Si solo tienes chat free, cambia model o habilita un image backend en OmniRoute.",
            text.chars().take(400).collect::<String>(),
            model
        )));
    }

    parse_openai_image_response(&text)
}

#[derive(Debug, Deserialize)]
struct OpenAiImagesResponse {
    data: Option<Vec<OpenAiImageItem>>,
}

#[derive(Debug, Deserialize)]
struct OpenAiImageItem {
    b64_json: Option<String>,
    url: Option<String>,
}

fn parse_openai_image_response(text: &str) -> Result<GeneratedImage, AppError> {
    let parsed: OpenAiImagesResponse = serde_json::from_str(text).map_err(|e| {
        AppError::Validation(format!(
            "respuesta OmniRoute no es JSON de images: {e} | {}",
            text.chars().take(200).collect::<String>()
        ))
    })?;
    let item = parsed
        .data
        .and_then(|d| d.into_iter().next())
        .ok_or_else(|| AppError::Validation("OmniRoute: data[] vacío".into()))?;

    if let Some(b64) = item.b64_json {
        let bytes = base64::engine::general_purpose::STANDARD
            .decode(b64.trim())
            .map_err(|e| AppError::Validation(format!("b64_json inválido: {e}")))?;
        let (mime, format, w, h) = sniff_image(&bytes);
        return Ok(GeneratedImage {
            bytes,
            mime,
            format,
            width: w,
            height: h,
            provider_id: "omniroute".into(),
        });
    }

    if let Some(url) = item.url {
        return download_image_url(&url);
    }

    Err(AppError::Validation(
        "OmniRoute: ni b64_json ni url en data[0]".into(),
    ))
}

fn download_image_url(url: &str) -> Result<GeneratedImage, AppError> {
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(60))
        .build()
        .map_err(|e| AppError::Internal(format!("http client: {e}")))?;
    let resp = client
        .get(url)
        .send()
        .map_err(|e| AppError::Storage(format!("descargar imagen OmniRoute: {e}")))?;
    if !resp.status().is_success() {
        return Err(AppError::Storage(format!(
            "descarga imagen HTTP {}",
            resp.status()
        )));
    }
    let bytes = resp
        .bytes()
        .map_err(|e| AppError::Storage(format!("bytes imagen: {e}")))?
        .to_vec();
    let (mime, format, w, h) = sniff_image(&bytes);
    Ok(GeneratedImage {
        bytes,
        mime,
        format,
        width: w,
        height: h,
        provider_id: "omniroute".into(),
    })
}

fn sniff_image(bytes: &[u8]) -> (String, String, i64, i64) {
    if bytes.len() >= 8 && bytes[0] == 0x89 && bytes[1] == b'P' {
        return ("image/png".into(), "png".into(), 1024, 1024);
    }
    if bytes.len() >= 3 && bytes[0] == 0xff && bytes[1] == 0xd8 {
        return ("image/jpeg".into(), "jpg".into(), 1024, 1024);
    }
    if bytes.len() >= 2 && bytes[0] == b'B' && bytes[1] == b'M' {
        return ("image/bmp".into(), "bmp".into(), 1024, 1024);
    }
    if bytes.len() >= 12 && &bytes[0..4] == b"RIFF" {
        return ("image/webp".into(), "webp".into(), 1024, 1024);
    }
    ("application/octet-stream".into(), "bin".into(), 0, 0)
}

/// Chat completion via OmniRoute (optional for script→needs later).
pub fn chat_via_omniroute(
    system: &str,
    user: &str,
    cfg: &IntegrationConfig,
) -> Result<String, AppError> {
    let base = normalize_base(&cfg.omniroute_base_url);
    let model = if cfg.omniroute_chat_model.trim().is_empty() {
        "auto".to_string()
    } else {
        cfg.omniroute_chat_model.trim().to_string()
    };
    let url = format!("{base}/chat/completions");
    let body = json!({
        "model": model,
        "messages": [
            {"role": "system", "content": system},
            {"role": "user", "content": user}
        ],
        "temperature": 0.3,
    });

    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(90))
        .build()
        .map_err(|e| AppError::Internal(format!("http client: {e}")))?;

    let mut req = client.post(&url).header("Content-Type", "application/json");
    let key = cfg.omniroute_api_key.trim();
    req = req.bearer_auth(if key.is_empty() { "omniroute" } else { key });

    let resp = req.json(&body).send().map_err(|e| {
        AppError::Validation(format!(
            "OmniRoute chat no responde ({url}): {e}. ¿Está corriendo el gateway?"
        ))
    })?;
    let status = resp.status();
    let text = resp.text().unwrap_or_default();
    if !status.is_success() {
        return Err(AppError::Validation(format!(
            "OmniRoute chat HTTP {status}: {}",
            text.chars().take(300).collect::<String>()
        )));
    }
    let v: serde_json::Value = serde_json::from_str(&text)
        .map_err(|e| AppError::Validation(format!("chat JSON: {e}")))?;
    v["choices"][0]["message"]["content"]
        .as_str()
        .map(|s| s.to_string())
        .ok_or_else(|| AppError::Validation("OmniRoute chat: sin content".into()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_adds_v1() {
        assert!(normalize_base("http://127.0.0.1:20128").ends_with("/v1"));
        assert_eq!(
            normalize_base("http://127.0.0.1:20128/v1"),
            "http://127.0.0.1:20128/v1"
        );
    }
}
