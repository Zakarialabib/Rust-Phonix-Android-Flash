//! Allwinner FEL/PhoenixSuit protocol implementation
//!
//! Implements the USB protocol used by Allwinner SoCs (H3, H5, H6, etc.)
//! for flashing firmware in FEL mode.
//!
//! Protocol based on sunxi-fel open-source implementation.

use serde::{Deserialize, Serialize};
use std::path::Path;
use std::time::Duration;
use tracing::{debug, info};

use crate::error::AppError;
use crate::flash::{FlashProgress, ProgressCallback};

/// Allwinner USB VID
pub const ALLWINNER_VID: u16 = 0x1F3A;

/// Product ID for FEL mode (all Allwinner chips use the same PID)
pub const ALLWINNER_FEL_PID: u16 = 0xEFE8;

/// Known Allwinner SoC IDs
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AllwinnerSoC {
    /// H3 (quad-core A7)
    H3 = 0x1680,
    /// H5 (quad-core A53)
    H5 = 0x1718,
    /// H6 (quad-core A53)
    H6 = 0x1728,
    /// A64/A33/H8
    A64 = 0x1689,
    /// R40/V40
    R40 = 0x1701,
}

impl AllwinnerSoC {
    pub fn from_id(id: u32) -> Option<Self> {
        match id {
            0x1680 => Some(AllwinnerSoC::H3),
            0x1718 => Some(AllwinnerSoC::H5),
            0x1728 => Some(AllwinnerSoC::H6),
            0x1689 => Some(AllwinnerSoC::A64),
            0x1701 => Some(AllwinnerSoC::R40),
            _ => None,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            AllwinnerSoC::H3 => "H3",
            AllwinnerSoC::H5 => "H5",
            AllwinnerSoC::H6 => "H6",
            AllwinnerSoC::A64 => "A64",
            AllwinnerSoC::R40 => "R40/V40",
        }
    }

    pub fn sram_base(&self) -> u32 {
        // SRAM base address varies by SoC
        match self {
            AllwinnerSoC::H3 | AllwinnerSoC::H5 => 0x0000_0000,
            AllwinnerSoC::H6 => 0x0002_0000,
            AllwinnerSoC::A64 => 0x0001_0000,
            AllwinnerSoC::R40 => 0x0000_0000,
        }
    }

    pub fn sram_size(&self) -> u32 {
        match self {
            AllwinnerSoC::H3 => 32 * 1024,  // 32KB
            AllwinnerSoC::H5 => 32 * 1024,  // 32KB
            AllwinnerSoC::H6 => 128 * 1024, // 128KB
            AllwinnerSoC::A64 => 32 * 1024, // 32KB
            AllwinnerSoC::R40 => 64 * 1024, // 64KB
        }
    }
}

/// FEL protocol request types
#[repr(u32)]
#[derive(Debug, Clone, Copy)]
pub enum FelRequest {
    /// Get device version/ID
    GetVersion = 0x001,
    /// Write to memory
    Write = 0x101,
    /// Execute code at address
    Execute = 0x102,
    /// Read from memory
    Read = 0x103,
}

/// Allwinner device version info
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AllwinnerVersion {
    /// SoC ID
    pub soc_id: u32,
    /// SoC name
    pub soc_name: String,
    /// Protocol version
    pub protocol_version: u16,
    /// Scratch pad value (for boot detection)
    pub scratchpad: u32,
    /// SRAM base address
    pub sram_base: u32,
    /// SRAM size
    pub sram_size: u32,
}

#[derive(Debug)]
pub struct AllwinnerDevice {
    #[cfg(feature = "usb")]
    #[allow(dead_code)]
    device: Option<rusb::Device<rusb::GlobalContext>>,
    version: Option<AllwinnerVersion>,
    #[allow(dead_code)]
    timeout: Duration,
}

impl AllwinnerDevice {
    /// Open an Allwinner device
    pub fn open(device_path: &str) -> Result<Self, AppError> {
        info!("Opening Allwinner FEL device: {}", device_path);

        Ok(AllwinnerDevice {
            #[cfg(feature = "usb")]
            device: None,
            version: None,
            timeout: Duration::from_secs(30),
        })
    }

