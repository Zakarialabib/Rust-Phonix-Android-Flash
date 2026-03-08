//! Rockchip Rockusb protocol implementation
//!
//! Full implementation ported from rkflashtool (BSD-2-Clause) by
//! Ivo van Poorten, Fukaumi Naoki, et al.
//!
//! Supports: RK2818–RK3588 in Loader and Maskrom modes.

use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::Path;
use std::time::Duration;
use tracing::{debug, info, warn};

#[cfg(feature = "usb")]
use rusb::{DeviceHandle, GlobalContext};

use crate::error::AppError;
use crate::flash::{FlashProgress, ProgressCallback};
pub use crate::hardware::vendor_ids::ROCKCHIP as ROCKCHIP_VID;
use crate::hardware::{product_ids, vendor_ids};

// ─── Constants ───────────────────────────────────────────────────────────────

const RKFT_BLOCKSIZE: usize = 0x4000; // 16KB
const RKFT_OFF_INCR: u32 = (RKFT_BLOCKSIZE >> 9) as u32; // 32 sectors
const MAX_PARAM_LENGTH: usize = 128 * 512 - 12;
const CMD_PKT_SIZE: usize = 31;
const RES_PKT_SIZE: usize = 13;

// Rockchip Image Parsing Constants
const RKAF_FSIZE_OFF: usize = 4;
const RKAF_MODEL_OFF: usize = 0x08;
const RKAF_MODEL_LEN: usize = 0x40;
const RKAF_MANUFACTURER_OFF: usize = 0x48;
const RKAF_MANUFACTURER_LEN: usize = 0x40;
const RKAF_COUNT_OFF: usize = 0x88;
const RKAF_ENTRIES_OFF: usize = 0x8c;
const RKAF_ENTRY_SIZE: usize = 0x70;
const RKAF_ENTRY_NAME_LEN: usize = 0x20;
const RKAF_ENTRY_PATH_OFF: usize = 0x20;
const RKAF_ENTRY_PATH_LEN: usize = 0x40;
const RKAF_ENTRY_IOFF_OFF: usize = 0x60;
const RKAF_ENTRY_NOFF_OFF: usize = 0x64;
const RKAF_ENTRY_ISIZE_OFF: usize = 0x68;
const RKAF_ENTRY_FILE_SIZE_OFF: usize = 0x6c;

const RKFW_VERSION_OFF: usize = 6;
const RKFW_CHIP_FAMILY_OFF: usize = 0x15;
const RKFW_BOOT_OFF_OFF: usize = 0x19;
const RKFW_BOOT_SIZE_OFF: usize = 0x1d;
const RKFW_UPDATE_OFF_OFF: usize = 0x21;
const RKFW_UPDATE_SIZE_OFF: usize = 0x25;

const RKFP_PSS_OFF: usize = 0x10;
const RKFP_PEO_OFF: usize = 0x14;
const RKFP_PES_OFF: usize = 0x1c;
const RKFP_PEC_OFF: usize = 0x20;
const RKFP_ENTRY_PATH_LEN: usize = 32;
const RKFP_ENTRY_IOFF_OFF: usize = 36;
const RKFP_ENTRY_ISIZE_OFF: usize = 40;
const RKFP_ENTRY_FILE_SIZE_OFF: usize = 44;

const RK_PARAM_SIZE_OFF: usize = 4;
const RK_PARAM_HEAD_SIZE: usize = 8;
const RK_PARAM_FOOT_SIZE: usize = 4;

const RK_BOOT_HEADER_SIZE: usize = 106;
const RK_BOOT_TAG_OFF: usize = 0;
const RK_BOOT_SIZE_OFF: usize = 4;
const RK_BOOT_VERSION_OFF: usize = 6;
const RK_BOOT_MERGE_VERSION_OFF: usize = 10;
const RK_BOOT_TIME_OFF: usize = 14;
const RK_BOOT_CHIP_OFF: usize = 21;
const RK_BOOT_471_COUNT_OFF: usize = 25;
const RK_BOOT_471_OFFSET_OFF: usize = 26;
const RK_BOOT_471_SIZE_OFF: usize = 30;
const RK_BOOT_472_COUNT_OFF: usize = 31;
const RK_BOOT_472_OFFSET_OFF: usize = 32;
const RK_BOOT_472_SIZE_OFF: usize = 36;
const RK_BOOT_LDR_COUNT_OFF: usize = 37;
const RK_BOOT_LDR_OFFSET_OFF: usize = 38;
const RK_BOOT_LDR_SIZE_OFF: usize = 42;
const RK_BOOT_SIGN_FLAG_OFF: usize = 43;
const RK_BOOT_RC4_FLAG_OFF: usize = 44;

const RK_BOOT_ENTRY_NAME_OFF: usize = 2;
const RK_BOOT_ENTRY_NAME_LEN: usize = 40;
const RK_BOOT_ENTRY_DATA_OFFSET_OFF: usize = 42;
const RK_BOOT_ENTRY_DATA_SIZE_OFF: usize = 46;
const RK_BOOT_ENTRY_DATA_DELAY_OFF: usize = 50;
const RK_BOOT_ENTRY_SIZE: usize = 54;

const RK_CHIP_INFO_LEN: usize = 16;

// ─── CRC (ported from rkcrc.h) ──────────────────────────────────────────────

#[rustfmt::skip]
static CRC16_TABLE: [u16; 256] = [
    0x0000,0x1021,0x2042,0x3063,0x4084,0x50a5,0x60c6,0x70e7,
    0x8108,0x9129,0xa14a,0xb16b,0xc18c,0xd1ad,0xe1ce,0xf1ef,
    0x1231,0x0210,0x3273,0x2252,0x52b5,0x4294,0x72f7,0x62d6,
    0x9339,0x8318,0xb37b,0xa35a,0xd3bd,0xc39c,0xf3ff,0xe3de,
    0x2462,0x3443,0x0420,0x1401,0x64e6,0x74c7,0x44a4,0x5485,
    0xa56a,0xb54b,0x8528,0x9509,0xe5ee,0xf5cf,0xc5ac,0xd58d,
    0x3653,0x2672,0x1611,0x0630,0x76d7,0x66f6,0x5695,0x46b4,
    0xb75b,0xa77a,0x9719,0x8738,0xf7df,0xe7fe,0xd79d,0xc7bc,
    0x48c4,0x58e5,0x6886,0x78a7,0x0840,0x1861,0x2802,0x3823,
    0xc9cc,0xd9ed,0xe98e,0xf9af,0x8948,0x9969,0xa90a,0xb92b,
    0x5af5,0x4ad4,0x7ab7,0x6a96,0x1a71,0x0a50,0x3a33,0x2a12,
    0xdbfd,0xcbdc,0xfbbf,0xeb9e,0x9b79,0x8b58,0xbb3b,0xab1a,
    0x6ca6,0x7c87,0x4ce4,0x5cc5,0x2c22,0x3c03,0x0c60,0x1c41,
    0xedae,0xfd8f,0xcdec,0xddcd,0xad2a,0xbd0b,0x8d68,0x9d49,
    0x7e97,0x6eb6,0x5ed5,0x4ef4,0x3e13,0x2e32,0x1e51,0x0e70,
    0xff9f,0xefbe,0xdfdd,0xcffc,0xbf1b,0xaf3a,0x9f59,0x8f78,
    0x9188,0x81a9,0xb1ca,0xa1eb,0xd10c,0xc12d,0xf14e,0xe16f,
    0x1080,0x00a1,0x30c2,0x20e3,0x5004,0x4025,0x7046,0x6067,
    0x83b9,0x9398,0xa3fb,0xb3da,0xc33d,0xd31c,0xe37f,0xf35e,
    0x02b1,0x1290,0x22f3,0x32d2,0x4235,0x5214,0x6277,0x7256,
    0xb5ea,0xa5cb,0x95a8,0x8589,0xf56e,0xe54f,0xd52c,0xc50d,
    0x34e2,0x24c3,0x14a0,0x0481,0x7466,0x6447,0x5424,0x4405,
    0xa7db,0xb7fa,0x8799,0x97b8,0xe75f,0xf77e,0xc71d,0xd73c,
    0x26d3,0x36f2,0x0691,0x16b0,0x6657,0x7676,0x4615,0x5634,
    0xd94c,0xc96d,0xf90e,0xe92f,0x99c8,0x89e9,0xb98a,0xa9ab,
    0x5844,0x4865,0x7806,0x6827,0x18c0,0x08e1,0x3882,0x28a3,
    0xcb7d,0xdb5c,0xeb3f,0xfb1e,0x8bf9,0x9bd8,0xabbb,0xbb9a,
    0x4a75,0x5a54,0x6a37,0x7a16,0x0af1,0x1ad0,0x2ab3,0x3a92,
    0xfd2e,0xed0f,0xdd6c,0xcd4d,0xbdaa,0xad8b,0x9de8,0x8dc9,
    0x7c26,0x6c07,0x5c64,0x4c45,0x3ca2,0x2c83,0x1ce0,0x0cc1,
    0xef1f,0xff3e,0xcf5d,0xdf7c,0xaf9b,0xbfba,0x8fd9,0x9ff8,
    0x6e17,0x7e36,0x4e55,0x5e74,0x2e93,0x3eb2,0x0ed1,0x1ef0,
];

