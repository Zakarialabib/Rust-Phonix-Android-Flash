use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug, Serialize, Deserialize)]
#[serde(tag = "code", content = "message", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AppError {
    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Hardware error: {0}")]
    HardwareError(String),

    #[error("IO error: {0}")]
    IoError(String),

    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Command failed: {0}")]
    CommandFailed(String),

    #[error("Device not found: {0}")]
    DeviceNotFound(String),

    #[error("Resource not found: {0}")]
    NotFound(String),

    #[error("Build failed: {0}")]
    BuildFailed(String),

    #[error("Settings load failed: {0}")]
    SettingsLoadFailed(String),

    #[error("Settings save failed: {0}")]
    SettingsSaveFailed(String),

    #[error("Asset base URL missing: {0}")]
    AssetBaseUrlMissing(String),

    #[error("Not implemented: {0}")]
    NotImplemented(String),

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Unknown error: {0}")]
    Unknown(String),
}

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        AppError::IoError(err.to_string())
    }
}

impl From<anyhow::Error> for AppError {
    fn from(err: anyhow::Error) -> Self {
        AppError::Unknown(err.to_string())
    }
}
