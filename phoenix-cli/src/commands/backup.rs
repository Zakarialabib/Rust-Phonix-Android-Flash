use anyhow::Result;
use colored::Colorize;
use phoenix_lib::backup::{BackupManager, BackupTarget};
use std::path::Path;

pub async fn dump(device: &str, output: &str, partition: Option<&str>) -> Result<()> {
    println!("{}", "Phoenix Backup Tool".bold().blue());
    println!("Device: {}", device);
    println!("Output: {}", output);

    let target = match partition {
        Some(part) => BackupTarget::Partition(part.to_string()),
        None => BackupTarget::Full,
    };

    BackupManager::create_backup(device, target, Path::new(output))
        .await
        .map_err(|e| anyhow::anyhow!(e.to_string()))?;

    println!("{}", "Backup completed successfully!".green());

    Ok(())
}

pub async fn extract(firmware: &str, output: &str, offset: u64, size: u64) -> Result<()> {
    println!("{}", "Phoenix Backup Extract".bold().blue());
    println!("Firmware: {}", firmware);
    println!("Output: {}", output);
    println!("Offset: {}", offset);
    println!("Size: {}", size);

    BackupManager::extract_from_image(Path::new(firmware), Path::new(output), offset, size)
        .await
        .map_err(|e| anyhow::anyhow!(e.to_string()))?;

    println!("{}", "Extraction completed successfully!".green());

    Ok(())
}

pub async fn verify(firmware: &str) -> Result<()> {
    println!("{}", "Phoenix Backup Verify".bold().blue());
    println!("Firmware: {}", firmware);

    let checksum = BackupManager::verify_backup(Path::new(firmware))
        .await
        .map_err(|e| anyhow::anyhow!(e.to_string()))?;

    println!("SHA256: {}", checksum.green());

    Ok(())
}
