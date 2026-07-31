use crate::catalog::dto::{ConceptDto, RepresentationDto, ThemeDto};
use crate::error::AppError;
use crate::ports::catalog::CatalogStore;

fn normalize_key(key: &str) -> Result<String, AppError> {
    let k = key.trim().to_lowercase().replace(' ', "-");
    if k.is_empty() {
        return Err(AppError::Validation("key no puede estar vacío".into()));
    }
    if !k
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        return Err(AppError::Validation(
            "key solo permite a-z, 0-9, '-' y '_'".into(),
        ));
    }
    Ok(k)
}

fn normalize_name(name: &str) -> Result<String, AppError> {
    let n = name.trim();
    if n.is_empty() {
        return Err(AppError::Validation("name no puede estar vacío".into()));
    }
    Ok(n.to_string())
}

pub fn list_themes(store: &impl CatalogStore) -> Result<Vec<ThemeDto>, AppError> {
    store.list_themes()
}

pub fn ensure_theme(
    store: &impl CatalogStore,
    name: &str,
    description: Option<&str>,
) -> Result<ThemeDto, AppError> {
    let name = normalize_name(name)?;
    store.ensure_theme(&name, description)
}

pub fn list_concepts(store: &impl CatalogStore) -> Result<Vec<ConceptDto>, AppError> {
    store.list_concepts()
}

pub fn ensure_concept(
    store: &impl CatalogStore,
    key: &str,
    name: &str,
    description: Option<&str>,
) -> Result<ConceptDto, AppError> {
    let key = normalize_key(key)?;
    let name = normalize_name(name)?;
    store.ensure_concept(&key, &name, description)
}

pub fn list_representations(
    store: &impl CatalogStore,
    concept_id: &str,
) -> Result<Vec<RepresentationDto>, AppError> {
    if concept_id.trim().is_empty() {
        return Err(AppError::Validation("concept_id requerido".into()));
    }
    store.list_representations(concept_id)
}

pub fn ensure_representation(
    store: &impl CatalogStore,
    concept_id: &str,
    key: &str,
    name: &str,
    orientation_default: Option<&str>,
) -> Result<RepresentationDto, AppError> {
    if concept_id.trim().is_empty() {
        return Err(AppError::Validation("concept_id requerido".into()));
    }
    let key = normalize_key(key)?;
    let name = normalize_name(name)?;
    let orient = orientation_default.unwrap_or("any").trim();
    let orient = if orient.is_empty() { "any" } else { orient };
    store.ensure_representation(concept_id, &key, &name, orient)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::sync::Mutex;

    #[derive(Default)]
    struct MemCatalog {
        themes: Mutex<HashMap<String, ThemeDto>>,
        concepts: Mutex<HashMap<String, ConceptDto>>,
        reps: Mutex<Vec<RepresentationDto>>,
    }

    impl CatalogStore for MemCatalog {
        fn list_themes(&self) -> Result<Vec<ThemeDto>, AppError> {
            Ok(self.themes.lock().unwrap().values().cloned().collect())
        }

        fn ensure_theme(
            &self,
            name: &str,
            description: Option<&str>,
        ) -> Result<ThemeDto, AppError> {
            let mut g = self.themes.lock().unwrap();
            if let Some(t) = g.values().find(|t| t.name == name) {
                return Ok(t.clone());
            }
            let t = ThemeDto {
                id: format!("th_{}", g.len()),
                name: name.to_string(),
                description: description.map(|s| s.to_string()),
                status: "active".into(),
            };
            g.insert(t.id.clone(), t.clone());
            Ok(t)
        }

        fn list_concepts(&self) -> Result<Vec<ConceptDto>, AppError> {
            Ok(self.concepts.lock().unwrap().values().cloned().collect())
        }

        fn ensure_concept(
            &self,
            key: &str,
            name: &str,
            description: Option<&str>,
        ) -> Result<ConceptDto, AppError> {
            let mut g = self.concepts.lock().unwrap();
            if let Some(c) = g.values().find(|c| c.key == key) {
                return Ok(c.clone());
            }
            let c = ConceptDto {
                id: format!("c_{}", g.len()),
                key: key.to_string(),
                name: name.to_string(),
                description: description.map(|s| s.to_string()),
                status: "active".into(),
            };
            g.insert(c.id.clone(), c.clone());
            Ok(c)
        }

        fn list_representations(
            &self,
            concept_id: &str,
        ) -> Result<Vec<RepresentationDto>, AppError> {
            Ok(self
                .reps
                .lock()
                .unwrap()
                .iter()
                .filter(|r| r.concept_id == concept_id)
                .cloned()
                .collect())
        }

        fn ensure_representation(
            &self,
            concept_id: &str,
            key: &str,
            name: &str,
            orientation_default: &str,
        ) -> Result<RepresentationDto, AppError> {
            let mut g = self.reps.lock().unwrap();
            if let Some(r) = g
                .iter()
                .find(|r| r.concept_id == concept_id && r.key == key)
            {
                return Ok(r.clone());
            }
            let r = RepresentationDto {
                id: format!("r_{}", g.len()),
                concept_id: concept_id.to_string(),
                key: key.to_string(),
                name: name.to_string(),
                orientation_default: orientation_default.to_string(),
                status: "active".into(),
            };
            g.push(r.clone());
            Ok(r)
        }
    }

    #[test]
    fn normalize_rejects_bad_key() {
        let store = MemCatalog::default();
        assert!(ensure_concept(&store, "Bad Key!", "X", None).is_err());
    }

    #[test]
    fn ensure_concept_idempotent_by_key() {
        let store = MemCatalog::default();
        let a = ensure_concept(&store, "hero", "Hero", None).unwrap();
        let b = ensure_concept(&store, "hero", "Hero 2", None).unwrap();
        assert_eq!(a.id, b.id);
        assert_eq!(a.key, "hero");
    }
}
