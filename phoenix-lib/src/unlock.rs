use crate::error::AppError;
use crate::profiles::ProfileDatabase;
use serde::{Deserialize, Serialize};
use tokio::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UnlockMethod {
    AmlogicUpdate,
    RockchipMaskrom,
    AllwinnerFel,
    FastbootOem,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UnlockInstruction {
    pub soc: String,
    pub methods: Vec<UnlockInstructionMethod>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UnlockInstructionMethod {
    pub name: String,
    pub description: String,
    pub steps: Vec<String>,
}

pub struct Unlocker;

impl Unlocker {
    pub async fn detect_mode() -> Result<String, AppError> {
        // Check for devices in various modes
        // 1. Check ADB
        let adb_output = Command::new("adb").arg("devices").output().await;

        if let Ok(output) = adb_output {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if stdout.contains("\tdevice") || stdout.contains("\trecovery") {
                return Ok("adb".to_string());
            }
        }

        // 2. Check Fastboot
        let fastboot_output = Command::new("fastboot").arg("devices").output().await;

        if let Ok(output) = fastboot_output {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if stdout.contains("\tfastboot") {
                return Ok("fastboot".to_string());
            }
        }

        // 3. Check Amlogic Update Mode (lsusb check usually, simpler placeholder here)
        // 4. Check FEL/Maskrom (lsusb)

        Ok("unknown".to_string())
    }

    pub async fn unlock_bootloader(method: UnlockMethod) -> Result<(), AppError> {
        match method {
            UnlockMethod::FastbootOem => {
                let status = Command::new("fastboot")
                    .arg("oem")
                    .arg("unlock")
                    .status()
                    .await
                    .map_err(|e| AppError::IoError(e.to_string()))?;

                if !status.success() {
                    return Err(AppError::CommandFailed(
                        "Fastboot unlock failed".to_string(),
                    ));
                }
            }
            _ => {
                return Err(AppError::CommandFailed(
                    "Method not implemented yet".to_string(),
                ))
            }
        }
        Ok(())
    }
}

pub fn get_unlock_instructions(soc: &str) -> Result<Vec<UnlockInstruction>, AppError> {
    let profiles = ProfileDatabase::from_file("profiles.toml")
        .unwrap_or_else(|_| crate::profiles::default_profiles());

    let matching_profiles: Vec<_> = profiles
        .profiles
        .iter()
        .filter(|p| p.soc.to_lowercase().contains(&soc.to_lowercase()))
        .collect();

    let mut instructions = Vec::new();

    for profile in matching_profiles {
        let mut methods = Vec::new();

        if profile.supported_modes.contains(&"maskrom".to_string()) {
            if profile.vendor_id == 0x1b8e {
                methods.push(UnlockInstructionMethod {
                    name: "Maskrom (Amlogic)".to_string(),
                    description: "Short eMMC pins to force BootROM mode".to_string(),
                    steps: vec![
                        "Open device case".to_string(),
                        "Locate eMMC chip".to_string(),
                        "Short pins 8-9 while powering on".to_string(),
                    ],
                });
            } else if profile.vendor_id == 0x2207 {
                methods.push(UnlockInstructionMethod {
                    name: "Maskrom (Rockchip)".to_string(),
                    description: "Short NAND/eMMC pins".to_string(),
                    steps: vec![
                        "Locate NAND/eMMC flash".to_string(),
                        "Short pins 29-30 (I/O lines)".to_string(),
                        "Connect USB while shorting".to_string(),
                    ],
                });
            }
        }

        if profile.supported_modes.contains(&"fel".to_string()) {
            methods.push(UnlockInstructionMethod {
                name: "FEL Mode".to_string(),
                description: "Allwinner FEL mode".to_string(),
                steps: vec![
                    "Hold FEL button if present".to_string(),
                    "Or short FEL pad to ground".to_string(),
                ],
            });
        }

        instructions.push(UnlockInstruction {
            soc: profile.soc.clone(),
            methods,
        });
    }

    Ok(instructions)
}
