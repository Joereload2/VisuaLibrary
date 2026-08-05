use serde::{Deserialize, Serialize};

use crate::error::AppError;
use crate::factory::{propose_needs_from_script, ProposeNeedsInput, ProposeNeedsResult};
use crate::integrations::config::default_needs_system_prompt;
use crate::integrations::IntegrationConfig;

/// Catalog entry for script→needs AI (choose in Settings).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptAiProviderInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    /// ready | missing_key | not_connected | always
    pub status: String,
    pub status_detail: String,
}

pub fn list_script_ai_providers(cfg: &IntegrationConfig) -> Vec<ScriptAiProviderInfo> {
    let xai_ready = !cfg.xai_api_key.trim().is_empty();
    let omni_base = cfg.omniroute_base_url.trim();
    vec![
        ScriptAiProviderInfo {
            id: "heuristic".into(),
            name: "Heurística local".into(),
            description: "Sin red. Parte el guion y arma needs BD. Siempre disponible.".into(),
            status: "always".into(),
            status_detail: "Listo".into(),
        },
        ScriptAiProviderInfo {
            id: "spacexai".into(),
            name: "SpaceXAI (xAI / Grok)".into(),
            description: "Proponer needs vía API xAI. Key en Settings; HTTP pendiente de conectar.".into(),
            status: if xai_ready {
                "not_connected".into()
            } else {
                "missing_key".into()
            },
            status_detail: if xai_ready {
                "Key presente — falta cablear HTTP. Fallback: heurística.".into()
            } else {
                "Falta API key (Settings → Keys)".into()
            },
        },
        ScriptAiProviderInfo {
            id: "omniroute".into(),
            name: "OmniRoute chat (Claude / free stack)".into(),
            description: "POST /v1/chat/completions. Pon model Claude (o auto) + arranca OmniRoute. Prompt editable en Settings.".into(),
            status: if omni_base.is_empty() {
                "missing_key".into()
            } else {
                "ready".into()
            },
            status_detail: format!(
                "Listo para conectar → {omni_base} · model=`{}`. Si el gateway no responde → heurística.",
                cfg.omniroute_chat_model
            ),
        },
    ]
}

/// Script → needs using selected provider.
/// `extra_user_instructions`: optional brief from Factory (tab Instrucciones) merged into the user message.
/// Remote failures always fall back to local heuristic so the product keeps working offline.
pub fn propose_needs_with_config(
    script: &str,
    max_needs: Option<usize>,
    cfg: &IntegrationConfig,
    extra_user_instructions: Option<&str>,
) -> Result<ProposeNeedsResult, AppError> {
    let mut result = match cfg.script_ai_provider.as_str() {
        "heuristic" | "" => propose_needs_from_script(ProposeNeedsInput {
            script: script.into(),
            max_needs,
        }),
        "spacexai" => {
            propose_via_spacexai_or_fallback(script, max_needs, cfg, extra_user_instructions)
        }
        "omniroute" => {
            propose_via_omniroute_or_fallback(script, max_needs, cfg, extra_user_instructions)
        }
        other => Err(AppError::Validation(format!(
            "script_ai_provider no soportado: {other}"
        ))),
    }?;
    // Stamp default image provider from integrations (so needs inherit omniroute when wired).
    let img = cfg.default_image_provider.trim();
    if !img.is_empty() {
        for n in result.needs.iter_mut() {
            if n.provider.as_deref().unwrap_or("stub") == "stub" && img != "stub" {
                n.provider = Some(img.to_string());
            } else if n.provider.is_none() {
                n.provider = Some(img.to_string());
            }
        }
    }
    Ok(result)
}

fn system_prompt(cfg: &IntegrationConfig) -> String {
    let p = cfg.needs_system_prompt.trim();
    if p.is_empty() {
        default_needs_system_prompt()
    } else {
        p.to_string()
    }
}

fn build_user_message(
    script: &str,
    max_needs: Option<usize>,
    extra_user_instructions: Option<&str>,
) -> String {
    let max = max_needs.unwrap_or(8);
    let mut parts = vec![
        format!("max_needs: {max}"),
        "Return ONLY the JSON object described in the system prompt.".into(),
    ];
    if let Some(extra) = extra_user_instructions
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        parts.push(format!(
            "Additional human instructions (follow carefully):\n{extra}"
        ));
    }
    parts.push(format!("SCRIPT:\n{}", script.trim()));
    parts.join("\n\n")
}

