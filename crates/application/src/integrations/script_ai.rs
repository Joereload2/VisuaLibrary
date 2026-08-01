use serde::{Deserialize, Serialize};

use crate::error::AppError;
use crate::factory::{propose_needs_from_script, ProposeNeedsInput, ProposeNeedsResult};
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
            description: "Proponer needs + instrucciones de guion vía API xAI. Conectar XAI_API_KEY en Settings.".into(),
            status: if xai_ready {
                "not_connected".into()
            } else {
                "missing_key".into()
            },
            status_detail: if xai_ready {
                "Key presente — falta cablear HTTP (listo para conectar). Fallback: heurística.".into()
            } else {
                "Falta API key (Settings → xAI)".into()
            },
        },
        ScriptAiProviderInfo {
            id: "omniroute".into(),
            name: "OmniRoute (gateway local)".into(),
            description: "Chat vía OmniRoute (/v1/chat/completions) para proponer needs. Free stack si el gateway lo expone.".into(),
            status: if cfg.omniroute_base_url.trim().is_empty() {
                "missing_key".into()
            } else {
                "ready".into()
            },
            status_detail: format!(
                "HTTP listo → {} model={}. Arranca OmniRoute; si falla → heurística.",
                cfg.omniroute_base_url, cfg.omniroute_chat_model
            ),
        },
    ]
}

/// Run script→needs using selected provider. Remote not fully wired → safe fallback.
pub fn propose_needs_with_config(
    script: &str,
    max_needs: Option<usize>,
    cfg: &IntegrationConfig,
) -> Result<ProposeNeedsResult, AppError> {
    match cfg.script_ai_provider.as_str() {
        "heuristic" | "" => propose_needs_from_script(ProposeNeedsInput {
            script: script.into(),
            max_needs,
        }),
        "spacexai" => propose_via_spacexai_or_fallback(script, max_needs, cfg),
        "omniroute" => propose_via_omniroute_or_fallback(script, max_needs, cfg),
        other => Err(AppError::Validation(format!(
            "script_ai_provider no soportado: {other}"
        ))),
    }
}

fn propose_via_omniroute_or_fallback(
    script: &str,
    max_needs: Option<usize>,
    cfg: &IntegrationConfig,
) -> Result<ProposeNeedsResult, AppError> {
    // Try chat; if gateway down or response unusable, heuristic.
    let system = "You extract educational visual needs for a YouTube lesson. \
        Reply ONLY with valid JSON: {\"script_instructions\": string, \"needs\": [ \
        {\"concept_key\",\"concept_name\",\"representation_key\",\"representation_name\",\
        \"prompt\",\"orientation\",\"style\",\"script_excerpt\",\"pedagogical_intent\",\
        \"variant_count\"} ] }. concept_key slug lowercase. variant_count 1-3.";
    let user = format!(
        "Script (max needs {}):\n{}",
        max_needs.unwrap_or(8),
        script
    );
    match crate::integrations::omniroute::chat_via_omniroute(system, &user, cfg) {
        Ok(content) => {
            if let Ok(parsed) = try_parse_needs_json(&content, max_needs) {
                return Ok(parsed);
            }
            let mut r = propose_needs_from_script(ProposeNeedsInput {
                script: script.into(),
                max_needs,
            })?;
            r.method = "fallback_heuristic_omniroute_bad_json".into();
            r.notes = format!(
                "OmniRoute respondió pero no era JSON usable. Heurística. Preview: {}",
                content.chars().take(120).collect::<String>()
            );
            Ok(r)
        }
        Err(e) => {
            let mut r = propose_needs_from_script(ProposeNeedsInput {
                script: script.into(),
                max_needs,
            })?;
            r.method = "fallback_heuristic_omniroute_offline".into();
            r.notes = format!("OmniRoute chat falló ({e}). Usando heurística local. {}", r.notes);
            Ok(r)
        }
    }
}

