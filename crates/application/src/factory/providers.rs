use serde::{Deserialize, Serialize};

use crate::error::AppError;

/// Image generation provider known to Manual Factory (multi-provider design).
/// Only **one** is selected per need/generate attempt.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ImageProvider {
    pub id: String,
    pub name: String,
    pub available: bool,
    /// 0 = cheapest preference weight (lower cost score = cheaper).
    pub cost_score: u8,
    /// 0–100 quality preference (higher is better).
    pub quality_score: u8,
    /// 0–100 availability/latency preference (higher is better).
    pub availability_score: u8,
    pub kind: String,
    pub notes: String,
}

/// MVP catalog: several providers; only `stub` executes generation today.
pub fn list_image_providers() -> Vec<ImageProvider> {
    vec![
        ImageProvider {
            id: "stub".into(),
            name: "Stub (local tile)".into(),
            available: true,
            cost_score: 0,
            quality_score: 20,
            availability_score: 100,
            kind: "local_stub".into(),
            notes: "Genera tile local de color; sin red. Default MVP.".into(),
        },
        ImageProvider {
            id: "spacexai-image".into(),
            name: "SpaceXAI / xAI image".into(),
            available: false,
            cost_score: 40,
            quality_score: 85,
            availability_score: 0,
            kind: "remote_api".into(),
            notes: "Reservado: activar con XAI_API_KEY + adapter real.".into(),
        },
        ImageProvider {
            id: "openai-image".into(),
            name: "OpenAI Images".into(),
            available: false,
            cost_score: 50,
            quality_score: 80,
            availability_score: 0,
            kind: "remote_api".into(),
            notes: "Reservado: multi-provider slot; no cableado en v1.".into(),
        },
        ImageProvider {
            id: "stability".into(),
            name: "Stability".into(),
            available: false,
            cost_score: 35,
            quality_score: 75,
            availability_score: 0,
            kind: "remote_api".into(),
            notes: "Reservado: multi-provider slot; no cableado en v1.".into(),
        },
    ]
}

/// Select exactly one provider: available first, then quality, availability, low cost.
/// If `preferred` is set and available, it wins. On catalog change, next generate re-runs this.
pub fn select_image_provider(preferred: Option<&str>) -> Result<ImageProvider, AppError> {
    let all = list_image_providers();
    if let Some(id) = preferred.map(str::trim).filter(|s| !s.is_empty()) {
        if let Some(p) = all.iter().find(|p| p.id == id) {
            if p.available {
                return Ok(p.clone());
            }
            // Preferred unavailable → fall through to auto-pick (2A: re-elegir).
        } else {
            return Err(AppError::Validation(format!(
                "provider desconocido: {id}"
            )));
        }
    }
    let mut available: Vec<_> = all.into_iter().filter(|p| p.available).collect();
    if available.is_empty() {
        return Err(AppError::Validation(
            "no hay providers de imagen disponibles".into(),
        ));
    }
    available.sort_by(|a, b| {
        b.quality_score
            .cmp(&a.quality_score)
            .then(b.availability_score.cmp(&a.availability_score))
            .then(a.cost_score.cmp(&b.cost_score))
            .then(a.id.cmp(&b.id))
    });
    Ok(available.remove(0))
}

/// Plantilla de prompt pre-diseñada + variables de need/BD (usuario puede editar después).
pub fn build_prompt_template(
    concept_name: &str,
    representation_name: &str,
    script_excerpt: &str,
    pedagogical_intent: Option<&str>,
    style: &str,
    orientation: &str,
) -> String {
    let intent = pedagogical_intent.unwrap_or("ilustrar el concepto de la lección");
    let excerpt = script_excerpt.trim();
    let excerpt = if excerpt.len() > 280 {
        format!("{}…", &excerpt[..280])
    } else {
        excerpt.to_string()
    };
    format!(
        "Educational YouTube lesson illustration.\n\
         Concept: {concept_name}\n\
         Representation: {representation_name}\n\
         Pedagogical intent: {intent}\n\
         Visual style: {style}; orientation: {orientation}\n\
         Discourse context: {excerpt}\n\
         Clear, didactic, single subject, no text overlays, high clarity."
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selects_available_stub_by_default() {
        let p = select_image_provider(None).unwrap();
        assert_eq!(p.id, "stub");
        assert!(p.available);
    }

    #[test]
    fn preferred_unavailable_falls_back() {
        let p = select_image_provider(Some("openai-image")).unwrap();
        assert_eq!(p.id, "stub");
    }

    #[test]
    fn unknown_provider_errors() {
        assert!(select_image_provider(Some("nope")).is_err());
    }
}