fn propose_via_omniroute_or_fallback(
    script: &str,
    max_needs: Option<usize>,
    cfg: &IntegrationConfig,
    extra_user_instructions: Option<&str>,
) -> Result<ProposeNeedsResult, AppError> {
    let system = system_prompt(cfg);
    let user = build_user_message(script, max_needs, extra_user_instructions);

    match crate::integrations::omniroute::chat_via_omniroute(&system, &user, cfg) {
        Ok(content) => match try_parse_needs_json(&content, max_needs, cfg) {
            Ok(mut parsed) => {
                // Track free/paid usage on the omniroute connector (best-effort).
                let mut cfg_bill = cfg.clone();
                let _ = crate::integrations::record_usage(&mut cfg_bill, "omniroute", 1);
                // Note: caller may not persist cfg_bill; optional side-effect for in-memory runs.
                // Persist happens when generate bills; chat usage is advisory unless we thread mut cfg.
                let _ = cfg_bill;
                parsed.method = "omniroute_chat_json_v1".into();
                parsed.notes = format!(
                    "Needs vía OmniRoute chat (model `{}`). Revisa y edita antes de generar.",
                    cfg.omniroute_chat_model
                );
                Ok(parsed)
            }
            Err(parse_err) => {
                let mut r = propose_needs_from_script(ProposeNeedsInput {
                    script: script.into(),
                    max_needs,
                })?;
                r.method = "fallback_heuristic_omniroute_bad_json".into();
                r.notes = format!(
                    "OmniRoute respondió pero el JSON no era usable ({parse_err}). Heurística. Preview: {}",
                    content.chars().take(140).collect::<String>()
                );
                Ok(r)
            }
        },
        Err(e) => {
            let mut r = propose_needs_from_script(ProposeNeedsInput {
                script: script.into(),
                max_needs,
            })?;
            r.method = "fallback_heuristic_omniroute_offline".into();
            r.notes = format!(
                "OmniRoute chat no disponible ({e}). Heurística local. \
                 Arranca el gateway, revisa base URL / model (Claude), y reintenta. {}",
                r.notes
            );
            Ok(r)
        }
    }
}

fn try_parse_needs_json(
    content: &str,
    max_needs: Option<usize>,
    cfg: &IntegrationConfig,
) -> Result<ProposeNeedsResult, AppError> {
    let json_str = extract_json_object(content)
        .ok_or_else(|| AppError::Validation("sin objeto JSON en la respuesta".into()))?;

    #[derive(Deserialize)]
    struct RawNeed {
        concept_key: String,
        concept_name: Option<String>,
        representation_key: Option<String>,
        representation_name: Option<String>,
        prompt: Option<String>,
        orientation: Option<String>,
        style: Option<String>,
        script_excerpt: Option<String>,
        pedagogical_intent: Option<String>,
        ai_instructions: Option<String>,
        variant_count: Option<u8>,
    }
    #[derive(Deserialize)]
    struct Raw {
        script_instructions: Option<String>,
        needs: Vec<RawNeed>,
    }

    let raw: Raw = serde_json::from_str(json_str)
        .map_err(|e| AppError::Validation(format!("json needs: {e}")))?;
    let max = max_needs.unwrap_or(8).clamp(1, 20);
    let default_provider = if cfg.default_image_provider.trim().is_empty() {
        "stub".into()
    } else {
        cfg.default_image_provider.clone()
    };

    let needs: Vec<crate::factory::ManualNeed> = raw
        .needs
        .into_iter()
        .take(max)
        .filter(|n| !n.concept_key.trim().is_empty())
        .map(|n| {
            let intent = n.pedagogical_intent.clone();
            let ai = n
                .ai_instructions
                .or_else(|| intent.clone())
                .filter(|s| !s.trim().is_empty());
            crate::factory::ManualNeed {
                concept_key: slugish(&n.concept_key),
                concept_name: n.concept_name.or_else(|| Some(n.concept_key.clone())),
                representation_key: slugish(
                    &n.representation_key.unwrap_or_else(|| "lesson".into()),
                ),
                representation_name: n
                    .representation_name
                    .or_else(|| Some("Lesson visual".into())),
                prompt: n.prompt,
                orientation: n.orientation.or_else(|| Some("landscape".into())),
                style: n.style.or_else(|| Some("didactic".into())),
                provider: Some(default_provider.clone()),
                script_excerpt: n.script_excerpt,
                ai_instructions: ai,
                pedagogical_intent: intent,
                included: Some(true),
                variant_count: Some(n.variant_count.unwrap_or(3).clamp(1, 3)),
                also_generate_if_found: Some(false),
            }
        })
        .collect();

    if needs.is_empty() {
        return Err(AppError::Validation("needs vacías tras parse".into()));
    }

    Ok(ProposeNeedsResult {
        needs,
        script_instructions: raw.script_instructions.unwrap_or_else(|| {
            "Instrucciones generadas por chat (OmniRoute). Edítalas si hace falta.".into()
        }),
        method: "omniroute_chat_json_v1".into(),
        notes: "OK".into(),
    })
}

