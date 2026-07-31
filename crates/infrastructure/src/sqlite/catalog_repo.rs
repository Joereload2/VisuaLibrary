use rusqlite::OptionalExtension;
use visual_library_application::catalog::{ConceptDto, RepresentationDto, ThemeDto};
use visual_library_application::ports::catalog::CatalogStore;
use visual_library_application::AppError;

use super::settings_repo::SqliteSettingsStore;

fn now() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    format!("unix:{secs}")
}

fn new_id(prefix: &str) -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let d = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    format!("{prefix}_{}_{}", d.as_secs(), d.subsec_nanos())
}

impl CatalogStore for SqliteSettingsStore {
    fn list_themes(&self) -> Result<Vec<ThemeDto>, AppError> {
        self.with_conn(|conn| {
            let mut stmt = conn
                .prepare(
                    "SELECT id, name, description, status FROM themes ORDER BY name COLLATE NOCASE",
                )
                .map_err(|e| AppError::Storage(e.to_string()))?;
            let rows = stmt
                .query_map([], |row| {
                    Ok(ThemeDto {
                        id: row.get(0)?,
                        name: row.get(1)?,
                        description: row.get(2)?,
                        status: row.get(3)?,
                    })
                })
                .map_err(|e| AppError::Storage(e.to_string()))?;
            let mut out = Vec::new();
            for r in rows {
                out.push(r.map_err(|e| AppError::Storage(e.to_string()))?);
            }
            Ok(out)
        })
    }

    fn ensure_theme(&self, name: &str, description: Option<&str>) -> Result<ThemeDto, AppError> {
        self.with_conn(|conn| {
            let existing: Option<ThemeDto> = conn
                .query_row(
                    "SELECT id, name, description, status FROM themes WHERE name = ?1",
                    [name],
                    |row| {
                        Ok(ThemeDto {
                            id: row.get(0)?,
                            name: row.get(1)?,
                            description: row.get(2)?,
                            status: row.get(3)?,
                        })
                    },
                )
                .optional()
                .map_err(|e| AppError::Storage(e.to_string()))?;
            if let Some(t) = existing {
                return Ok(t);
            }
            let id = new_id("th");
            let ts = now();
            conn.execute(
                "INSERT INTO themes (id, name, description, status, created_at, updated_at)
                 VALUES (?1, ?2, ?3, 'active', ?4, ?4)",
                rusqlite::params![id, name, description, ts],
            )
            .map_err(|e| AppError::Storage(e.to_string()))?;
            Ok(ThemeDto {
                id,
                name: name.to_string(),
                description: description.map(|s| s.to_string()),
                status: "active".into(),
            })
        })
    }

    fn list_concepts(&self) -> Result<Vec<ConceptDto>, AppError> {
        self.with_conn(|conn| {
            let mut stmt = conn
                .prepare(
                    "SELECT id, key, name, description, status FROM concepts ORDER BY key COLLATE NOCASE",
                )
                .map_err(|e| AppError::Storage(e.to_string()))?;
            let rows = stmt
                .query_map([], |row| {
                    Ok(ConceptDto {
                        id: row.get(0)?,
                        key: row.get(1)?,
                        name: row.get(2)?,
                        description: row.get(3)?,
                        status: row.get(4)?,
                    })
                })
                .map_err(|e| AppError::Storage(e.to_string()))?;
            let mut out = Vec::new();
            for r in rows {
                out.push(r.map_err(|e| AppError::Storage(e.to_string()))?);
            }
            Ok(out)
        })
    }

    fn ensure_concept(
        &self,
        key: &str,
        name: &str,
        description: Option<&str>,
    ) -> Result<ConceptDto, AppError> {
        self.with_conn(|conn| {
            let existing: Option<ConceptDto> = conn
                .query_row(
                    "SELECT id, key, name, description, status FROM concepts WHERE key = ?1",
                    [key],
                    |row| {
                        Ok(ConceptDto {
                            id: row.get(0)?,
                            key: row.get(1)?,
                            name: row.get(2)?,
                            description: row.get(3)?,
                            status: row.get(4)?,
                        })
                    },
                )
                .optional()
                .map_err(|e| AppError::Storage(e.to_string()))?;
            if let Some(c) = existing {
                return Ok(c);
            }
            let id = new_id("c");
            let ts = now();
            conn.execute(
                "INSERT INTO concepts (id, key, name, description, status, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, 'active', ?5, ?5)",
                rusqlite::params![id, key, name, description, ts],
            )
            .map_err(|e| AppError::Storage(e.to_string()))?;
            Ok(ConceptDto {
                id,
                key: key.to_string(),
                name: name.to_string(),
                description: description.map(|s| s.to_string()),
                status: "active".into(),
            })
        })
    }

    fn list_representations(&self, concept_id: &str) -> Result<Vec<RepresentationDto>, AppError> {
        self.with_conn(|conn| {
            let mut stmt = conn
                .prepare(
                    "SELECT id, concept_id, key, name, orientation_default, status
                     FROM representations WHERE concept_id = ?1 ORDER BY key COLLATE NOCASE",
                )
                .map_err(|e| AppError::Storage(e.to_string()))?;
            let rows = stmt
                .query_map([concept_id], |row| {
                    Ok(RepresentationDto {
                        id: row.get(0)?,
                        concept_id: row.get(1)?,
                        key: row.get(2)?,
                        name: row.get(3)?,
                        orientation_default: row.get(4)?,
                        status: row.get(5)?,
                    })
                })
                .map_err(|e| AppError::Storage(e.to_string()))?;
            let mut out = Vec::new();
            for r in rows {
                out.push(r.map_err(|e| AppError::Storage(e.to_string()))?);
            }
            Ok(out)
        })
    }

    fn ensure_representation(
        &self,
        concept_id: &str,
        key: &str,
        name: &str,
        orientation_default: &str,
    ) -> Result<RepresentationDto, AppError> {
        self.with_conn(|conn| {
            let exists: bool = conn
                .query_row(
                    "SELECT EXISTS(SELECT 1 FROM concepts WHERE id = ?1)",
                    [concept_id],
                    |r| r.get(0),
                )
                .map_err(|e| AppError::Storage(e.to_string()))?;
            if !exists {
                return Err(AppError::NotFound(format!(
                    "concept no encontrado: {concept_id}"
                )));
            }

            let existing: Option<RepresentationDto> = conn
                .query_row(
                    "SELECT id, concept_id, key, name, orientation_default, status
                     FROM representations WHERE concept_id = ?1 AND key = ?2",
                    [concept_id, key],
                    |row| {
                        Ok(RepresentationDto {
                            id: row.get(0)?,
                            concept_id: row.get(1)?,
                            key: row.get(2)?,
                            name: row.get(3)?,
                            orientation_default: row.get(4)?,
                            status: row.get(5)?,
                        })
                    },
                )
                .optional()
                .map_err(|e| AppError::Storage(e.to_string()))?;
            if let Some(r) = existing {
                return Ok(r);
            }

            let id = new_id("r");
            let ts = now();
            conn.execute(
                "INSERT INTO representations
                 (id, concept_id, key, name, orientation_default, status, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, 'active', ?6, ?6)",
                rusqlite::params![id, concept_id, key, name, orientation_default, ts],
            )
            .map_err(|e| AppError::Storage(e.to_string()))?;
            Ok(RepresentationDto {
                id,
                concept_id: concept_id.to_string(),
                key: key.to_string(),
                name: name.to_string(),
                orientation_default: orientation_default.to_string(),
                status: "active".into(),
            })
        })
    }
}
