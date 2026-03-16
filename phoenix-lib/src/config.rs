//! Device configuration loading and validation

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::path::Path;

use crate::error::AppError;
use crate::profiles::DeviceProfile;

/// Root device configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceConfig {
    pub device: DeviceInfo,
    pub hardware: HardwareConfig,
    pub boot: BootConfig,
    #[serde(default)]
    pub profiles: std::collections::HashMap<String, BuildProfile>,
    #[serde(default)]
    pub build: BuildConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceInfo {
    pub name: String,
    #[serde(default)]
    pub manufacturer: String,
    pub soc: String,
    #[serde(default)]
    pub variant: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareConfig {
    pub memory: MemoryConfig,
    pub storage: StorageConfig,
    #[serde(default)]
    pub wifi: Option<WifiConfig>,
    #[serde(default)]
    pub ethernet: Option<EthernetConfig>,
    #[serde(default)]
    pub capabilities: Option<Capabilities>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryConfig {
    #[serde(rename = "type")]
    pub mem_type: String,
    pub size_mb: u32,
    #[serde(default)]
    pub chip: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    #[serde(rename = "type")]
    pub storage_type: String,
    pub size_gb: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WifiConfig {
    pub chip: String,
    pub driver: String,
    #[serde(default)]
    pub firmware: String,
    #[serde(default)]
    pub nvram: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EthernetConfig {
    #[serde(rename = "type")]
    pub eth_type: String,
    pub speed: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Capabilities {
    #[serde(default)]
    pub gpu: Option<String>,
    #[serde(default)]
    pub vpu: Option<String>,
    #[serde(default)]
    pub wifi_supported: bool,
    #[serde(default)]
    pub ethernet_supported: bool,
    #[serde(default)]
    pub hdmi_cec_supported: bool,
    #[serde(default)]
    pub has_emmc: bool,
    #[serde(default)]
    pub has_sd_slot: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BootConfig {
    #[serde(default)]
    pub secure_boot: bool,
    pub reference_dtb: String,
    #[serde(default)]
    pub uart: Option<UartConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UartConfig {
    pub port: String,
    pub baud: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildProfile {
    pub rootfs: String,
    pub kernel: String,
    #[serde(default)]
    pub packages: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BuildConfig {
    #[serde(default)]
    pub buildroot_defconfig: String,
    #[serde(default)]
    pub kernel_fragments: Vec<String>,
    #[serde(default)]
    pub uboot_config: String,
}

impl DeviceConfig {
    /// Load configuration from a YAML file
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = std::fs::read_to_string(path.as_ref())
            .with_context(|| format!("Failed to read config file: {:?}", path.as_ref()))?;
        Self::from_str(&content)
    }

    /// Parse configuration from YAML string
    pub fn from_str(yaml: &str) -> Result<Self> {
        serde_yaml::from_str(yaml).context("Failed to parse YAML configuration")
    }

    /// Save configuration to a YAML file
    pub fn to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let yaml = serde_yaml::to_string(self)?;
        std::fs::write(path.as_ref(), yaml)
            .with_context(|| format!("Failed to write config file: {:?}", path.as_ref()))?;
        Ok(())
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<(), AppError> {
        // Check required fields
        if self.device.name.is_empty() {
            return Err(AppError::ValidationError(
                "Device name is required".to_string(),
            ));
        }
        if self.device.soc.is_empty() {
            return Err(AppError::ValidationError(
                "SoC type is required".to_string(),
            ));
        }
        if self.boot.reference_dtb.is_empty() {
            return Err(AppError::ValidationError(
                "Reference DTB is required".to_string(),
            ));
        }
        // Additional checks
        if self.hardware.memory.size_mb == 0 {
            return Err(AppError::ValidationError(
                "Memory size cannot be 0".to_string(),
            ));
        }
        if self.hardware.storage.size_gb == 0 {
            return Err(AppError::ValidationError(
                "Storage size cannot be 0".to_string(),
            ));
        }

        // Schema-style checks
        let allowed_mems = ["DDR3", "DDR4", "LPDDR3", "LPDDR4"];
        if !allowed_mems.contains(&self.hardware.memory.mem_type.as_str()) {
            return Err(AppError::ValidationError(format!(
                "Unsupported memory type: {}",
                self.hardware.memory.mem_type
            )));
        }

        let allowed_storage = ["eMMC", "sd", "spi_nor"];
        if !allowed_storage.contains(&self.hardware.storage.storage_type.as_str()) {
            return Err(AppError::ValidationError(format!(
                "Unsupported storage type: {}",
                self.hardware.storage.storage_type
            )));
        }

        if let Some(wifi) = &self.hardware.wifi {
            if wifi.chip.is_empty() {
                return Err(AppError::ValidationError(
                    "WiFi chip cannot be empty".to_string(),
                ));
            }
        }

        if let Some(cap) = &self.hardware.capabilities {
            if cap.has_emmc && self.hardware.storage.storage_type.to_lowercase() != "emmc" {
                return Err(AppError::ValidationError(
                    "Capability has_emmc conflicts with storage_type".to_string(),
                ));
            }
        }
        Ok(())
    }

    pub fn validate_schema_yaml(yaml: &str) -> Result<(), AppError> {
        let yaml_value: serde_yaml::Value = serde_yaml::from_str(yaml)
            .map_err(|e| AppError::ValidationError(format!("Invalid YAML: {}", e)))?;
        let json_value = serde_json::to_value(yaml_value)
            .map_err(|e| AppError::ValidationError(format!("Schema conversion failed: {}", e)))?;

        let schema = device_config_schema();
        let compiled = jsonschema::Validator::new(&schema)
            .map_err(|e| AppError::ValidationError(format!("Schema compile error: {}", e)))?;

        if let Err(errors) = compiled.validate(&json_value) {
            let messages = errors.map(|e| e.to_string()).collect::<Vec<_>>();
            return Err(AppError::ValidationError(messages.join("; ")));
        }
        Ok(())
    }

    pub fn populate_capabilities_from_profile(&mut self, profile: &DeviceProfile) {
        let has_emmc = profile.storage_type.to_lowercase() == "emmc";
        let has_sd_slot = profile.storage_type.to_lowercase() == "sd";
        self.hardware.capabilities = Some(Capabilities {
            gpu: Some("panfrost".to_string()),
            vpu: Some("v4l2-m2m".to_string()),
            wifi_supported: self.hardware.wifi.is_some(),
            ethernet_supported: self.hardware.ethernet.is_some(),
            hdmi_cec_supported: true,
            has_emmc,
            has_sd_slot,
        });
    }
}

fn device_config_schema() -> JsonValue {
    serde_json::json!({
        "type": "object",
        "required": ["device", "hardware", "boot"],
        "properties": {
            "device": {
                "type": "object",
                "required": ["name", "soc"],
                "properties": {
                    "name": { "type": "string", "minLength": 1 },
                    "manufacturer": { "type": "string" },
                    "soc": { "type": "string", "minLength": 1 },
                    "variant": { "type": "string" }
                }
            },
            "hardware": {
                "type": "object",
                "required": ["memory", "storage"],
                "properties": {
                    "memory": {
                        "type": "object",
                        "required": ["type", "size_mb"],
                        "properties": {
                            "type": { "type": "string" },
                            "size_mb": { "type": "number", "minimum": 1 },
                            "chip": { "type": "string" }
                        }
                    },
                    "storage": {
                        "type": "object",
                        "required": ["type", "size_gb"],
                        "properties": {
                            "type": { "type": "string" },
                            "size_gb": { "type": "number", "minimum": 1 }
                        }
                    },
                    "wifi": {
                        "type": ["object", "null"],
                        "properties": {
                            "chip": { "type": "string" },
                            "driver": { "type": "string" },
                            "firmware": { "type": "string" },
                            "nvram": { "type": "string" }
                        }
                    },
                    "ethernet": {
                        "type": ["object", "null"],
                        "properties": {
                            "type": { "type": "string" },
                            "speed": { "type": "string" }
                        }
                    },
                    "capabilities": {
                        "type": ["object", "null"],
                        "properties": {
                            "gpu": { "type": ["string", "null"] },
                            "vpu": { "type": ["string", "null"] },
                            "wifiSupported": { "type": "boolean" },
                            "ethernetSupported": { "type": "boolean" },
                            "hdmiCecSupported": { "type": "boolean" },
                            "hasEmmc": { "type": "boolean" },
                            "hasSdSlot": { "type": "boolean" }
                        }
                    }
                }
            },
            "boot": {
                "type": "object",
                "required": ["reference_dtb"],
                "properties": {
                    "secure_boot": { "type": "boolean" },
                    "reference_dtb": { "type": "string" },
                    "uart": {
                        "type": ["object", "null"],
                        "properties": {
                            "port": { "type": "string" },
                            "baud": { "type": "number" }
                        }
                    }
                }
            },
            "profiles": { "type": "object" },
            "build": { "type": "object" }
        }
    })
}

/// Create a default configuration for a given SoC
pub fn create_default_config(soc: &str, name: &str) -> DeviceConfig {
    let (dtb, mem_size) = match soc {
        "s905w" => ("meson-gxl-s905w-p281.dtb", 2048),
        "s905x" => ("meson-gxl-s905x-p212.dtb", 2048),
        "rk3229" => ("rk3229-evb.dtb", 1024),
        _ => ("unknown.dtb", 1024),
    };

    DeviceConfig {
        device: DeviceInfo {
            name: name.to_string(),
            manufacturer: "Generic".to_string(),
            soc: soc.to_string(),
            variant: format!("{}MB", mem_size),
        },
        hardware: HardwareConfig {
            memory: MemoryConfig {
                mem_type: "DDR3".to_string(),
                size_mb: mem_size,
                chip: "Unknown".to_string(),
            },
            storage: StorageConfig {
                storage_type: "eMMC".to_string(),
                size_gb: 16,
            },
            wifi: Some(WifiConfig {
                chip: "AP6212".to_string(),
                driver: "brcmfmac".to_string(),
                firmware: "brcmfmac43430-sdio.bin".to_string(),
                nvram: "nvram_ap6212.txt".to_string(),
            }),
            ethernet: Some(EthernetConfig {
                eth_type: "internal".to_string(),
                speed: "100Mbps".to_string(),
            }),
            capabilities: Some(Capabilities {
                gpu: Some("panfrost".to_string()),
                vpu: Some("v4l2-m2m".to_string()),
                wifi_supported: true,
                ethernet_supported: true,
                hdmi_cec_supported: true,
                has_emmc: true,
                has_sd_slot: true,
            }),
        },
        boot: BootConfig {
            secure_boot: false,
            reference_dtb: dtb.to_string(),
            uart: Some(UartConfig {
                port: "ttyAML0".to_string(),
                baud: 115200,
            }),
        },
        profiles: [
            (
                "minimal".to_string(),
                BuildProfile {
                    rootfs: "buildroot".to_string(),
                    kernel: "mainline-6.6".to_string(),
                    packages: vec![],
                },
            ),
            (
                "signage".to_string(),
                BuildProfile {
                    rootfs: "buildroot".to_string(),
                    kernel: "mainline-6.6".to_string(),
                    packages: vec!["cage".to_string(), "weston".to_string()],
                },
            ),
        ]
        .into_iter()
        .collect(),
        build: BuildConfig {
            buildroot_defconfig: format!("phoenix_{}_defconfig", soc),
            kernel_fragments: vec!["amlogic-base.config".to_string()],
            uboot_config: "libretech-cc_defconfig".to_string(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_default_config() {
        // Test S905W
        let config = create_default_config("s905w", "My S905W Box");
        assert_eq!(config.device.name, "My S905W Box");
        assert_eq!(config.device.soc, "s905w");
        assert_eq!(config.hardware.memory.size_mb, 2048);
        assert_eq!(config.boot.reference_dtb, "meson-gxl-s905w-p281.dtb");

        // Test RK3229
        let config = create_default_config("rk3229", "My RK3229 Box");
        assert_eq!(config.device.name, "My RK3229 Box");
        assert_eq!(config.device.soc, "rk3229");
        assert_eq!(config.hardware.memory.size_mb, 1024);
        assert_eq!(config.boot.reference_dtb, "rk3229-evb.dtb");

        // Test Unknown
        let config = create_default_config("unknown_soc", "Generic Box");
        assert_eq!(config.device.name, "Generic Box");
        assert_eq!(config.device.soc, "unknown_soc");
        assert_eq!(config.hardware.memory.size_mb, 1024);
        assert_eq!(config.boot.reference_dtb, "unknown.dtb");
    }

    #[test]
    fn test_validate_schema_valid() {
        let yaml = r#"
device:
  name: "Test Device"
  soc: "s905x"
hardware:
  memory:
    type: "DDR3"
    size_mb: 1024
  storage:
    type: "eMMC"
    size_gb: 16
boot:
  reference_dtb: "test.dtb"
"#;
        assert!(DeviceConfig::validate_schema_yaml(yaml).is_ok());
    }

    #[test]
    fn test_validate_schema_invalid_syntax() {
        let yaml = "invalid: : yaml";
        assert!(DeviceConfig::validate_schema_yaml(yaml).is_err());
    }

    #[test]
    fn test_validate_schema_missing_top_level() {
        let yaml = r#"
hardware:
  memory:
    type: "DDR3"
    size_mb: 1024
  storage:
    type: "eMMC"
    size_gb: 16
boot:
  reference_dtb: "test.dtb"
"#;
        let result = DeviceConfig::validate_schema_yaml(yaml);
        assert!(result
            .expect_err("Validation should fail for missing top-level property")
            .to_string()
            .contains("\"device\" is a required property"));
    }

    #[test]
    fn test_validate_schema_missing_nested() {
        let yaml = r#"
device:
  name: "Test Device"
hardware:
  memory:
    type: "DDR3"
    size_mb: 1024
  storage:
    type: "eMMC"
    size_gb: 16
boot:
  reference_dtb: "test.dtb"
"#;
        let result = DeviceConfig::validate_schema_yaml(yaml);
        assert!(result
            .expect_err("Validation should fail for missing nested property")
            .to_string()
            .contains("\"soc\" is a required property"));
    }

    #[test]
    fn test_validate_schema_minimum_value() {
        let yaml = r#"
device:
  name: "Test Device"
  soc: "s905x"
hardware:
  memory:
    type: "DDR3"
    size_mb: 0
  storage:
    type: "eMMC"
    size_gb: 16
boot:
  reference_dtb: "test.dtb"
"#;
        let result = DeviceConfig::validate_schema_yaml(yaml);
        assert!(result
            .expect_err("Validation should fail for value below minimum")
            .to_string()
            .contains("0 is less than the minimum of 1"));
    }

    #[test]
    fn test_validate_schema_minimum_length() {
        let yaml = r#"
device:
  name: ""
  soc: "s905x"
hardware:
  memory:
    type: "DDR3"
    size_mb: 1024
  storage:
    type: "eMMC"
    size_gb: 16
boot:
  reference_dtb: "test.dtb"
"#;
        let result = DeviceConfig::validate_schema_yaml(yaml);
        let err = result
            .expect_err("Validation should fail for empty name")
            .to_string();
        println!("Error: {}", err);
        assert!(err.contains("shorter than 1") || err.contains("length"));
    }

    #[test]
    fn test_validate_schema_type_mismatch() {
        let yaml = r#"
device:
  name: "Test Device"
  soc: "s905x"
hardware:
  memory:
    type: "DDR3"
    size_mb: "1024"
  storage:
    type: "eMMC"
    size_gb: 16
boot:
  reference_dtb: "test.dtb"
"#;
        let result = DeviceConfig::validate_schema_yaml(yaml);
        assert!(result
            .expect_err("Validation should fail for type mismatch")
            .to_string()
            .contains("is not of type \"number\""));
    }
}
