use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Configuration load error: {0}")]
    ConfigLoadError(String),

    #[error("IO error: {0}")]
    IoError(String),

    #[error("Template error: {0}")]
    TemplateError(String),

    #[error("Invalid path: {0}")]
    InvalidPath(String),
}
