use rusqlite::OptionalExtension;
use visual_library_application::ports::plans::PlanStore;
use visual_library_application::AppError;
use visual_library_application::{PlanDto, PlanItemDto};

use super::settings_repo::SqliteSettingsStore;

fn map_plan(row: &rusqlite::Row<'_>) -> rusqlite::Result<PlanDto> {
    Ok(PlanDto {
        id: row.get(0)?,
        theme_id: row.get(1)?,
        name: row.get(2)?,
        description: row.get(3)?,
        status: row.get(4)?,
        approved_at: row.get(5)?,
        created_at: row.get(6)?,
        updated_at: row.get(7)?,
    })
}

fn map_item(row: &rusqlite::Row<'_>) -> rusqlite::Result<PlanItemDto> {
    Ok(PlanItemDto {
        id: row.get(0)?,
        plan_id: row.get(1)?,
        concept_id: row.get(2)?,
        representation_id: row.get(3)?,
        concept_key: row.get(4)?,
        representation_key: row.get(5)?,
        action: row.get(6)?,
        priority: row.get(7)?,
        target_count: row.get(8)?,
        constraints_json: row.get(9)?,
        status: row.get(10)?,
        created_at: row.get(11)?,
        updated_at: row.get(12)?,
    })
}

impl PlanStore for SqliteSettingsStore {
    fn insert_plan(&self, plan: &PlanDto) -> Result<(), AppError> {
        self.with_conn(|conn| {
            conn.execute(
                "INSERT INTO coverage_plans
                 (id, theme_id, name, description, status, approved_at, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                rusqlite::params![
                    plan.id,
                    plan.theme_id,
                    plan.name,
                    plan.description,
                    plan.status,
                    plan.approved_at,
                    plan.created_at,
                    plan.updated_at,
                ],
            )
            .map_err(|e| AppError::Storage(e.to_string()))?;
            Ok(())
        })
    }

    fn get_plan(&self, id: &str) -> Result<Option<PlanDto>, AppError> {
        self.with_conn(|conn| {
            conn.query_row(
                "SELECT id, theme_id, name, description, status, approved_at, created_at, updated_at
                 FROM coverage_plans WHERE id = ?1",
                [id],
                map_plan,
            )
            .optional()
            .map_err(|e| AppError::Storage(e.to_string()))
        })
    }

    fn list_plans(&self) -> Result<Vec<PlanDto>, AppError> {
        self.with_conn(|conn| {
            let mut stmt = conn
                .prepare(
                    "SELECT id, theme_id, name, description, status, approved_at, created_at, updated_at
                     FROM coverage_plans ORDER BY created_at DESC",
                )
                .map_err(|e| AppError::Storage(e.to_string()))?;
            let rows = stmt
                .query_map([], map_plan)
                .map_err(|e| AppError::Storage(e.to_string()))?;
            let mut out = Vec::new();
            for r in rows {
                out.push(r.map_err(|e| AppError::Storage(e.to_string()))?);
            }
            Ok(out)
        })
    }

    fn update_plan_status(
        &self,
        id: &str,
        status: &str,
        approved_at: Option<&str>,
    ) -> Result<(), AppError> {
        self.with_conn(|conn| {
            use std::time::{SystemTime, UNIX_EPOCH};
            let ts = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0);
            let updated = format!("unix:{ts}");
            let n = conn
                .execute(
                    "UPDATE coverage_plans SET status = ?1,
                     approved_at = COALESCE(?2, approved_at), updated_at = ?3
                     WHERE id = ?4",
                    rusqlite::params![status, approved_at, updated, id],
                )
                .map_err(|e| AppError::Storage(e.to_string()))?;
            if n == 0 {
                return Err(AppError::NotFound(format!("plan {id}")));
            }
            Ok(())
        })
    }

    fn insert_item(&self, item: &PlanItemDto) -> Result<(), AppError> {
        self.with_conn(|conn| {
            conn.execute(
                "INSERT INTO coverage_plan_items
                 (id, plan_id, concept_id, representation_id, concept_key, representation_key,
                  action, priority, target_count, constraints_json, status, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
                rusqlite::params![
                    item.id,
                    item.plan_id,
                    item.concept_id,
                    item.representation_id,
                    item.concept_key,
                    item.representation_key,
                    item.action,
                    item.priority,
                    item.target_count,
                    item.constraints_json,
                    item.status,
                    item.created_at,
                    item.updated_at,
                ],
            )
            .map_err(|e| AppError::Storage(e.to_string()))?;
            Ok(())
        })
    }

    fn list_items(&self, plan_id: &str) -> Result<Vec<PlanItemDto>, AppError> {
        self.with_conn(|conn| {
            let mut stmt = conn
                .prepare(
                    "SELECT id, plan_id, concept_id, representation_id, concept_key, representation_key,
                            action, priority, target_count, constraints_json, status, created_at, updated_at
                     FROM coverage_plan_items WHERE plan_id = ?1
                     ORDER BY priority ASC, created_at ASC",
                )
                .map_err(|e| AppError::Storage(e.to_string()))?;
            let rows = stmt
                .query_map([plan_id], map_item)
                .map_err(|e| AppError::Storage(e.to_string()))?;
            let mut out = Vec::new();
            for r in rows {
                out.push(r.map_err(|e| AppError::Storage(e.to_string()))?);
            }
            Ok(out)
        })
    }

    fn update_item_status(&self, id: &str, status: &str) -> Result<(), AppError> {
        self.with_conn(|conn| {
            use std::time::{SystemTime, UNIX_EPOCH};
            let ts = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0);
            let updated = format!("unix:{ts}");
            let n = conn
                .execute(
                    "UPDATE coverage_plan_items SET status = ?1, updated_at = ?2 WHERE id = ?3",
                    rusqlite::params![status, updated, id],
                )
                .map_err(|e| AppError::Storage(e.to_string()))?;
            if n == 0 {
                return Err(AppError::NotFound(format!("plan item {id}")));
            }
            Ok(())
        })
    }
}
