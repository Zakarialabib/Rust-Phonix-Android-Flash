use crate::error::AppError;
use std::path::Path;
use walkdir::WalkDir;

pub struct Extractor;

impl Extractor {
    pub async fn extract_wifi_firmware(
        mount_point: &Path,
        output_dir: &Path,
    ) -> Result<u32, AppError> {
        let firmware_paths = ["system/etc/wifi", "vendor/etc/wifi", "lib/firmware"];
        let mut copied = 0u32;

        for subpath in firmware_paths {
            let source = mount_point.join(subpath);
            if source.exists() {
                let dest = output_dir.join(subpath);
                copied += copy_dir_recursive(&source, &dest).await?;
            }
        }

        Ok(copied)
    }

    pub async fn extract_dtb_from_mount(
        mount_point: &Path,
        output_dir: &Path,
    ) -> Result<u32, AppError> {
        copy_matching_files(mount_point, output_dir, |path| {
            path.extension()
                .and_then(|e| e.to_str())
                .map(|e| e.eq_ignore_ascii_case("dtb"))
                .unwrap_or(false)
        })
        .await
    }

    pub async fn extract_ddr_timings(
        mount_point: &Path,
        output_dir: &Path,
    ) -> Result<u32, AppError> {
        copy_matching_files(mount_point, output_dir, |path| {
            path.file_name()
                .and_then(|n| n.to_str())
                .map(|name| name.to_lowercase().contains("ddr"))
                .unwrap_or(false)
        })
        .await
    }

    pub async fn extract_kernel_config(
        mount_point: &Path,
        output_dir: &Path,
    ) -> Result<u32, AppError> {
        copy_matching_files(mount_point, output_dir, |path| {
            path.file_name()
                .and_then(|n| n.to_str())
                .map(|name| name == "config.gz" || name.starts_with("config-"))
                .unwrap_or(false)
        })
        .await
    }
}

async fn copy_matching_files<F>(
    mount_point: &Path,
    output_dir: &Path,
    matcher: F,
) -> Result<u32, AppError>
where
    F: Fn(&Path) -> bool + Send + 'static,
{
    let mount_point = mount_point.to_path_buf();
    let output_dir = output_dir.to_path_buf();

    tokio::task::spawn_blocking(move || {
        let mut copied = 0u32;
        for entry in WalkDir::new(&mount_point).into_iter() {
            let entry = entry.map_err(|e| AppError::IoError(e.to_string()))?;
            if !entry.file_type().is_file() {
                continue;
            }
            let path = entry.path();
            if !matcher(path) {
                continue;
            }
            let relative = path
                .strip_prefix(&mount_point)
                .map_err(|e| AppError::IoError(e.to_string()))?;
            let destination = output_dir.join(relative);
            if let Some(parent) = destination.parent() {
                std::fs::create_dir_all(parent).map_err(|e| AppError::IoError(e.to_string()))?;
            }
            std::fs::copy(path, &destination).map_err(|e| AppError::IoError(e.to_string()))?;
            copied += 1;
        }
        Ok(copied)
    })
    .await
    .map_err(|e| AppError::Unknown(format!("Task join error: {}", e)))?
}

async fn copy_dir_recursive(source: &Path, destination: &Path) -> Result<u32, AppError> {
    let source = source.to_path_buf();
    let destination = destination.to_path_buf();

    tokio::task::spawn_blocking(move || {
        let mut copied = 0u32;
        for entry in WalkDir::new(&source).into_iter() {
            let entry = entry.map_err(|e| AppError::IoError(e.to_string()))?;
            let path = entry.path();
            let relative = path
                .strip_prefix(&source)
                .map_err(|e| AppError::IoError(e.to_string()))?;
            let target = destination.join(relative);

            if entry.file_type().is_dir() {
                std::fs::create_dir_all(&target).map_err(|e| AppError::IoError(e.to_string()))?;
            } else if entry.file_type().is_file() {
                if let Some(parent) = target.parent() {
                    std::fs::create_dir_all(parent)
                        .map_err(|e| AppError::IoError(e.to_string()))?;
                }
                std::fs::copy(path, &target).map_err(|e| AppError::IoError(e.to_string()))?;
                copied += 1;
            }
        }
        Ok(copied)
    })
    .await
    .map_err(|e| AppError::Unknown(format!("Task join error: {}", e)))?
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use std::time::Instant;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_copy_matching_files_perf() {
        let dir = tempdir().unwrap();
        let src = dir.path().join("src");
        let dest = dir.path().join("dest");
        std::fs::create_dir(&src).unwrap();

        // Create 1000 files in nested directories
        for i in 0..10 {
            let subdir = src.join(format!("dir_{}", i));
            std::fs::create_dir(&subdir).unwrap();
            for j in 0..100 {
                let p = subdir.join(format!("file_{}.txt", j));
                let mut f = File::create(p).unwrap();
                f.write_all(b"content").unwrap();
            }
        }

        let start = Instant::now();
        let count = copy_matching_files(&src, &dest, |_| true).await.unwrap();
        let duration = start.elapsed();

        println!("Copied {} files in {:?}", count, duration);
        assert_eq!(count, 1000);

        // Verify destination exists and has files
        assert!(dest.exists());
        assert!(dest.join("dir_0/file_0.txt").exists());
    }
}
