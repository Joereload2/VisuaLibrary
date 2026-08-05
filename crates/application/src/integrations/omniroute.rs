//! OmniRoute gateway adapter (local AI gateway).
//! Docs: https://github.com/diegosouzapw/OmniRoute
//! Product checklist: docs/providers/CONNECT-OMNIROUTE.md
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

fn image_model(cfg: &IntegrationConfig) -> String {
    let m = cfg.omniroute_image_model.trim();
    // OmniRoute rejects bare "auto" for images — needs provider/model.
    if m.is_empty() || m.eq_ignore_ascii_case("auto") {
        "pollinations/flux".into()
    } else {
        m.to_string()
    }
}

fn chat_model(cfg: &IntegrationConfig) -> String {
    let m = cfg.omniroute_chat_model.trim();
    // Bare "auto" is invalid; combos look like auto/best-free, auto/chat, …
    if m.is_empty() || m.eq_ignore_ascii_case("auto") {
        "auto/best-free".into()
    } else {
        m.to_string()
    }
}

fn auth_req(
    req: reqwest::blocking::RequestBuilder,
    cfg: &IntegrationConfig,
) -> reqwest::blocking::RequestBuilder {
    let key = cfg.omniroute_api_key.trim();
    req.bearer_auth(if key.is_empty() { "omniroute" } else { key })
}

/// Result of a lightweight connectivity probe (no generation billed if images skipped).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct OmniRouteProbeResult {
    pub base_url: String,
    pub models_ok: bool,
    pub models_detail: String,
    pub images_ok: bool,
    pub images_detail: String,
    pub chat_ok: bool,
    pub chat_detail: String,
    /// True if gateway is reachable enough to try Manual generate / needs.
    pub overall_ok: bool,
    pub summary: String,
}

/// Catalog for Settings dropdowns (fetched from live OmniRoute + curated image ids).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct OmniRouteModelCatalog {
    pub base_url: String,
    pub ok: bool,
    pub detail: String,
    /// Chat / combo model ids (`auto/best-free`, `oc/…`, …).
    pub chat_models: Vec<String>,
    /// Image model ids (`provider/model`).
    pub image_models: Vec<String>,
}

/// Built-in image ids OmniRoute accepts (format provider/model). Live list may be empty until
/// providers are connected; still show these so the UI is a menu, not a free-typed field.
fn curated_image_models() -> Vec<String> {
    // Match OmniRoute Pollinations image registry (as of 2026); live GET overrides these.
    [
        "pollinations/flux",
        "pollinations/klein",
        "pollinations/zimage",
        "pollinations/qwen-image",
        "pollinations/wan-image",
        "pollinations/gptimage",
        "pollinations/gpt-image-2",
        "pollinations/gptimage-large",
    ]
    .into_iter()
    .map(str::to_string)
    .collect()
}

fn curated_chat_models() -> Vec<String> {
    [
        "auto/best-free",
        "auto/chat",
        "auto/best-chat",
        "auto/fast",
        "auto/cheap",
        "auto/smart",
        "auto/claude-sonnet",
        "auto/claude-opus",
        "auto/best-coding",
        "auto/vision",
    ]
    .into_iter()
    .map(str::to_string)
    .collect()
}

fn parse_openai_model_ids(text: &str) -> Vec<String> {
    let v: serde_json::Value = match serde_json::from_str(text) {
        Ok(v) => v,
        Err(_) => return vec![],
    };
    let mut ids = Vec::new();
    if let Some(arr) = v.get("data").and_then(|d| d.as_array()) {
        for item in arr {
            if let Some(id) = item.get("id").and_then(|x| x.as_str()) {
                let id = id.trim();
                if !id.is_empty() {
                    ids.push(id.to_string());
                }
            }
        }
    }
    ids.sort();
    ids.dedup();
    ids
}

fn looks_like_image_model(id: &str) -> bool {
    let l = id.to_ascii_lowercase();
    l.contains("image")
        || l.contains("flux")
        || l.contains("dall")
        || l.contains("sdxl")
        || l.contains("stable-diffusion")
        || l.contains("pollinations/")
        || l.contains("imagen")
        || l.contains("gpt-image")
        || l.contains("picasso")
        || l.contains("draw")
}

fn merge_unique(primary: Vec<String>, extra: Vec<String>) -> Vec<String> {
    let mut out = primary;
    for e in extra {
        if !out.iter().any(|x| x == &e) {
            out.push(e);
        }
    }
    out
}

