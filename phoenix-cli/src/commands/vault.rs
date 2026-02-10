//! Vault command for secure firmware backups

use anyhow::Result;
use phoenix_lib::vault::VaultManager;
use phoenix_lib::hardware;
use std::path::Path;
use tracing::info;

/// Create a new vault backup
pub async fn create(device: Option<&str>, name: &str) -> Result<()> {
    info!("Initializing vault creation for device {:?}", device);
    
    let vault = VaultManager::default();
    
    // 1. Detect device hardware
    let report = hardware::perform_deep_scan(device)?;
    
    // 2. Create vault
    let manifest = vault.create_vault(name, &report)?;
    
    println!("✅ Vault '{}' created successfully", name);
    println!("   Timestamp: {}", manifest.timestamp);
    println!("   Device: {}", manifest.device_id);
    println!("   Partitions: {}", manifest.partitions.len());
    
    Ok(())
}

/// List available vaults
pub async fn list() -> Result<()> {
    let vault = VaultManager::default();
    let vaults = vault.list_vaults()?;
    
    if vaults.is_empty() {
        println!("No vaults found.");
        return Ok(());
    }
    
    println!("\nAvailable Vaults:");
    println!("{}", "=".repeat(60));
    println!("{:<20} {:<25} {:<15}", "Name", "Timestamp", "Device");
    println!("{}", "-".repeat(60));
    
    for v in vaults {
        println!("{:<20} {:<25} {:<15}", v.name, v.timestamp, v.device_id);
    }
    println!();
    
    Ok(())
}

/// Verify vault integrity
pub async fn verify(name: &str) -> Result<()> {
    let vault = VaultManager::default();
    if vault.verify_vault(name)? {
        println!("✅ Vault '{}' integrity verified", name);
    } else {
        println!("❌ Vault '{}' integrity check failed!", name);
        std::process::exit(1);
    }
    Ok(())
}

/// Restore a vault to a device
pub async fn restore(name: &str, device: Option<&str>) -> Result<()> {
    let vault = VaultManager::default();
    vault.restore_vault(name, device)?;
    println!("✅ Vault '{}' restored successfully to {:?}", name, device);
    Ok(())
}

/// Extract data from a vault
pub async fn extract(name: &str, partition: &str, output: &str) -> Result<()> {
    let vault = VaultManager::default();
    vault.extract_from_vault(name, partition, Path::new(output))?;
    println!("✅ Extracted {} from vault '{}' to {}", partition, name, output);
    Ok(())
}
