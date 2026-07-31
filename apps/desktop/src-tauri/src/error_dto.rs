use serde::Serialize;
use visual_library_application::AppError;

#[derive(Debug, Serialize)]
pub struct CommandError {
    pub code: String,
    pub message: String,
    pub retryable: bool,
    pub suggested_action: String,
    pub detail: Option<String>,
}

impl From<AppError> for CommandError {
    fn from(value: AppError) -> Self {
        Self {
            code: value.code().to_string(),
            message: value.to_string(),
            retryable: value.retryable(),
            suggested_action: value.suggested_action().to_string(),
            detail: None,
        }
    }
}