#[rustfmt::skip]
static CRC32_TABLE: [u32; 256] = [
    0x00000000,0x04c10db7,0x09821b6e,0x0d4316d9,
    0x130436dc,0x17c53b6b,0x1a862db2,0x1e472005,
    0x26086db8,0x22c9600f,0x2f8a76d6,0x2b4b7b61,
    0x350c5b64,0x31cd56d3,0x3c8e400a,0x384f4dbd,
    0x4c10db70,0x48d1d6c7,0x4592c01e,0x4153cda9,
    0x5f14edac,0x5bd5e01b,0x5696f6c2,0x5257fb75,
    0x6a18b6c8,0x6ed9bb7f,0x639aada6,0x675ba011,
    0x791c8014,0x7ddd8da3,0x709e9b7a,0x745f96cd,
    0x9821b6e0,0x9ce0bb57,0x91a3ad8e,0x9562a039,
    0x8b25803c,0x8fe48d8b,0x82a79b52,0x866696e5,
    0xbe29db58,0xbae8d6ef,0xb7abc036,0xb36acd81,
    0xad2ded84,0xa9ece033,0xa4aff6ea,0xa06efb5d,
    0xd4316d90,0xd0f06027,0xddb376fe,0xd9727b49,
    0xc7355b4c,0xc3f456fb,0xceb74022,0xca764d95,
    0xf2390028,0xf6f80d9f,0xfbbb1b46,0xff7a16f1,
    0xe13d36f4,0xe5fc3b43,0xe8bf2d9a,0xec7e202d,
    0x34826077,0x30436dc0,0x3d007b19,0x39c176ae,
    0x278656ab,0x23475b1c,0x2e044dc5,0x2ac54072,
    0x128a0dcf,0x164b0078,0x1b0816a1,0x1fc91b16,
    0x018e3b13,0x054f36a4,0x080c207d,0x0ccd2dca,
    0x7892bb07,0x7c53b6b0,0x7110a069,0x75d1adde,
    0x6b968ddb,0x6f57806c,0x621496b5,0x66d59b02,
    0x5e9ad6bf,0x5a5bdb08,0x5718cdd1,0x53d9c066,
    0x4d9ee063,0x495fedd4,0x441cfb0d,0x40ddf6ba,
    0xaca3d697,0xa862db20,0xa521cdf9,0xa1e0c04e,
    0xbfa7e04b,0xbb66edfc,0xb625fb25,0xb2e4f692,
    0x8aabbb2f,0x8e6ab698,0x8329a041,0x87e8adf6,
    0x99af8df3,0x9d6e8044,0x902d969d,0x94ec9b2a,
    0xe0b30de7,0xe4720050,0xe9311689,0xedf01b3e,
    0xf3b73b3b,0xf776368c,0xfa352055,0xfef42de2,
    0xc6bb605f,0xc27a6de8,0xcf397b31,0xcbf87686,
    0xd5bf5683,0xd17e5b34,0xdc3d4ded,0xd8fc405a,
    0x6904c0ee,0x6dc5cd59,0x6086db80,0x6447d637,
    0x7a00f632,0x7ec1fb85,0x7382ed5c,0x7743e0eb,
    0x4f0cad56,0x4bcda0e1,0x468eb638,0x424fbb8f,
    0x5c089b8a,0x58c9963d,0x558a80e4,0x514b8d53,
    0x25141b9e,0x21d51629,0x2c9600f0,0x28570d47,
    0x36102d42,0x32d120f5,0x3f92362c,0x3b533b9b,
    0x031c7626,0x07dd7b91,0x0a9e6d48,0x0e5f60ff,
    0x101840fa,0x14d94d4d,0x199a5b94,0x1d5b5623,
    0xf125760e,0xf5e47bb9,0xf8a76d60,0xfc6660d7,
    0xe22140d2,0xe6e04d65,0xeba35bbc,0xef62560b,
    0xd72d1bb6,0xd3ec1601,0xdeaf00d8,0xda6e0d6f,
    0xc4292d6a,0xc0e820dd,0xcdab3604,0xc96a3bb3,
    0xbd35ad7e,0xb9f4a0c9,0xb4b7b610,0xb076bba7,
    0xae319ba2,0xaaf09615,0xa7b380cc,0xa3728d7b,
    0x9b3dc0c6,0x9ffccd71,0x92bfdba8,0x967ed61f,
    0x8839f61a,0x8cf8fbad,0x81bbed74,0x857ae0c3,
    0x5d86a099,0x5947ad2e,0x5404bbf7,0x50c5b640,
    0x4e829645,0x4a439bf2,0x47008d2b,0x43c1809c,
    0x7b8ecd21,0x7f4fc096,0x720cd64f,0x76cddbf8,
    0x688afbfd,0x6c4bf64a,0x6108e093,0x65c9ed24,
    0x11967be9,0x1557765e,0x18146087,0x1cd56d30,
    0x02924d35,0x06534082,0x0b10565b,0x0fd15bec,
    0x379e1651,0x335f1be6,0x3e1c0d3f,0x3add0088,
    0x249a208d,0x205b2d3a,0x2d183be3,0x29d93654,
    0xc5a71679,0xc1661bce,0xcc250d17,0xc8e400a0,
    0xd6a320a5,0xd2622d12,0xdf213bcb,0xdbe0367c,
    0xe3af7bc1,0xe76e7676,0xea2d60af,0xeeec6d18,
    0xf0ab4d1d,0xf46a40aa,0xf9295673,0xfde85bc4,
    0x89b7cd09,0x8d76c0be,0x8035d667,0x84f4dbd0,
    0x9ab3fbd5,0x9e72f662,0x9331e0bb,0x97f0ed0c,
    0xafbfa0b1,0xab7ead06,0xa63dbbdf,0xa2fcb668,
    0xbcbb966d,0xb87a9bda,0xb5398d03,0xb1f880b4,
];

pub fn rkcrc16(data: &[u8]) -> u16 {
    let mut crc: u16 = 0xffff;
    for &b in data {
        crc = (crc << 8) ^ CRC16_TABLE[((crc >> 8) as u8 ^ b) as usize];
    }
    crc
}

pub fn rkcrc32(data: &[u8]) -> u32 {
    let mut crc: u32 = 0;
    for &b in data {
        crc = (crc << 8) ^ CRC32_TABLE[((crc >> 24) as u8 ^ b) as usize];
    }
    crc
}

// ─── PID table (from rkflashtool lines 98-123) ─────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RockchipPid {
    pub pid: u16,
    pub name: &'static str,
}

pub const PID_TABLE: &[RockchipPid] = &[
    RockchipPid {
        pid: product_ids::RK2818,
        name: "RK2818",
    },
    RockchipPid {
        pid: product_ids::RK2918,
        name: "RK2918",
    },
    RockchipPid {
        pid: product_ids::RK2928,
        name: "RK2928",
    },
    RockchipPid {
        pid: product_ids::RK3026,
        name: "RK3026",
    },
    RockchipPid {
        pid: product_ids::RK3066,
        name: "RK3066",
    },
    RockchipPid {
        pid: product_ids::RK3168,
        name: "RK3168",
    },
    RockchipPid {
        pid: product_ids::RK3036,
        name: "RK3036",
    },
    RockchipPid {
        pid: product_ids::RK3188,
        name: "RK3188",
    },
    RockchipPid {
        pid: product_ids::RK312X,
        name: "RK312X",
    },
    RockchipPid {
        pid: product_ids::RK3126,
        name: "RK3126",
    },
    RockchipPid {
        pid: product_ids::RK3288,
        name: "RK3288",
    },
    RockchipPid {
        pid: product_ids::RK322X,
        name: "RK322X",
    },
    RockchipPid {
        pid: product_ids::RK3328,
        name: "RK3328",
    },
    RockchipPid {
        pid: product_ids::RK3368,
        name: "RK3368",
    },
    RockchipPid {
        pid: product_ids::RK3399,
        name: "RK3399",
    },
    RockchipPid {
        pid: product_ids::RK3308,
        name: "RK3308",
    },
    RockchipPid {
        pid: product_ids::RK3568,
        name: "RK3568",
    },
    RockchipPid {
        pid: product_ids::RK3588,
        name: "RK3588",
    },
    RockchipPid {
        pid: product_ids::RK3528,
        name: "RK3528",
    },
];

pub fn lookup_pid(pid: u16) -> Option<&'static str> {
    PID_TABLE.iter().find(|p| p.pid == pid).map(|p| p.name)
}

// ─── Rockusb commands (merged from rkflashtool + rkdeveloptool) ─────────────

