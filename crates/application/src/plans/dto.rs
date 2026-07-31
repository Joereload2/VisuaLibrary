use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PlanDto {
    pub id: String,
    pub theme_id: Option<String>,
    pub name: String,
    pub description: Option<String>,
    pub status: String,
    pub approved_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PlanItemDto {
    pub id: String,
    pub plan_id: String,
    pub concept_id: Option<String>,
    pub representation_id: Option<String>,
    pub concept_key: Option<String>,
    pub representation_key: Option<String>,
    pub action: String,
    pub priority: i64,
    pub target_count: i64,
    pub constraints_json: Option<String>,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanWithItemsDto {
    pub plan: PlanDto,
    pub items: Vec<PlanItemDto>,
}
