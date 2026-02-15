//! Remote configuration command

use crate::cli::RemoteAction;
use anyhow::Result;
use phoenix_lib::remote_config::RemoteConfigDatabase;
use std::fs;
use tracing::info;

pub async fn run(action: RemoteAction) -> Result<()> {
    match action {
        RemoteAction::List { format } => list(&format).await,
        RemoteAction::GenerateConf { name, output } => generate_conf(&name, &output).await,
        RemoteAction::GenerateKeylayout { name, output } => {
            generate_keylayout(&name, &output).await
        }
    }
}

/// List all available remote configurations
pub async fn list(format: &str) -> Result<()> {
    let db = RemoteConfigDatabase::default_database();

    match format {
        "json" => {
            let json = serde_json::to_string_pretty(&db)?;
            println!("{}", json);
        }
        _ => {
            println!("\nAvailable Remote Configurations:");
            println!("{}", "=".repeat(50));

            for remote in &db.remotes {
                println!();
                println!("  Name: {}", remote.name);
                println!("  Factory Code: 0x{:04X}", remote.factory_code);
                println!("  Protocol: {}", remote.protocol);
                println!("  Source: {:?}", remote.source);
                println!("  Compatible: {}", remote.compatible_devices.join(", "));
            }
            println!();
        }
    }

    Ok(())
}

/// Generate remote.conf for a specific remote
pub async fn generate_conf(name: &str, output: &str) -> Result<()> {
    let db = RemoteConfigDatabase::default_database();

    let remote = db
        .find_by_name(name)
        .ok_or_else(|| anyhow::anyhow!("Remote not found: {}", name))?;

    let content = remote.generate_remote_conf();

    if output == "-" {
        println!("{}", content);
    } else {
        fs::write(output, &content)?;
        info!("Generated remote.conf: {}", output);
        println!("✅ Generated {} for '{}'", output, remote.name);
    }

    Ok(())
}

/// Generate Android keylayout for a specific remote
pub async fn generate_keylayout(name: &str, output: &str) -> Result<()> {
    let db = RemoteConfigDatabase::default_database();

    let remote = db
        .find_by_name(name)
        .ok_or_else(|| anyhow::anyhow!("Remote not found: {}", name))?;

    let content = remote.generate_keylayout();

    if output == "-" {
        println!("{}", content);
    } else {
        fs::write(output, &content)?;
        info!("Generated keylayout: {}", output);
        println!("✅ Generated {} for '{}'", output, remote.name);
    }

    Ok(())
}
