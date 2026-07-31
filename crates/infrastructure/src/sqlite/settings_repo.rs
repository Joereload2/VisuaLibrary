use std::sync::Mutex;

use rusqlite::Connection;
use visual_library_application::ports::settings::SettingsStore;
use visual_library_application::AppError;

/// Settings store backed by SQLite `settings` table.
pub struct SqliteSettingsStore {
    conn: Mutex<Connection>,
}

impl SqliteSettingsStore {
    pub fn new(conn: Connection) -> Self {
        Self {
            conn: Mutex::new(conn),
        }
    }

    pub fn with_conn<F, T>(&self, f: F) -> Result<T, AppError>
    where
        F: FnOnce(&Connection) -> Result<T, AppError>,
    {
        let guard = self
            .conn
            .lock()
            .map_err(|_| AppError::Internal("db mutex poisoned".into()))?;
        f(&guard)
    }

    pub fn connection_mutex(&self) -> &Mutex<Connection> {
        &self.conn
    }
}

impl SettingsStore for SqliteSettingsStore {
    fn get_json(&self, key: &str) -> Result<Option<String>, AppError> {
        self.with_conn(|conn| {
            let mut stmt = conn
                .prepare("SELECT value_json FROM settings WHERE key = ?1")
                .map_err(|e| AppError::Storage(e.to_string()))?;
            let mut rows = stmt
                .query([key])
                .map_err(|e| AppError::Storage(e.to_string()))?;
            match rows.next().map_err(|e| AppError::Storage(e.to_string()))? {
                Some(row) => {
                    let v: String = row.get(0).map_err(|e| AppError::Storage(e.to_string()))?;
                    Ok(Some(v))
                }
                None => Ok(None),
            }
        })
    }

    fn set_json(&self, key: &str, value_json: &str) -> Result<(), AppError> {
        self.with_conn(|conn| {
            let now = {
                use std::time::{SystemTime, UNIX_EPOCH};
                let secs = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .map(|d| d.as_secs())
                    .unwrap_or(0);
                format!("unix:{secs}")
            };
            conn.execute(
                "INSERT INTO settings (key, value_json, updated_at) VALUES (?1, ?2, ?3)
                 ON CONFLICT(key) DO UPDATE SET value_json = excluded.value_json, updated_at = excluded.updated_at",
                (key, value_json, now.as_str()),
            )
            .map_err(|e| AppError::Storage(e.to_string()))?;
            Ok(())
        })
    }
}
