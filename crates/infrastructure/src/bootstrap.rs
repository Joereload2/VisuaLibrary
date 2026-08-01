use std::path::PathBuf;
use std::sync::Arc;

use visual_library_application::AppError;

use crate::error::InfraError;
use crate::paths::AppLayout;
use crate::sqlite::{migrate, open_database, SqliteSettingsStore};

pub struct Platform {
    pub layout: AppLayout,
    pub settings: Arc<SqliteSettingsStore>,
}

/// Open DB, migrate, ensure directories. `app_data_root` is OS app data (parent of visual-library/).
pub fn bootstrap(app_data_root: PathBuf) -> Result<Platform, AppError> {
    let layout = AppLayout::from_app_data_root(app_data_root);
    layout
        .ensure_directories()
        .map_err(|e: InfraError| AppError::from(e))?;

    let conn = open_database(&layout.db_path).map_err(AppError::from)?;
    migrate(&conn).map_err(AppError::from)?;

    let settings = Arc::new(SqliteSettingsStore::new(conn));
    // Durable jobs: recover work left in `running` after crash/kill.
    let _recovered = settings.recover_running_jobs()?;

    Ok(Platform {
        layout,
        settings,
    })
}
