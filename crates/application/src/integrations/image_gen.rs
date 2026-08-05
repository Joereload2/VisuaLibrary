use serde::{Deserialize, Serialize};

use crate::error::AppError;
use crate::integrations::config::IntegrationConfig;
use crate::jobs::colored_stub_bmp;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageProviderInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub kind: String,
    /// ready | missing_key | not_connected | disabled | always | budget_exhausted
    pub status: String,
    pub status_detail: String,
    pub cost_score: u8,
    pub quality_score: u8,
    pub availability_score: u8,
    pub enabled: bool,
    pub unit_cost_cents: u64,
    pub spent_cents: u64,
    pub budget_limit_cents: u64,
    pub available_budget_cents: Option<u64>,
    pub free_remaining: Option<u64>,
    pub is_free: bool,
    pub can_afford_one: bool,
}

#[derive(Debug, Clone)]
pub struct GeneratedImage {
    pub bytes: Vec<u8>,
    pub mime: String,
    pub format: String,
    pub width: i64,
    pub height: i64,
    pub provider_id: String,
}

/// Full **runtime** catalog (Tier 0). Research Tier-1: `docs/providers/CATALOG.md`.
/// kind: local_stub | gateway | remote_api | local_runtime (D-039).
/// cost_score: lower = cheaper preference · quality/availability: higher = better.
pub fn catalog_image_providers() -> Vec<(&'static str, &'static str, &'static str, u8, u8, u8)> {
    // id, name, kind, cost, quality, availability
    vec![
        ("stub", "Stub local (tile color)", "local_stub", 0, 20, 100),
        (
            "omniroute",
            "OmniRoute gateway (free stack)",
            "gateway",
            5,
            70,
            90,
        ),
        (
            "spacexai-image",
            "SpaceXAI / xAI image",
            "remote_api",
            40,
            85,
            70,
        ),
        ("openai-image", "OpenAI Images", "remote_api", 50, 80, 70),
        ("stability", "Stability", "remote_api", 35, 75, 65),
    ]
}

/// Weights for ranking runnable providers (see `docs/providers/SCORING.md`).
#[derive(Debug, Clone, Copy)]
pub struct ProviderScoreWeights {
    pub free: u32,
    pub quality: u32,
    pub cost: u32,
    pub availability: u32,
}

impl ProviderScoreWeights {
    /// Automatic Factory / free-first product default.
    pub const AUTOMATIC: Self = Self {
        free: 40,
        quality: 15,
        cost: 25,
        availability: 20,
    };
}

/// Higher is better. Non-runnable providers get a very low score.
pub fn score_image_provider(p: &ImageProviderInfo, w: &ProviderScoreWeights) -> i64 {
    let runnable = p.enabled
        && p.can_afford_one
        && (p.id == "stub" || p.status == "always" || p.status == "ready");
    if !runnable {
        return i64::MIN / 4;
    }
    let free_c: u32 = if p.is_free || p.unit_cost_cents == 0 {
        100
    } else {
        0
    };
    // cost_score is "expense preference" (0 = cheapest).
    let cost_c = 100u32.saturating_sub(u32::from(p.cost_score));
    let q = u32::from(p.quality_score);
    let a = u32::from(p.availability_score);
    let num = w.free * free_c + w.quality * q + w.cost * cost_c + w.availability * a;
    let den = (w.free + w.quality + w.cost + w.availability).max(1);
    i64::from(num / den)
}

pub fn list_image_providers_with_config(cfg: &IntegrationConfig) -> Vec<ImageProviderInfo> {
    use crate::integrations::budget::{ensure_default_ledgers, ConnectorLedger};

    let mut cfg_clone = cfg.clone();
    ensure_default_ledgers(&mut cfg_clone);

    catalog_image_providers()
        .into_iter()
        .map(|(id, name, kind, cost, quality, avail)| {
            let enabled = cfg.enabled_image_providers.iter().any(|e| e == id) || id == "stub";
            let led = cfg_clone
                .connector_ledgers
                .iter()
                .find(|l| l.provider_id == id)
                .cloned()
                .unwrap_or_else(|| ConnectorLedger::default_for(id));
            let (mut status, mut detail) = provider_status(id, enabled, cfg);
            if enabled && status != "disabled" && !led.can_afford(1) {
                status = "budget_exhausted".into();
                detail = format!(
                    "Sin presupuesto/cuota (spent {}¢ / limit {}¢, free {}/{})",
                    led.spent_cents, led.budget_limit_cents, led.free_used, led.free_quota
                );
            }
            ImageProviderInfo {
                id: id.into(),
                name: name.into(),
                description: match id {
                    "stub" => {
                        "Tier 0 local_stub. Tile de color; gratis; dev/offline. Ver docs/providers."
                            .into()
                    }
                    "spacexai-image" => {
                        "Tier 0 remote_api. xAI image: key + HTTP pendiente. Presupuesto por conector."
                            .into()
                    }
                    "openai-image" => {
                        "Tier 0 remote_api. OpenAI Images: key + HTTP pendiente. Manual premium."
                            .into()
                    }
                    "stability" => {
                        "Tier 0 remote_api. Stability: key + HTTP pendiente. Presupuesto/cuota."
                            .into()
                    }
                    "omniroute" => {
                        "Tier 0 gateway (no es el modelo). /v1/images + chat needs. Free stack. docs/providers."
                            .into()
                    }
                    _ => "".into(),
                },
                kind: kind.into(),
                status,
                status_detail: detail,
                cost_score: cost,
                quality_score: quality,
                availability_score: avail,
                enabled,
                unit_cost_cents: led.unit_cost_cents,
                spent_cents: led.spent_cents,
                budget_limit_cents: led.budget_limit_cents,
                available_budget_cents: led.available_budget_cents(),
                free_remaining: led.free_remaining(),
                is_free: led.is_free || led.unit_cost_cents == 0,
                can_afford_one: led.can_afford(1),
            }
        })
        .collect()
}

