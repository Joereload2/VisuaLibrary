use crate::error::AppError;
use crate::jobs::JobDto;

pub trait JobStore {
    fn insert(&self, job: &JobDto) -> Result<(), AppError>;
    fn get(&self, id: &str) -> Result<Option<JobDto>, AppError>;
    fn get_by_idempotency_key(&self, key: &str) -> Result<Option<JobDto>, AppError>;
    fn update(
        &self,
        id: &str,
        status: &str,
        attempts: i64,
        last_error: Option<&str>,
        outputs_json: Option<&str>,
        started_at: Option<&str>,
        finished_at: Option<&str>,
    ) -> Result<(), AppError>;
}