/// Prefer fenced ```json ... ``` then first `{...}` span.
fn extract_json_object(content: &str) -> Option<&str> {
    let trimmed = content.trim();
    if let Some(rest) = trimmed.strip_prefix("```json") {
        let body = rest.strip_prefix('\n').unwrap_or(rest);
        if let Some(end) = body.find("```") {
            let inner = body[..end].trim();
            if inner.starts_with('{') {
                return Some(inner);
            }
        }
    }
    if let Some(rest) = trimmed.strip_prefix("```") {
        let body = rest.strip_prefix('\n').unwrap_or(rest);
        if let Some(end) = body.find("```") {
            let inner = body[..end].trim();
            if inner.starts_with('{') {
                return Some(inner);
            }
        }
    }
    let start = trimmed.find('{')?;
    let end = trimmed.rfind('}')?;
    if end >= start {
        Some(&trimmed[start..=end])
    } else {
        None
    }
}

fn slugish(s: &str) -> String {
    let s = s.trim().to_lowercase();
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        if c.is_ascii_alphanumeric() {
            out.push(c);
        } else if c == ' ' || c == '_' || c == '-' {
            if !out.ends_with('-') {
                out.push('-');
            }
        }
    }
    let out = out.trim_matches('-').to_string();
    if out.is_empty() {
        "concept".into()
    } else {
        out
    }
}

/// Placeholder for real SpaceXAI call. When key missing or HTTP not wired, falls back to heuristic.
fn propose_via_spacexai_or_fallback(
    script: &str,
    max_needs: Option<usize>,
    cfg: &IntegrationConfig,
    extra_user_instructions: Option<&str>,
) -> Result<ProposeNeedsResult, AppError> {
    let _ = extra_user_instructions;
    if cfg.xai_api_key.trim().is_empty() {
        let mut r = propose_needs_from_script(ProposeNeedsInput {
            script: script.into(),
            max_needs,
        })?;
        r.method = "fallback_heuristic_missing_xai_key".into();
        r.notes = format!(
            "SpaceXAI sin API key. Heurística. Conecta key o usa OmniRoute+Claude. {}",
            r.notes
        );
        return Ok(r);
    }

    // --- CONNECT xAI HTTP HERE (same JSON schema as OmniRoute) ---
    // system = system_prompt(cfg); user = build_user_message(...);
    // POST chat completions → try_parse_needs_json
    let mut r = propose_needs_from_script(ProposeNeedsInput {
        script: script.into(),
        max_needs,
    })?;
    r.method = "fallback_heuristic_spacexai_pending_http".into();
    r.notes = format!(
        "Key xAI presente; HTTP aún no conectado. Preferible: Settings → script AI = omniroute + Claude. {}",
        r.notes
    );
    Ok(r)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn heuristic_works() {
        let cfg = IntegrationConfig::default();
        let r = propose_needs_with_config(
            "Lección sobre el agua. El ciclo hidrológico es esencial para la vida en la Tierra.",
            Some(4),
            &cfg,
            None,
        )
        .unwrap();
        assert!(!r.needs.is_empty());
    }

    #[test]
    fn spacexai_without_key_falls_back() {
        let mut cfg = IntegrationConfig::default();
        cfg.script_ai_provider = "spacexai".into();
        let r = propose_needs_with_config(
            "Lección sobre el agua. El ciclo hidrológico es esencial para la vida en la Tierra.",
            Some(3),
            &cfg,
            None,
        )
        .unwrap();
        assert!(r.method.contains("fallback"));
    }

    #[test]
    fn parse_needs_json_from_model_reply() {
        let cfg = IntegrationConfig::default();
        let raw = r#"Here you go:
```json
{
  "script_instructions": "Focus on clarity",
  "needs": [
    {
      "concept_key": "Water Cycle",
      "concept_name": "Water cycle",
      "representation_key": "diagram",
      "prompt": "simple water cycle diagram",
      "orientation": "landscape",
      "style": "didactic",
      "script_excerpt": "ciclo hidrológico",
      "pedagogical_intent": "show evaporation",
      "ai_instructions": "clear arrows",
      "variant_count": 2
    }
  ]
}
```"#;
        let p = try_parse_needs_json(raw, Some(5), &cfg).unwrap();
        assert_eq!(p.needs.len(), 1);
        assert_eq!(p.needs[0].concept_key, "water-cycle");
        assert_eq!(p.needs[0].variant_count, Some(2));
        assert_eq!(p.needs[0].ai_instructions.as_deref(), Some("clear arrows"));
        assert!(p.script_instructions.contains("clarity"));
    }

    #[test]
    fn default_system_prompt_is_substantial() {
        let p = default_needs_system_prompt();
        assert!(p.contains("JSON"));
        assert!(p.len() > 200);
    }

    #[test]
    fn omniroute_offline_falls_back() {
        let mut cfg = IntegrationConfig::default();
        cfg.script_ai_provider = "omniroute".into();
        cfg.omniroute_base_url = "http://127.0.0.1:1/v1".into(); // nothing listening
        let r = propose_needs_with_config(
            "Lección sobre el agua. El ciclo hidrológico es esencial para la vida en la Tierra.",
            Some(2),
            &cfg,
            Some("Prioriza diagramas"),
        )
        .unwrap();
        assert!(r.method.contains("fallback"));
        assert!(!r.needs.is_empty());
    }
}
