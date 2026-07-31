use crate::catalog::{ConceptDto, RepresentationDto, ThemeDto};
use crate::error::AppError;

pub trait CatalogStore {
    fn list_themes(&self) -> Result<Vec<ThemeDto>, AppError>;
    fn ensure_theme(&self, name: &str, description: Option<&str>) -> Result<ThemeDto, AppError>;

    fn list_concepts(&self) -> Result<Vec<ConceptDto>, AppError>;
    fn ensure_concept(
        &self,
        key: &str,
        name: &str,
        description: Option<&str>,
    ) -> Result<ConceptDto, AppError>;

    fn list_representations(&self, concept_id: &str) -> Result<Vec<RepresentationDto>, AppError>;
    fn ensure_representation(
        &self,
        concept_id: &str,
        key: &str,
        name: &str,
        orientation_default: &str,
    ) -> Result<RepresentationDto, AppError>;
}
