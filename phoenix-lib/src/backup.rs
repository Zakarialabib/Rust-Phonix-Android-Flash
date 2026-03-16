use crate::error::AppError;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::Path;
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BackupTarget {
    Full,
    Partition(String),
    Bootloader,
}

pub struct BackupManager;

impl BackupManager {
    async fn validate_device_file(file: &File, path: &str) -> Result<(), AppError> {
        let metadata = file
            .metadata()
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

        // Open file first to get handle
        let mut input_file = File::open(input_path)
            .await
            .map_err(|_| AppError::DeviceNotFound(input_path.to_string()))?;

        // Validate on the open file handle (prevents TOCTOU)
        Self::validate_device_file(&input_file, input_path).await?;

        let mut output_file = File::create(output_path)
            .await
            .map_err(|e| AppError::IoError(e.to_string()))?;

        // ⚡ Bolt: Pipeline I/O reads and writes using double buffering.
        // This producer-consumer pattern uses channels to prevent leaving
        // the storage bus idle during sequential await calls.
        let (tx, mut rx) = tokio::sync::mpsc::channel::<Result<(Vec<u8>, usize), AppError>>(2);
        let (recycle_tx, mut recycle_rx) = tokio::sync::mpsc::channel::<Vec<u8>>(2);

        // Pre-allocate 2 buffers and send them to the recycle channel
        for _ in 0..2 {
            recycle_tx.send(vec![0u8; 4 * 1024 * 1024]).await.ok();
        }

        tokio::spawn(async move {
            loop {
                let mut buffer = match recycle_rx.recv().await {
                    Some(b) => b,
                    None => break, // Writer closed or finished
                };

                match input_file.read(&mut buffer).await {
                    Ok(0) => break, // EOF
                    Ok(n) => {
                        if tx.send(Ok((buffer, n))).await.is_err() {
                            break; // Receiver dropped
                        }
                    }
                    Err(e) => {
                        let _ = tx
                            .send(Err(AppError::IoError(format!("Read failed: {}", e))))
                            .await;
                        break;
                    }
                }
            }
        });

        while let Some(result) = rx.recv().await {
            let (chunk, n) = result?;

            output_file
                .write_all(&chunk[..n])
                .await
                .map_err(|e| AppError::IoError(format!("Write failed: {}", e)))?;

            // Recycle the buffer
            let _ = recycle_tx.send(chunk).await;
        }

        output_file
            .flush()
            .await
            .map_err(|e| AppError::IoError(e.to_string()))?;

        Ok(())
    }

    pub async fn verify_backup(image_path: &Path) -> Result<String, AppError> {
        let image_path = image_path.to_path_buf();
        tokio::task::spawn_blocking(move || {
            let mut file =
                std::fs::File::open(&image_path).map_err(|e| AppError::IoError(e.to_string()))?;
            let mut hasher = Sha256::new();
            let mut buffer = vec![0u8; 1024 * 1024];

            loop {
                use std::io::Read;
                let read = file
                    .read(&mut buffer)
                    .map_err(|e| AppError::IoError(e.to_string()))?;
                if read == 0 {
                    break;
                }
                hasher.update(&buffer[..read]);
            }

            Ok(format!("{:x}", hasher.finalize()))
        })
        .await
        .map_err(|e| AppError::Unknown(format!("Task join error: {}", e)))?
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
    async fn test_extract_from_image() {
        let dir = tempdir().unwrap();
        let image_path = dir.path().join("source.img");
        let output_path = dir.path().join("extracted.img");

        // Create a dummy file with 16 bytes: 0, 1, 2, ..., 15
        let data: Vec<u8> = (0..16).collect();
        tokio::fs::write(&image_path, &data)
            .await
            .expect("Failed to create test image");

        // Test extraction from middle: offset 4, size 8 (should be 4, 5, 6, 7, 8, 9, 10, 11)
        BackupManager::extract_from_image(&image_path, &output_path, 4, 8)
            .await
            .expect("Extraction failed");

        let extracted = tokio::fs::read(&output_path)
            .await
            .expect("Failed to read extracted file");
        assert_eq!(extracted, vec![4, 5, 6, 7, 8, 9, 10, 11]);

        // Test extraction from start: offset 0, size 4 (should be 0, 1, 2, 3)
        BackupManager::extract_from_image(&image_path, &output_path, 0, 4)
            .await
            .expect("Extraction failed");
        let extracted = tokio::fs::read(&output_path)
            .await
            .expect("Failed to read extracted file");
        assert_eq!(extracted, vec![0, 1, 2, 3]);

        // Test extraction exceeding file size
        let result = BackupManager::extract_from_image(&image_path, &output_path, 10, 10).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            AppError::ValidationError(msg) => {
                assert!(msg.contains("smaller than requested extraction size"));
            }
            err => panic!("Expected ValidationError, got {:?}", err),
        }
    }

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

#[cfg(test)]
mod performance_tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;
    use std::time::Instant;
    use tempfile::tempdir;
    use tokio::time::{sleep, Duration};

    #[tokio::test]
    async fn test_verify_backup_performance() {
        // Setup: Create a large temporary file (e.g., 50MB)
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("large_file.img");
        let size_mb = 50;
        {
            let mut file = File::create(&file_path).unwrap();
            let data = vec![0u8; 1024 * 1024]; // 1MB chunk
            for _ in 0..size_mb {
                file.write_all(&data).unwrap();
            }
        }

        // Measure: Spawn a background task to count ticks (simulating other work)
        let ticks = Arc::new(AtomicUsize::new(0));
        let ticks_clone = ticks.clone();

        let handle = tokio::spawn(async move {
            loop {
                sleep(Duration::from_millis(1)).await;
                ticks_clone.fetch_add(1, Ordering::Relaxed);
            }
        });

        println!("Starting verification of {}MB file...", size_mb);
        let start = Instant::now();

        // Execute the function under test
        let result = BackupManager::verify_backup(&file_path).await;

        let duration = start.elapsed();
        handle.abort(); // Stop the counter

        assert!(result.is_ok());
        println!("Verification took: {:?}", duration);
        println!("Background ticks: {}", ticks.load(Ordering::Relaxed));
    }
}
