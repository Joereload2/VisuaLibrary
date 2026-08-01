//! Variantes / matices por need (Manual Factory v1.1).
//! Mezcla: literal vs metafórico (B) + estilo visual (C).

/// Cuántas variantes generar (1..=3). Default producto = 3.
pub fn clamp_variant_count(n: Option<u8>) -> usize {
    n.unwrap_or(3).clamp(1, 3) as usize
}

/// (label corta, sufijo de prompt para el matiz).
pub fn matiz_specs(count: usize) -> Vec<(&'static str, &'static str)> {
    let all = [
        (
            "literal-didactic",
            "Matiz: representación LITERAL y didáctica del concepto; estilo limpio, claro, realista-educativo; sin metáforas abstractas.",
        ),
        (
            "metaphor-warm",
            "Matiz: representación METAFÓRICA que ayude a comprender la lección; estilo visual más cálido y expresivo, simbólico pero legible.",
        ),
        (
            "hybrid-graphic",
            "Matiz: híbrido literal+símbolo con estilo gráfico/ilustrado de lección; énfasis en claridad pedagógica y variedad visual para el canal.",
        ),
    ];
    let n = count.clamp(1, 3);
    all.into_iter().take(n).collect()
}

pub fn apply_matiz_to_prompt(base: &str, suffix: &str, variant_index: usize, total: usize) -> String {
    format!("{base}\n\n---\nVariant {variant_index}/{total}\n{suffix}")
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
}
