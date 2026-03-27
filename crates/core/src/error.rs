use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("IO error: {0}")]
    IoError(String),

    #[error("TOML parse error: {0}")]
    TomlParse(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("File not found: {0:?}")]
    FileNotFound(std::path::PathBuf),

    #[error("Directory not found: {0:?}")]
    DirectoryNotFound(std::path::PathBuf),

    #[error("Invalid path: {0}")]
    InvalidPath(String),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Provider not found: {0}")]
    ProviderNotFound(String),

    #[error("Model not found: {0}")]
    ModelNotFound(String),
}