/// List models for Settings selects. Always returns curated fallbacks even if offline.
pub fn list_omniroute_model_catalog(cfg: &IntegrationConfig) -> OmniRouteModelCatalog {
    let base = normalize_base(&cfg.omniroute_base_url);
    let mut chat = curated_chat_models();
    let mut image = curated_image_models();
    let client = match reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(20))
        .build()
    {
        Ok(c) => c,
        Err(e) => {
            return OmniRouteModelCatalog {
                base_url: base,
                ok: false,
                detail: format!("http client: {e}"),
                chat_models: chat,
                image_models: image,
            };
        }
    };

    let models_url = format!("{base}/models");
    let (ok, mut detail, live) = match auth_req(client.get(&models_url), cfg).send() {
        Ok(resp) => {
            let status = resp.status();
            let text = resp.text().unwrap_or_default();
            if status.is_success() {
                let ids = parse_openai_model_ids(&text);
                (
                    true,
                    format!("HTTP {status} · {} models from /v1/models", ids.len()),
                    ids,
                )
            } else {
                (
                    false,
                    format!(
                        "HTTP {status}: {}",
                        text.chars().take(120).collect::<String>()
                    ),
                    vec![],
                )
            }
        }
        Err(e) => (false, format!("sin conexión: {e}"), vec![]),
    };

    if !live.is_empty() {
        let mut live_chat: Vec<String> = live
            .iter()
            .filter(|id| id.starts_with("auto/") || !looks_like_image_model(id))
            .cloned()
            .collect();
        live_chat.sort_by(|a, b| {
            let sa = a.starts_with("auto/");
            let sb = b.starts_with("auto/");
            sb.cmp(&sa).then(a.cmp(b))
        });
        chat = merge_unique(live_chat, chat);
    }

    // Prefer dedicated image catalog (type=image) — not the chat model dump.
    let img_url = format!("{base}/images/generations");
    if let Ok(resp) = auth_req(client.get(&img_url), cfg).send() {
        if resp.status().is_success() {
            if let Ok(text) = resp.text() {
                let extra = parse_openai_model_ids(&text);
                if !extra.is_empty() {
                    image = extra; // live image list wins
                    detail = format!("{detail} · {} image models", image.len());
                }
            }
        }
    }

    OmniRouteModelCatalog {
        base_url: base,
        ok,
        detail,
        chat_models: chat,
        image_models: image,
    }
}

/// Probe models + optional tiny image/chat. Safe to call from Settings UI.
pub fn probe_omniroute(
    cfg: &IntegrationConfig,
    try_image: bool,
    try_chat: bool,
) -> OmniRouteProbeResult {
    let base = normalize_base(&cfg.omniroute_base_url);
    let client = match reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(25))
        .build()
    {
        Ok(c) => c,
        Err(e) => {
            return OmniRouteProbeResult {
                base_url: base,
                models_ok: false,
                models_detail: format!("http client: {e}"),
                images_ok: false,
                images_detail: "skipped".into(),
                chat_ok: false,
                chat_detail: "skipped".into(),
                overall_ok: false,
                summary: "No se pudo crear cliente HTTP".into(),
            };
        }
    };

    // --- models ---
    let models_url = format!("{base}/models");
    let (models_ok, models_detail) = match auth_req(client.get(&models_url), cfg).send() {
        Ok(resp) => {
            let status = resp.status();
            let text = resp.text().unwrap_or_default();
            if status.is_success() {
                let n = serde_json::from_str::<serde_json::Value>(&text)
                    .ok()
                    .and_then(|v| v.get("data").and_then(|d| d.as_array()).map(|a| a.len()))
                    .unwrap_or(0);
                (
                    true,
                    format!("HTTP {status} · ~{n} model(s) en data[] (si aplica)"),
                )
            } else {
                (
                    false,
                    format!(
                        "HTTP {status}: {}",
                        text.chars().take(160).collect::<String>()
                    ),
                )
            }
        }
        Err(e) => (
            false,
            format!("sin conexión a {models_url}: {e}. ¿OmniRoute arrancado?"),
        ),
    };

    // --- images (tiny prompt; may consume free quota) ---
    let (images_ok, images_detail) = if !try_image {
        (false, "omitido".into())
    } else if !models_ok {
        (false, "omitido (models falló)".into())
    } else {
        match generate_image_via_omniroute("tiny solid blue square educational icon, no text", cfg)
        {
            Ok(img) => (
                true,
                format!(
                    "ok · {} bytes · {}×{} · {}",
                    img.bytes.len(),
                    img.width,
                    img.height,
                    img.format
                ),
            ),
            Err(e) => (false, e.to_string()),
        }
    };

    // --- chat ---
    let (chat_ok, chat_detail) = if !try_chat {
        (false, "omitido".into())
    } else if !models_ok {
        (false, "omitido (models falló)".into())
    } else {
        match chat_via_omniroute("Reply with exactly: pong", "ping", cfg) {
            Ok(s) => (
                true,
                format!("ok · preview: {}", s.chars().take(80).collect::<String>()),
            ),
            Err(e) => (false, e.to_string()),
        }
    };

    let overall_ok = models_ok;
    let summary = if overall_ok && images_ok {
        "OmniRoute listo para generar imágenes (y models OK).".into()
    } else if overall_ok && !try_image {
        "OmniRoute alcanzable (models OK). Prueba imagen cuando quieras.".into()
    } else if overall_ok && !images_ok {
        "Gateway up, pero /images falló — revisa image model / backend de imagen.".into()
    } else {
        "OmniRoute no alcanzable. Arranca el gateway y revisa base URL.".into()
    };

    OmniRouteProbeResult {
        base_url: base,
        models_ok,
        models_detail,
        images_ok,
        images_detail,
        chat_ok,
        chat_detail,
        overall_ok,
        summary,
    }
}

