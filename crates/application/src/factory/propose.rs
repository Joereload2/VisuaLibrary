use serde::{Deserialize, Serialize};

use crate::error::AppError;
use crate::factory::manual::ManualNeed;
use crate::factory::providers::{build_prompt_template, select_image_provider};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProposeNeedsInput {
    pub script: String,
    /// Max needs to propose (default 8).
    pub max_needs: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProposeNeedsResult {
    pub needs: Vec<ManualNeed>,
    /// Instrucciones globales de la IA sobre el guion (el humano puede editarlas).
    pub script_instructions: String,
    pub method: String,
    pub notes: String,
}

/// From script text → proposed visual needs (human must edit/approve).
/// Needs = DB requirements; script_instructions = AI brief on the script.
pub fn propose_needs_from_script(input: ProposeNeedsInput) -> Result<ProposeNeedsResult, AppError> {
    let script = input.script.trim();
    if script.is_empty() {
        return Err(AppError::Validation("el guion no puede estar vacío".into()));
    }
    if script.len() < 20 {
        return Err(AppError::Validation(
            "guion demasiado corto (mín. ~20 caracteres) para proponer needs".into(),
        ));
    }

    let max = input.max_needs.unwrap_or(8).clamp(1, 20);
    let chunks = split_script_chunks(script, max);
    let provider = select_image_provider(None)?;

    let script_instructions = build_script_instructions(script, chunks.len());

    let mut needs = Vec::with_capacity(chunks.len());
    for (i, chunk) in chunks.into_iter().enumerate() {
        let (concept_key, concept_name) = concept_from_chunk(&chunk, i);
        let representation_key = "lesson".to_string();
        let representation_name = "Lesson visual".to_string();
        let orientation = "landscape".to_string();
        let style = "didactic".to_string();
        let intent = Some(format!(
            "Representar gráficamente el tramo {} del discurso de la lección",
            i + 1
        ));
        let ai_instructions = Some(format!(
            "Del guion: ilustrar este tramo de forma didáctica. \
             Prioriza claridad para YouTube. \
             Contexto del tramo: {}",
            chunk.chars().take(200).collect::<String>()
        ));
        let prompt = build_prompt_template(
            &concept_name,
            &representation_name,
            &chunk,
            intent.as_deref(),
            &style,
            &orientation,
        );
        needs.push(ManualNeed {
            concept_key,
            concept_name: Some(concept_name),
            representation_key,
            representation_name: Some(representation_name),
            prompt: Some(prompt),
            orientation: Some(orientation),
            style: Some(style),
            provider: Some(provider.id.clone()),
            script_excerpt: Some(chunk),
            ai_instructions,
            pedagogical_intent: intent,
            included: Some(true),
            variant_count: Some(3),
            also_generate_if_found: Some(false),
        });
    }

    Ok(ProposeNeedsResult {
        needs,
        script_instructions,
        method: "heuristic_script_chunks_v1_1".into(),
        notes: format!(
            "Needs = requerimientos BD. Default 3 variantes (literal/metáfora+estilo). \
             Provider: {}. Edita instrucciones del guion y cada need.",
            provider.id
        ),
    })
}

fn build_script_instructions(script: &str, need_count: usize) -> String {
    let preview: String = script.chars().take(240).collect();
    format!(
        "Instrucciones (propuesta heurística v1.1 — editable):\n\
         - Objetivo: apoyar un video de YouTube / lección con imágenes conceptuales.\n\
         - Misión de cada imagen: representar gráficamente el discurso del tramo.\n\
         - Needs detectadas (aprox.): {need_count}. Son filas de requerimiento de BD, no prompts sueltos.\n\
         - Por need se generarán 1–3 variantes (matices literal/metafórico + estilo) salvo FOUND sin enriquecer.\n\
         - Resumen del guion: {preview}…"
    )
}

fn split_script_chunks(script: &str, max: usize) -> Vec<String> {
    let mut parts: Vec<String> = script
        .split("\n\n")
        .map(str::trim)
        .filter(|s| s.len() >= 12)
        .map(|s| s.to_string())
        .collect();
    if parts.len() < 2 {
        parts = script
            .split(|c| c == '.' || c == '!' || c == '?')
            .map(str::trim)
            .filter(|s| s.len() >= 12)
            .map(|s| s.to_string())
            .collect();
    }
    if parts.is_empty() {
        parts.push(script.chars().take(400).collect());
    }
    if parts.len() > max {
        let mut merged = Vec::with_capacity(max);
        let bucket = (parts.len() + max - 1) / max;
        for chunk in parts.chunks(bucket) {
            merged.push(chunk.join(". "));
            if merged.len() == max {
                break;
            }
        }
        return merged;
    }
    parts
}

fn concept_from_chunk(chunk: &str, index: usize) -> (String, String) {
    let words: Vec<&str> = chunk
        .split_whitespace()
        .filter(|w| w.chars().any(|c| c.is_alphanumeric()))
        .take(4)
        .collect();
    let name = if words.is_empty() {
        format!("Concepto {}", index + 1)
    } else {
        words.join(" ")
    };
    let key = slugify(&name, index);
    (key, name)
}

fn slugify(s: &str, index: usize) -> String {
    let mut out = String::new();
    for c in s.chars().flat_map(|c| c.to_lowercase()) {
        if c.is_ascii_alphanumeric() {
            out.push(c);
        } else if c.is_whitespace() || c == '-' || c == '_' {
            if !out.ends_with('-') && !out.is_empty() {
                out.push('-');
            }
        }
    }
    let out = out.trim_matches('-').to_string();
    if out.len() < 2 {
        format!("need-{}", index + 1)
    } else {
        format!("{}-{}", out.chars().take(40).collect::<String>(), index + 1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn proposes_from_short_lesson() {
        let script = "Hoy hablamos de fotosíntesis. Las plantas capturan luz solar.\n\n\
            El CO2 entra por los estomas. El resultado es glucosa y oxígeno.\n\n\
            En la práctica, la clorofila es la molécula clave del proceso.";
        let r = propose_needs_from_script(ProposeNeedsInput {
            script: script.into(),
            max_needs: Some(5),
        })
        .unwrap();
        assert!(r.needs.len() >= 2);
        assert!(!r.script_instructions.is_empty());
        assert!(r.needs.iter().all(|n| n.variant_count == Some(3)));
        assert!(r.needs.iter().all(|n| n.ai_instructions.is_some()));
    }

    #[test]
    fn empty_script_fails() {
        assert!(propose_needs_from_script(ProposeNeedsInput {
            script: "  ".into(),
            max_needs: None,
        })
        .is_err());
    }
}