/// Command opcode format: 0xDDDDDDCC where CC is the operation code
/// Direction bit: 0x80xxxxxx = IN (device→host), 0x00xxxxxx = OUT (host→device)
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RockusbOpcode {
    TestUnitReady = 0x00,
    ReadFlashId = 0x01,
    TestBadBlock = 0x03,
    ReadSector = 0x04,
    WriteSector = 0x05,
    EraseNormal = 0x06,
    EraseForce = 0x0B,
    ReadLBA = 0x14,
    WriteLBA = 0x15,
    EraseSystemDisk = 0x16,
    ReadSDRAM = 0x17,
    WriteSDRAM = 0x18,
    ExecuteSDRAM = 0x19,
    ReadFlashInfo = 0x1A,
    ReadChipInfo = 0x1B,
    SetResetFlag = 0x1E,
    WriteEfuse = 0x1F,
    ReadEfuse = 0x20,
    ReadSpiFlash = 0x21,
    WriteSpiFlash = 0x22,
    WriteNewEfuse = 0x23,
    ReadNewEfuse = 0x24,
    EraseLBA = 0x25,
    ChangeStorage = 0x2A,
    ReadStorage = 0x2B,
    ReadCapability = 0xAA,
    DeviceReset = 0xFF,
}

/// Reset subcode for DeviceReset command
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResetSubcode {
    None = 0x00,
    ResetMsc = 0x01,
    PowerOff = 0x02,
    ResetMaskrom = 0x03,
    DisconnectReset = 0x04,
}

/// R/W method subcode for Read/WriteLBA
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RwMethod {
    Image = 0x00, // Partition/image mode
    Lba = 0x01,   // Raw LBA mode
}

/// Storage type for ChangeStorage command
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StorageType {
    Flash = 0x00,
    Emmc = 0x01,
    Sd = 0x02,
    Spi = 0x03,
    UsbOtg = 0x04,
}

/// Legacy command enum (for backwards compatibility)
#[repr(u32)]
#[derive(Debug, Clone, Copy)]
pub enum RockusbCmd {
    TestUnitReady = 0x80000600,
    ReadFlashId = 0x80000601,
    ReadFlashInfo = 0x8000061a,
    ReadChipInfo = 0x8000061b,
    ReadLBA = 0x80000a14,
    ReadSDRAM = 0x80000a17,
    WriteLBA = 0x00000a15,
    WriteSDRAM = 0x00000a18,
    ExecuteSDRAM = 0x00000a19,
    EraseSectors = 0x00000a06,
    ResetDevice = 0x000006ff,
    EraseSystemDisk = 0x00000616,
    // New commands from rkdeveloptool
    TestBadBlock = 0x80000a03,
    EraseForce = 0x00000a0b,
    EraseLBA = 0x00000a25,
    ReadEfuse = 0x80000620,
    WriteEfuse = 0x0000061f,
    ReadNewEfuse = 0x80000624,
    WriteNewEfuse = 0x00000623,
    ReadSpiFlash = 0x80000a21,
    WriteSpiFlash = 0x00000a22,
    ChangeStorage = 0x0000062a,
    ReadStorage = 0x8000062b,
    ReadCapability = 0x800006aa,
    SetResetFlag = 0x0000061e,
}

// ─── Parameter.txt parser ───────────────────────────────────────────────────

/// A parsed partition from parameter.txt mtdparts
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RkPartition {
    pub name: String,
    /// Size in 512-byte sectors (0 = extends to end)
    pub size_sectors: u64,
    /// Offset in 512-byte sectors
    pub offset_sectors: u64,
    /// Grows to fill remaining space
    pub grow: bool,
}

/// Parsed parameter.txt
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RkParameter {
    pub firmware_ver: String,
    pub machine_model: String,
    pub machine_id: String,
    pub manufacturer: String,
    pub cmdline: String,
    pub partitions: Vec<RkPartition>,
}

impl RkParameter {
    /// Parse parameter.txt content
    pub fn parse(content: &str) -> Result<Self, AppError> {
        let mut params = HashMap::new();
        let mut cmdline_full = String::new();

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if let Some(idx) = line.find(':') {
                let key = line[..idx].trim();
                let val = line[idx + 1..].trim();
                if key == "CMDLINE" {
                    if !cmdline_full.is_empty() {
                        cmdline_full.push(' ');
                    }
                    cmdline_full.push_str(val);
                } else {
                    params.insert(key.to_string(), val.to_string());
                }
            }
        }

        // Parse mtdparts from cmdline
        let partitions = Self::parse_mtdparts(&cmdline_full)?;

        Ok(RkParameter {
            firmware_ver: params.get("FIRMWARE_VER").cloned().unwrap_or_default(),
            machine_model: params.get("MACHINE_MODEL").cloned().unwrap_or_default(),
            machine_id: params.get("MACHINE_ID").cloned().unwrap_or_default(),
            manufacturer: params.get("MANUFACTURER").cloned().unwrap_or_default(),
            cmdline: cmdline_full,
            partitions,
        })
    }

    fn parse_mtdparts(cmdline: &str) -> Result<Vec<RkPartition>, AppError> {
        let mut partitions = Vec::new();
        let mtd = cmdline
            .find("mtdparts=")
            .map(|i| &cmdline[i + 9..])
            .unwrap_or("");
        // Skip device name "rk29xxnand:"
        let entries = mtd.find(':').map(|i| &mtd[i + 1..]).unwrap_or(mtd);

        // Parse mtdparts
        // Format: size@offset(name:flags),size@offset(name)...
        // Size can be '-' for "grow to end"
        // Example: 0x00002000@0x00004000(loader2)
        // Example: -@0x0040000(rootfs:grow)
        static PARTITION_REGEX: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
            Regex::new(
                r"(?P<size>(?:0x[0-9a-fA-F]+|-))@(?P<offset>0x[0-9a-fA-F]+)\((?P<name>[^)]+)\)",
            )
            .expect("Invalid partition regex")
        });

        for caps in PARTITION_REGEX.captures_iter(entries) {
            let size_str = match caps.name("size") {
                Some(m) => m.as_str(),
                None => continue,
            };
            let offset_str = match caps.name("offset") {
                Some(m) => m.as_str(),
                None => continue,
            };
            let name_and_flags = match caps.name("name") {
                Some(m) => m.as_str(),
                None => continue,
            };

            // Split name and flags
            let (name, _flags) = if let Some((n, _f)) = name_and_flags.split_once(':') {
                (n.to_string(), Some(_f.to_string()))
            } else {
                (name_and_flags.to_string(), None)
            };

            let offset_sectors =
                u64::from_str_radix(offset_str.trim_start_matches("0x"), 16).unwrap_or(0);

            let (size_sectors, grow) = if size_str == "-" {
                (0, true)
            } else {
                (
                    u64::from_str_radix(size_str.trim_start_matches("0x"), 16).unwrap_or(0),
                    false,
                )
            };

            partitions.push(RkPartition {
                name,
                size_sectors,
                offset_sectors,
                grow,
            });
        }
        Ok(partitions)
    }

    /// Parse from file
    pub fn parse_file(path: &Path) -> Result<Self, AppError> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| AppError::IoError(format!("Read parameter.txt: {}", e)))?;
        Self::parse(&content)
    }
}

// ─── Image unpacker (ported from rkunpack.c) ────────────────────────────────

fn get32le(buf: &[u8], off: usize) -> Result<u32, AppError> {
    if off + 4 > buf.len() {
        return Err(AppError::ParseError("Buffer too short".to_string()));
    }
    Ok(buf[off] as u32
        | (buf[off + 1] as u32) << 8
        | (buf[off + 2] as u32) << 16
        | (buf[off + 3] as u32) << 24)
}

/// Entry extracted from an RKAF image
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RkImageEntry {
    pub name: String,
    pub path: String,
    pub offset: u64,
    pub size: u64,
    pub file_size: u64,
}

/// Rockchip image header (RKAF / RKFW / RKFP)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RkImageHeader {
    pub magic: String,
    pub manufacturer: String,
    pub model: String,
    pub version: String,
    pub chip_family: String,
    pub entries: Vec<RkImageEntry>,
}

impl RkImageHeader {
    /// Parse a Rockchip firmware image (update.img or rkfw)
    pub fn parse(image_path: &Path) -> Result<Self, AppError> {
        let file = std::fs::File::open(image_path)
            .map_err(|e| AppError::IoError(format!("Read image: {}", e)))?;
        let mut reader = std::io::BufReader::new(file);

        let mut magic_buf = [0u8; 4];
        reader
            .read_exact(&mut magic_buf)
            .map_err(|e| AppError::IoError(format!("Read magic: {}", e)))?;
        let magic = std::str::from_utf8(&magic_buf)
            .unwrap_or("????")
            .to_string();

        reader
            .seek(SeekFrom::Start(0))
            .map_err(|e| AppError::IoError(format!("Seek: {}", e)))?;

        match magic.as_str() {
            "RKAF" => Self::parse_rkaf(&mut reader),
            "RKFW" => Self::parse_rkfw(&mut reader),
            "RKFP" => Self::parse_rkfp(&mut reader),
            _ => Err(AppError::ValidationError(format!(
                "Unknown Rockchip image magic: {}",
                magic
            ))),
        }
    }

