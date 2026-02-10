use std::path::Path;
use std::process::Command;
use anyhow::Result;
use crate::error::AppError;

pub fn preflight(image_path: &Path, target_device: &str) -> Result<(), AppError> {
    if target_device.trim().is_empty() {
        return Err(AppError::ValidationError("Target device cannot be empty".to_string()));
    }

    if !image_path.exists() {
        return Err(AppError::ValidationError(format!("Image file not found: {:?}", image_path)));
    }

    let metadata = std::fs::metadata(image_path).map_err(|e| AppError::IoError(e.to_string()))?;
    if metadata.len() == 0 {
        return Err(AppError::ValidationError("Image file is empty".to_string()));
    }

    which::which("dd").map_err(|_| AppError::CommandFailed("Required tool 'dd' not found".to_string()))?;

    Ok(())
}

/// Flash an image to a target device using dd
pub fn flash_image(image_path: &Path, target_device: &str) -> Result<(), AppError> {
    preflight(image_path, target_device)?;

    // Construct dd command
    // Note: status=progress is a GNU dd extension, might not work on all BSD/Mac variants
    // bs=4M is a reasonable default
    let status = Command::new("dd")
        .arg(format!("if={}", image_path.to_string_lossy()))
        .arg(format!("of={}", target_device))
        .arg("bs=4M")
        .arg("status=progress")
        .arg("conv=fsync")
        .status()
        .map_err(|e| AppError::CommandFailed(format!("Failed to execute dd: {}", e)))?;

    if !status.success() {
        return Err(AppError::CommandFailed(format!("dd command failed with exit code: {:?}", status.code())));
    }

    Ok(())
}
