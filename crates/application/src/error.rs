use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("{0}")]
    Validation(String),
    #[error("{0}")]
    NotFound(String),
    #[error("{0}")]
    Storage(String),
    #[error("{0}")]
    Internal(String),
}

impl AppError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::Validation(_) => "validation",
            Self::NotFound(_) => "not_found",
            Self::Storage(_) => "storage",
            Self::Internal(_) => "internal",
        }
    }

    pub fn retryable(&self) -> bool {
        matches!(self, Self::Storage(_))
    }

    pub fn suggested_action(&self) -> &'static str {
        match self {
            Self::Validation(_) => "Corrige el valor e inténtalo de nuevo.",
            Self::NotFound(_) => "Comprueba la clave o ruta solicitada.",
            Self::Storage(_) => "Verifica permisos y espacio en disco.",
            Self::Internal(_) => "Revisa los logs locales de la aplicación.",
        }
    }
}
