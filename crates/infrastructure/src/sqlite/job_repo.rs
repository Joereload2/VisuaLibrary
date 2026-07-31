use rusqlite::OptionalExtension;
use visual_library_application::ports::jobs::JobStore;
use visual_library_application::AppError;
use visual_library_application::JobDto;

use super::settings_repo::SqliteSettingsStore;

fn map_job(row: &rusqlite::Row<'_>) -> rusqlite::Result<JobDto> {
    Ok(JobDto {
        id: row.get(0)?,
        job_type: row.get(1)?,
        payload_json: row.get(2)?,
        status: row.get(3)?,
        priority: row.get(4)?,
        attempts: row.get(5)?,
        max_attempts: row.get(6)?,
        last_error: row.get(7)?,
        related_entity_type: row.get(8)?,
        related_entity_id: row.get(9)?,
        idempotency_key: row.get(10)?,
        outputs_json: row.get(11)?,
        created_at: row.get(12)?,
        updated_at: row.get(13)?,
        started_at: row.get(14)?,
        finished_at: row.get(15)?,
    })
}

const SELECT_COLS: &str = "id, job_type, payload_json, status, priority, attempts, max_attempts,
    last_error, related_entity_type, related_entity_id, idempotency_key, outputs_json,
    created_at, updated_at, started_at, finished_at";

impl JobStore for SqliteSettingsStore {
    fn insert(&self, job: &JobDto) -> Result<(), AppError> {
        self.with_conn(|conn| {
            conn.execute(
                "INSERT INTO jobs (
                    id, job_type, payload_json, status, priority, attempts, max_attempts,
                    last_error, related_entity_type, related_entity_id, idempotency_key,
                    outputs_json, created_at, updated_at, started_at, finished_at
                ) VALUES (
                    ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16
                )",
                rusqlite::params![
                    job.id,
                    job.job_type,
                    job.payload_json,
                    job.status,
                    job.priority,
                    job.attempts,
                    job.max_attempts,
                    job.last_error,
                    job.related_entity_type,
                    job.related_entity_id,
                    job.idempotency_key,
                    job.outputs_json,
                    job.created_at,
                    job.updated_at,
                    job.started_at,
                    job.finished_at,
                ],
            )
            .map_err(|e| AppError::Storage(e.to_string()))?;
            Ok(())
        })
    }

    fn get(&self, id: &str) -> Result<Option<JobDto>, AppError> {
        self.with_conn(|conn| {
            conn.query_row(
                &format!("SELECT {SELECT_COLS} FROM jobs WHERE id = ?1"),
                [id],
                map_job,
            )
            .optional()
            .map_err(|e| AppError::Storage(e.to_string()))
        })
    }

    fn get_by_idempotency_key(&self, key: &str) -> Result<Option<JobDto>, AppError> {
        self.with_conn(|conn| {
            conn.query_row(
                &format!("SELECT {SELECT_COLS} FROM jobs WHERE idempotency_key = ?1"),
                [key],
                map_job,
            )
            .optional()
            .map_err(|e| AppError::Storage(e.to_string()))
        })
    }

    fn update(
        &self,
        id: &str,
        status: &str,
        attempts: i64,
        last_error: Option<&str>,
        outputs_json: Option<&str>,
        started_at: Option<&str>,
        finished_at: Option<&str>,
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
                    "UPDATE jobs SET status = ?1, attempts = ?2,
                     last_error = COALESCE(?3, last_error),
                     outputs_json = COALESCE(?4, outputs_json),
                     started_at = COALESCE(?5, started_at),
                     finished_at = COALESCE(?6, finished_at),
                     updated_at = ?7
                     WHERE id = ?8",
                    rusqlite::params![
                        status,
                        attempts,
                        last_error,
                        outputs_json,
                        started_at,
                        finished_at,
                        updated,
                        id
                    ],
                )
                .map_err(|e| AppError::Storage(e.to_string()))?;
            if n == 0 {
                return Err(AppError::NotFound(format!("job {id}")));
            }
            Ok(())
        })
    }
}
