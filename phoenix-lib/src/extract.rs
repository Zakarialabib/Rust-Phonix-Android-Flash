use crate::error::AppError;
use std::path::Path;
use tokio::fs;
use walkdir::WalkDir;

pub struct Extractor;

impl Extractor {
    pub async fn extract_wifi_firmware(mount_point: &Path, output_dir: &Path) -> Result<u32, AppError> {
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

    pub async fn extract_dtb_from_mount(mount_point: &Path, output_dir: &Path) -> Result<u32, AppError> {
        copy_matching_files(mount_point, output_dir, |path| {
            path.extension().and_then(|e| e.to_str()).map(|e| e.eq_ignore_ascii_case("dtb")).unwrap_or(false)
        })
        .await
    }

    pub async fn extract_ddr_timings(mount_point: &Path, output_dir: &Path) -> Result<u32, AppError> {
        copy_matching_files(mount_point, output_dir, |path| {
            path.file_name()
                .and_then(|n| n.to_str())
                .map(|name| name.to_lowercase().contains("ddr"))
                .unwrap_or(false)
        })
        .await
    }

    pub async fn extract_kernel_config(mount_point: &Path, output_dir: &Path) -> Result<u32, AppError> {
        copy_matching_files(mount_point, output_dir, |path| {
            path.file_name()
                .and_then(|n| n.to_str())
                .map(|name| name == "config.gz" || name.starts_with("config-"))
                .unwrap_or(false)
        })
        .await
    }
}

async fn copy_matching_files<F>(mount_point: &Path, output_dir: &Path, matcher: F) -> Result<u32, AppError>
where
    F: Fn(&Path) -> bool,
{
    let mut copied = 0u32;
    for entry in WalkDir::new(mount_point).into_iter() {
        let entry = entry.map_err(|e| AppError::IoError(e.to_string()))?;
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path();
        if !matcher(path) {
            continue;
        }
        let relative = path
            .strip_prefix(mount_point)
            .map_err(|e| AppError::IoError(e.to_string()))?;
        let destination = output_dir.join(relative);
        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent)
                .await
                .map_err(|e| AppError::IoError(e.to_string()))?;
        }
        fs::copy(path, &destination)
            .await
            .map_err(|e| AppError::IoError(e.to_string()))?;
        copied += 1;
    }

    Ok(copied)
}

async fn copy_dir_recursive(source: &Path, destination: &Path) -> Result<u32, AppError> {
    let mut copied = 0u32;
    for entry in WalkDir::new(source).into_iter() {
        let entry = entry.map_err(|e| AppError::IoError(e.to_string()))?;
        let path = entry.path();
        let relative = path
            .strip_prefix(source)
            .map_err(|e| AppError::IoError(e.to_string()))?;
        let target = destination.join(relative);

        if entry.file_type().is_dir() {
            fs::create_dir_all(&target)
                .await
                .map_err(|e| AppError::IoError(e.to_string()))?;
        } else if entry.file_type().is_file() {
            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent)
                    .await
                    .map_err(|e| AppError::IoError(e.to_string()))?;
            }
            fs::copy(path, &target)
                .await
                .map_err(|e| AppError::IoError(e.to_string()))?;
            copied += 1;
        }
    }

    Ok(copied)
}
