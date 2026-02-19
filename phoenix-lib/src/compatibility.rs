use serde::{Deserialize, Serialize};
use std::path::Path;
use std::str::FromStr;

use crate::config::DeviceConfig;
use crate::error::AppError;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum SoC {
    S905W,
    S905X,
    RK3229,
    RK3036,
    RK3328,
    RK3399,
    H3,
    Unknown(String),
}

impl FromStr for SoC {
    type Err = std::convert::Infallible;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Ok(match value.to_lowercase().as_str() {
            "s905w" | "amlogic_s905w" => SoC::S905W,
            "s905x" | "amlogic_s905x" => SoC::S905X,
            "rk3229" => SoC::RK3229,
            "rk3036" => SoC::RK3036,
            "rk3328" => SoC::RK3328,
            "rk3399" => SoC::RK3399,
            "h3" | "allwinner_h3" => SoC::H3,
            other => SoC::Unknown(other.to_string()),
        })
    }
}

impl SoC {
    fn is_unknown(&self) -> bool {
        matches!(self, SoC::Unknown(_))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum PcbVariant {
    P281,
    P282,
    Unknown(String),
}

impl FromStr for PcbVariant {
    type Err = std::convert::Infallible;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Ok(match value.to_lowercase().as_str() {
            "p281" => PcbVariant::P281,
            "p282" => PcbVariant::P282,
            other => PcbVariant::Unknown(other.to_string()),
        })
    }
}

impl PcbVariant {
    fn is_unknown(&self) -> bool {
        matches!(self, PcbVariant::Unknown(_))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum RamVendor {
    Samsung,
    Hynix,
    Micron,
    Unknown(String),
}

impl FromStr for RamVendor {
    type Err = std::convert::Infallible;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let lower = value.to_lowercase();
        Ok(if lower.contains("samsung") {
            RamVendor::Samsung
        } else if lower.contains("hynix") || lower.contains("skhynix") {
            RamVendor::Hynix
        } else if lower.contains("micron") {
            RamVendor::Micron
        } else {
            RamVendor::Unknown(value.to_string())
        })
    }
}

impl RamVendor {
    fn is_unknown(&self) -> bool {
        matches!(self, RamVendor::Unknown(_))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum WifiChip {
    AP6212,
    RTL8189FS,
    SSV6051,
    Unknown(String),
}

impl FromStr for WifiChip {
    type Err = std::convert::Infallible;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Ok(match value.to_lowercase().as_str() {
            "ap6212" => WifiChip::AP6212,
            "rtl8189fs" => WifiChip::RTL8189FS,
            "ssv6051" => WifiChip::SSV6051,
            other => WifiChip::Unknown(other.to_string()),
        })
    }
}

impl WifiChip {
    fn is_unknown(&self) -> bool {
        matches!(self, WifiChip::Unknown(_))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum EmmcVendor {
    Samsung,
    Toshiba,
    SkHynix,
    Sandisk,
    Unknown(String),
}

impl EmmcVendor {
    fn is_unknown(&self) -> bool {
        matches!(self, EmmcVendor::Unknown(_))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum HdmiPhyVersion {
    V1_0,
    V1_1,
    Unknown(String),
}

impl HdmiPhyVersion {
    fn is_unknown(&self) -> bool {
        matches!(self, HdmiPhyVersion::Unknown(_))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum OsType {
    Android,
    Linux,
    CoreElec,
    Armbian,
    Unknown(String),
}

impl FromStr for OsType {
    type Err = std::convert::Infallible;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Ok(match value.to_lowercase().as_str() {
            "android" => OsType::Android,
            "linux" => OsType::Linux,
            "coreelec" => OsType::CoreElec,
            "armbian" => OsType::Armbian,
            other => OsType::Unknown(other.to_string()),
        })
    }
}

impl OsType {
    fn is_unknown(&self) -> bool {
        matches!(self, OsType::Unknown(_))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HardwareProfile {
    pub soc: SoC,
    pub pcb_variant: PcbVariant,
    pub ram_vendor: RamVendor,
    pub wifi_chip: WifiChip,
    pub emmc_vendor: EmmcVendor,
    pub hdmi_phy: HdmiPhyVersion,
}

impl HardwareProfile {
    pub fn from_device_config(config: &DeviceConfig) -> Self {
        let pcb_variant = if config.device.variant.is_empty() {
            PcbVariant::Unknown("unknown".to_string())
        } else {
            PcbVariant::from_str(&config.device.variant).unwrap()
        };

        let ram_vendor = RamVendor::from_str(&config.hardware.memory.chip).unwrap();
        let wifi_chip = config
            .hardware
            .wifi
            .as_ref()
            .map(|wifi| WifiChip::from_str(&wifi.chip).unwrap())
            .unwrap_or_else(|| WifiChip::Unknown("unknown".to_string()));

        HardwareProfile {
            soc: SoC::from_str(&config.device.soc).unwrap(),
            pcb_variant,
            ram_vendor,
            wifi_chip,
            emmc_vendor: EmmcVendor::Unknown("unknown".to_string()),
            hdmi_phy: HdmiPhyVersion::Unknown("unknown".to_string()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FirmwareTarget {
    pub os_type: OsType,
    pub version: String,
    pub kernel: String,
}

impl FirmwareTarget {
    pub fn from_inputs(
        firmware_path: &Path,
        os_override: Option<&str>,
        version_override: Option<&str>,
        kernel_override: Option<&str>,
    ) -> Result<Self, AppError> {
        if !firmware_path.exists() {
            return Err(AppError::ValidationError(format!(
                "Firmware path not found: {}",
                firmware_path.display()
            )));
        }

        let file_name = firmware_path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or_default();

        let os_type = match os_override {
            Some(value) => OsType::from_str(value).unwrap(),
            None => infer_os_type(file_name),
        };

        let version = match version_override {
            Some(value) => value.to_string(),
            None => extract_version(file_name).unwrap_or_else(|| "unknown".to_string()),
        };

        let kernel = match kernel_override {
            Some(value) => value.to_string(),
            None => "unknown".to_string(),
        };

        Ok(FirmwareTarget {
            os_type,
            version,
            kernel,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompatibilityEntry {
    pub hardware: HardwareProfile,
    pub firmware: FirmwareTarget,
    pub status: CompatibilityStatus,
    pub issues: Vec<KnownIssue>,
    pub required_patches: Vec<PatchId>,
    pub confidence: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum CompatibilityStatus {
    Works,
    WorksWithPatches,
    Broken,
    Untested,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum KnownIssue {
    Wifi5GhzIntermittent,
    HdmiCecNoAudio,
    DdrTrainingFail,
    GpuKernelPanic,
    GpuBlobMissing,
    WifiCalibrationMissing,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum PatchId {
    BrcmfmacFix5Ghz,
    HdmiCecAudioWorkaround,
    MaliBlobExtract,
    Ap6212NvramFix,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompatibilityReport {
    pub hardware: HardwareProfile,
    pub firmware: FirmwareTarget,
    pub status: CompatibilityStatus,
    pub issues: Vec<KnownIssue>,
    pub required_patches: Vec<PatchId>,
    pub confidence: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PatchPlanStep {
    pub step: u8,
    pub patch: PatchId,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PatchPlan {
    pub steps: Vec<PatchPlanStep>,
    pub risk_level: String,
    pub success_probability: u8,
}

#[derive(Debug, Clone)]
pub struct CompatibilityMatrix {
    entries: Vec<CompatibilityEntry>,
}

impl CompatibilityMatrix {
    pub fn default_matrix() -> Self {
        let entries = vec![
            CompatibilityEntry {
                hardware: HardwareProfile {
                    soc: SoC::S905W,
                    pcb_variant: PcbVariant::P281,
                    ram_vendor: RamVendor::Samsung,
                    wifi_chip: WifiChip::AP6212,
                    emmc_vendor: EmmcVendor::Unknown("unknown".to_string()),
                    hdmi_phy: HdmiPhyVersion::Unknown("unknown".to_string()),
                },
                firmware: FirmwareTarget {
                    os_type: OsType::Android,
                    version: "11".to_string(),
                    kernel: "unknown".to_string(),
                },
                status: CompatibilityStatus::WorksWithPatches,
                issues: vec![KnownIssue::Wifi5GhzIntermittent, KnownIssue::GpuBlobMissing],
                required_patches: vec![PatchId::BrcmfmacFix5Ghz, PatchId::MaliBlobExtract],
                confidence: 85,
            },
            CompatibilityEntry {
                hardware: HardwareProfile {
                    soc: SoC::S905W,
                    pcb_variant: PcbVariant::P282,
                    ram_vendor: RamVendor::Hynix,
                    wifi_chip: WifiChip::Unknown("unknown".to_string()),
                    emmc_vendor: EmmcVendor::Unknown("unknown".to_string()),
                    hdmi_phy: HdmiPhyVersion::Unknown("unknown".to_string()),
                },
                firmware: FirmwareTarget {
                    os_type: OsType::Android,
                    version: "11".to_string(),
                    kernel: "unknown".to_string(),
                },
                status: CompatibilityStatus::Broken,
                issues: vec![KnownIssue::DdrTrainingFail, KnownIssue::GpuKernelPanic],
                required_patches: vec![],
                confidence: 95,
            },
            CompatibilityEntry {
                hardware: HardwareProfile {
                    soc: SoC::S905W,
                    pcb_variant: PcbVariant::Unknown("unknown".to_string()),
                    ram_vendor: RamVendor::Unknown("unknown".to_string()),
                    wifi_chip: WifiChip::AP6212,
                    emmc_vendor: EmmcVendor::Unknown("unknown".to_string()),
                    hdmi_phy: HdmiPhyVersion::Unknown("unknown".to_string()),
                },
                firmware: FirmwareTarget {
                    os_type: OsType::Linux,
                    version: "unknown".to_string(),
                    kernel: "6.".to_string(),
                },
                status: CompatibilityStatus::WorksWithPatches,
                issues: vec![KnownIssue::WifiCalibrationMissing],
                required_patches: vec![PatchId::Ap6212NvramFix],
                confidence: 78,
            },
        ];

        CompatibilityMatrix { entries }
    }

    pub fn evaluate(
        &self,
        hardware: HardwareProfile,
        firmware: FirmwareTarget,
    ) -> CompatibilityReport {
        let mut best_match: Option<&CompatibilityEntry> = None;

        for entry in &self.entries {
            if hardware_matches(entry, &hardware) && firmware_matches(entry, &firmware) {
                let replace = match best_match {
                    Some(existing) => entry.confidence > existing.confidence,
                    None => true,
                };

                if replace {
                    best_match = Some(entry);
                }
            }
        }

        if let Some(entry) = best_match {
            CompatibilityReport {
                hardware,
                firmware,
                status: entry.status.clone(),
                issues: entry.issues.clone(),
                required_patches: entry.required_patches.clone(),
                confidence: entry.confidence,
            }
        } else {
            CompatibilityReport {
                hardware,
                firmware,
                status: CompatibilityStatus::Untested,
                issues: vec![],
                required_patches: vec![],
                confidence: 0,
            }
        }
    }
}

pub fn build_patch_plan(report: &CompatibilityReport) -> PatchPlan {
    let mut steps = Vec::new();

    for (idx, patch) in report.required_patches.iter().enumerate() {
        let description = match patch {
            PatchId::BrcmfmacFix5Ghz => "Apply brcmfmac 5GHz stability patch",
            PatchId::HdmiCecAudioWorkaround => "Apply HDMI CEC audio workaround",
            PatchId::MaliBlobExtract => "Extract Mali GPU blob from backup",
            PatchId::Ap6212NvramFix => "Apply AP6212 NVRAM calibration",
        };

        steps.push(PatchPlanStep {
            step: (idx + 1) as u8,
            patch: patch.clone(),
            description: description.to_string(),
        });
    }

    PatchPlan {
        steps,
        risk_level: risk_level(report).to_string(),
        success_probability: report.confidence,
    }
}

fn hardware_matches(entry: &CompatibilityEntry, hardware: &HardwareProfile) -> bool {
    matches_field(&entry.hardware.soc, &hardware.soc, SoC::is_unknown)
        && matches_field(
            &entry.hardware.pcb_variant,
            &hardware.pcb_variant,
            PcbVariant::is_unknown,
        )
        && matches_field(
            &entry.hardware.ram_vendor,
            &hardware.ram_vendor,
            RamVendor::is_unknown,
        )
        && matches_field(
            &entry.hardware.wifi_chip,
            &hardware.wifi_chip,
            WifiChip::is_unknown,
        )
        && matches_field(
            &entry.hardware.emmc_vendor,
            &hardware.emmc_vendor,
            EmmcVendor::is_unknown,
        )
        && matches_field(
            &entry.hardware.hdmi_phy,
            &hardware.hdmi_phy,
            HdmiPhyVersion::is_unknown,
        )
}

fn firmware_matches(entry: &CompatibilityEntry, firmware: &FirmwareTarget) -> bool {
    if !entry.firmware.os_type.is_unknown() && entry.firmware.os_type != firmware.os_type {
        return false;
    }

    if !entry.firmware.version.is_empty()
        && entry.firmware.version != "unknown"
        && !firmware.version.starts_with(&entry.firmware.version)
    {
        return false;
    }

    if !entry.firmware.kernel.is_empty()
        && entry.firmware.kernel != "unknown"
        && !firmware.kernel.starts_with(&entry.firmware.kernel)
    {
        return false;
    }

    true
}

fn matches_field<T: PartialEq>(entry: &T, value: &T, is_unknown: fn(&T) -> bool) -> bool {
    if is_unknown(entry) {
        return true;
    }

    if is_unknown(value) {
        return false;
    }

    entry == value
}

fn risk_level(report: &CompatibilityReport) -> &'static str {
    match report.status {
        CompatibilityStatus::Works => "Low",
        CompatibilityStatus::WorksWithPatches => "Medium",
        CompatibilityStatus::Broken => "High",
        CompatibilityStatus::Untested => "Unknown",
    }
}

fn infer_os_type(name: &str) -> OsType {
    let lower = name.to_lowercase();
    if lower.contains("android") {
        OsType::Android
    } else if lower.contains("coreelec") {
        OsType::CoreElec
    } else if lower.contains("armbian") {
        OsType::Armbian
    } else if lower.contains("linux") {
        OsType::Linux
    } else {
        OsType::Unknown("unknown".to_string())
    }
}

fn extract_version(input: &str) -> Option<String> {
    let mut current = String::new();
    for ch in input.chars() {
        if ch.is_ascii_digit() || (ch == '.' && !current.is_empty()) {
            current.push(ch);
        } else if !current.is_empty() {
            break;
        }
    }

    if current.is_empty() {
        None
    } else {
        Some(current)
    }
}

pub fn resolve_firmware_target(
    firmware_path: &Path,
    os_override: Option<&str>,
    version_override: Option<&str>,
    kernel_override: Option<&str>,
) -> Result<FirmwareTarget, AppError> {
    FirmwareTarget::from_inputs(
        firmware_path,
        os_override,
        version_override,
        kernel_override,
    )
}

pub fn resolve_hardware_profile(config: &DeviceConfig) -> HardwareProfile {
    HardwareProfile::from_device_config(config)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FirmwareRecommendation {
    pub name: String,
    pub version: String,
    pub url: String,
    pub notes: String,
}

pub fn get_recommendations(profile: &HardwareProfile) -> Vec<FirmwareRecommendation> {
    let mut recs = Vec::new();

    match profile.soc {
        SoC::S905W => {
            recs.push(FirmwareRecommendation {
                name: "SlimBoxTV".to_string(),
                version: "v15.2 (Android 9)".to_string(),
                url: "https://slimboxtv.ru/amlogic-s905w/".to_string(),
                notes: if profile.pcb_variant == PcbVariant::P282 {
                    "⚠️ Best for p282. Stable HDMI PHY.".to_string()
                } else {
                    "Recommended for p281. Smooth performance.".to_string()
                },
            });
            recs.push(FirmwareRecommendation {
                name: "Aidan's ROM".to_string(),
                version: "v7 (Android 9)".to_string(),
                url: "https://forum.xda-developers.com/t/rom-aidans-rom-android-tv-9-0/"
                    .to_string(),
                notes: "Excellent for Widevine L3 and stable streaming.".to_string(),
            });
        }
        SoC::S905X => {
            recs.push(FirmwareRecommendation {
                name: "SlimBoxTV".to_string(),
                version: "v15.0 (Android 9/11)".to_string(),
                url: "https://slimboxtv.ru/amlogic-s905x/".to_string(),
                notes: "Top choice for S905X. Supports Android 11 beta.".to_string(),
            });
        }
        SoC::RK3229 => {
            recs.push(FirmwareRecommendation {
                name: "SlimBoxTV".to_string(),
                version: "v10.0".to_string(),
                url: "https://slimboxtv.ru/rockchip-rk3229/".to_string(),
                notes: "Brings modern Android TV UI to legacy RK3229.".to_string(),
            });
        }
        _ => {}
    }

    recs
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_soc_from_str() {
        // Test Amlogic
        assert_eq!(SoC::from_str("s905w").unwrap(), SoC::S905W);
        assert_eq!(SoC::from_str("S905W").unwrap(), SoC::S905W);
        assert_eq!(SoC::from_str("amlogic_s905w").unwrap(), SoC::S905W);

        assert_eq!(SoC::from_str("s905x").unwrap(), SoC::S905X);
        assert_eq!(SoC::from_str("amlogic_s905x").unwrap(), SoC::S905X);

        // Test Rockchip
        assert_eq!(SoC::from_str("rk3229").unwrap(), SoC::RK3229);
        assert_eq!(SoC::from_str("RK3229").unwrap(), SoC::RK3229);
        assert_eq!(SoC::from_str("rk3036").unwrap(), SoC::RK3036);
        assert_eq!(SoC::from_str("rk3328").unwrap(), SoC::RK3328);
        assert_eq!(SoC::from_str("rk3399").unwrap(), SoC::RK3399);

        // Test Allwinner
        assert_eq!(SoC::from_str("h3").unwrap(), SoC::H3);
        assert_eq!(SoC::from_str("allwinner_h3").unwrap(), SoC::H3);

        // Test Unknown
        assert!(
            matches!(SoC::from_str("unknown_soc"), Ok(SoC::Unknown(ref s)) if s == "unknown_soc")
        );

        // Test parsing via string parse method
        assert_eq!("s905w".parse::<SoC>(), Ok(SoC::S905W));
    }

    #[test]
    fn test_soc_is_unknown() {
        assert!(!SoC::S905W.is_unknown());
        assert!(SoC::Unknown("test".to_string()).is_unknown());
    }

    #[test]
    fn test_os_type_from_str() {
        assert_eq!(OsType::from_str("android").unwrap(), OsType::Android);
        assert_eq!(OsType::from_str("Android").unwrap(), OsType::Android);
        assert_eq!(OsType::from_str("linux").unwrap(), OsType::Linux);
        assert_eq!(OsType::from_str("coreelec").unwrap(), OsType::CoreElec);
        assert_eq!(OsType::from_str("armbian").unwrap(), OsType::Armbian);

        assert!(
            matches!(OsType::from_str("windows"), Ok(OsType::Unknown(ref s)) if s == "windows")
        );
    }

    #[test]
    fn test_extract_version() {
        assert_eq!(
            extract_version("android-11-image.img"),
            Some("11".to_string())
        );
        assert_eq!(
            extract_version("linux-v5.10-test.bin"),
            Some("5.10".to_string())
        );
        assert_eq!(extract_version("no-version"), None);
    }

    #[test]
    fn test_infer_os_type() {
        assert_eq!(infer_os_type("my-android-box.img"), OsType::Android);
        assert_eq!(infer_os_type("CoreELEC-9.2.img"), OsType::CoreElec);
        assert_eq!(infer_os_type("Armbian_21.img"), OsType::Armbian);
        assert_eq!(infer_os_type("linux_firmware.bin"), OsType::Linux);
        assert!(matches!(infer_os_type("unknown.bin"), OsType::Unknown(_)));
    }
}