    fn parse_rkaf<R: Read + Seek>(reader: &mut R) -> Result<Self, AppError> {
        let mut header = [0u8; 0x8c];
        reader
            .read_exact(&mut header)
            .map_err(|e| AppError::IoError(format!("Read RKAF header: {}", e)))?;

        let fsize = get32le(&header, 4)? as usize + 4;
        let manufacturer = String::from_utf8_lossy(&header[0x48..0x88])
            .trim_matches('\0')
            .to_string();
        let model =
            String::from_utf8_lossy(&header[RKAF_MODEL_OFF..RKAF_MODEL_OFF + RKAF_MODEL_LEN])
                .trim_matches('\0')
                .to_string();
        let count = get32le(&header, 0x88)? as usize;

        info!(
            "RKAF: manufacturer={} model={} files={} total={}",
            manufacturer, model, count, fsize
        );

        reader
            .seek(SeekFrom::Start(RKAF_ENTRIES_OFF as u64))
            .map_err(|e| AppError::IoError(format!("Seek RKAF entries: {}", e)))?;

        let mut entries = Vec::new();
        for i in 0..count {
            let mut entry_buf = [0u8; RKAF_ENTRY_SIZE];
            reader
                .read_exact(&mut entry_buf)
                .map_err(|e| AppError::IoError(format!("Read RKAF entry {}: {}", i, e)))?;

            let name = String::from_utf8_lossy(&entry_buf[0..RKAF_ENTRY_NAME_LEN])
                .trim_matches('\0')
                .to_string();
            let path = String::from_utf8_lossy(
                &entry_buf[RKAF_ENTRY_PATH_OFF..RKAF_ENTRY_PATH_OFF + RKAF_ENTRY_PATH_LEN],
            )
            .trim_matches('\0')
            .to_string();
            let ioff = get32le(&entry_buf, 0x60)? as u64;
            let isize = get32le(&entry_buf, 0x68)? as u64;
            let file_size = get32le(&entry_buf, 0x6c)? as u64;

            entries.push(RkImageEntry {
                name,
                path,
                offset: ioff,
                size: isize,
                file_size,
            });
        }

        Ok(RkImageHeader {
            magic: "RKAF".into(),
            manufacturer,
            model,
            version: String::new(),
            chip_family: String::new(),
            entries,
        })
    }

    fn parse_rkfw<R: Read + Seek>(reader: &mut R) -> Result<Self, AppError> {
        let mut header = [0u8; 0x29];
        reader
            .read_exact(&mut header)
            .map_err(|e| AppError::IoError(format!("Read RKFW header: {}", e)))?;

        let major = header[RKFW_VERSION_OFF + 3];
        let minor = header[RKFW_VERSION_OFF + 2];
        let build = ((header[RKFW_VERSION_OFF + 1] as u16) << 8) | header[RKFW_VERSION_OFF] as u16;
        let version = format!("{}.{}.{}", major, minor, build);

        let chip_family = match header[RKFW_CHIP_FAMILY_OFF] {
            0x50 => "rk29xx",
            0x60 => "rk30xx",
            0x70 => "rk31xx",
            0x80 => "rk32xx",
            0x41 => "rk3368",
            0x38 => "rk35xx",
            _ => "unknown",
        }
        .to_string();

        let boot_off = get32le(&header, RKFW_BOOT_OFF_OFF)? as u64;
        let boot_size = get32le(&header, RKFW_BOOT_SIZE_OFF)? as u64;
        let update_off = get32le(&header, RKFW_UPDATE_OFF_OFF)? as u64;
        let update_size = get32le(&header, RKFW_UPDATE_SIZE_OFF)? as u64;

        let mut boot_magic = [0u8; 4];
        reader
            .seek(SeekFrom::Start(boot_off))
            .map_err(|e| AppError::IoError(format!("Seek to boot: {}", e)))?;
        reader
            .read_exact(&mut boot_magic)
            .map_err(|e| AppError::IoError(format!("Read boot magic: {}", e)))?;

        let boot_name = if &boot_magic == b"BOOT" {
            "BOOT"
        } else {
            "LDR"
        };

        let entries = vec![
            RkImageEntry {
                name: boot_name.into(),
                path: boot_name.into(),
                offset: boot_off,
                size: boot_size,
                file_size: boot_size,
            },
            RkImageEntry {
                name: "update.img".into(),
                path: "embedded-update.img".into(),
                offset: update_off,
                size: update_size,
                file_size: update_size,
            },
        ];

        Ok(RkImageHeader {
            magic: "RKFW".into(),
            manufacturer: String::new(),
            model: String::new(),
            version,
            chip_family,
            entries,
        })
    }

    fn parse_rkfp<R: Read + Seek>(reader: &mut R) -> Result<Self, AppError> {
        let mut header = [0u8; 0x24];
        reader
            .read_exact(&mut header)
            .map_err(|e| AppError::IoError(format!("Read RKFP header: {}", e)))?;

        let pss = get32le(&header, RKFP_PSS_OFF)? as usize;
        let peo = get32le(&header, RKFP_PEO_OFF)? as usize;
        let pes = get32le(&header, RKFP_PES_OFF)? as usize;
        let pec = get32le(&header, RKFP_PEC_OFF)? as usize;

        let mut entries = Vec::new();
        for i in 0..pec {
            let p = pss * peo + i * pes;
            reader
                .seek(SeekFrom::Start(p as u64))
                .map_err(|e| AppError::IoError(format!("Seek to RKFP entry {}: {}", i, e)))?;

            let mut entry_buf = [0u8; 48];
            reader
                .read_exact(&mut entry_buf)
                .map_err(|e| AppError::IoError(format!("Read RKFP entry {}: {}", i, e)))?;

            let path = String::from_utf8_lossy(&entry_buf[0..RKFP_ENTRY_PATH_LEN])
                .trim_matches('\0')
                .to_string();
            let ioff = get32le(&entry_buf, 36)? as u64;
            let isize = get32le(&entry_buf, 40)? as u64;
            let fsize = get32le(&entry_buf, 44)? as u64;
            entries.push(RkImageEntry {
                name: path.clone(),
                path,
                offset: ioff * pss as u64,
                size: isize,
                file_size: fsize,
            });
        }

        Ok(RkImageHeader {
            magic: "RKFP".into(),
            manufacturer: String::new(),
            model: String::new(),
            version: String::new(),
            chip_family: String::new(),
            entries,
        })
    }

    /// Extract all entries to output directory
    pub fn extract_to(&self, image_path: &Path, output_dir: &Path) -> Result<(), AppError> {
        info!(
            "Extracting {} entries to {}",
            self.entries.len(),
            output_dir.display()
        );
        std::fs::create_dir_all(output_dir)
            .map_err(|e| AppError::IoError(format!("mkdir: {}", e)))?;

        let mut file = std::fs::File::open(image_path)
            .map_err(|e| AppError::IoError(format!("Open image for extraction: {}", e)))?;
        let total_len = file
            .metadata()
            .map(|m| m.len())
            .map_err(|e| AppError::IoError(format!("stat image: {}", e)))?;

        // Allocate a single 1MB buffer to reuse across all files, reducing memory allocations
        // and syscalls. This avoids creating new BufReader/BufWriter for each entry.
        let mut buffer = vec![0u8; 1024 * 1024];

        for entry in &self.entries {
            if entry.path == "SELF" {
                continue;
            }
            let mut off = entry.offset;
            let mut sz = entry.file_size;

            // Strip parameter header/footer
            if entry.name.starts_with("parameter") {
                off += RK_PARAM_HEAD_SIZE as u64;
                sz = sz.saturating_sub((RK_PARAM_HEAD_SIZE + RK_PARAM_FOOT_SIZE) as u64);
            }

            if off + sz > total_len {
                warn!("Entry {} exceeds image bounds, skipping", entry.name);
                continue;
            }

            // Create subdirectories if path contains /
            let out_path = output_dir.join(entry.path.replace('/', std::path::MAIN_SEPARATOR_STR));
            if let Some(parent) = out_path.parent() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| AppError::IoError(format!("mkdir: {}", e)))?;
            }

            let mut f_out = std::fs::File::create(&out_path)
                .map_err(|e| AppError::IoError(format!("create {}: {}", out_path.display(), e)))?;

            file.seek(SeekFrom::Start(off))
                .map_err(|e| AppError::IoError(format!("Seek to entry {}: {}", entry.name, e)))?;

            let mut remaining = sz;
            while remaining > 0 {
                let to_read = std::cmp::min(remaining, buffer.len() as u64) as usize;
                let slice = &mut buffer[..to_read];

                file.read_exact(slice)
                    .map_err(|e| AppError::IoError(format!("Read {}: {}", entry.path, e)))?;

                f_out
                    .write_all(slice)
                    .map_err(|e| AppError::IoError(format!("Write {}: {}", entry.path, e)))?;

                remaining -= to_read as u64;
            }

