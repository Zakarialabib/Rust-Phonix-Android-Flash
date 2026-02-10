use crate::error::AppError;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::Path;
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt};
use tokio::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BackupTarget {
    Full,
    Partition(String),
    Bootloader,
}

pub struct BackupManager;

impl BackupManager {
    pub async fn create_backup(device_path: &str, target: BackupTarget, output_path: &Path) -> Result<(), AppError> {
        let input_arg = match target {
            BackupTarget::Full => format!("if={}", device_path),
            BackupTarget::Partition(p) => format!("if={}", p),
            BackupTarget::Bootloader => format!("if={}", device_path),
        };

        let status = Command::new("dd")
            .arg(&input_arg)
            .arg(format!("of={}", output_path.display()))
            .arg("bs=4M")
            .arg("status=progress")
            .status()
            .await
            .map_err(|e| AppError::IoError(e.to_string()))?;

        if !status.success() {
            return Err(AppError::CommandFailed(format!("Backup failed for {}", device_path)));
        }

        Ok(())
    }

    pub async fn verify_backup(image_path: &Path) -> Result<String, AppError> {
        let mut file = File::open(image_path)
            .await
            .map_err(|e| AppError::IoError(e.to_string()))?;
        let mut hasher = Sha256::new();
        let mut buffer = vec![0u8; 1024 * 1024];

        loop {
            let read = file
                .read(&mut buffer)
                .await
                .map_err(|e| AppError::IoError(e.to_string()))?;
            if read == 0 {
                break;
            }
            hasher.update(&buffer[..read]);
        }

        Ok(format!("{:x}", hasher.finalize()))
    }

    pub async fn extract_from_image(
        image_path: &Path,
        output_path: &Path,
        offset: u64,
        size: u64,
    ) -> Result<(), AppError> {
        let mut input = File::open(image_path)
            .await
            .map_err(|e| AppError::IoError(e.to_string()))?;
        input
            .seek(std::io::SeekFrom::Start(offset))
            .await
            .map_err(|e| AppError::IoError(e.to_string()))?;

        let mut output = File::create(output_path)
            .await
            .map_err(|e| AppError::IoError(e.to_string()))?;

        let mut remaining = size;
        let mut buffer = vec![0u8; 1024 * 1024];

        while remaining > 0 {
            let chunk_size = std::cmp::min(remaining as usize, buffer.len());
            let read = input
                .read(&mut buffer[..chunk_size])
                .await
                .map_err(|e| AppError::IoError(e.to_string()))?;
            if read == 0 {
                break;
            }
            output
                .write_all(&buffer[..read])
                .await
                .map_err(|e| AppError::IoError(e.to_string()))?;
            remaining = remaining.saturating_sub(read as u64);
        }

        if remaining > 0 {
            return Err(AppError::ValidationError(
                "Image is smaller than requested extraction size".to_string(),
            ));
        }

        Ok(())
    }
}