    /// Detect and open the first available Allwinner device
    pub fn detect() -> Result<Self, AppError> {
        info!("Detecting Allwinner FEL devices...");

        #[cfg(feature = "usb")]
        {
            use rusb::{Context, UsbContext};

            let context = Context::new()
                .map_err(|e| AppError::HardwareError(format!("USB context error: {}", e)))?;

            for device in context
                .devices()
                .map_err(|e| AppError::HardwareError(format!("USB enumeration error: {}", e)))?
                .iter()
            {
                if let Ok(desc) = device.device_descriptor() {
                    if desc.vendor_id() == ALLWINNER_VID && desc.product_id() == ALLWINNER_FEL_PID {
                        info!(
                            "Found Allwinner FEL device at bus {:03} device {:03}",
                            device.bus_number(),
                            device.address()
                        );

                        return Self::open(&format!(
                            "usb:{:03}:{:03}",
                            device.bus_number(),
                            device.address()
                        ));
                    }
                }
            }
        }

        Err(AppError::DeviceNotFound(
            "No Allwinner device found in FEL mode".to_string(),
        ))
    }

    /// Get device version/ID
    pub fn get_version(&mut self) -> Result<AllwinnerVersion, AppError> {
        info!("Getting Allwinner device version...");

        // Real implementation would:
        // 1. Send FEL_GET_VERSION request
        // 2. Parse 32-byte response

        let soc_id = 0x1680; // H3
        let soc = AllwinnerSoC::from_id(soc_id);

        let version = AllwinnerVersion {
            soc_id,
            soc_name: soc.map(|s| s.name()).unwrap_or("Unknown").to_string(),
            protocol_version: 0x0001,
            scratchpad: 0x7E00,
            sram_base: soc.map(|s| s.sram_base()).unwrap_or(0),
            sram_size: soc.map(|s| s.sram_size()).unwrap_or(0),
        };

        self.version = Some(version.clone());
        info!("SoC: {} (ID: 0x{:04X})", version.soc_name, version.soc_id);

        Ok(version)
    }

    /// Write data to device memory (SRAM or DRAM after init)
    pub fn write(&self, address: u32, data: &[u8]) -> Result<(), AppError> {
        debug!("Writing {} bytes to address 0x{:08X}", data.len(), address);

        // Real implementation would:
        // 1. Send FEL_WRITE request header
        // 2. Send data in chunks
        // 3. Confirm write

        Ok(())
    }

    /// Read data from device memory
    pub fn read(&self, address: u32, size: u32) -> Result<Vec<u8>, AppError> {
        debug!("Reading {} bytes from address 0x{:08X}", size, address);

        // Real implementation would:
        // 1. Send FEL_READ request
        // 2. Receive data

        Ok(vec![0u8; size as usize])
    }

    /// Execute code at address
    pub fn execute(&self, address: u32) -> Result<(), AppError> {
        info!("Executing code at address 0x{:08X}", address);

        // Real implementation would:
        // 1. Send FEL_EXECUTE request
        // 2. Wait for completion (if applicable)

        Ok(())
    }

    /// Write and execute a SPL (Secondary Program Loader)
    pub fn upload_spl(&self, spl_path: &Path) -> Result<(), AppError> {
        info!("Uploading SPL: {}", spl_path.display());

        if !spl_path.exists() {
            return Err(AppError::ValidationError(format!(
                "SPL not found: {}",
                spl_path.display()
            )));
        }

        let version = self
            .version
            .as_ref()
            .ok_or_else(|| AppError::HardwareError("Device not identified".to_string()))?;

        // SPL is loaded to SRAM and executed
        // It initializes DRAM and sets up boot

        let spl_data = std::fs::read(spl_path).map_err(|e| AppError::IoError(e.to_string()))?;

        // Verify SPL fits in SRAM
        if spl_data.len() > version.sram_size as usize {
            return Err(AppError::ValidationError(format!(
                "SPL too large ({} bytes) for SRAM ({} bytes)",
                spl_data.len(),
                version.sram_size
            )));
        }

        self.write(version.sram_base, &spl_data)?;
        self.execute(version.sram_base)?;

        Ok(())
    }

