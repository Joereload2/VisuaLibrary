use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct JobDto {
    pub id: String,
    pub job_type: String,
    pub payload_json: String,
    pub status: String,
    pub priority: i64,
    pub attempts: i64,
    pub max_attempts: i64,
    pub last_error: Option<String>,
    pub related_entity_type: Option<String>,
    pub related_entity_id: Option<String>,
    pub idempotency_key: Option<String>,
    pub outputs_json: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub started_at: Option<String>,
    pub finished_at: Option<String>,
}
