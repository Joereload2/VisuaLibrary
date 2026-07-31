use std::fs;
use std::path::{Path, PathBuf};

use visual_library_application::AppPathsDto;

use crate::error::InfraError;

/// On-disk layout under app data (ARCHITECTURE.md).
#[derive(Debug, Clone)]
pub struct AppLayout {
    pub root: PathBuf,
    pub db_path: PathBuf,
    pub media_root: PathBuf,
    pub exports_dir: PathBuf,
    pub tmp_dir: PathBuf,
    pub logs_dir: PathBuf,
}

impl AppLayout {
    pub fn from_app_data_root(app_data_root: impl Into<PathBuf>) -> Self {
        let root = app_data_root.into().join("visual-library");
        Self {
            db_path: root.join("db").join("visual_library.sqlite"),
            media_root: root.join("media"),
            exports_dir: root.join("exports"),
            tmp_dir: root.join("tmp").join("jobs"),
            logs_dir: root.join("logs"),
            root,
        }
    }

    pub fn ensure_directories(&self) -> Result<(), InfraError> {
        for dir in [
            self.root.join("db"),
            self.media_root.clone(),
            self.exports_dir.clone(),
            self.tmp_dir.clone(),
            self.logs_dir.clone(),
        ] {
            fs::create_dir_all(&dir)?;
        }
        Ok(())
    }

    pub fn to_dto(&self) -> AppPathsDto {
        AppPathsDto {
            app_data_root: self.root.to_string_lossy().into_owned(),
            db_path: self.db_path.to_string_lossy().into_owned(),
            media_root: self.media_root.to_string_lossy().into_owned(),
            exports_dir: self.exports_dir.to_string_lossy().into_owned(),
            tmp_dir: self.tmp_dir.to_string_lossy().into_owned(),
            logs_dir: self.logs_dir.to_string_lossy().into_owned(),
        }
    }
}

pub fn resolve_media_root(layout: &AppLayout, configured: Option<&str>) -> PathBuf {
    match configured {
        Some(p) if !p.trim().is_empty() => PathBuf::from(p.trim()),
        _ => layout.media_root.clone(),
    }
}

pub fn ensure_dir(path: &Path) -> Result<(), InfraError> {
    fs::create_dir_all(path)?;
    Ok(())
}