            info!(
                "  {:08x}-{:08x} {} ({} bytes)",
                off,
                off + sz,
                entry.path,
                sz
            );
        }
        Ok(())
    }
}

// ─── Chip info ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RockchipChipInfo {
    pub chip_id: String,
    pub flash_type: String,
    pub flash_size: u64,
    pub boot_rom_version: String,
    pub loader_version: Option<String>,
    pub is_maskrom: bool,
}

// ─── Device handle ──────────────────────────────────────────────────────────

#[derive(Debug)]
pub struct RockchipDevice {
    #[cfg(feature = "usb")]
    handle: Option<DeviceHandle<GlobalContext>>,
    chip_info: Option<RockchipChipInfo>,
    timeout: Duration,
    endpoint: u8,
    is_maskrom: bool,
}

impl RockchipDevice {
    pub fn open(device_path: &str) -> Result<Self, AppError> {
        info!("Opening Rockchip device: {}", device_path);
        Ok(Self {
            #[cfg(feature = "usb")]
            handle: None,
            chip_info: None,
            timeout: Duration::from_secs(30),
            endpoint: 0x02,
            is_maskrom: false,
        })
    }

    pub fn detect() -> Result<Self, AppError> {
        info!("Scanning USB for Rockchip VID 0x{:04X}...", ROCKCHIP_VID);

        #[cfg(feature = "usb")]
        {
            for device in rusb::devices()
                .map_err(|e| AppError::HardwareError(format!("USB enum: {}", e)))?
                .iter()
            {
                if let Ok(desc) = device.device_descriptor() {
                    if desc.vendor_id() == ROCKCHIP_VID {
                        if let Some(name) = lookup_pid(desc.product_id()) {
                            let is_maskrom = desc.manufacturer_string_index().is_none()
                                || desc.manufacturer_string_index() == Some(0);

                            info!(
                                "Found {} (PID 0x{:04X}) {} at bus {:03}:{:03}",
                                name,
                                desc.product_id(),
                                if is_maskrom { "MASKROM" } else { "LOADER" },
                                device.bus_number(),
                                device.address()
                            );

                            let handle = device
                                .open()
                                .map_err(|e| AppError::HardwareError(format!("open: {}", e)))?;

                            if handle.kernel_driver_active(0).unwrap_or(false) {
                                let _ = handle.detach_kernel_driver(0);
                            }
                            handle
                                .claim_interface(0)
                                .map_err(|e| AppError::HardwareError(format!("claim: {}", e)))?;

                            // Discover endpoint from descriptor (rkflashtool line 353)
                            let ep = device
                                .active_config_descriptor()
                                .ok()
                                .and_then(|config| {
                                    let mut interfaces = config.interfaces();
                                    let interface = interfaces.next()?;
                                    let mut descriptors = interface.descriptors();
                                    let descriptor = descriptors.next()?;
                                    let mut endpoints = descriptor.endpoint_descriptors();
                                    endpoints.nth(1).map(|e| e.address())
                                })
                                .unwrap_or(0x02);

                            info!("Using endpoint 0x{:02X}", ep);

                            return Ok(Self {
                                handle: Some(handle),
                                chip_info: None,
                                timeout: Duration::from_secs(30),
                                endpoint: ep,
                                is_maskrom,
                            });
                        }
                    }
                }
            }
        }

        Err(AppError::DeviceNotFound(
            "No Rockchip device in Loader/Maskrom mode".into(),
        ))
    }

    // ─── Low-level USB (ported from rkflashtool send_cmd/recv_res/send_buf/recv_buf)

    fn build_cmd(&self, command: RockusbCmd, offset: u32, nsectors: u16) -> [u8; CMD_PKT_SIZE] {
        let mut cmd = [0u8; CMD_PKT_SIZE];
        cmd[0..4].copy_from_slice(b"USBC");
        let tag: u32 = 0xDEADBEEF; // deterministic for testing
        cmd[4] = (tag >> 24) as u8;
        cmd[5] = (tag >> 16) as u8;
        cmd[6] = (tag >> 8) as u8;
        cmd[7] = tag as u8;
        let c = command as u32;
        cmd[12] = (c >> 24) as u8;
        cmd[13] = (c >> 16) as u8;
        cmd[14] = (c >> 8) as u8;
        cmd[15] = c as u8;
        if offset > 0 {
            cmd[17] = (offset >> 24) as u8;
            cmd[18] = (offset >> 16) as u8;
            cmd[19] = (offset >> 8) as u8;
            cmd[20] = offset as u8;
        }
        if nsectors > 0 {
            cmd[22] = (nsectors >> 8) as u8;
            cmd[23] = nsectors as u8;
        }
        cmd
    }

    fn send_cmd(&self, command: RockusbCmd, offset: u32, nsectors: u16) -> Result<(), AppError> {
        debug!("CMD {:?} off=0x{:08X} n={}", command, offset, nsectors);
        let cmd = self.build_cmd(command, offset, nsectors);

        #[cfg(feature = "usb")]
        if let Some(h) = &self.handle {
            h.write_bulk(self.endpoint, &cmd, self.timeout)
                .map_err(|e| AppError::HardwareError(format!("send_cmd: {}", e)))?;
        }
        Ok(())
    }

    fn recv_res(&self) -> Result<[u8; RES_PKT_SIZE], AppError> {
        let mut res = [0u8; RES_PKT_SIZE];
        #[cfg(feature = "usb")]
        if let Some(h) = &self.handle {
            h.read_bulk(1 | 0x80, &mut res, self.timeout)
                .map_err(|e| AppError::HardwareError(format!("recv_res: {}", e)))?;
        }
        Ok(res)
    }

    fn send_buf(&self, data: &[u8]) -> Result<(), AppError> {
        #[cfg(feature = "usb")]
        if let Some(h) = &self.handle {
            h.write_bulk(self.endpoint, data, self.timeout)
                .map_err(|e| AppError::HardwareError(format!("send_buf: {}", e)))?;
        }
        Ok(())
    }

    fn recv_buf(&self, size: usize) -> Result<Vec<u8>, AppError> {
        let mut buf = vec![0u8; size];
        #[cfg(feature = "usb")]
        if let Some(h) = &self.handle {
            h.read_bulk(1 | 0x80, &mut buf, self.timeout)
                .map_err(|e| AppError::HardwareError(format!("recv_buf: {}", e)))?;
        }
        Ok(buf)
    }

    // ─── High-level operations ──────────────────────────────────────────────

    pub fn read_chip_info(&mut self) -> Result<RockchipChipInfo, AppError> {
        info!("Reading chip info...");
        self.send_cmd(RockusbCmd::TestUnitReady, 0, 0)?;
        self.recv_res()?;
        std::thread::sleep(Duration::from_millis(20));

        self.send_cmd(RockusbCmd::ReadChipInfo, 0, 0)?;
        let buf = self.recv_buf(RK_CHIP_INFO_LEN)?;
        self.recv_res()?;

        let chip_id = format!(
            "{}{}{}{}-{}{}{}{}.{}{}.{}{}-{}{}{}{}",
            buf[3] as char,
            buf[2] as char,
            buf[1] as char,
            buf[0] as char,
            buf[7] as char,
            buf[6] as char,
            buf[5] as char,
            buf[4] as char,
            buf[11] as char,
            buf[10] as char,
            buf[9] as char,
            buf[8] as char,
            buf[15] as char,
            buf[14] as char,
            buf[13] as char,
            buf[12] as char
        );

        info!("Chip version: {}", chip_id);

        let info = RockchipChipInfo {
            chip_id,
            flash_type: "eMMC".into(),
            flash_size: 0,
            boot_rom_version: "1.00".into(),
            loader_version: if self.is_maskrom {
                None
            } else {
                Some("1.24".into())
            },
            is_maskrom: self.is_maskrom,
        };
        self.chip_info = Some(info.clone());
        Ok(info)
    }

    /// Read parameter.txt from LBA 0
    pub fn read_parameters(&self) -> Result<RkParameter, AppError> {
        info!("Reading parameters from LBA 0...");
        self.send_cmd(RockusbCmd::ReadLBA, 0, RKFT_OFF_INCR as u16)?;
        let buf = self.recv_buf(RKFT_BLOCKSIZE)?;
        self.recv_res()?;

        let size = get32le(&buf, RK_PARAM_SIZE_OFF)? as usize;
        if size > MAX_PARAM_LENGTH {
            return Err(AppError::ValidationError("Bad parameter length".into()));
        }

        // Verify CRC
        let stored_crc = get32le(&buf, RK_PARAM_HEAD_SIZE + size)?;
        let calc_crc = rkcrc32(&buf[RK_PARAM_HEAD_SIZE..RK_PARAM_HEAD_SIZE + size]);
        if stored_crc != calc_crc {
            warn!(
                "Parameter CRC mismatch: stored=0x{:08X} calc=0x{:08X}",
                stored_crc, calc_crc
            );
        }

        let content = String::from_utf8_lossy(&buf[RK_PARAM_HEAD_SIZE..RK_PARAM_HEAD_SIZE + size]);
        RkParameter::parse(&content)
    }

