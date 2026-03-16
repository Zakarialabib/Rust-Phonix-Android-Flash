use crate::error::AppError;
use anyhow::Result;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::process::Command;
use tokio::fs::OpenOptions;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

fn validate_target_device(device: &str) -> Result<(), AppError> {
    if device.trim().is_empty() {
        return Err(AppError::ValidationError(
            "Target device cannot be empty".to_string(),
        ));
    }

    #[cfg(windows)]
    {
        if !device.starts_with(r"\\.\PhysicalDrive") {
            return Err(AppError::ValidationError(
                "Target device must be a physical drive (e.g., \\\\.\\PhysicalDrive1)".to_string(),
            ));
        }
    }

    #[cfg(unix)]
    {
        // Resolve path to handle symlinks and relative paths (e.g., /dev/../tmp/foo)
        let path = std::fs::canonicalize(device)
            .map_err(|e| AppError::DeviceNotFound(format!("Device not found: {}", e)))?;
        let path_str = path.to_string_lossy();

        if !path_str.starts_with("/dev/") {
            return Err(AppError::ValidationError(
                "Target device must be a block device path in /dev/".to_string(),
            ));
        }

        // Linux-specific system drive protection
        #[cfg(target_os = "linux")]
        {
            if is_system_device(&path_str)? {
                return Err(AppError::ValidationError(
                    "Operation on primary system drive and its partitions is restricted."
                        .to_string(),
                ));
            }
        }
    }

    Ok(())
}

fn is_system_device(path: &str) -> Result<bool, AppError> {
    // We use a regex to identify common primary system drives and their partitions
    // to prevent accidental overwriting of the host OS on Linux.
    // - sda: Primary SATA/USB drive (blocks sda, sda1... but NOT sdb or sdaa)
    // - vda: Primary VirtIO disk (common in VMs)
    // - nvmeXnY: All NVMe namespaces and partitions (e.g., nvme0n1, nvme0n1p1)
    // - mmcblkX: All EMMC/SD devices and partitions (e.g., mmcblk0, mmcblk0p1)
    static SYSTEM_DEVICE_REGEX: std::sync::LazyLock<Result<Regex, regex::Error>> =
        std::sync::LazyLock::new(|| {
            Regex::new(
                r"^/dev/(sda[0-9]*|vda[0-9]*|nvme[0-9]+n[0-9]+(p[0-9]+)?|mmcblk[0-9]+(p[0-9]+)?)$",
            )
        });
    SYSTEM_DEVICE_REGEX
        .as_ref()
        .map(|re| re.is_match(path))
        .map_err(|e| AppError::ParseError(format!("Invalid system device regex: {}", e)))
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
        return Err(AppError::ValidationError(format!(
            "Image file not found: {:?}",
            image_path
        )));
    }

    let metadata = std::fs::metadata(image_path).map_err(|e| AppError::IoError(e.to_string()))?;
    if metadata.len() == 0 {
        return Err(AppError::ValidationError("Image file is empty".to_string()));
    }

    which::which("dd")
        .map_err(|_| AppError::CommandFailed("Required tool 'dd' not found".to_string()))?;

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
            // Create a dummy file to act as the "device" so canonicalize succeeds
            let device_file = NamedTempFile::new().unwrap();
            let device_path = device_file.path().to_str().unwrap();

            // This path is in /tmp (usually), so it should fail validation because it's not in /dev/
            let result = preflight(image_path, device_path);
            assert!(
                matches!(result, Err(AppError::ValidationError(ref msg)) if msg.contains("block device path")),
                "Expected ValidationError for non-/dev path on Linux, got {:?}",
                result
            );

            // Test non-existent device
            let result = preflight(image_path, "/tmp/non_existent_device_12345");
            assert!(
                matches!(result, Err(AppError::DeviceNotFound(_))),
                "Expected DeviceNotFound for non-existent device"
            );
        }
    }

    #[test]
    fn test_is_system_device() {
        // System drives and partitions should be detected
        assert!(is_system_device("/dev/sda").unwrap());
        assert!(is_system_device("/dev/sda1").unwrap());
        assert!(is_system_device("/dev/sda15").unwrap());

        assert!(is_system_device("/dev/vda").unwrap());
        assert!(is_system_device("/dev/vda2").unwrap());

        assert!(is_system_device("/dev/nvme0n1").unwrap());
        // NVMe partitions usually follow p<digits> pattern
        assert!(is_system_device("/dev/nvme0n1p1").unwrap());
        assert!(is_system_device("/dev/nvme0n1p12").unwrap());
        assert!(is_system_device("/dev/nvme1n1").unwrap()); // Multiple NVMe slots

        assert!(is_system_device("/dev/mmcblk0").unwrap());
        assert!(is_system_device("/dev/mmcblk0p2").unwrap());
        assert!(is_system_device("/dev/mmcblk1").unwrap());

        // Other devices should be safe
        assert!(!is_system_device("/dev/sdb").unwrap());
        assert!(!is_system_device("/dev/sdb1").unwrap());
        assert!(!is_system_device("/dev/sdc").unwrap());

        // Similar prefixes but different devices
        assert!(!is_system_device("/dev/sdaa").unwrap()); // 27th disk
        assert!(!is_system_device("/dev/sdaa1").unwrap());
        assert!(!is_system_device("/dev/vdab").unwrap());
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

    let metadata = f_in
        .metadata()
        .await
        .map_err(|e| AppError::IoError(format!("Failed to get image metadata: {}", e)))?;
    let total_bytes = metadata.len();

    let mut f_out = OpenOptions::new()
        .write(true)
        .open(target_device)
        .await
        .map_err(|e| AppError::IoError(format!("Failed to open target device: {}", e)))?;

    // Create channels for pipelined reading/writing with buffer recycling
    // Capacity of 2 allows for double buffering (one being read, one being written)
    let (tx, mut rx) = tokio::sync::mpsc::channel::<Result<(Vec<u8>, usize), AppError>>(2);
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
                    // Send the buffer with valid length, avoid truncating
                    if tx.send(Ok((buffer, n))).await.is_err() {
                        break; // Receiver dropped
                    }
                }
                Err(e) => {
                    let _ = tx
                        .send(Err(AppError::IoError(format!("Read error: {}", e))))
                        .await;
                    break;
                }
            }
        }
    });

    let mut bytes_transferred = 0u64;
    let start_time = std::time::Instant::now();

    while let Some(result) = rx.recv().await {
        let (chunk, n) = result?;

        f_out
            .write_all(&chunk[..n])
            .await
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
                percent: if total_bytes > 0 {
                    ((bytes_transferred * 100) / total_bytes) as u8
                } else {
                    0
                },
                bytes_transferred,
                total_bytes,
                speed_bps,
            });
        }

        // Recycle the buffer
        // It wasn't truncated, so it stays at full capacity/length without zeroing
        let _ = recycle_tx.send(chunk).await;
    }

    f_out
        .sync_all()
        .await
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
        return Err(AppError::CommandFailed(format!(
            "dd command failed with exit code: {:?}",
            status.code()
        )));
    }

    Ok(())
}