fn provider_status(id: &str, enabled: bool, cfg: &IntegrationConfig) -> (String, String) {
    if id == "stub" {
        return ("always".into(), "Listo · free".into());
    }
    if !enabled {
        return ("disabled".into(), "Deshabilitado en Settings".into());
    }
    if id == "omniroute" {
        let base = cfg.omniroute_base_url.trim();
        if base.is_empty() {
            return (
                "missing_key".into(),
                "Falta omniroute_base_url (ej. http://127.0.0.1:20128/v1)".into(),
            );
        }
        // HTTP wired; "connect" = arrancar OmniRoute + modelos free
        return (
            "ready".into(),
            format!(
                "HTTP listo → {base} model={} (prefer_free={}). Arranca OmniRoute para generar.",
                cfg.omniroute_image_model, cfg.omniroute_prefer_free
            ),
        );
    }
    let key_ok = match id {
        "spacexai-image" => !cfg.xai_api_key.trim().is_empty(),
        "openai-image" => !cfg.openai_api_key.trim().is_empty(),
        "stability" => !cfg.stability_api_key.trim().is_empty(),
        _ => false,
    };
    if !key_ok {
        return ("missing_key".into(), "Falta API key en Settings".into());
    }
    (
        "not_connected".into(),
        "Key presente — implementar HTTP en integrations/image_gen.rs".into(),
    )
}

/// Select one provider: preferred if enabled+usable, else best available (stub always works).
pub fn select_image_provider_with_config(
    preferred: Option<&str>,
    cfg: &IntegrationConfig,
) -> Result<ImageProviderInfo, AppError> {
    let all = list_image_providers_with_config(cfg);
    // Runnable: enabled + budget OK; stub always; remotes/gateway when status ready.
    let is_runnable = |p: &ImageProviderInfo| {
        if !p.enabled || !p.can_afford_one {
            return false;
        }
        p.id == "stub" || p.status == "always" || p.status == "ready"
    };

    if let Some(id) = preferred.map(str::trim).filter(|s| !s.is_empty()) {
        if let Some(p) = all.iter().find(|p| p.id == id) {
            if is_runnable(p) {
                return Ok(p.clone());
            }
            // preferred not runnable → fall through (re-pick free stack)
        } else {
            return Err(AppError::Validation(format!("provider desconocido: {id}")));
        }
    }

    // Prefer free first when omniroute_prefer_free (Automatic-friendly).
    let prefer_free = cfg.omniroute_prefer_free;

    if prefer_free {
        if let Some(p) = all
            .iter()
            .find(|p| is_runnable(p) && p.is_free && p.id == "omniroute")
        {
            return Ok(p.clone());
        }
        if let Some(p) = all
            .iter()
            .find(|p| is_runnable(p) && p.is_free && p.id != "stub")
        {
            return Ok(p.clone());
        }
    }

    // Prefer default from config if runnable
    if let Some(p) = all
        .iter()
        .find(|p| p.id == cfg.default_image_provider && is_runnable(p))
    {
        return Ok(p.clone());
    }

    let weights = ProviderScoreWeights::AUTOMATIC;
    let mut candidates: Vec<_> = all.into_iter().filter(|p| is_runnable(p)).collect();
    // D-039 / docs/providers/SCORING.md — free-first Automatic ranking.
    candidates.sort_by(|a, b| {
        score_image_provider(b, &weights)
            .cmp(&score_image_provider(a, &weights))
            .then(a.id.cmp(&b.id))
    });
    candidates
        .into_iter()
        .next()
        .ok_or_else(|| AppError::Validation("no hay providers de imagen utilizables".into()))
}

