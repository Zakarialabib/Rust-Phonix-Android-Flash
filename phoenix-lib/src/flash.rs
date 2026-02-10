use std::path::Path;
use std::process::Command;
use tokio::fs::OpenOptions;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use crate::error::AppError;

/// Flash progress information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FlashProgress {
    /// Current operation
    pub operation: String,
    /// Partition being flashed
    pub partition: Option<String>,
    /// Progress percentage (0-100)
    pub percent: u8,
    /// Bytes transferred
    pub bytes_transferred: u64,
    /// Total bytes
    pub total_bytes: u64,
    /// Current speed in bytes/sec
    pub speed_bps: u64,
}

/// Flash progress callback type
pub type ProgressCallback = Box<dyn Fn(FlashProgress) + Send>;

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

/// Flash an image to a target device asynchronously with progress reporting
pub async fn flash_image_async(
    image_path: &Path,
    target_device: &str,
    progress: Option<ProgressCallback>,
) -> Result<(), AppError> {
    preflight(image_path, target_device)?;

    let mut f_in = tokio::fs::File::open(image_path)
        .await
        .map_err(|e| AppError::IoError(format!("Failed to open image: {}", e)))?;

    let metadata = f_in.metadata().await
        .map_err(|e| AppError::IoError(format!("Failed to get image metadata: {}", e)))?;
    let total_bytes = metadata.len();

    let mut f_out = OpenOptions::new()
        .write(true)
        .open(target_device)
        .await
        .map_err(|e| AppError::IoError(format!("Failed to open target device: {}", e)))?;

    let mut buffer = vec![0u8; 4 * 1024 * 1024]; // 4MB buffer
    let mut bytes_transferred = 0u64;
    let start_time = std::time::Instant::now();

    loop {
        let n = f_in.read(&mut buffer).await
            .map_err(|e| AppError::IoError(format!("Read error: {}", e)))?;

        if n == 0 {
            break;
        }

        f_out.write_all(&buffer[..n]).await
            .map_err(|e| AppError::IoError(format!("Write error: {}", e)))?;

        bytes_transferred += n as u64;

        if let Some(ref cb) = progress {
            let elapsed = start_time.elapsed().as_secs_f64();
            let speed_bps = if elapsed > 0.0 {
                (bytes_transferred as f64 / elapsed) as u64
            } else {
                0
            };

            cb(FlashProgress {
                operation: "Writing".to_string(),
                partition: None,
                percent: if total_bytes > 0 { ((bytes_transferred * 100) / total_bytes) as u8 } else { 0 },
                bytes_transferred,
                total_bytes,
                speed_bps,
            });
        }
    }

    f_out.sync_all().await
        .map_err(|e| AppError::IoError(format!("Sync error: {}", e)))?;

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