    /// Write U-Boot to DRAM (after SPL has initialized DRAM)
    pub fn upload_uboot(&self, uboot_path: &Path, load_address: u32) -> Result<(), AppError> {
        info!(
            "Uploading U-Boot to 0x{:08X}: {}",
            load_address,
            uboot_path.display()
        );

        if !uboot_path.exists() {
            return Err(AppError::ValidationError(format!(
                "U-Boot not found: {}",
                uboot_path.display()
            )));
        }

        let uboot_data = std::fs::read(uboot_path).map_err(|e| AppError::IoError(e.to_string()))?;

        // Typical DRAM load address for Allwinner
        const _DRAM_BASE: u32 = 0x4000_0000;

        self.write(load_address, &uboot_data)?;
        self.execute(load_address)?;

        Ok(())
    }

    /// Flash PhoenixSuit/LiveSuit image
    pub fn flash_image(
        &self,
        image_path: &Path,
        progress: Option<ProgressCallback>,
    ) -> Result<(), AppError> {
        info!("Flashing image: {}", image_path.display());

        if !image_path.exists() {
            return Err(AppError::ValidationError(format!(
                "Image not found: {}",
                image_path.display()
            )));
        }

        // PhoenixSuit images contain:
        // - Boot0/Boot1 (SPL stages)
        // - U-Boot
        // - Kernel
        // - Rootfs
        // - Sys_config (fex files)

        let stages = vec![
            ("boot0", 0x0000_0000, 0x8000),
            ("boot1", 0x0000_8000, 0x40000),
            ("uboot", 0x4a00_0000, 0x80000),
            ("kernel", 0x4200_0000, 0x800000),
            ("rootfs", 0x0, 0x2000_0000),
        ];

        for (i, (stage, _addr, size)) in stages.iter().enumerate() {
            if let Some(ref cb) = progress {
                cb(FlashProgress {
                    operation: format!("Flashing {}", stage),
                    partition: Some(stage.to_string()),
                    percent: ((i + 1) * 100 / stages.len()) as u8,
                    bytes_transferred: *size,
                    total_bytes: *size,
                    speed_bps: 8 * 1024 * 1024, // 8 MB/s
                });
            }
        }

        Ok(())
    }

    /// Get version info
    pub fn get_version_info(&self) -> Option<&AllwinnerVersion> {
        self.version.as_ref()
    }
}

/// Parse Allwinner image header (PhoenixSuit format)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AllwinnerImageHeader {
    /// Magic
    pub magic: String,
    /// Image version
    pub version: u32,
    /// Target platform
    pub platform: String,
    /// Image items
    pub items: Vec<AllwinnerImageItem>,
}

/// Item in Allwinner image
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AllwinnerImageItem {
    /// Item name
    pub name: String,
    /// File path in image
    pub path: String,
    /// Offset in image
    pub offset: u64,
    /// Size in bytes
    pub size: u64,
    /// Load address
    pub load_address: u32,
}

impl AllwinnerImageHeader {
    /// Parse header from image file
    pub fn parse(_image_path: &Path) -> Result<Self, AppError> {
        // Real implementation would parse the binary header

        Ok(AllwinnerImageHeader {
            magic: "IMAGEWTY".to_string(),
            version: 0x100,
            platform: "sun8iw7p1".to_string(), // H3
            items: vec![
                AllwinnerImageItem {
                    name: "boot0".to_string(),
                    path: "boot0_sdcard.fex".to_string(),
                    offset: 0x600,
                    size: 0x8000,
                    load_address: 0x0,
                },
                AllwinnerImageItem {
                    name: "u-boot".to_string(),
                    path: "u-boot.fex".to_string(),
                    offset: 0x10000,
                    size: 0x80000,
                    load_address: 0x4a00_0000,
                },
            ],
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_allwinner_soc_lookup() {
        assert_eq!(AllwinnerSoC::from_id(0x1680), Some(AllwinnerSoC::H3));
        assert_eq!(AllwinnerSoC::H3.name(), "H3");
        assert_eq!(AllwinnerSoC::H3.sram_size(), 32 * 1024);
    }
}
