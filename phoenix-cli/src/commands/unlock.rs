use anyhow::Result;
use colored::Colorize;
use phoenix_lib::hardware::{detect_devices, list_serial_ports};
use phoenix_lib::unlock::get_unlock_instructions;

pub async fn detect(method: &str, port: Option<&str>) -> Result<()> {
    println!("{}", "Phoenix Unlock Detect".bold().magenta());

    match method {
        "usb" => print_usb_devices()?,
        "uart" => print_uart_ports(port)?,
        "all" => {
            print_usb_devices()?;
            print_uart_ports(port)?;
        }
        _ => {
            println!("Unknown method: {}", method);
            println!("Available methods: usb, uart, all");
        }
    }

    Ok(())
}

pub async fn maskrom(soc: &str) -> Result<()> {
    println!("{}", "Phoenix Unlock Assistant".bold().magenta());

    let instructions = get_unlock_instructions(soc)
        .map_err(|e: phoenix_lib::error::AppError| anyhow::anyhow!(e.to_string()))?;

    if instructions.is_empty() {
        println!("No profiles found for SoC: {}", soc);
        println!("Generic unlock instructions:");
        println!("  1. Amlogic: Hold reset button inside AV port while powering on.");
        println!("  2. Rockchip: Hold recovery button while connecting USB, or short eMMC pins.");
        println!("  3. Allwinner: Hold FEL button (if present) or short eMMC pins.");
        return Ok(());
    }

    for instruction in instructions {
        println!("\nSoC: {}", instruction.soc.bold());
        for method in instruction.methods {
            println!("\n{}", method.name.bold());
            println!("{}", method.description);
            for step in method.steps {
                println!("  - {}", step);
            }
        }
    }

    Ok(())
}

pub async fn status(method: &str) -> Result<()> {
    println!("{}", "Phoenix Unlock Status".bold().magenta());

    match method {
        "usb" | "all" => {
            let devices = detect_devices().map_err(|e| anyhow::anyhow!(e.to_string()))?;
            if devices.is_empty() {
                println!("No USB devices detected.");
            } else {
                for device in devices {
                    println!(
                        "{} {:04x}:{:04x} {} {}",
                        device.vendor_name,
                        device.vendor_id,
                        device.product_id,
                        device.soc_model.unwrap_or_default(),
                        device.mode
                    );
                }
            }
        }
        _ => {}
    }

    match method {
        "uart" | "all" => {
            let ports = list_serial_ports().map_err(|e| anyhow::anyhow!(e.to_string()))?;
            if ports.is_empty() {
                println!("No UART ports detected.");
            } else {
                for port in ports {
                    println!("UART: {}", port);
                }
            }
        }
        _ => {}
    }

    Ok(())
}

fn print_usb_devices() -> Result<()> {
    let devices = detect_devices().map_err(|e| anyhow::anyhow!(e.to_string()))?;
    if devices.is_empty() {
        println!("No USB devices detected.");
    } else {
        for device in devices {
            println!(
                "{} {:04x}:{:04x} {} {}",
                device.vendor_name,
                device.vendor_id,
                device.product_id,
                device.soc_model.unwrap_or_default(),
                device.mode
            );
        }
    }

    Ok(())
}

fn print_uart_ports(port: Option<&str>) -> Result<()> {
    let ports = list_serial_ports().map_err(|e| anyhow::anyhow!(e.to_string()))?;
    if let Some(filter) = port {
        let matches: Vec<_> = ports.into_iter().filter(|p| p == filter).collect();
        if matches.is_empty() {
            println!("UART port not found: {}", filter);
        } else {
            for p in matches {
                println!("UART: {}", p);
            }
        }
    } else if ports.is_empty() {
        println!("No UART ports detected.");
    } else {
        for p in ports {
            println!("UART: {}", p);
        }
    }

    Ok(())
}
