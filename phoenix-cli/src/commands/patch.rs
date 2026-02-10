use std::path::Path;

use anyhow::Result;
use phoenix_lib::compatibility::{
    build_patch_plan, resolve_firmware_target, resolve_hardware_profile,
    CompatibilityMatrix, CompatibilityReport, PatchPlan,
};
use phoenix_lib::config::DeviceConfig;
use phoenix_lib::workflow::{Phase, PhaseStatus};
use crate::commands::phase;
use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct PatchPlanOutput {
    report: CompatibilityReport,
    plan: PatchPlan,
}

pub async fn plan(
    profile: &str,
    firmware: &str,
    os_type: Option<&str>,
    version: Option<&str>,
    kernel: Option<&str>,
    format: &str,
) -> Result<()> {
    phase::emit(Phase::PatchPlan, PhaseStatus::Started, None);
    let config = DeviceConfig::from_file(profile)?;
    config.validate().map_err(|e| anyhow::anyhow!(e.to_string()))?;

    let hardware = resolve_hardware_profile(&config);
    let firmware_target = resolve_firmware_target(
        Path::new(firmware),
        os_type,
        version,
        kernel,
    )?;

    let matrix = CompatibilityMatrix::default_matrix();
    let report = matrix.evaluate(hardware, firmware_target);
    let plan = build_patch_plan(&report);

    if format == "json" {
        let output = PatchPlanOutput { report, plan };
        println!("{}", serde_json::to_string_pretty(&output)?);
        phase::emit(Phase::PatchPlan, PhaseStatus::Completed, None);
        return Ok(());
    }

    println!("Patch Plan");
    println!("Status: {:?}", report.status);
    println!("Risk Level: {}", plan.risk_level);
    println!("Success Probability: {}%", plan.success_probability);

    if plan.steps.is_empty() {
        println!("Steps: none");
    } else {
        println!("Steps:");
        for step in &plan.steps {
            println!("  {}. {} ({:?})", step.step, step.description, step.patch);
        }
    }

    phase::emit(Phase::PatchPlan, PhaseStatus::Completed, None);
    Ok(())
}
