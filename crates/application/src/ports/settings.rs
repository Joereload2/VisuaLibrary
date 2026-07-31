use crate::error::AppError;

/// Port for key/value settings (JSON values as text).
pub trait SettingsStore {
    fn get_json(&self, key: &str) -> Result<Option<String>, AppError>;
    fn set_json(&self, key: &str, value_json: &str) -> Result<(), AppError>;
}
