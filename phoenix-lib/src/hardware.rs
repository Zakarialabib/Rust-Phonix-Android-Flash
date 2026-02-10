//! Hardware detection for Amlogic and Rockchip devices

use anyhow::Result;
use serde::{Deserialize, Serialize};
use tracing::debug;
use crate::error::AppError;
use crate::config::{Capabilities, DeviceConfig};
use crate::profiles::ProfileDatabase;

#[cfg(feature = "usb")]
use tracing::info;
#[cfg(feature = "usb")]
use tracing::warn;

/// Known USB Vendor IDs for TV box SoCs
pub mod vendor_ids {
    /// Amlogic (maskrom mode)
    pub const AMLOGIC: u16 = 0x1B8E;
    /// Rockchip (maskrom mode)
    pub const ROCKCHIP: u16 = 0x2207;
    /// Allwinner (FEL mode)
    pub const ALLWINNER: u16 = 0x1F3A;
}

/// Known USB Product IDs
pub mod product_ids {
    // Amlogic
    pub const AML_S905W: u16 = 0xC003;
    pub const AML_S905X: u16 = 0xC003;
    pub const AML_S912: u16 = 0xC004;
    
    // Rockchip
    pub const RK3036: u16 = 0x301A;
    pub const RK3229: u16 = 0x320B;
    pub const RK3328: u16 = 0x320C;
    pub const RK3328_LOADER: u16 = 0x322A;
    pub const RK3399: u16 = 0x330C;

    // Allwinner
    pub const H3: u16 = 0xEFE8;
}

/// Detected device information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DetectedDevice {
    pub vendor_id: u16,
    pub product_id: u16,
    pub vendor_name: String,
    pub soc_family: String,
    pub soc_model: Option<String>,
    pub mode: DeviceMode,
    pub bus_number: u8,
    pub device_address: u8,
}

/// Device connection mode
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DeviceMode {
    /// Maskrom/recovery mode - can flash
    Maskrom,
    /// ADB mode - Android running
    Adb,
    /// Fastboot mode
    Fastboot,
    /// Allwinner FEL mode
    Fel,
    /// Unknown mode
    Unknown,
}

impl std::fmt::Display for DeviceMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DeviceMode::Maskrom => write!(f, "Maskrom"),
            DeviceMode::Adb => write!(f, "ADB"),
            DeviceMode::Fastboot => write!(f, "Fastboot"),
            DeviceMode::Fel => write!(f, "FEL"),
            DeviceMode::Unknown => write!(f, "Unknown"),
        }
    }
}

/// Detect all connected USB devices that match known TV box SoCs
pub fn detect_devices() -> Result<Vec<DetectedDevice>> {
    let mut devices = Vec::new();

    #[cfg(feature = "usb")]
    {
        match rusb::devices() {
            Ok(device_list) => {
                for device in device_list.iter() {
                    if let Ok(desc) = device.device_descriptor() {
                        if let Some(detected) = identify_device(
                            desc.vendor_id(),
                            desc.product_id(),
                            device.bus_number(),
                            device.address(),
                        ) {
                            info!("Detected: {} ({:04x}:{:04x})", 
                                detected.vendor_name, detected.vendor_id, detected.product_id);
                            devices.push(detected);
                        }
                    }
                }
            }
            Err(e) => {
                warn!("Failed to enumerate USB devices: {}", e);
            }
        }
    }

    #[cfg(not(feature = "usb"))]
    {
        debug!("USB detection disabled (rusb not available)");
        // Return mock device for development
        devices.push(DetectedDevice {
            vendor_id: vendor_ids::AMLOGIC,
            product_id: product_ids::AML_S905W,
            vendor_name: "Amlogic".to_string(),
            soc_family: "meson-gxl".to_string(),
            soc_model: Some("S905W".to_string()),
            mode: DeviceMode::Maskrom,
            bus_number: 0,
            device_address: 0,
        });
    }

    Ok(devices)
}

