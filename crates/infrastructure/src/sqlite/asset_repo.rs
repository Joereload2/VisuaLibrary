use rusqlite::OptionalExtension;
use visual_library_application::ports::assets::AssetStore;
use visual_library_application::AppError;
use visual_library_application::AssetDto;

use super::settings_repo::SqliteSettingsStore;

fn map_asset(row: &rusqlite::Row<'_>) -> rusqlite::Result<AssetDto> {
    Ok(AssetDto {
        id: row.get(0)?,
        concept_id: row.get(1)?,
        representation_id: row.get(2)?,
        status: row.get(3)?,
        storage_path: row.get(4)?,
        content_hash: row.get(5)?,
        width: row.get(6)?,
        height: row.get(7)?,
        mime: row.get(8)?,
        format: row.get(9)?,
        orientation: row.get(10)?,
        style: row.get(11)?,
        provider: row.get(12)?,
        prompt: row.get(13)?,
        generation_request_id: row.get(14)?,
        review_notes: row.get(15)?,
        reject_reason: row.get(16)?,
        duplicate_of_asset_id: row.get(17)?,
        approved_at: row.get(18)?,
        rejected_at: row.get(19)?,
        created_at: row.get(20)?,
        updated_at: row.get(21)?,
        package_id: row.get(22)?,
        package_path: row.get(23)?,
        beat_id: row.get(24)?,
        package_concept_key: row.get(25)?,
    })
}

const SELECT_COLS: &str = "id, concept_id, representation_id, status, storage_path, content_hash,
    width, height, mime, format, orientation, style, provider, prompt, generation_request_id,
    review_notes, reject_reason, duplicate_of_asset_id, approved_at, rejected_at, created_at, updated_at,
    package_id, package_path, beat_id, package_concept_key";

impl AssetStore for SqliteSettingsStore {
    fn insert(&self, asset: &AssetDto) -> Result<(), AppError> {
        self.with_conn(|conn| {
            conn.execute(
                "INSERT INTO assets (
                    id, concept_id, representation_id, status, storage_path, content_hash,
                    width, height, mime, format, orientation, style, provider, prompt,
                    generation_request_id, review_notes, reject_reason, duplicate_of_asset_id,
                    approved_at, rejected_at, created_at, updated_at,
                    package_id, package_path, beat_id, package_concept_key
                ) VALUES (
                    ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21, ?22,
                    ?23, ?24, ?25, ?26
                )",
                rusqlite::params![
                    asset.id,
                    asset.concept_id,
                    asset.representation_id,
                    asset.status,
                    asset.storage_path,
                    asset.content_hash,
                    asset.width,
                    asset.height,
                    asset.mime,
                    asset.format,
                    asset.orientation,
                    asset.style,
                    asset.provider,
                    asset.prompt,
                    asset.generation_request_id,
                    asset.review_notes,
                    asset.reject_reason,
                    asset.duplicate_of_asset_id,
                    asset.approved_at,
                    asset.rejected_at,
                    asset.created_at,
                    asset.updated_at,
                    asset.package_id,
                    asset.package_path,
                    asset.beat_id,
                    asset.package_concept_key,
                ],
            )
            .map_err(|e| AppError::Storage(e.to_string()))?;
            Ok(())
        })
    }

    fn get(&self, id: &str) -> Result<Option<AssetDto>, AppError> {
        self.with_conn(|conn| {
            conn.query_row(
                &format!("SELECT {SELECT_COLS} FROM assets WHERE id = ?1"),
                [id],
                map_asset,
            )
            .optional()
            .map_err(|e| AppError::Storage(e.to_string()))
        })
    }

    fn list_by_status(&self, status: &str) -> Result<Vec<AssetDto>, AppError> {
        self.with_conn(|conn| {
            let mut stmt = conn
                .prepare(&format!(
                    "SELECT {SELECT_COLS} FROM assets WHERE status = ?1 ORDER BY created_at ASC"
                ))
                .map_err(|e| AppError::Storage(e.to_string()))?;
            let rows = stmt
                .query_map([status], map_asset)
                .map_err(|e| AppError::Storage(e.to_string()))?;
            let mut out = Vec::new();
            for r in rows {
                out.push(r.map_err(|e| AppError::Storage(e.to_string()))?);
            }
            Ok(out)
        })
    }

    fn update_status(
        &self,
        id: &str,
        status: &str,
        approved_at: Option<&str>,
        rejected_at: Option<&str>,
        reject_reason: Option<&str>,
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
                    "UPDATE assets SET status = ?1, approved_at = COALESCE(?2, approved_at),
                     rejected_at = COALESCE(?3, rejected_at),
                     reject_reason = COALESCE(?4, reject_reason),
                     updated_at = ?5
                     WHERE id = ?6",
                    rusqlite::params![status, approved_at, rejected_at, reject_reason, updated, id],
                )
                .map_err(|e| AppError::Storage(e.to_string()))?;
            if n == 0 {
                return Err(AppError::NotFound(format!("asset {id}")));
            }
            Ok(())
        })
    }

    fn find_approved_match(
        &self,
        representation_id: &str,
        orientation: &str,
        style: &str,
    ) -> Result<Option<AssetDto>, AppError> {
        self.with_conn(|conn| {
            // Pull recent approved for representation; filter orientation/style in app policy terms.
            let mut stmt = conn
                .prepare(&format!(
                    "SELECT {SELECT_COLS} FROM assets
                     WHERE status = 'approved' AND representation_id = ?1
                     ORDER BY approved_at DESC, created_at DESC
                     LIMIT 20"
                ))
                .map_err(|e| AppError::Storage(e.to_string()))?;
            let rows = stmt
                .query_map([representation_id], map_asset)
                .map_err(|e| AppError::Storage(e.to_string()))?;
            for r in rows {
                let a = r.map_err(|e| AppError::Storage(e.to_string()))?;
                if visual_library_domain::field_matches(orientation, a.orientation.as_deref())
                    && visual_library_domain::field_matches(style, a.style.as_deref())
                {
                    return Ok(Some(a));
                }
            }
            Ok(None)
        })
    }

    fn update_metadata(
        &self,
        id: &str,
        review_notes: Option<&str>,
        orientation: Option<&str>,
        style: Option<&str>,
        prompt: Option<&str>,
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
                    "UPDATE assets SET
                     review_notes = COALESCE(?1, review_notes),
                     orientation = COALESCE(?2, orientation),
                     style = COALESCE(?3, style),
                     prompt = COALESCE(?4, prompt),
                     updated_at = ?5
                     WHERE id = ?6",
                    rusqlite::params![review_notes, orientation, style, prompt, updated, id],
                )
                .map_err(|e| AppError::Storage(e.to_string()))?;
            if n == 0 {
                return Err(AppError::NotFound(format!("asset {id}")));
            }
            Ok(())
        })
    }

    fn set_duplicate_of(&self, id: &str, of_asset_id: &str) -> Result<(), AppError> {
        self.with_conn(|conn| {
            use std::time::{SystemTime, UNIX_EPOCH};
            let ts = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0);
            let updated = format!("unix:{ts}");
            let n = conn
                .execute(
                    "UPDATE assets SET duplicate_of_asset_id = ?1, updated_at = ?2 WHERE id = ?3",
                    rusqlite::params![of_asset_id, updated, id],
                )
                .map_err(|e| AppError::Storage(e.to_string()))?;
            if n == 0 {
                return Err(AppError::NotFound(format!("asset {id}")));
            }
            Ok(())
        })
    }
}