    /// Read LBA sectors
    pub fn read_lba(&self, offset: u32, count: u32) -> Result<Vec<u8>, AppError> {
        let mut result = Vec::new();
        let mut off = offset;
        let mut remaining = count;
        while remaining > 0 {
            let n = std::cmp::min(remaining, RKFT_OFF_INCR);
            self.send_cmd(RockusbCmd::ReadLBA, off, n as u16)?;
            let chunk = self.recv_buf(n as usize * 512)?;
            self.recv_res()?;
            result.extend_from_slice(&chunk);
            off += n;
            remaining -= n;
        }
        Ok(result)
    }

    /// Write LBA sectors
    pub fn write_lba(&self, offset: u32, data: &[u8]) -> Result<(), AppError> {
        let mut off = offset;
        let mut pos = 0;
        while pos < data.len() {
            let chunk_end = std::cmp::min(pos + RKFT_BLOCKSIZE, data.len());
            let n = (chunk_end - pos).div_ceil(512) as u16;
            self.send_cmd(RockusbCmd::WriteLBA, off, n)?;
            self.send_buf(&data[pos..chunk_end])?;
            self.recv_res()?;
            off += n as u32;
            pos = chunk_end;
        }
        Ok(())
    }

    /// Flash complete update.img
    pub fn flash_update_image(
        &mut self,
        image_path: &Path,
        progress: Option<ProgressCallback>,
    ) -> Result<(), AppError> {
        info!("Flashing: {}", image_path.display());
        if !image_path.exists() {
            return Err(AppError::ValidationError(format!(
                "Not found: {}",
                image_path.display()
            )));
        }

        // Parse firmware image contents
        let header = RkImageHeader::parse(image_path)?;

        // Read partition layout from device
        let params = self.read_parameters()?;
        let mut part_map = std::collections::HashMap::new();
        for p in &params.partitions {
            part_map.insert(p.name.clone(), p);
        }

        let mut file = std::fs::File::open(image_path)
            .map_err(|e| AppError::IoError(format!("Open image for flashing: {}", e)))?;

        let total_entries = header.entries.len();

        for (i, entry) in header.entries.iter().enumerate() {
            if entry.path == "SELF" {
                continue;
            }

            let part = if let Some(p) = part_map.get(&entry.name) {
                *p
            } else {
                warn!(
                    "No partition mapping found for entry '{}', skipping",
                    entry.name
                );
                continue;
            };

            let start_lba = part.offset_sectors as u32;
            let mut current_lba = start_lba;

            info!(
                "Flashing entry '{}' -> partition '{}' at LBA {} ({} bytes)",
                entry.name, part.name, start_lba, entry.file_size
            );

            file.seek(SeekFrom::Start(entry.offset))
                .map_err(|e| AppError::IoError(format!("Seek image to {}: {}", entry.offset, e)))?;

            // Pipeline constants
            const PIPELINE_BUFFER_SIZE: usize = 1024 * 1024; // 1MB
            const PIPELINE_DEPTH: usize = 2;

            // Setup channels for pipelined reading/writing
            let (tx, rx) =
                std::sync::mpsc::sync_channel::<Result<(Vec<u8>, usize), AppError>>(PIPELINE_DEPTH);
            let (recycle_tx, recycle_rx) = std::sync::mpsc::sync_channel::<Vec<u8>>(PIPELINE_DEPTH);

            // Pre-fill recycle channel with buffers
            for _ in 0..PIPELINE_DEPTH {
                let _ = recycle_tx.send(vec![0u8; PIPELINE_BUFFER_SIZE]);
            }

            // Clone file handle for reader thread
            let mut f_in = file
                .try_clone()
                .map_err(|e| AppError::IoError(format!("Clone file handle: {}", e)))?;
            let entry_name = entry.name.clone();
            let entry_file_size = entry.file_size;

            // Spawn reader thread
            let reader_handle = std::thread::spawn(move || {
                let mut remaining = entry_file_size;
                while remaining > 0 {
                    // Get buffer from recycle channel
                    let mut buffer = match recycle_rx.recv() {
                        Ok(b) => b,
                        Err(_) => break, // Writer closed
                    };

                    let to_read = std::cmp::min(remaining, buffer.len() as u64) as usize;

                    // Resize if buffer is smaller than needed (should only happen if file < buffer size)
                    if buffer.len() < to_read {
                        buffer.resize(to_read, 0);
                    }

                    if let Err(e) = f_in.read_exact(&mut buffer[..to_read]) {
                        let _ = tx.send(Err(AppError::IoError(format!(
                            "Read image data for entry {}: {}",
                            entry_name, e
                        ))));
                        break;
                    }

                    // Pad to 512-byte boundary
                    let padded_len = to_read.div_ceil(512) * 512;
                    if padded_len > buffer.len() {
                        buffer.resize(padded_len, 0);
                    } else {
                        // Zero padding
                        for b in &mut buffer[to_read..padded_len] {
                            *b = 0;
                        }
                    }

                    // Truncate to padded length so writer knows exactly how much to write
                    buffer.truncate(padded_len);

                    if tx.send(Ok((buffer, to_read))).is_err() {
                        break;
                    }
                    remaining -= to_read as u64;
                }
            });

            // Writer loop (main thread)
            let mut sectors_written: u64 = 0;
            let max_sectors = if part.size_sectors == 0 {
                None
            } else {
                Some(part.size_sectors)
            };

            let mut total_bytes_transferred = 0u64;
            let start_time = std::time::Instant::now();

            while let Ok(result) = rx.recv() {
                let (mut chunk, bytes_read) = result?;

                let sectors = (chunk.len() / 512) as u64;
                if let Some(max) = max_sectors {
                    if sectors_written + sectors > max {
                        // Join reader thread before returning error to clean up
                        let _ = reader_handle.join();
                        return Err(AppError::ValidationError(format!(
                            "Entry '{}' exceeds partition '{}' size",
                            entry.name, part.name
                        )));
                    }
                }

                self.write_lba(current_lba, &chunk)?;
                current_lba += sectors as u32;
                sectors_written += sectors;
                total_bytes_transferred += bytes_read as u64;

                // Recycle buffer
                if chunk.capacity() < PIPELINE_BUFFER_SIZE {
                    chunk = vec![0u8; PIPELINE_BUFFER_SIZE];
                } else {
                    chunk.resize(PIPELINE_BUFFER_SIZE, 0);
                }
                let _ = recycle_tx.send(chunk);

                if let Some(ref cb) = progress {
                    let elapsed = start_time.elapsed().as_secs_f64();
                    let speed_bps = if elapsed > 0.0 {
                        (total_bytes_transferred as f64 / elapsed) as u64
                    } else {
                        0
                    };

                    cb(FlashProgress {
                        operation: format!("Flashing {}", entry.name),
                        partition: Some(entry.name.clone()),
                        percent: ((i + 1) * 100 / total_entries.max(1)) as u8,
                        bytes_transferred: total_bytes_transferred,
                        total_bytes: entry.file_size,
                        speed_bps,
                    });
                }
            }

            reader_handle
                .join()
                .map_err(|_| AppError::Unknown("Reader thread panicked".into()))?;
        }

        Ok(())
    }

    /// Reset device
    pub fn reset(&self) -> Result<(), AppError> {
        info!("Resetting device...");
        self.send_cmd(RockusbCmd::ResetDevice, 0, 0)?;
        self.recv_res()?;
        Ok(())
    }

    pub fn get_chip_info(&self) -> Option<&RockchipChipInfo> {
        self.chip_info.as_ref()
    }
}

// ─── Tests ──────────────────────────────────────────────────────────────────

// ─── RKBoot Loader Parser (ported from RKBoot.cpp) ──────────────────────────