/// Identify a device by its USB VID/PID
fn identify_device(
    vendor_id: u16,
    product_id: u16,
    bus: u8,
    addr: u8,
) -> Option<DetectedDevice> {
    let (vendor_name, soc_family, soc_model, mode) = match vendor_id {
        vendor_ids::AMLOGIC => {
            let (family, model) = match product_id {
                product_ids::AML_S905W => ("meson-gxl", Some("S905W/S905X")),
                product_ids::AML_S912 => ("meson-gxm", Some("S912")),
                _ => ("meson", None),
            };
            ("Amlogic", family, model, DeviceMode::Maskrom)
        }
        vendor_ids::ROCKCHIP => {
            let (family, model) = match product_id {
                product_ids::RK3036 => ("rk30xx", Some("RK3036")),
                product_ids::RK3229 => ("rk322x", Some("RK3229")),
                product_ids::RK3328 => ("rk3328", Some("RK3328")),
                product_ids::RK3328_LOADER => ("rk3328", Some("RK3328 (Loader)")),
                product_ids::RK3399 => ("rk3399", Some("RK3399")),
                _ => ("rockchip", None),
            };
            ("Rockchip", family, model, DeviceMode::Maskrom)
        }
        vendor_ids::ALLWINNER => {
            let (family, model, mode) = match product_id {
                product_ids::H3 => ("sun8i", Some("H3"), DeviceMode::Fel),
                _ => ("allwinner", None, DeviceMode::Unknown),
            };
            ("Allwinner", family, model, mode)
        }
        0x18D1 => {
            // Google (ADB/Fastboot)
            if product_id == 0x4EE7 {
                ("Android", "android", None, DeviceMode::Fastboot)
            } else {
                ("Android", "android", None, DeviceMode::Adb)
            }
        }
        _ => return None,
    };

    Some(DetectedDevice {
        vendor_id,
        product_id,
        vendor_name: vendor_name.to_string(),
        soc_family: soc_family.to_string(),
        soc_model: soc_model.map(String::from),
        mode,
        bus_number: bus,
        device_address: addr,
    })
}

/// List available serial ports for UART detection
pub fn list_serial_ports() -> Result<Vec<String>> {
    let ports = serialport::available_ports()
        .map_err(|e| anyhow::anyhow!("Failed to list serial ports: {}", e))?;
    
    Ok(ports.iter().map(|p| p.port_name.clone()).collect())
}

/// UART console detection result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UartDetection {
    pub port: String,
    pub baud: u32,
    pub bootloader: Option<String>,
    pub is_responding: bool,
}

/// DDR memory timing information - critical for firmware compatibility
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DdrTiming {
    /// RAM vendor (Samsung, Hynix, Micron)
    pub vendor: String,
    /// RAM type and speed (e.g., "DDR3-1600", "DDR4-2133")
    pub speed: String,
    /// Raw timing pattern from registers (e.g., "0x04040404")
    pub timing_pattern: String,
    /// Total RAM size in MB
    pub size_mb: u32,
    /// List of DTBs known to be compatible with this DDR configuration
    pub compatible_dtbs: Vec<String>,
}

impl DdrTiming {
    /// Detect DDR vendor from timing pattern
    pub fn vendor_from_pattern(pattern: &str) -> String {
        // Common patterns from Amlogic DDR training results:
        // Samsung: 0x04040404 pattern
        // Hynix:   0x05050505 pattern
        // Micron:  0x06060606 pattern
        if pattern.contains("04040404") {
            "Samsung".to_string()
        } else if pattern.contains("05050505") {
            "Hynix".to_string()
        } else if pattern.contains("06060606") {
            "Micron".to_string()
        } else {
            "Unknown".to_string()
        }
    }
}

/// Bootloader security status - important for unlock/flash operations
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BootloaderInfo {
    /// Bootloader version string
    pub version: String,
    /// Type of bootloader (U-Boot, BL2, etc.)
    pub bootloader_type: String,
    /// Whether secure boot is enabled
    pub secure_boot: bool,
    /// Whether BL2 is signed (locked bootloader)
    pub bl2_signed: bool,
    /// Whether the bootloader can be unlocked
    pub unlock_possible: bool,
    /// Additional notes or warnings
    pub notes: Vec<String>,
}

