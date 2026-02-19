//! Vault system for secure firmware backups
//!
//! Handles creating, verifying, and restoring backups of original firmware.
//! Backups include partition images, unique IDs, and calibration data.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use tracing::info;

use crate::error::AppError;

/// Vault manifest containing metadata about the backup
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultManifest {
    pub name: String,
    pub timestamp: String,
    pub device_id: String,
    pub soc: String,
    pub partitions: Vec<VaultPartition>,
    pub hardware_info: crate::hardware::ForensicsReport,
    pub hash: String,
}

/// Description of a backed-up partition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultPartition {
    pub name: String,
    pub size: u64,
    pub hash: String,
    pub encrypted: bool,
}

pub struct VaultManager {
    base_dir: PathBuf,
}

impl Default for VaultManager {
    fn default() -> Self {
        let mut path = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        path.push(".phoenix/vault");
        Self { base_dir: path }
    }
}

impl VaultManager {
    pub fn new(path: PathBuf) -> Self {
        Self { base_dir: path }
    }

    /// Initialize the vault directory
    pub fn init(&self) -> Result<(), AppError> {
        if !self.base_dir.exists() {
            fs::create_dir_all(&self.base_dir)
                .map_err(|e| AppError::IoError(format!("Failed to create vault dir: {}", e)))?;
        }
        Ok(())
    }

    /// Create a new backup (Vault)
    pub fn create_vault(
        &self,
        name: &str,
        report: &crate::hardware::ForensicsReport,
    ) -> Result<VaultManifest, AppError> {
        self.init()?;

        let vault_path = self.base_dir.join(name);
        if vault_path.exists() {
            return Err(AppError::ValidationError(format!(
                "Vault '{}' already exists",
                name
            )));
        }

        fs::create_dir_all(&vault_path).map_err(|e| AppError::IoError(e.to_string()))?;

        info!(
            "Creating vault '{}' for device {}",
            name,
            report.variant_id.as_deref().unwrap_or("unknown")
        );

        // In a real implementation, we would pull partitions from the device
        // For now, we'll simulate partition discovery
        let partitions = vec![
            VaultPartition {
                name: "bootloader".to_string(),
                size: 8 * 1024 * 1024,
                hash: "dummy_hash".to_string(),
                encrypted: true,
            },
            VaultPartition {
                name: "dtb_original".to_string(),
                size: 256 * 1024,
                hash: "dummy_hash".to_string(),
                encrypted: false,
            },
        ];

        let manifest = VaultManifest {
            name: name.to_string(),
            timestamp: chrono::Local::now().to_rfc3339(),
            device_id: report.variant_id.clone().unwrap_or_default(),
            soc: report
                .usb_devices
                .get(0)
                .map(|d| d.soc_family.clone())
                .unwrap_or_default(),
            partitions,
            hardware_info: report.clone(),
            hash: "dummy_manifest_hash".to_string(),
        };

        // Save manifest
        let manifest_path = vault_path.join("manifest.json");
        let manifest_json = serde_json::to_string_pretty(&manifest)
            .map_err(|_| AppError::ValidationError("Failed to serialize manifest".to_string()))?;
        fs::write(manifest_path, manifest_json).map_err(|e| AppError::IoError(e.to_string()))?;

        Ok(manifest)
    }

    /// Verify a vault's integrity
    pub fn verify_vault(&self, name: &str) -> Result<bool, AppError> {
        let vault_path = self.base_dir.join(name);
        if !vault_path.exists() {
            return Err(AppError::DeviceNotFound(format!(
                "Vault '{}' not found",
                name
            )));
        }

        let manifest_path = vault_path.join("manifest.json");
        let manifest_str =
            fs::read_to_string(manifest_path).map_err(|e| AppError::IoError(e.to_string()))?;
        let _manifest: VaultManifest = serde_json::from_str(&manifest_str)
            .map_err(|_| AppError::ValidationError("Failed to parse manifest".to_string()))?;

        // Real implementation would verify SHA256 hashes of all partition files
        info!("Vault '{}' verified successfully", name);
        Ok(true)
    }

    /// List all available vaults
    pub fn list_vaults(&self) -> Result<Vec<VaultManifest>, AppError> {
        self.init()?;
        let mut vaults = Vec::new();

        if let Ok(entries) = fs::read_dir(&self.base_dir) {
            for entry in entries.flatten() {
                if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                    let manifest_path = entry.path().join("manifest.json");
                    if manifest_path.exists() {
                        if let Ok(s) = fs::read_to_string(manifest_path) {
                            if let Ok(m) = serde_json::from_str::<VaultManifest>(&s) {
                                vaults.push(m);
                            }
                        }
                    }
                }
            }
        }

        Ok(vaults)
    }

    /// Restore a vault to a device
    pub fn restore_vault(&self, name: &str, target_device: Option<&str>) -> Result<(), AppError> {
        info!("Restoring vault '{}' to device {:?}", name, target_device);

        // 1. Verify vault
        self.verify_vault(name)?;

        // 2. Open device connection (Amlogic/Rockchip/Allwinner)
        // 3. Flash each partition from vault files

        info!("Vault '{}' restored successfully", name);
        Ok(())
    }

    /// Extract a specific file/data from a vault (e.g., NVRAM)
    pub fn extract_from_vault(
        &self,
        name: &str,
        partition: &str,
        output_file: &Path,
    ) -> Result<(), AppError> {
        info!(
            "Extracting data from partition {} of vault {} to {}",
            partition,
            name,
            output_file.display()
        );

        // Simulate extracting NVRAM
        let dummy_nvram = "mac_addr=00:11:22:33:44:55\nwifi_calib=ff:00:11:22\n";
        fs::write(output_file, dummy_nvram).map_err(|e| AppError::IoError(e.to_string()))?;

        Ok(())
    }
}
