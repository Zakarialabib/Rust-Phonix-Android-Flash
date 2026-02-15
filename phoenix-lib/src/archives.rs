use crate::error::AppError;
use flate2::read::GzDecoder;
use std::fs::{self, File};
use std::io::{BufReader, BufWriter};
use std::path::Path;
use tar::Archive;
use tracing::info;
use zip::ZipArchive;

/// Extract various archive formats to a destination directory
/// Supports: .zip, .tar, .tar.gz, .tgz
pub fn extract_archive(archive_path: &Path, destination: &Path) -> Result<(), AppError> {
    if !archive_path.exists() {
        return Err(AppError::ValidationError(format!(
            "Archive not found: {:?}",
            archive_path
        )));
    }

    if !destination.exists() {
        fs::create_dir_all(destination).map_err(|e| AppError::IoError(e.to_string()))?;
    }

    let extension = archive_path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");
    let file = File::open(archive_path).map_err(|e| AppError::IoError(e.to_string()))?;
    let file = BufReader::new(file);

    match extension {
        "zip" => extract_zip(file, destination),
        "tar" => extract_tar(file, destination),
        "gz" | "tgz" => extract_targz(file, destination),
        _ => {
            // Check for complex extensions like .tar.gz handled by file extension splitting
            if archive_path.to_string_lossy().ends_with(".tar.gz") {
                extract_targz(file, destination)
            } else {
                Err(AppError::ValidationError(format!(
                    "Unsupported archive format: {}",
                    extension
                )))
            }
        }
    }
}

fn extract_zip(file: BufReader<File>, destination: &Path) -> Result<(), AppError> {
    let mut archive =
        ZipArchive::new(file).map_err(|e| AppError::IoError(format!("Invalid zip: {}", e)))?;

    for i in 0..archive.len() {
        let mut file = archive
            .by_index(i)
            .map_err(|e| AppError::IoError(e.to_string()))?;
        let outpath = match file.enclosed_name() {
            Some(path) => destination.join(path),
            None => continue,
        };

        if file.name().ends_with('/') {
            fs::create_dir_all(&outpath).map_err(|e| AppError::IoError(e.to_string()))?;
        } else {
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    fs::create_dir_all(p).map_err(|e| AppError::IoError(e.to_string()))?;
                }
            }
            let outfile = File::create(&outpath).map_err(|e| AppError::IoError(e.to_string()))?;
            let mut outfile = BufWriter::new(outfile);
            std::io::copy(&mut file, &mut outfile).map_err(|e| AppError::IoError(e.to_string()))?;
        }
    }

    info!("Extracted zip successfully to {:?}", destination);
    Ok(())
}

fn extract_tar(file: BufReader<File>, destination: &Path) -> Result<(), AppError> {
    let mut archive = Archive::new(file);
    archive
        .unpack(destination)
        .map_err(|e| AppError::IoError(format!("Failed to unpack tar: {}", e)))?;

    info!("Extracted tar successfully to {:?}", destination);
    Ok(())
}

fn extract_targz(file: BufReader<File>, destination: &Path) -> Result<(), AppError> {
    let tar = GzDecoder::new(file);
    let mut archive = Archive::new(tar);
    archive
        .unpack(destination)
        .map_err(|e| AppError::IoError(format!("Failed to unpack tar.gz: {}", e)))?;

    info!("Extracted tar.gz successfully to {:?}", destination);
    Ok(())
}
