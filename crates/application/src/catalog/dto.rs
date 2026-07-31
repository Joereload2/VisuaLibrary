use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ThemeDto {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ConceptDto {
    pub id: String,
    pub key: String,
    pub name: String,
    pub description: Option<String>,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RepresentationDto {
    pub id: String,
    pub concept_id: String,
    pub key: String,
    pub name: String,
    pub orientation_default: String,
    pub status: String,
}
