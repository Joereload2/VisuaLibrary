use thiserror::Error;
use visual_library_application::AppError;

#[derive(Debug, Error)]
pub enum InfraError {
    #[error("sqlite: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("{0}")]
    Message(String),
}

impl From<InfraError> for AppError {
    fn from(value: InfraError) -> Self {
        match value {
            InfraError::Sqlite(e) => AppError::Storage(e.to_string()),
            InfraError::Io(e) => AppError::Storage(e.to_string()),
            InfraError::Message(m) => AppError::Storage(m),
        }
    }
}
