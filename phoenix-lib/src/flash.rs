use std::path::Path;
use std::process::Command;
use tokio::fs::OpenOptions;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use crate::error::AppError;

fn validate_target_device(device: &str) -> Result<(), AppError> {
    if device.trim().is_empty() {
        return Err(AppError::ValidationError("Target device cannot be empty".to_string()));
    }

    if cfg!(target_os = "windows") {
        if !device.starts_with(r"\\.\PhysicalDrive") {
            return Err(AppError::ValidationError(
                "Target device must be a physical drive (e.g., \\\\.\\PhysicalDrive1)".to_string(),
            ));
        }
    } else if cfg!(target_os = "linux") {
        if !device.starts_with("/dev/") {
            return Err(AppError::ValidationError(
                "Target device must be a block device path (e.g., /dev/sdX)".to_string(),
            ));
        }
        // Basic protection for system drives
        if device == "/dev/sda" || device == "/dev/nvme0n1" {
             return Err(AppError::ValidationError(
                "Operation on primary system drive /dev/sda or /dev/nvme0n1 is restricted.".to_string(),
            ));
        }
    }
    Ok(())
}

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
    validate_target_device(target_device)?;

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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_preflight_validation() {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(b"test data").unwrap();
        let image_path = file.path();

        // Empty string should fail
        let result = preflight(image_path, "");
        assert!(matches!(result, Err(AppError::ValidationError(_))));

        // On Linux, non-/dev path should fail
        if cfg!(target_os = "linux") {
            let result = preflight(image_path, "/tmp/not_a_device");
            // This assertion currently FAILS because preflight doesn't check
            // After fix, it should PASS
            assert!(matches!(result, Err(AppError::ValidationError(_))), "Expected ValidationError for non-/dev path on Linux");
        }
    }
}

/// Flash an image to a target device asynchronously with progress reporting
pub async fn flash_image_async(
    image_path: &Path,
    target_device: &str,
    progress: Option<ProgressCallback>,
) -> Result<(), AppError> {
    let image_path_buf = image_path.to_path_buf();
    let target_device_owned = target_device.to_string();
    
    tokio::task::spawn_blocking(move || preflight(&image_path_buf, &target_device_owned))
        .await
        .map_err(|e| AppError::Unknown(format!("Preflight join error: {}", e)))??;

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

    // Create channels for pipelined reading/writing with buffer recycling
    // Capacity of 2 allows for double buffering (one being read, one being written)
    let (tx, mut rx) = tokio::sync::mpsc::channel::<Result<Vec<u8>, AppError>>(2);
    let (recycle_tx, mut recycle_rx) = tokio::sync::mpsc::channel::<Vec<u8>>(2);

    // Pre-allocate 2 buffers and send them to the recycle channel
    for _ in 0..2 {
        recycle_tx.send(vec![0u8; 4 * 1024 * 1024]).await.ok();
    }

    // Spawn a reader task
    tokio::spawn(async move {
        loop {
            // Get a buffer from the recycle channel
            let mut buffer = match recycle_rx.recv().await {
                Some(b) => b,
                None => break, // Writer closed or finished
            };

            match f_in.read(&mut buffer).await {
                Ok(0) => break, // EOF
                Ok(n) => {
                    // Truncate to actual data size
                    buffer.truncate(n);
                    // Send the data chunk
                    if tx.send(Ok(buffer)).await.is_err() {
                        break; // Receiver dropped
                    }
                }
                Err(e) => {
                    let _ = tx.send(Err(AppError::IoError(format!("Read error: {}", e)))).await;
                    break;
                }
            }
        }
    });

    let mut bytes_transferred = 0u64;
    let start_time = std::time::Instant::now();

    while let Some(result) = rx.recv().await {
        let mut chunk = result?;

        f_out.write_all(&chunk).await
            .map_err(|e| AppError::IoError(format!("Write error: {}", e)))?;

        bytes_transferred += chunk.len() as u64;

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

        // Recycle the buffer
        // Resize back to 4MB capacity for the next read
        chunk.resize(4 * 1024 * 1024, 0);
        let _ = recycle_tx.send(chunk).await;
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
