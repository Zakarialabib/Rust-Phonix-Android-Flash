use serde::{Deserialize, Serialize};
use std::path::Path;
use anyhow::{Context, Result};
use std::fs;

/// Device Profile Database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileDatabase {
    pub profiles: Vec<DeviceProfile>,
}

/// Hardware Capability Manifest for a specific device model
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceProfile {
    pub vendor_id: u16,
    pub product_id: u16,
    pub name: String,
    pub soc: String,
    pub ram_mb: u32,
    pub storage_type: String,
    pub bootloader_offset: u32,
    pub supported_modes: Vec<String>,
}

impl ProfileDatabase {
    /// Load database from TOML file
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = fs::read_to_string(path.as_ref())
            .with_context(|| format!("Failed to read profiles file: {:?}", path.as_ref()))?;
        toml::from_str(&content).context("Failed to parse profiles TOML")
    }

    /// Find profile by Vendor ID and Product ID
    pub fn find(&self, vid: u16, pid: u16) -> Option<&DeviceProfile> {
        self.profiles.iter().find(|p| p.vendor_id == vid && p.product_id == pid)
    }
}

/// Default built-in profiles (fallback if file missing)
pub fn default_profiles() -> ProfileDatabase {
    ProfileDatabase {
        profiles: vec![
            DeviceProfile {
                vendor_id: 0x1b8e,
                product_id: 0xc003,
                name: "Amlogic S905W (Generic)".to_string(),
                soc: "s905w".to_string(),
                ram_mb: 2048,
                storage_type: "emmc".to_string(),
                bootloader_offset: 512,
                supported_modes: vec!["maskrom".to_string()],
            },
            DeviceProfile {
                vendor_id: 0x1f3a,
                product_id: 0xefe8,
                name: "Allwinner H3 (Generic)".to_string(),
                soc: "h3".to_string(),
                ram_mb: 1024,
                storage_type: "sd".to_string(),
                bootloader_offset: 8,
                supported_modes: vec!["fel".to_string()],
            },
            DeviceProfile {
                vendor_id: 0x1b8e,
                product_id: 0xc003,
                name: "Amlogic S905X (Generic)".to_string(),
                soc: "s905x".to_string(),
                ram_mb: 2048,
                storage_type: "emmc".to_string(),
                bootloader_offset: 512,
                supported_modes: vec!["maskrom".to_string()],
            },
            DeviceProfile {
                vendor_id: 0x1b8e,
                product_id: 0xc003,
                name: "Amlogic S905X3 (Generic)".to_string(),
                soc: "s905x3".to_string(),
                ram_mb: 4096,
                storage_type: "emmc".to_string(),
                bootloader_offset: 512,
                supported_modes: vec!["maskrom".to_string(), "burn_mode".to_string()],
            },
            DeviceProfile {
                vendor_id: 0x2207,
                product_id: 0x320b,
                name: "Rockchip RK3229 (Generic)".to_string(),
                soc: "rk3229".to_string(),
                ram_mb: 1024,
                storage_type: "emmc".to_string(),
                bootloader_offset: 64,
                supported_modes: vec!["maskrom".to_string(), "loader".to_string()],
            },
            DeviceProfile {
                vendor_id: 0x2207,
                product_id: 0x301a,
                name: "Rockchip RK3036 (AnyCast)".to_string(),
                soc: "rk3036".to_string(),
                ram_mb: 256,
                storage_type: "spi_nor".to_string(),
                bootloader_offset: 64,
                supported_modes: vec!["maskrom".to_string(), "loader".to_string()],
            },
            DeviceProfile {
                vendor_id: 0x2207,
                product_id: 0x322a,
                name: "Rockchip RK3328 (Generic)".to_string(),
                soc: "rk3328".to_string(),
                ram_mb: 2048,
                storage_type: "emmc".to_string(),
                bootloader_offset: 64,
                supported_modes: vec!["maskrom".to_string(), "loader".to_string()],
            },
            DeviceProfile {
                vendor_id: 0x2207,
                product_id: 0x330c,
                name: "Rockchip RK3399 (Generic)".to_string(),
                soc: "rk3399".to_string(),
                ram_mb: 4096,
                storage_type: "emmc".to_string(),
                bootloader_offset: 64,
                supported_modes: vec!["maskrom".to_string(), "loader".to_string()],
            },
            DeviceProfile {
                vendor_id: 0x1b8e,
                product_id: 0xc003,
                name: "Amlogic S905D (Generic)".to_string(),
                soc: "s905d".to_string(),
                ram_mb: 2048,
                storage_type: "emmc".to_string(),
                bootloader_offset: 512,
                supported_modes: vec!["maskrom".to_string(), "burn_mode".to_string()],
            },
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_existing() {
        let db = default_profiles();
        // Allwinner H3 (Generic)
        let profile = db.find(0x1f3a, 0xefe8).expect("Should find Allwinner H3");
        assert_eq!(profile.name, "Allwinner H3 (Generic)");
        assert_eq!(profile.soc, "h3");
    }

    #[test]
    fn test_find_non_existent() {
        let db = default_profiles();
        let profile = db.find(0x0000, 0x0000);
        assert!(profile.is_none());
    }

    #[test]
    fn test_find_multiple_matches() {
        let db = default_profiles();
        // Multiple profiles have 0x1b8e:0xc003. The first one is S905W.
        let profile = db.find(0x1b8e, 0xc003).expect("Should find Amlogic profile");
        assert_eq!(profile.name, "Amlogic S905W (Generic)");
        assert_eq!(profile.soc, "s905w");
    }

    #[test]
    fn test_from_file_valid() {
        use std::io::Write;
        let mut tmpfile = tempfile::NamedTempFile::new().unwrap();
        let toml_content = r#"
[[profiles]]
vendorId = 4660
productId = 22136
name = "Test Device"
soc = "test-soc"
ramMb = 1024
storageType = "emmc"
bootloaderOffset = 0
supportedModes = ["test"]
"#;
        write!(tmpfile, "{}", toml_content).unwrap();

        let db = ProfileDatabase::from_file(tmpfile.path()).expect("Should load valid TOML");
        assert_eq!(db.profiles.len(), 1);
        assert_eq!(db.profiles[0].name, "Test Device");
        assert_eq!(db.profiles[0].vendor_id, 0x1234);
        assert_eq!(db.profiles[0].product_id, 0x5678);
    }

    #[test]
    fn test_from_file_invalid_toml() {
        use std::io::Write;
        let mut tmpfile = tempfile::NamedTempFile::new().unwrap();
        write!(tmpfile, "invalid = [toml").unwrap();

        let result = ProfileDatabase::from_file(tmpfile.path());
        assert!(result.is_err());
    }

    #[test]
    fn test_from_file_missing_file() {
        let result = ProfileDatabase::from_file("non_existent_file.toml");
        assert!(result.is_err());
    }

    #[test]
    fn test_from_file_malformed_structure() {
        use std::io::Write;
        let mut tmpfile = tempfile::NamedTempFile::new().unwrap();
        // Missing required fields like 'soc' or 'vendorId'
        let toml_content = r#"
[[profiles]]
name = "Incomplete Device"
"#;
        write!(tmpfile, "{}", toml_content).unwrap();

        let result = ProfileDatabase::from_file(tmpfile.path());
        assert!(result.is_err());
    }
}
