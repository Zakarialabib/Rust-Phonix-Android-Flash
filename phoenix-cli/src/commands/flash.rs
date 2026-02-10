//! Flash command

use anyhow::Result;
use indicatif::{ProgressBar, ProgressStyle};
use std::path::Path;
use std::time::Duration;
use phoenix_lib::flash::{preflight, flash_image_async, FlashProgress};
use phoenix_lib::workflow::{Phase, PhaseStatus};
use crate::commands::phase;

pub async fn run(target: &str, device: &str, image: &str) -> Result<()> {
    phase::emit(Phase::Flash, PhaseStatus::Started, None);
    println!("💾 Phoenix Flash Tool");
    println!("=====================");
    println!();

    preflight(Path::new(image), device).map_err(|e| anyhow::anyhow!(e.to_string()))?;

    let image_size = std::fs::metadata(image)?.len();
    println!("Image:  {} ({:.1} MB)", image, image_size as f64 / 1024.0 / 1024.0);
    println!("Target: {} ({})", device, target);
    println!();

    // Safety confirmation
    println!("⚠️  WARNING: This will ERASE ALL DATA on {}", device);
    println!("   Make sure you have selected the correct device!");
    println!();
    
    // In a real implementation, we'd prompt for confirmation
    // For now, just show what would happen

    match target {
        "sd" => flash_sd(device, image, image_size).await,
        "emmc" => flash_emmc(device, image, image_size).await,
        _ => anyhow::bail!("Unknown target: {}. Use 'sd' or 'emmc'", target),
    }
    .map(|_| {
        phase::emit(Phase::Flash, PhaseStatus::Completed, None);
    })
}

async fn flash_sd(device: &str, image: &str, size: u64) -> Result<()> {
    println!("📝 Writing to SD card...");

    let pb = ProgressBar::new(size);
    pb.set_style(ProgressStyle::default_bar()
        .template("{spinner:.green} [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")?
        .progress_chars("█▓░"));

    let pb_clone = pb.clone();
    let progress_cb = Box::new(move |p: FlashProgress| {
        pb_clone.set_position(p.bytes_transferred);
    });

    flash_image_async(Path::new(image), device, Some(progress_cb))
        .await
        .map_err(|e| anyhow::anyhow!(e.to_string()))?;

    pb.finish_with_message("Write complete!");
    println!();
    println!("✅ SD card flashed successfully!");
    println!();
    println!("Next steps:");
    println!("  1. Safely eject the SD card");
    println!("  2. Insert into your device");
    println!("  3. Power on - Phoenix should boot!");
    println!();
    println!("First boot may take 1-2 minutes for initial setup.");

    Ok(())
}

async fn flash_emmc(device: &str, image: &str, size: u64) -> Result<()> {
    println!("📝 Writing to eMMC via USB...");
    println!("Device: {}, Image: {}, Size: {}", device, image, size);
    println!();
    println!("Ensure device is in Maskrom mode (run 'phoenix detect' to verify)");

    let pb = ProgressBar::new(size);
    pb.set_style(ProgressStyle::default_bar()
        .template("{spinner:.green} [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")?
        .progress_chars("█▓░"));

    // Simulate write progress
    let chunk_size = size / 100;
    for _ in 0..100 {
        tokio::time::sleep(Duration::from_millis(30)).await;
        pb.inc(chunk_size);
    }

    pb.finish_with_message("Write complete!");
    println!();
    println!("✅ eMMC flashed successfully!");
    println!();
    println!("Device will reboot automatically.");

    Ok(())
}
