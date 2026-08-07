use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AssetDto {
    pub id: String,
    pub concept_id: String,
    pub representation_id: String,
    pub status: String,
    pub storage_path: String,
    pub content_hash: Option<String>,
    pub width: Option<i64>,
    pub height: Option<i64>,
    pub mime: Option<String>,
    pub format: Option<String>,
    pub orientation: Option<String>,
    pub style: Option<String>,
    pub provider: Option<String>,
    pub prompt: Option<String>,
    pub generation_request_id: Option<String>,
    pub review_notes: Option<String>,
    pub reject_reason: Option<String>,
    pub duplicate_of_asset_id: Option<String>,
    pub approved_at: Option<String>,
    pub rejected_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    /// FacelessStudio package handoff (optional; set when generated from package needs).
    #[serde(default)]
    pub package_id: Option<String>,
    #[serde(default)]
    pub package_path: Option<String>,
    #[serde(default)]
    pub beat_id: Option<String>,
    #[serde(default)]
    pub package_concept_key: Option<String>,
}
