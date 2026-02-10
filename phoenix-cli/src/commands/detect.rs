//! Device detection command

use anyhow::Result;
use phoenix_lib::hardware::{detect_devices, list_serial_ports, probe_uart, DeviceMode};
use phoenix_lib::workflow::{Phase, PhaseStatus};
use crate::commands::phase;
use tracing::info;

pub async fn run(method: &str, port: Option<&str>, format: &str) -> Result<()> {
    info!("Detecting devices using method: {}", method);
    phase::emit(Phase::Detect, PhaseStatus::Started, None);

    let mut found_devices = Vec::new();

    // USB detection
    if method == "all" || method == "usb" {
        println!("🔍 Scanning USB devices...");
        match detect_devices() {
            Ok(devices) => {
                for device in devices {
                    found_devices.push(serde_json::json!({
                        "type": "usb",
                        "vendor": device.vendor_name,
                        "soc_family": device.soc_family,
                        "soc_model": device.soc_model,
                        "mode": device.mode.to_string(),
                        "vid": format!("{:04x}", device.vendor_id),
                        "pid": format!("{:04x}", device.product_id),
                    }));

                    if format == "text" {
                        let mode_icon = match device.mode {
                            DeviceMode::Maskrom => "🔓",
                            DeviceMode::Adb => "📱",
                            DeviceMode::Fastboot => "⚡",
                            DeviceMode::Fel => "🛠️",
                            DeviceMode::Unknown => "❓",
                        };
                        println!(
                            "  {} {} {} ({:04x}:{:04x}) - {}",
                            mode_icon,
                            device.vendor_name,
                            device.soc_model.as_deref().unwrap_or("Unknown"),
                            device.vendor_id,
                            device.product_id,
                            device.mode
                        );
                    }
                }
            }
            Err(e) => {
                if format == "text" {
                    println!("  ❌ USB detection failed: {}", e);
                }
            }
        }
    }

    // UART detection
    if method == "all" || method == "uart" {
        println!("🔍 Scanning serial ports...");
        match list_serial_ports() {
            Ok(ports) => {
                if ports.is_empty() {
                    if format == "text" {
                        println!("  No serial ports found");
                    }
                } else {
                    for port_name in &ports {
                        if let Some(filter_port) = port {
                            if port_name != filter_port {
                                continue;
                            }
                        }

                        if format == "text" {
                            print!("  📡 {} ... ", port_name);
                        }

                        match probe_uart(port_name, 115200) {
                            Ok(result) => {
                                found_devices.push(serde_json::json!({
                                    "type": "uart",
                                    "port": result.port,
                                    "baud": result.baud,
                                    "responding": result.is_responding,
                                    "bootloader": result.bootloader,
                                }));

                                if format == "text" {
                                    if result.is_responding {
                                        println!("✅ Responding");
                                    } else {
                                        println!("❌ No response");
                                    }
                                }
                            }
                            Err(e) => {
                                if format == "text" {
                                    println!("❌ Error: {}", e);
                                }
                            }
                        }
                    }
                }
            }
            Err(e) => {
                if format == "text" {
                    println!("  ❌ Serial port listing failed: {}", e);
                }
            }
        }
    }

    // Output results
    if format == "json" {
        println!("{}", serde_json::to_string_pretty(&found_devices)?);
    } else {
        println!();
        if found_devices.is_empty() {
            println!("No compatible devices found.");
            println!();
            println!("Tips:");
            println!("  • For Amlogic: Hold reset while powering on, or short eMMC pins");
            println!("  • For Rockchip: Connect USB while holding recovery button");
            println!("  • Ensure USB cable supports data (not charge-only)");
        } else {
            println!("Found {} device(s)", found_devices.len());
        }
    }

    phase::emit(Phase::Detect, PhaseStatus::Completed, None);
    Ok(())
}