/// POST OpenAI-compatible image generation through OmniRoute.
pub fn generate_image_via_omniroute(
    prompt: &str,
    cfg: &IntegrationConfig,
) -> Result<GeneratedImage, AppError> {
    let base = normalize_base(&cfg.omniroute_base_url);
    let model = image_model(cfg);
    let url = format!("{base}/images/generations");

    // Prefer b64; on parse failure some gateways only return url — second attempt.
    match generate_images_request(&url, &model, prompt, "b64_json", cfg) {
        Ok(img) => Ok(img),
        Err(e_b64) => match generate_images_request(&url, &model, prompt, "url", cfg) {
            Ok(img) => Ok(img),
            Err(e_url) => Err(AppError::Validation(format!(
                "OmniRoute images falló (b64: {e_b64}; url: {e_url}). \
                 Model=`{model}`. Ver docs/providers/CONNECT-OMNIROUTE.md"
            ))),
        },
    }
}

fn generate_images_request(
    url: &str,
    model: &str,
    prompt: &str,
    response_format: &str,
    cfg: &IntegrationConfig,
) -> Result<GeneratedImage, AppError> {
    let body = json!({
        "model": model,
        "prompt": prompt,
        "n": 1,
        "size": "1024x1024",
        "response_format": response_format,
    });

    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(120))
        .build()
        .map_err(|e| AppError::Internal(format!("http client: {e}")))?;

    let req = auth_req(
        client.post(url).header("Content-Type", "application/json"),
        cfg,
    );

    let resp = req.json(&body).send().map_err(|e| {
        AppError::Validation(format!(
            "OmniRoute no responde en {url}: {e}. \
             Arranca OmniRoute y revisa base_url (docs/providers/CONNECT-OMNIROUTE.md)."
        ))
    })?;

    let status = resp.status();
    let text = resp
        .text()
        .map_err(|e| AppError::Storage(format!("leer body OmniRoute: {e}")))?;

    if !status.is_success() {
        let snippet = text.chars().take(280).collect::<String>();
        let low = snippet.to_ascii_lowercase();
        let hint = if status.as_u16() == 401
            || low.contains("unauthorized")
            || low.contains("authentication required")
        {
            " Pista: en OmniRoute → Proveedores → Pollinations pega la API key de enter.pollinations.ai."
        } else if status.as_u16() == 402
            || low.contains("insufficient balance")
            || low.contains("payment_required")
            || low.contains("pollen")
        {
            " Pista: la key de Pollinations es válida pero la cuenta no tiene saldo (pollen=0). En enter.pollinations.ai revisa créditos free / recarga, o usa otro provider de imagen con free tier."
        } else {
            ""
        };
        return Err(AppError::Validation(format!(
            "HTTP {status}: {snippet} | model=`{model}` format={response_format}.{hint}"
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
    // Some gateways wrap or add markdown; extract first JSON object.
    let json_str = extract_json_object(text).unwrap_or(text.trim());
    let parsed: OpenAiImagesResponse = serde_json::from_str(json_str).map_err(|e| {
        AppError::Validation(format!(
            "respuesta no es JSON images: {e} | {}",
            text.chars().take(200).collect::<String>()
        ))
    })?;
    let item = parsed
        .data
        .and_then(|d| d.into_iter().next())
        .ok_or_else(|| AppError::Validation("OmniRoute: data[] vacío".into()))?;

    if let Some(b64) = item.b64_json.filter(|s| !s.trim().is_empty()) {
        let raw = b64.trim();
        // data:image/png;base64,....
        let raw = raw.split(',').next_back().unwrap_or(raw).trim();
        let bytes = base64::engine::general_purpose::STANDARD
            .decode(raw)
            .or_else(|_| base64::engine::general_purpose::STANDARD_NO_PAD.decode(raw))
            .map_err(|e| AppError::Validation(format!("b64_json inválido: {e}")))?;
        if bytes.len() < 32 {
            return Err(AppError::Validation("b64_json demasiado corto".into()));
        }
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

    if let Some(url) = item.url.filter(|s| !s.trim().is_empty()) {
        return download_image_url(url.trim());
    }

    Err(AppError::Validation(
        "OmniRoute: ni b64_json ni url en data[0]".into(),
    ))
}

fn extract_json_object(content: &str) -> Option<&str> {
    let trimmed = content.trim();
    let start = trimmed.find('{')?;
    let end = trimmed.rfind('}')?;
    if end >= start {
        Some(&trimmed[start..=end])
    } else {
        None
    }
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
    if bytes.len() < 32 {
        return Err(AppError::Validation("imagen descargada vacía".into()));
    }
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
        let (w, h) = png_size(bytes).unwrap_or((1024, 1024));
        return ("image/png".into(), "png".into(), w, h);
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

fn png_size(bytes: &[u8]) -> Option<(i64, i64)> {
    // IHDR at offset 16 after 8-byte sig + 8-byte chunk header
    if bytes.len() < 24 {
        return None;
    }
    let w = u32::from_be_bytes([bytes[16], bytes[17], bytes[18], bytes[19]]);
    let h = u32::from_be_bytes([bytes[20], bytes[21], bytes[22], bytes[23]]);
    if w == 0 || h == 0 {
        return None;
    }
    Some((i64::from(w), i64::from(h)))
}

/// Chat completion via OmniRoute (script→needs, probes).
pub fn chat_via_omniroute(
    system: &str,
    user: &str,
    cfg: &IntegrationConfig,
) -> Result<String, AppError> {
    let base = normalize_base(&cfg.omniroute_base_url);
    let model = chat_model(cfg);
    let url = format!("{base}/chat/completions");
    // OmniRoute often defaults to SSE streaming; VL needs a single JSON object.
    let body = json!({
        "model": model,
        "messages": [
            {"role": "system", "content": system},
            {"role": "user", "content": user}
        ],
        "temperature": 0.3,
        "stream": false,
    });

    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(90))
        .build()
        .map_err(|e| AppError::Internal(format!("http client: {e}")))?;

    let req = auth_req(
        client.post(&url).header("Content-Type", "application/json"),
        cfg,
    );

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
    let v: serde_json::Value =
        serde_json::from_str(&text).map_err(|e| AppError::Validation(format!("chat JSON: {e}")))?;
    // content as string or array of parts (OpenAI multi-part)
    if let Some(s) = v["choices"][0]["message"]["content"].as_str() {
        return Ok(s.to_string());
    }
    if let Some(arr) = v["choices"][0]["message"]["content"].as_array() {
        let mut out = String::new();
        for part in arr {
            if let Some(t) = part.get("text").and_then(|x| x.as_str()) {
                out.push_str(t);
            }
        }
        if !out.is_empty() {
            return Ok(out);
        }
    }
    Err(AppError::Validation("OmniRoute chat: sin content".into()))
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

    #[test]
    fn parse_b64_png_fixture() {
        // 1x1 PNG
        let png_b64 = "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mP8z8BQDwAEhQGAhKmMIQAAAABJRU5ErkJggg==";
        let json = format!(r#"{{"data":[{{"b64_json":"{png_b64}"}}]}}"#);
        let img = parse_openai_image_response(&json).unwrap();
        assert_eq!(img.provider_id, "omniroute");
        assert_eq!(img.format, "png");
        assert!(img.bytes.len() > 20);
        assert_eq!(img.width, 1);
        assert_eq!(img.height, 1);
    }

    #[test]
    fn parse_rejects_empty_data() {
        let err = parse_openai_image_response(r#"{"data":[]}"#).unwrap_err();
        assert!(err.to_string().contains("vacío") || err.to_string().contains("data"));
    }

    #[test]
    fn probe_offline_is_not_ok() {
        let mut cfg = IntegrationConfig::default();
        cfg.omniroute_base_url = "http://127.0.0.1:1/v1".into();
        let r = probe_omniroute(&cfg, false, false);
        assert!(!r.overall_ok);
        assert!(!r.models_ok);
    }
}
