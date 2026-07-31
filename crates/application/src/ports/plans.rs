use crate::error::AppError;
use crate::plans::{PlanDto, PlanItemDto};

pub trait PlanStore {
    fn insert_plan(&self, plan: &PlanDto) -> Result<(), AppError>;
    fn get_plan(&self, id: &str) -> Result<Option<PlanDto>, AppError>;
    fn list_plans(&self) -> Result<Vec<PlanDto>, AppError>;
    fn update_plan_status(
        &self,
        id: &str,
        status: &str,
        approved_at: Option<&str>,
    ) -> Result<(), AppError>;

    fn insert_item(&self, item: &PlanItemDto) -> Result<(), AppError>;
    fn list_items(&self, plan_id: &str) -> Result<Vec<PlanItemDto>, AppError>;
    fn update_item_status(&self, id: &str, status: &str) -> Result<(), AppError>;
}