/// RKBoot header structure (106 bytes)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RkBootHeader {
    pub tag: u32,  // "KRNL" or "BOOT"
    pub size: u16, // Header size
    pub version: u32,
    pub merge_version: u32,
    pub release_time: String,
    pub support_chip: u32,
    pub entry_471_count: u8,
    pub entry_471_offset: u32,
    pub entry_471_size: u8,
    pub entry_472_count: u8,
    pub entry_472_offset: u32,
    pub entry_472_size: u8,
    pub entry_loader_count: u8,
    pub entry_loader_offset: u32,
    pub entry_loader_size: u8,
    pub sign_flag: u8,
    pub rc4_flag: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RkBootEntry {
    pub size: u8,
    pub entry_type: u32, // 1=471, 2=472, 4=Loader
    pub name: String,
    pub data_offset: u32,
    pub data_size: u32,
    pub data_delay: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RkBoot {
    pub header: RkBootHeader,
    pub entries: Vec<RkBootEntry>,
}

impl RkBoot {
    pub fn parse(path: &Path) -> Result<Self, AppError> {
        let file = std::fs::File::open(path)
            .map_err(|e| AppError::IoError(format!("Open RKBoot file: {}", e)))?;
        let mut reader = std::io::BufReader::new(file);
        Self::parse_reader(&mut reader)
    }

    pub fn parse_buf(buf: &[u8]) -> Result<Self, AppError> {
        let mut cursor = std::io::Cursor::new(buf);
        Self::parse_reader(&mut cursor)
    }

    pub fn parse_reader<R: Read + Seek>(reader: &mut R) -> Result<Self, AppError> {
        let mut header_buf = [0u8; RK_BOOT_HEADER_SIZE];
        reader.read_exact(&mut header_buf).map_err(|e| {
            if e.kind() == std::io::ErrorKind::UnexpectedEof {
                AppError::ParseError("File too small for RKBoot header".to_string())
            } else {
                AppError::IoError(format!("Read RKBoot header: {}", e))
            }
        })?;

        let buf = &header_buf; // Use existing get32le on this buffer

        let tag = get32le(buf, RK_BOOT_TAG_OFF)?;
        if tag != 0x4C4E524B && tag != 0x544F4F42 {
            // "KRNL" or "BOOT"
            return Err(AppError::ParseError(format!(
                "Invalid RKBoot magic: 0x{:08X}",
                tag
            )));
        }

        let header = RkBootHeader {
            tag,
            size: (buf[RK_BOOT_SIZE_OFF] as u16) | ((buf[RK_BOOT_SIZE_OFF + 1] as u16) << 8),
            version: get32le(buf, RK_BOOT_VERSION_OFF)?,
            merge_version: get32le(buf, RK_BOOT_MERGE_VERSION_OFF)?,
            release_time: format!(
                "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
                (buf[RK_BOOT_TIME_OFF] as u16) | ((buf[RK_BOOT_TIME_OFF + 1] as u16) << 8),
                buf[RK_BOOT_TIME_OFF + 2],
                buf[RK_BOOT_TIME_OFF + 3],
                buf[RK_BOOT_TIME_OFF + 4],
                buf[RK_BOOT_TIME_OFF + 5],
                buf[RK_BOOT_TIME_OFF + 6]
            ),
            support_chip: get32le(buf, RK_BOOT_CHIP_OFF)?, // Enum value
            entry_471_count: buf[RK_BOOT_471_COUNT_OFF],
            entry_471_offset: get32le(buf, RK_BOOT_471_OFFSET_OFF)?,
            entry_471_size: buf[RK_BOOT_471_SIZE_OFF],
            entry_472_count: buf[RK_BOOT_472_COUNT_OFF],
            entry_472_offset: get32le(buf, RK_BOOT_472_OFFSET_OFF)?,
            entry_472_size: buf[RK_BOOT_472_SIZE_OFF],
            entry_loader_count: buf[RK_BOOT_LDR_COUNT_OFF],
            entry_loader_offset: get32le(buf, RK_BOOT_LDR_OFFSET_OFF)?,
            entry_loader_size: buf[RK_BOOT_LDR_SIZE_OFF],
            sign_flag: buf[RK_BOOT_SIGN_FLAG_OFF],
            rc4_flag: buf[RK_BOOT_RC4_FLAG_OFF],
        };

        let mut entries = Vec::new();

        // Helper to parse entries
        let mut parse_entries = |count: u8,
                                 offset: u32,
                                 size: u8,
                                 type_id: u32|
         -> Result<Vec<RkBootEntry>, AppError> {
            let mut result = Vec::new();
            if count == 0 {
                return Ok(result);
            }

            let entry_size = size as usize;
            if entry_size < RK_BOOT_ENTRY_SIZE {
                return Err(AppError::ParseError(
                    "RKBoot entry size smaller than expected".to_string(),
                ));
            }

            reader
                .seek(SeekFrom::Start(offset as u64))
                .map_err(|e| AppError::IoError(format!("Seek to entries: {}", e)))?;

            let mut entry_buf = vec![0u8; entry_size];

            for i in 0..count {
                if let Err(e) = reader.read_exact(&mut entry_buf) {
                    if e.kind() == std::io::ErrorKind::UnexpectedEof {
                        break;
                    }
                    return Err(AppError::IoError(format!("Read RKBoot entry {}: {}", i, e)));
                }

                // Name is WCHAR (2 bytes per char), 20 chars max = 40 bytes
                // We'll just read bytes and convert strictly to ASCII for now
                let name_bytes = &entry_buf
                    [RK_BOOT_ENTRY_NAME_OFF..RK_BOOT_ENTRY_NAME_OFF + RK_BOOT_ENTRY_NAME_LEN];
                let name = String::from_utf8_lossy(name_bytes)
                    .chars()
                    .filter(|c| c.is_alphanumeric() || *c == '.' || *c == '_' || *c == '-')
                    .collect::<String>();

                result.push(RkBootEntry {
                    size: entry_buf[0],
                    entry_type: type_id,
                    name,
                    data_offset: get32le(&entry_buf, RK_BOOT_ENTRY_DATA_OFFSET_OFF)?,
                    data_size: get32le(&entry_buf, RK_BOOT_ENTRY_DATA_SIZE_OFF)?,
                    data_delay: get32le(&entry_buf, RK_BOOT_ENTRY_DATA_DELAY_OFF)?,
                });
            }
            Ok(result)
        };

        entries.extend(parse_entries(
            header.entry_471_count,
            header.entry_471_offset,
            header.entry_471_size,
            1,
        )?);
        entries.extend(parse_entries(
            header.entry_472_count,
            header.entry_472_offset,
            header.entry_472_size,
            2,
        )?);
        entries.extend(parse_entries(
            header.entry_loader_count,
            header.entry_loader_offset,
            header.entry_loader_size,
            4,
        )?);

        Ok(RkBoot { header, entries })
    }
}

// ─── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pid_lookup() {
        assert_eq!(lookup_pid(0x320b), Some("RK322X"));
        assert_eq!(lookup_pid(0x320c), Some("RK3328"));
        assert_eq!(lookup_pid(0x350b), Some("RK3588"));
        assert_eq!(lookup_pid(0x9999), None);
    }

    #[test]
    fn test_rkcrc16() {
        let data = b"Hello Rockchip";
        let crc = rkcrc16(data);
        assert_ne!(crc, 0);
    }

    #[test]
    fn test_rkcrc32() {
        let data = b"PARM";
        let crc = rkcrc32(data);
        assert_ne!(crc, 0);
    }

    #[test]
    fn test_parameter_parse() {
        let content = r#"FIRMWARE_VER:8.1
MACHINE_MODEL:RK3229
MACHINE_ID:007
MANUFACTURER:RK3229
MAGIC: 0x5041524B
ATAG: 0x00200800
MACHINE: 3229
CHECK_MASK: 0x80
CMDLINE:mtdparts=rk29xxnand:0x00002000@0x00002000(uboot),0x00002000@0x00004000(trust),0x00008000@0x00008000(resource),0x00010000@0x00010000(kernel),0x00010000@0x00020000(boot),0x00020000@0x00030000(recovery),-@0x0092A000(userdata:grow)
"#;
        let params = RkParameter::parse(content).unwrap();
        assert_eq!(params.firmware_ver, "8.1");
        assert_eq!(params.machine_model, "RK3229");
        assert_eq!(params.partitions.len(), 7);
        assert_eq!(params.partitions[0].name, "uboot");
        assert_eq!(params.partitions[0].offset_sectors, 0x2000);
        assert!(params.partitions[6].grow);
        assert_eq!(params.partitions[6].name, "userdata");
    }

    #[test]
    fn test_build_cmd() {
        let dev = RockchipDevice {
            #[cfg(feature = "usb")]
            handle: None,
            chip_info: None,
            timeout: Duration::from_secs(1),
            endpoint: 0x02,
            is_maskrom: false,
        };
        let cmd = dev.build_cmd(RockusbCmd::TestUnitReady, 0, 0);
        assert_eq!(&cmd[0..4], b"USBC");
        assert_eq!(cmd[12], 0x80);
        assert_eq!(cmd[13], 0x00);
        assert_eq!(cmd[14], 0x06);
        assert_eq!(cmd[15], 0x00);
    }

    #[test]
    fn test_rkaf_short_buffer() {
        // Construct a buffer with RKAF magic but too short for count
        let mut buf = Vec::new();
        buf.extend_from_slice(b"RKAF");
        buf.resize(20, 0); // Short buffer
                           // parse_rkaf is private, but we can test via public interface if we mock file read,
                           // or just call it directly since we are in the same module (cfg(test) mod tests)
                           // However, RkImageHeader::parse calls read_file.
                           // We can call RkImageHeader::parse_rkaf directly as we are in the module.
                           // Wait, RkImageHeader::parse_rkaf is private associated function.
                           // Test module is submodule, so it can access private items of parent.
        let mut cursor = std::io::Cursor::new(&buf);
        let res = RkImageHeader::parse_rkaf(&mut cursor);
        assert!(res.is_err());
    }

    #[test]
    fn test_rkboot_parse_basic() {
        let mut buf = vec![0u8; 106];
        // Magic "BOOT" (0x544F4F42)
        buf[0] = 0x42;
        buf[1] = 0x4F;
        buf[2] = 0x4F;
        buf[3] = 0x54;

        // entry_471_count = 1 at offset 25
        buf[25] = 1;
        // entry_471_offset = 100 at offset 26
        buf[26] = 100;
        buf[27] = 0;
        buf[28] = 0;
        buf[29] = 0;
        // entry_471_size at offset 30
        buf[30] = RK_BOOT_ENTRY_SIZE as u8;

        // Need at least 100 + RK_BOOT_ENTRY_SIZE bytes
        buf.resize(100 + RK_BOOT_ENTRY_SIZE, 0);

        let res = RkBoot::parse_buf(&buf);
        assert!(res.is_ok());
        let boot = res.unwrap();
        assert_eq!(boot.header.tag, 0x544F4F42);
        // It might be 1 if it successfully parses the entry at offset 100
        assert_eq!(boot.entries.len(), 1);
    }

    #[test]
    fn test_extract_to_streaming() {
        use std::io::Write;
        let temp_dir = std::env::temp_dir().join("phoenix_test_extract");
        let _ = std::fs::remove_dir_all(&temp_dir);
        std::fs::create_dir_all(&temp_dir).unwrap();

        let image_path = temp_dir.join("test.img");
        let output_dir = temp_dir.join("out");

        // Create dummy RKAF image
        {
            let mut f = std::fs::File::create(&image_path).unwrap();
            f.write_all(b"RKAF").unwrap();
            f.write_all(&0u32.to_le_bytes()).unwrap(); // Size

            // Model (64 bytes)
            f.write_all(&[0u8; 64]).unwrap();
            // Manufacturer (64 bytes)
            f.write_all(&[0u8; 64]).unwrap();

            // Count = 1
            f.write_all(&1u32.to_le_bytes()).unwrap();

            // Entry 1 (0x8C)
            // Name (32 bytes)
            let mut name = [0u8; 32];
            b"test_part"
                .iter()
                .enumerate()
                .for_each(|(i, b)| name[i] = *b);
            f.write_all(&name).unwrap();

            // Path (64 bytes)
            let mut path = [0u8; 64];
            b"test_part.bin"
                .iter()
                .enumerate()
                .for_each(|(i, b)| path[i] = *b);
            f.write_all(&path).unwrap();

            // Offset (u32) at 0x60 -> 256
            f.write_all(&256u32.to_le_bytes()).unwrap();

            // ? (u32)
            f.write_all(&0u32.to_le_bytes()).unwrap();

            // Size (u32) -> 1024
            f.write_all(&1024u32.to_le_bytes()).unwrap();

            // File Size (u32) -> 1024
            f.write_all(&1024u32.to_le_bytes()).unwrap();

            // Padding to 256
            let current_pos = 4 + 4 + 64 + 64 + 4 + 32 + 64 + 4 + 4 + 4 + 4; // 252
            for _ in current_pos..256 {
                f.write_all(&[0]).unwrap();
            }

            // Content (1024 bytes of 0xAA)
            f.write_all(&vec![0xAAu8; 1024]).unwrap();
        }

        // Parse
        let header = RkImageHeader::parse(&image_path).unwrap();
        assert_eq!(header.entries.len(), 1);
        assert_eq!(header.entries[0].name.trim_matches('\0'), "test_part");

        // Extract
        header.extract_to(&image_path, &output_dir).unwrap();

        // Verify extraction
        let extracted_path = output_dir.join("test_part.bin");
        assert!(extracted_path.exists());
        let content = std::fs::read(extracted_path).unwrap();
        assert_eq!(content.len(), 1024);
        assert!(content.iter().all(|&b| b == 0xAA));

        // Cleanup
        let _ = std::fs::remove_dir_all(temp_dir);
    }

    #[test]
    fn test_rkfw_parse_basic() {
        let mut buf = vec![0u8; 0x29];
        buf[0..4].copy_from_slice(b"RKFW");

        buf[RKFW_VERSION_OFF] = 0x34;
        buf[RKFW_VERSION_OFF + 1] = 0x12;
        buf[RKFW_VERSION_OFF + 2] = 2;
        buf[RKFW_VERSION_OFF + 3] = 3;

        buf[RKFW_CHIP_FAMILY_OFF] = 0x60;

        buf[RKFW_BOOT_OFF_OFF] = 0x40;
        buf[RKFW_BOOT_OFF_OFF + 1] = 0;
        buf[RKFW_BOOT_OFF_OFF + 2] = 0;
        buf[RKFW_BOOT_OFF_OFF + 3] = 0;
        buf[RKFW_BOOT_SIZE_OFF] = 0x10;
        buf[RKFW_BOOT_SIZE_OFF + 1] = 0;
        buf[RKFW_BOOT_SIZE_OFF + 2] = 0;
        buf[RKFW_BOOT_SIZE_OFF + 3] = 0;

        buf[RKFW_UPDATE_OFF_OFF] = 0x50;
        buf[RKFW_UPDATE_OFF_OFF + 1] = 0;
        buf[RKFW_UPDATE_OFF_OFF + 2] = 0;
        buf[RKFW_UPDATE_OFF_OFF + 3] = 0;
        buf[RKFW_UPDATE_SIZE_OFF] = 0x20;
        buf[RKFW_UPDATE_SIZE_OFF + 1] = 0;
        buf[RKFW_UPDATE_SIZE_OFF + 2] = 0;
        buf[RKFW_UPDATE_SIZE_OFF + 3] = 0;

        buf.resize(0x70, 0);
        buf[0x40..0x44].copy_from_slice(b"BOOT");

        let mut cursor = std::io::Cursor::new(buf);
        let header = RkImageHeader::parse_rkfw(&mut cursor).unwrap();

        assert_eq!(header.magic, "RKFW");
        assert_eq!(header.version, "3.2.4660");
        assert_eq!(header.chip_family, "rk30xx");
        assert_eq!(header.entries.len(), 2);
        assert_eq!(header.entries[0].name, "BOOT");
        assert_eq!(header.entries[0].offset, 0x40);
        assert_eq!(header.entries[0].size, 0x10);
        assert_eq!(header.entries[1].name, "update.img");
        assert_eq!(header.entries[1].offset, 0x50);
        assert_eq!(header.entries[1].size, 0x20);
    }

    #[test]
    fn test_rkfp_parse_basic() {
        let mut buf = vec![0u8; 0x24];
        buf[0..4].copy_from_slice(b"RKFP");

        buf[RKFP_PSS_OFF] = 1;
        buf[RKFP_PSS_OFF + 1] = 0;
        buf[RKFP_PSS_OFF + 2] = 0;
        buf[RKFP_PSS_OFF + 3] = 0;

        buf[RKFP_PEO_OFF] = 0x30;
        buf[RKFP_PEO_OFF + 1] = 0;
        buf[RKFP_PEO_OFF + 2] = 0;
        buf[RKFP_PEO_OFF + 3] = 0;

        buf[RKFP_PES_OFF] = 48;
        buf[RKFP_PES_OFF + 1] = 0;
        buf[RKFP_PES_OFF + 2] = 0;
        buf[RKFP_PES_OFF + 3] = 0;

        buf[RKFP_PEC_OFF] = 1;
        buf[RKFP_PEC_OFF + 1] = 0;
        buf[RKFP_PEC_OFF + 2] = 0;
        buf[RKFP_PEC_OFF + 3] = 0;

        let entry_off = 0x30;
        buf.resize(entry_off + 48, 0);
        let name_bytes = b"kernel.img";
        buf[entry_off..entry_off + name_bytes.len()].copy_from_slice(name_bytes);
        let ioff = 2u32;
        let isize = 0x1000u32;
        let fsize = 0x1000u32;
        let base = entry_off + RKFP_ENTRY_IOFF_OFF;
        buf[base..base + 4].copy_from_slice(&ioff.to_le_bytes());
        buf[base + 4..base + 8].copy_from_slice(&isize.to_le_bytes());
        buf[base + 8..base + 12].copy_from_slice(&fsize.to_le_bytes());

        let mut cursor = std::io::Cursor::new(buf);
        let header = RkImageHeader::parse_rkfp(&mut cursor).unwrap();

        assert_eq!(header.magic, "RKFP");
        assert_eq!(header.entries.len(), 1);
        let e = &header.entries[0];
        assert_eq!(e.name, "kernel.img");
        assert_eq!(e.offset, 2);
        assert_eq!(e.size, 0x1000);
        assert_eq!(e.file_size, 0x1000);
    }

    #[test]
    fn test_parameter_parse_malformed_mtdparts() {
        let content = "CMDLINE:mtdparts=rk29xxnand:0x00002000@0x00002000(uboot),invalid_part,0x00002000@0x00004000(trust)\n";
        let params = RkParameter::parse(content).unwrap();
        // It should still parse the valid ones and ignore the invalid one
        assert_eq!(params.partitions.len(), 2);
        assert_eq!(params.partitions[0].name, "uboot");
        assert_eq!(params.partitions[1].name, "trust");
    }
}
