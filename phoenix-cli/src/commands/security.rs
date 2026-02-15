//! Security scanning command

use crate::cli::SecurityAction;
use anyhow::Result;
use phoenix_lib::security::{SecurityReport, SecurityScanner};
use std::path::Path;
use tracing::info;

pub async fn run(action: SecurityAction) -> Result<()> {
    match action {
        SecurityAction::Scan { image, format } => scan(&image, &format).await,
    }
}

/// Scan an image for known malware
pub async fn scan(image_path: &str, format: &str) -> Result<()> {
    info!("Scanning {} for malware...", image_path);

    let path = Path::new(image_path);
    let report = SecurityScanner::scan_image(path)?;

    match format {
        "json" => {
            let json = serde_json::to_string_pretty(&report)?;
            println!("{}", json);
        }
        _ => {
            print_report(&report);
        }
    }

    if report.is_infected {
        std::process::exit(1);
    }

    Ok(())
}

fn print_report(report: &SecurityReport) {
    println!("\n{}\n", "=".repeat(60));
    println!("PHOENIX SECURITY SCAN REPORT");
    println!("{}\n", "=".repeat(60));

    println!("Scan Path: {}", report.scan_path);
    println!("Scan Type: {:?}", report.scan_type);
    println!();

    // Status summary
    println!("{}", report.summary());
    println!();

    if !report.threats.is_empty() {
        println!("THREATS DETECTED:");
        println!("{}", "-".repeat(40));

        for (i, threat) in report.threats.iter().enumerate() {
            println!();
            println!("  [{}] {} ({})", i + 1, threat.name, threat.severity);
            println!("      Path: {}", threat.path);
            println!("      Description: {}", threat.description);
            println!("      Remediation: {}", threat.remediation);
        }
        println!();
    }

    if !report.recommendations.is_empty() {
        println!("RECOMMENDATIONS:");
        println!("{}", "-".repeat(40));
        for rec in &report.recommendations {
            println!("  • {}", rec);
        }
        println!();
    }

    println!("{}", "=".repeat(60));
}
