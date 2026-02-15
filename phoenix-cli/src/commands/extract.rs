use crate::cli::ExtractTarget;
use anyhow::Result;
use colored::Colorize;
use phoenix_lib::extract::Extractor;
use std::path::Path;

pub async fn run(target: ExtractTarget) -> Result<()> {
    match target {
        ExtractTarget::Wifi { mount, output } => wifi(&mount, &output).await,
        ExtractTarget::Ddr { mount, output } => ddr(&mount, &output).await,
        ExtractTarget::Dtb { mount, output } => dtb(&mount, &output).await,
        ExtractTarget::Config { mount, output } => config(&mount, &output).await,
    }
}

pub async fn wifi(mount: &str, output: &str) -> Result<()> {
    println!("{}", "Phoenix Artifact Extractor".bold().cyan());
    let copied = Extractor::extract_wifi_firmware(Path::new(mount), Path::new(output))
        .await
        .map_err(|e| anyhow::anyhow!(e.to_string()))?;
    println!("Copied {} files", copied);
    Ok(())
}

pub async fn ddr(mount: &str, output: &str) -> Result<()> {
    println!("{}", "Phoenix Artifact Extractor".bold().cyan());
    let copied = Extractor::extract_ddr_timings(Path::new(mount), Path::new(output))
        .await
        .map_err(|e| anyhow::anyhow!(e.to_string()))?;
    println!("Copied {} files", copied);
    Ok(())
}

pub async fn dtb(mount: &str, output: &str) -> Result<()> {
    println!("{}", "Phoenix Artifact Extractor".bold().cyan());
    let copied = Extractor::extract_dtb_from_mount(Path::new(mount), Path::new(output))
        .await
        .map_err(|e| anyhow::anyhow!(e.to_string()))?;
    println!("Copied {} files", copied);
    Ok(())
}

pub async fn config(mount: &str, output: &str) -> Result<()> {
    println!("{}", "Phoenix Artifact Extractor".bold().cyan());
    let copied = Extractor::extract_kernel_config(Path::new(mount), Path::new(output))
        .await
        .map_err(|e| anyhow::anyhow!(e.to_string()))?;
    println!("Copied {} files", copied);
    Ok(())
}