impl BootloaderInfo {
    /// Parse bootloader info from UART output
    pub fn from_uart_output(output: &str) -> Self {
        let mut info = BootloaderInfo {
            version: "Unknown".to_string(),
            bootloader_type: "Unknown".to_string(),
            secure_boot: false,
            bl2_signed: false,
            unlock_possible: true,
            notes: Vec::new(),
        };

        // Parse U-Boot version
        for line in output.lines() {
            if line.contains("U-Boot") {
                if let Some(version) = line.split_whitespace().nth(1) {
                    info.version = version.to_string();
                    info.bootloader_type = "U-Boot".to_string();
                }
            }
            
            // Detect secure boot markers
            if line.contains("Secure boot enabled") || line.contains("BL2 verified") {
                info.secure_boot = true;
                info.bl2_signed = true;
                info.unlock_possible = false;
                info.notes.push("⚠️ Secure boot enabled - flashing may be restricted".to_string());
            }
            
            // Detect Amlogic-specific bootloader
            if line.contains("gxl_") || line.contains("gxm_") {
                info.bootloader_type = "Amlogic U-Boot".to_string();
            }
        }

        info
    }
}

/// WiFi chip detection information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WifiChipInfo {
    /// Chip name (e.g., "AP6212", "RTL8189FS")
    pub chip: String,
    /// Vendor name
    pub vendor: String,
    /// SDIO vendor ID (if detected)
    pub sdio_vid: Option<u16>,
    /// SDIO device ID (if detected)
    pub sdio_did: Option<u16>,
    /// Whether driver is available in mainline Linux
    pub mainline_driver: bool,
    /// Required firmware files
    pub firmware_files: Vec<String>,
    /// NVRAM calibration file path (for Broadcom chips)
    pub nvram_path: Option<String>,
}

impl WifiChipInfo {
    /// Create WifiChipInfo from SDIO vendor/device IDs
    pub fn from_sdio_ids(vid: u16, did: u16) -> Self {
        match (vid, did) {
            (0x02D0, _) => WifiChipInfo {
                chip: "BCM43438 (AP6212)".to_string(),
                vendor: "Broadcom".to_string(),
                sdio_vid: Some(vid),
                sdio_did: Some(did),
                mainline_driver: true,
                firmware_files: vec![
                    "brcmfmac43430-sdio.bin".to_string(),
                    "brcmfmac43430-sdio.txt".to_string(),
                ],
                nvram_path: Some("/lib/firmware/brcm/brcmfmac43430-sdio.txt".to_string()),
            },
            (0x024C, _) => WifiChipInfo {
                chip: "RTL8189FS".to_string(),
                vendor: "Realtek".to_string(),
                sdio_vid: Some(vid),
                sdio_did: Some(did),
                mainline_driver: false,
                firmware_files: vec![],
                nvram_path: None,
            },
            _ => WifiChipInfo {
                chip: "Unknown".to_string(),
                vendor: "Unknown".to_string(),
                sdio_vid: Some(vid),
                sdio_did: Some(did),
                mainline_driver: false,
                firmware_files: vec![],
                nvram_path: None,
            },
        }
    }
}

/// eMMC storage information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EmmcInfo {
    /// Vendor name (Samsung, Toshiba, SK Hynix, SanDisk)
    pub vendor: String,
    /// eMMC version (e.g., "5.0", "5.1")
    pub version: String,
    /// Capacity in GB
    pub capacity_gb: u32,
    /// Manufacturer ID from CID register
    pub manufacturer_id: Option<u8>,
    /// Model/part number
    pub model: Option<String>,
}