fn try_parse_needs_json(
    content: &str,
    max_needs: Option<usize>,
) -> Result<ProposeNeedsResult, AppError> {
    // Extract JSON object from possible markdown fences
    let trimmed = content.trim();
    let json_str = if let Some(start) = trimmed.find('{') {
        let end = trimmed.rfind('}').unwrap_or(trimmed.len() - 1);
        &trimmed[start..=end]
    } else {
        trimmed
    };
    #[derive(serde::Deserialize)]
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
        variant_count: Option<u8>,
    }
    #[derive(serde::Deserialize)]
    struct Raw {
        script_instructions: Option<String>,
        needs: Vec<RawNeed>,
    }
    let raw: Raw = serde_json::from_str(json_str)
        .map_err(|e| AppError::Validation(format!("json needs: {e}")))?;
    let max = max_needs.unwrap_or(8);
    let needs: Vec<crate::factory::ManualNeed> = raw
        .needs
        .into_iter()
        .take(max)
        .map(|n| crate::factory::ManualNeed {
            concept_key: n.concept_key,
            concept_name: n.concept_name,
            representation_key: n.representation_key.unwrap_or_else(|| "lesson".into()),
            representation_name: n.representation_name.or_else(|| Some("Lesson visual".into())),
            prompt: n.prompt,
            orientation: n.orientation.or_else(|| Some("landscape".into())),
            style: n.style.or_else(|| Some("didactic".into())),
            provider: Some("omniroute".into()),
            script_excerpt: n.script_excerpt,
            ai_instructions: n.pedagogical_intent.clone(),
            pedagogical_intent: n.pedagogical_intent,
            included: Some(true),
            variant_count: Some(n.variant_count.unwrap_or(3).clamp(1, 3)),
            also_generate_if_found: Some(false),
        })
        .collect();
    if needs.is_empty() {
        return Err(AppError::Validation("needs vacías".into()));
    }
    Ok(ProposeNeedsResult {
        needs,
        script_instructions: raw
            .script_instructions
            .unwrap_or_else(|| "Instrucciones generadas vía OmniRoute.".into()),
        method: "omniroute_chat_json_v1".into(),
        notes: "Needs propuestas por OmniRoute (chat). Revisa y edita antes de generar.".into(),
    })
}

/// Placeholder for real SpaceXAI call. When key missing or HTTP not wired, falls back to heuristic
/// and annotates method so the UI shows what ran.
fn propose_via_spacexai_or_fallback(
    script: &str,
    max_needs: Option<usize>,
    cfg: &IntegrationConfig,
) -> Result<ProposeNeedsResult, AppError> {
    if cfg.xai_api_key.trim().is_empty() {
        let mut r = propose_needs_from_script(ProposeNeedsInput {
            script: script.into(),
            max_needs,
        })?;
        r.method = "fallback_heuristic_missing_xai_key".into();
        r.notes = format!(
            "SpaceXAI seleccionado pero sin API key. Usando heurística. \
             Conecta key en Settings y completa el adapter HTTP en integrations/script_ai.rs. {}",
            r.notes
        );
        return Ok(r);
    }

    // --- CONNECT API HERE ---
    // When ready: POST https://api.x.ai/v1/chat/completions or /responses
    // Authorization: Bearer {cfg.xai_api_key}
    // Prompt: ask for JSON array of ManualNeed fields + script_instructions
    // Parse JSON → ProposeNeedsResult
    // On success: return Ok(result) with method = "spacexai_v1"
    //
    // Until connected, fall back so the product still works:
    let mut r = propose_needs_from_script(ProposeNeedsInput {
        script: script.into(),
        max_needs,
    })?;
    r.method = "fallback_heuristic_spacexai_pending_http".into();
    r.notes = format!(
        "Key xAI detectada, pero el adapter HTTP aún no está conectado \
         (integrations/script_ai.rs → propose_via_spacexai). Fallback heurística. {}",
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
        )
        .unwrap();
        assert!(r.method.contains("fallback"));
    }
}
