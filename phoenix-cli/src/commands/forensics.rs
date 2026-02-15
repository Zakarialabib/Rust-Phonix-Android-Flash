use crate::cli::ForensicsAction;
use crate::commands::phase;
use anyhow::Result;
use phoenix_lib::config::create_default_config;
use phoenix_lib::hardware::{generate_forensics_report, populate_config_from_detection};
use phoenix_lib::profiles::{default_profiles, ProfileDatabase};
use phoenix_lib::workflow::{Phase, PhaseStatus};
use serde::Serialize;

pub async fn run(action: ForensicsAction) -> Result<()> {
    match action {
        ForensicsAction::DeepScan { device, format } => deep_scan(device.as_deref(), &format).await,
    }
}

pub async fn deep_scan(device: Option<&str>, format: &str) -> Result<()> {
    phase::emit(Phase::Detect, PhaseStatus::Started, None);
    let report = generate_forensics_report(device).map_err(|e| anyhow::anyhow!(e.to_string()))?;

    let profiles =
        ProfileDatabase::from_file("profiles.toml").unwrap_or_else(|_| default_profiles());
    let inferred_config = report.usb_devices.first().map(|detected| {
        let profile = profiles.find(detected.vendor_id, detected.product_id);
        let name = profile
            .map(|p| p.name.as_str())
            .unwrap_or_else(|| detected.vendor_name.as_str());
        let soc = profile
            .map(|p| p.soc.as_str())
            .unwrap_or_else(|| detected.soc_model.as_deref().unwrap_or("unknown"));
        let mut config = create_default_config(&soc.to_lowercase(), name);
        populate_config_from_detection(&mut config, detected, Some(&profiles));
        config
    });

    if format == "json" {
        #[derive(Serialize)]
        #[serde(rename_all = "camelCase")]
        struct Output {
            report: phoenix_lib::hardware::ForensicsReport,
            inferred_config: Option<phoenix_lib::config::DeviceConfig>,
        }

        let output = Output {
            report,
            inferred_config,
        };
        println!("{}", serde_json::to_string_pretty(&output)?);
        phase::emit(Phase::Detect, PhaseStatus::Completed, None);
        return Ok(());
    }

    println!("Forensics Report");
    if let Some(target) = report.target_device.as_deref() {
        println!("Target: {}", target);
    }

    if report.usb_devices.is_empty() {
        println!("USB: No compatible devices detected");
    } else {
        println!("USB: {} device(s) detected", report.usb_devices.len());
        for device in &report.usb_devices {
            println!(
                "  - {} {} ({:04x}:{:04x}) [{}]",
                device.vendor_name,
                device.soc_model.as_deref().unwrap_or("Unknown"),
                device.vendor_id,
                device.product_id,
                device.mode
            );
        }
    }

    if let Some(config) = inferred_config {
        println!(
            "Inferred Config: {} ({})",
            config.device.name, config.device.soc
        );
    }

    if report.uart_ports.is_empty() {
        println!("UART: No serial ports detected");
    } else {
        println!("UART: {} port(s) available", report.uart_ports.len());
        for port in &report.uart_ports {
            println!("  - {}", port);
        }
    }

    if let Some(uart_probe) = report.uart_probe {
        println!(
            "UART Probe: {} @ {} baud (responding: {})",
            uart_probe.port, uart_probe.baud, uart_probe.is_responding
        );
    }

    phase::emit(Phase::Detect, PhaseStatus::Completed, None);
    Ok(())
}
