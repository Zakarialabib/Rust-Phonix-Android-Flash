use std::path::Path;

use crate::commands::phase;
use anyhow::Result;
use phoenix_lib::compatibility::{
    resolve_firmware_target, resolve_hardware_profile, CompatibilityMatrix, CompatibilityStatus,
};
use phoenix_lib::config::DeviceConfig;
use phoenix_lib::workflow::{Phase, PhaseStatus};

pub async fn run(
    profile: &str,
    firmware: &str,
    os_type: Option<&str>,
    version: Option<&str>,
    kernel: Option<&str>,
    format: &str,
) -> Result<()> {
    phase::emit(Phase::Check, PhaseStatus::Started, None);
    let config = DeviceConfig::from_file(profile)?;
    config
        .validate()
        .map_err(|e| anyhow::anyhow!(e.to_string()))?;

    let hardware = resolve_hardware_profile(&config);
    let firmware_target = resolve_firmware_target(Path::new(firmware), os_type, version, kernel)?;

    let matrix = CompatibilityMatrix::default_matrix();
    let report = matrix.evaluate(hardware, firmware_target);

    if format == "json" {
        println!("{}", serde_json::to_string_pretty(&report)?);
        phase::emit(Phase::Check, PhaseStatus::Completed, None);
        return Ok(());
    }

    println!("Compatibility Check");
    println!("Status: {}", status_label(&report.status));
    println!("Confidence: {}%", report.confidence);

    if report.issues.is_empty() {
        println!("Issues: none");
    } else {
        println!("Issues:");
        for issue in report.issues {
            println!("  - {:?}", issue);
        }
    }

    if report.required_patches.is_empty() {
        println!("Patches: none");
    } else {
        println!("Patches:");
        for patch in report.required_patches {
            println!("  - {:?}", patch);
        }
    }

    phase::emit(Phase::Check, PhaseStatus::Completed, None);
    Ok(())
}

fn status_label(status: &CompatibilityStatus) -> &'static str {
    match status {
        CompatibilityStatus::Works => "Works",
        CompatibilityStatus::WorksWithPatches => "WorksWithPatches",
        CompatibilityStatus::Broken => "Broken",
        CompatibilityStatus::Untested => "Untested",
    }
}
