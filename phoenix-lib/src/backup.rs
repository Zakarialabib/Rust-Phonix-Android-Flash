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
    async fn validate_device_path(path: &str) -> Result<(), AppError> {
        let path_obj = Path::new(path);

        if !path_obj.exists() {
            return Err(AppError::DeviceNotFound(path.to_string()));
        }

        let metadata = tokio::fs::metadata(path_obj)
            .await
            .map_err(|e| AppError::IoError(e.to_string()))?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::FileTypeExt;
            let file_type = metadata.file_type();
            if !file_type.is_block_device() && !file_type.is_char_device() {
                return Err(AppError::ValidationError(format!(
                    "Path is not a valid device: {}. Only block and character devices are allowed.",
                    path
                )));
            }
        }

        #[cfg(not(unix))]
        {
            if metadata.is_file() {
                return Err(AppError::ValidationError(format!(
                    "Path is a regular file: {}. Regular files are not allowed for backup source.",
                    path
                )));
            }
        }

        Ok(())
    }

    pub async fn create_backup(
        device_path: &str,
        target: BackupTarget,
        output_path: &Path,
    ) -> Result<(), AppError> {
        let input_path = match &target {
            BackupTarget::Full => device_path,
            BackupTarget::Partition(p) => p.as_str(),
            BackupTarget::Bootloader => device_path,
        };

        Self::validate_device_path(input_path).await?;

        let input_arg = format!("if={}", input_path);

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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_create_backup_rejects_regular_file() {
        let dir = tempdir().unwrap();
        let regular_file = dir.path().join("regular_file");
        File::create(&regular_file).unwrap();
        let output = dir.path().join("output.img");

        let result = BackupManager::create_backup(
            regular_file.to_str().unwrap(),
            BackupTarget::Full,
            &output,
        )
        .await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        match err {
            AppError::ValidationError(msg) => {
                assert!(
                    msg.contains("Only block and character devices are allowed")
                        || msg.contains("Regular files are not allowed")
                );
            }
            _ => panic!("Expected ValidationError, got {:?}", err),
        }
    }

    #[tokio::test]
    async fn test_create_backup_rejects_non_existent_path() {
        let dir = tempdir().unwrap();
        let non_existent = dir.path().join("non_existent");
        let output = dir.path().join("output.img");

        let result = BackupManager::create_backup(
            non_existent.to_str().unwrap(),
            BackupTarget::Full,
            &output,
        )
        .await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        match err {
            AppError::DeviceNotFound(_) => {}
            _ => panic!("Expected DeviceNotFound, got {:?}", err),
        }
    }
}
