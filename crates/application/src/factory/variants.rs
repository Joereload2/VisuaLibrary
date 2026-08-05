//! Variantes / matices por need (Manual Factory v1.1).
//! Mezcla: literal vs metafórico (B) + estilo visual (C).

/// Appended to every generation prompt so models avoid garbled letters.
pub const NO_TEXT_IN_IMAGE_GUARD: &str = "\
STRICT: pure visual illustration only. No text, no letters, no words, no numbers, no captions, \
no labels, no signs, no logos, no watermarks, no UI chrome, no speech bubbles with writing, \
no readable typography of any kind. Teach with shapes, color, composition, and characters only.";

/// Cuántas variantes generar (1..=3). Default producto = 3.
pub fn clamp_variant_count(n: Option<u8>) -> usize {
    n.unwrap_or(3).clamp(1, 3) as usize
}

/// (label corta, sufijo de prompt para el matiz).
pub fn matiz_specs(count: usize) -> Vec<(&'static str, &'static str)> {
    let all = [
        (
            "literal-didactic",
            "Matiz: representación LITERAL y didáctica del concepto; estilo limpio, claro, realista-educativo; sin metáforas abstractas. Sin texto en la imagen.",
        ),
        (
            "metaphor-warm",
            "Matiz: representación METAFÓRICA visual (símbolos, no carteles con palabras); estilo cálido y expresivo, legible sin leer.",
        ),
        (
            "hybrid-graphic",
            "Matiz: híbrido literal+símbolo, estilo gráfico de lección; claridad pedagógica sin tipografía ni etiquetas escritas.",
        ),
    ];
    let n = count.clamp(1, 3);
    all.into_iter().take(n).collect()
}

pub fn apply_matiz_to_prompt(
    base: &str,
    suffix: &str,
    variant_index: usize,
    total: usize,
) -> String {
    format!("{base}\n\n---\nVariant {variant_index}/{total}\n{suffix}\n\n{NO_TEXT_IN_IMAGE_GUARD}")
}

/// Ensure a free-form prompt still carries the no-text guard (idempotent).
pub fn with_no_text_guard(prompt: &str) -> String {
    let p = prompt.trim();
    if p.to_ascii_lowercase().contains("no text")
        || p.to_ascii_lowercase().contains("no letters")
        || p.contains("STRICT: pure visual")
    {
        p.to_string()
    } else {
        format!("{p}\n\n{NO_TEXT_IN_IMAGE_GUARD}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_three() {
        assert_eq!(clamp_variant_count(None), 3);
        assert_eq!(clamp_variant_count(Some(0)), 1);
        assert_eq!(clamp_variant_count(Some(9)), 3);
    }

    #[test]
    fn two_matizes() {
        assert_eq!(matiz_specs(2).len(), 2);
        assert!(matiz_specs(2)[0].0.contains("literal"));
    }

    #[test]
    fn apply_matiz_includes_no_text_guard() {
        let p = apply_matiz_to_prompt("A tree", matiz_specs(1)[0].1, 1, 1);
        assert!(p.to_ascii_lowercase().contains("no text") || p.contains("STRICT"));
    }

    #[test]
    fn with_no_text_guard_is_idempotent() {
        let once = with_no_text_guard("A river");
        let twice = with_no_text_guard(&once);
        assert_eq!(once.matches("STRICT: pure visual").count(), 1);
        assert_eq!(twice.matches("STRICT: pure visual").count(), 1);
    }
}
