use std::path::{Component, Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::AppError;
use crate::ports::settings::SettingsStore;
use crate::settings::keys;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AppPathsDto {
    pub app_data_root: String,
    pub db_path: String,
    pub media_root: String,
    pub exports_dir: String,
    pub tmp_dir: String,
    pub logs_dir: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SettingsDto {
    pub media_root: String,
}

pub fn validate_media_root(path: &str) -> Result<PathBuf, AppError> {
    let trimmed = path.trim();
    if trimmed.is_empty() {
        return Err(AppError::Validation(
            "media_root no puede estar vacío".into(),
        ));
    }
    let p = PathBuf::from(trimmed);
    if p.as_os_str().is_empty() {
        return Err(AppError::Validation("media_root inválido".into()));
    }
    // Reject path segments that escape via `..` when treated as relative under a root.
    if p.components().any(|c| matches!(c, Component::ParentDir)) {
        return Err(AppError::Validation(
            "media_root no puede contener '..'".into(),
        ));
    }
    Ok(p)
}

pub fn get_settings(
    store: &impl SettingsStore,
    default_media_root: &Path,
) -> Result<SettingsDto, AppError> {
    let media = match store.get_json(keys::MEDIA_ROOT)? {
        Some(raw) => {
            let parsed: String = serde_json::from_str(&raw).map_err(|e| {
                AppError::Storage(format!("settings.media_root JSON inválido: {e}"))
            })?;
            validate_media_root(&parsed)?.to_string_lossy().into_owned()
        }
        None => default_media_root.to_string_lossy().into_owned(),
    };
    Ok(SettingsDto { media_root: media })
}

pub fn update_media_root(
    store: &impl SettingsStore,
    media_root: &str,
) -> Result<SettingsDto, AppError> {
    let path = validate_media_root(media_root)?;
    let as_str = path.to_string_lossy();
    let json = serde_json::to_string(as_str.as_ref())
        .map_err(|e| AppError::Internal(format!("serialize media_root: {e}")))?;
    store.set_json(keys::MEDIA_ROOT, &json)?;
    Ok(SettingsDto {
        media_root: as_str.into_owned(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::sync::Mutex;

    struct MemStore(Mutex<HashMap<String, String>>);

    impl SettingsStore for MemStore {
        fn get_json(&self, key: &str) -> Result<Option<String>, AppError> {
            Ok(self.0.lock().unwrap().get(key).cloned())
        }

        fn set_json(&self, key: &str, value_json: &str) -> Result<(), AppError> {
            self.0
                .lock()
                .unwrap()
                .insert(key.to_string(), value_json.to_string());
            Ok(())
        }
    }

    #[test]
    fn rejects_parent_dir_in_media_root() {
        let err = validate_media_root("foo/../bar").unwrap_err();
        assert!(matches!(err, AppError::Validation(_)));
    }

    #[test]
    fn update_and_get_media_root() {
        let store = MemStore(Mutex::new(HashMap::new()));
        let default = PathBuf::from("C:/data/media");
        let before = get_settings(&store, &default).unwrap();
        assert_eq!(before.media_root, default.to_string_lossy());

        update_media_root(&store, r"D:\vl\media").unwrap();
        let after = get_settings(&store, &default).unwrap();
        assert!(after.media_root.contains("media"));
    }
}