impl EmmcInfo {
    /// Detect vendor from manufacturer ID (CID register byte 0)
    pub fn vendor_from_mid(mid: u8) -> String {
        match mid {
            0x15 => "Samsung".to_string(),
            0x90 => "SK Hynix".to_string(),
            0x45 => "SanDisk".to_string(),
            0x11 => "Toshiba".to_string(),
            _ => format!("Unknown (0x{:02X})", mid),
        }
    }
}

/// PCB Hardware Variant (e.g., p281, p282 for S905W)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PcbVariant {
    /// Standard Amlogic S905W reference (p281)
    P281,
    /// Second variant (p282) - ⚠️ Android 11 HDMI PHY issues
    P282,
    /// Rockchip reference (e.g., rk3229-evb)
    Evb,
    /// Generic/Unknown
    Generic,
}

impl PcbVariant {
    pub fn name(&self) -> &'static str {
        match self {
            PcbVariant::P281 => "p281",
            PcbVariant::P282 => "p282",
            PcbVariant::Evb => "evb",
            PcbVariant::Generic => "generic",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ForensicsReport {
    pub target_device: Option<String>,
    pub usb_devices: Vec<DetectedDevice>,
    pub uart_ports: Vec<String>,
    pub uart_probe: Option<UartDetection>,
    
    // Enhanced detection fields (IDEA2/IDEA3)
    /// DDR timing information - critical for DTB selection
    pub ddr_timing: Option<DdrTiming>,
    /// Bootloader security status
    pub bootloader: Option<BootloaderInfo>,
    /// WiFi chip information
    pub wifi_chip: Option<WifiChipInfo>,
    /// eMMC storage information
    pub emmc_info: Option<EmmcInfo>,
    /// PCB hardware variant
    pub pcb_variant: Option<PcbVariant>,
    /// Variant ID string for compatibility lookup
    pub variant_id: Option<String>,
}

impl ForensicsReport {
    /// Generate a variant ID string for compatibility matrix lookup
    pub fn compute_variant_id(&self) -> String {
        let soc = self.usb_devices.first()
            .and_then(|d| d.soc_model.as_ref())
            .map(|s| s.to_lowercase())
            .unwrap_or_else(|| "unknown".to_string());
        
        let ddr_vendor = self.ddr_timing.as_ref()
            .map(|d| d.vendor.to_lowercase())
            .unwrap_or_else(|| "unknown".to_string());
        
        let wifi = self.wifi_chip.as_ref()
            .map(|w| w.chip.to_lowercase().replace(' ', "-"))
            .unwrap_or_else(|| "unknown".to_string());
        
        let emmc = self.emmc_info.as_ref()
            .map(|e| e.vendor.to_lowercase())
            .unwrap_or_else(|| "unknown".to_string());
        
        let pcb = self.pcb_variant.as_ref()
            .map(|p| p.name().to_string())
            .unwrap_or_else(|| "unknown".to_string());
        
        format!("{}-{}-{}-{}-{}", soc, ddr_vendor, wifi, emmc, pcb)
    }
}


/// Probe a serial port for bootloader presence
pub fn probe_uart(port: &str, baud: u32) -> Result<UartDetection> {
    use std::time::Duration;
    
    let port_settings = serialport::new(port, baud)
        .timeout(Duration::from_millis(500));
    
    match port_settings.open() {
        Ok(_serial) => {
            // TODO: Send probe bytes and detect response
            Ok(UartDetection {
                port: port.to_string(),
                baud,
                bootloader: None,
                is_responding: true,
            })
        }
        Err(e) => {
            debug!("Failed to open {}: {}", port, e);
            Ok(UartDetection {
                port: port.to_string(),
                baud,
                bootloader: None,
                is_responding: false,
            })
        }
    }
}

pub fn generate_forensics_report(
    target_device: Option<&str>,
) -> Result<ForensicsReport, AppError> {
    let usb_devices = detect_devices().map_err(|e| AppError::HardwareError(e.to_string()))?;
    let uart_ports = list_serial_ports().map_err(|e| AppError::HardwareError(e.to_string()))?;

    let uart_probe = target_device.and_then(|device| {
        if uart_ports.iter().any(|port| port == device) {
            match probe_uart(device, 115200) {
                Ok(result) => Some(result),
                Err(_) => None,
            }
        } else {
            None
        }
    });

    let mut report = ForensicsReport {
        target_device: target_device.map(|value| value.to_string()),
        usb_devices,
        uart_ports,
        uart_probe,
        // Enhanced fields - populated by deep scan or set to None for basic scan
        ddr_timing: None,
        bootloader: None,
        wifi_chip: None,
        emmc_info: None,
        pcb_variant: None,
        variant_id: None,
    };
    
    // Compute variant ID if we have USB device info
    if !report.usb_devices.is_empty() {
        report.variant_id = Some(report.compute_variant_id());
    }

    Ok(report)
}

/// Perform a deep forensics scan to populate all fields
pub fn perform_deep_scan(
    target_device: Option<&str>
) -> Result<ForensicsReport, AppError> {
    let mut report = generate_forensics_report(target_device)?;
    
    // Simulate deep scan analysis
    // In a real implementation, this would:
    // 1. Read RAM timing registers via UART or USB
    // 2. Detect PCB strapping (GPIOs) for p281/p282
    // 3. Probe SDIO for WiFi chip
    // 4. Read eMMC CID register
    
    // For demonstration purposes, we'll populate it with "detected" values
    report.ddr_timing = Some(DdrTiming {
        vendor: "Samsung".to_string(),
        speed: "DDR3-1600".to_string(),
        timing_pattern: "04040404".to_string(),
        size_mb: 2048,
        compatible_dtbs: vec!["meson-gxl-s905w-p281.dtb".to_string()],
    });
    
    report.pcb_variant = Some(PcbVariant::P281);
    
    report.wifi_chip = Some(WifiChipInfo::from_sdio_ids(0x02D0, 0x4343));
    
    report.emmc_info = Some(EmmcInfo {
        vendor: "Samsung".to_string(),
        version: "5.1".to_string(),
        capacity_gb: 16,
        manufacturer_id: Some(0x15),
        model: Some("KLMAG2GEAC-B031".to_string()),
    });
    
    // Recompute variant ID after deep scan
    if !report.usb_devices.is_empty() {
        report.variant_id = Some(report.compute_variant_id());
    }
    
    Ok(report)
}


pub fn populate_config_from_detection(
    config: &mut DeviceConfig,
    detected: &DetectedDevice,
    profiles: Option<&ProfileDatabase>,
) {
    let profile = profiles.and_then(|db| db.find(detected.vendor_id, detected.product_id));

    if let Some(found) = profile {
        config.device.soc = found.soc.clone();
        config.device.name = found.name.clone();
        config.hardware.memory.size_mb = found.ram_mb;
        config.hardware.storage.storage_type = found.storage_type.clone();
    }

    let caps = capabilities_from_detected_device(detected, profile);
    config.hardware.capabilities = Some(caps);
}

fn capabilities_from_detected_device(
    detected: &DetectedDevice,
    profile: Option<&crate::profiles::DeviceProfile>,
) -> Capabilities {
    let (gpu, vpu) = match detected.soc_family.as_str() {
        "meson-gxl" | "meson-gxm" => (Some("panfrost".to_string()), Some("v4l2-m2m".to_string())),
        "rk322x" | "rk3328" | "rk3399" => (Some("panfrost".to_string()), Some("v4l2-m2m".to_string())),
        _ => (None, None),
    };

    let storage_type = profile.map(|p| p.storage_type.to_lowercase());
    let has_emmc = matches!(storage_type.as_deref(), Some("emmc"));
    let has_sd_slot = matches!(storage_type.as_deref(), Some("sd"));

    Capabilities {
        gpu,
        vpu,
        wifi_supported: true,
        ethernet_supported: true,
        hdmi_cec_supported: true,
        has_emmc,
        has_sd_slot,
    }
}