/// Generate image bytes for a provider. Stub works; remotes: connect HTTP here.
pub fn generate_image_bytes(
    provider_id: &str,
    prompt: &str,
    seed: &str,
    cfg: &IntegrationConfig,
) -> Result<GeneratedImage, AppError> {
    let info = list_image_providers_with_config(cfg)
        .into_iter()
        .find(|p| p.id == provider_id)
        .ok_or_else(|| AppError::Validation(format!("provider desconocido: {provider_id}")))?;

    if !info.enabled && provider_id != "stub" {
        return Err(AppError::Validation(format!(
            "provider `{provider_id}` deshabilitado en Settings"
        )));
    }

    match provider_id {
        "stub" => {
            let bytes = colored_stub_bmp(seed);
            Ok(GeneratedImage {
                bytes,
                mime: "image/bmp".into(),
                format: "bmp".into(),
                width: 128,
                height: 128,
                provider_id: "stub".into(),
            })
        }
        "spacexai-image" => generate_remote_placeholder(
            provider_id,
            prompt,
            &cfg.xai_api_key,
            // CONNECT: POST api.x.ai image endpoint with Bearer key
        ),
        "openai-image" => generate_remote_placeholder(
            provider_id,
            prompt,
            &cfg.openai_api_key,
            // CONNECT: POST api.openai.com/v1/images/generations
        ),
        "stability" => generate_remote_placeholder(
            provider_id,
            prompt,
            &cfg.stability_api_key,
            // CONNECT: Stability REST generate
        ),
        "omniroute" => crate::integrations::omniroute::generate_image_via_omniroute(prompt, cfg),
        other => Err(AppError::Validation(format!(
            "provider sin adapter: {other}"
        ))),
    }
}

fn generate_remote_placeholder(
    provider_id: &str,
    _prompt: &str,
    api_key: &str,
) -> Result<GeneratedImage, AppError> {
    if api_key.trim().is_empty() {
        return Err(AppError::Validation(format!(
            "provider `{provider_id}`: falta API key (Settings). O usa stub."
        )));
    }
    // Key is present — HTTP not implemented yet. Clear error so product can choose stub.
    Err(AppError::Validation(format!(
        "provider `{provider_id}`: API key configurada, pero la llamada HTTP aún no está conectada. \
         Implementa generate_image_bytes para `{provider_id}` en crates/application/src/integrations/image_gen.rs \
         (o usa provider stub mientras tanto)."
    )))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stub_generates_bytes() {
        let cfg = IntegrationConfig::default();
        let img = generate_image_bytes("stub", "hello", "seed-1", &cfg).unwrap();
        assert!(img.bytes.len() > 50);
        assert_eq!(img.provider_id, "stub");
    }

    #[test]
    fn remote_without_key_errors() {
        let cfg = IntegrationConfig::default();
        assert!(generate_image_bytes("openai-image", "p", "s", &cfg).is_err());
    }

    #[test]
    fn select_prefers_free_omniroute_when_enabled() {
        let cfg = IntegrationConfig::default();
        // default enables omniroute + prefer_free → omniroute first
        let p = select_image_provider_with_config(None, &cfg).unwrap();
        assert_eq!(p.id, "omniroute");
    }

    #[test]
    fn select_stub_when_only_stub() {
        let mut cfg = IntegrationConfig::default();
        cfg.enabled_image_providers = vec!["stub".into()];
        cfg.default_image_provider = "stub".into();
        cfg.omniroute_prefer_free = false;
        let p = select_image_provider_with_config(None, &cfg).unwrap();
        assert_eq!(p.id, "stub");
    }

    #[test]
    fn score_prefers_free_over_paid_when_weights_automatic() {
        let cfg = IntegrationConfig::default();
        let all = list_image_providers_with_config(&cfg);
        let stub = all.iter().find(|p| p.id == "stub").unwrap();
        let omni = all.iter().find(|p| p.id == "omniroute").unwrap();
        // Both free-ish; omniroute has higher quality → can beat stub when both runnable.
        let w = ProviderScoreWeights::AUTOMATIC;
        let s_stub = score_image_provider(stub, &w);
        let s_omni = score_image_provider(omni, &w);
        assert!(s_omni > s_stub || s_omni == s_stub);
        assert!(s_stub > i64::MIN / 4);
    }

    #[test]
    fn catalog_kinds_match_taxonomy() {
        for (_, _, kind, _, _, _) in catalog_image_providers() {
            assert!(
                matches!(
                    kind,
                    "local_stub" | "gateway" | "remote_api" | "local_runtime"
                ),
                "unexpected kind {kind}"
            );
        }
    }
}
