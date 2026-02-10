//! Configuration management commands

use anyhow::Result;
use phoenix_lib::config::{create_default_config, DeviceConfig};

pub fn init(soc: &str, name: &str, output: Option<&str>) -> Result<()> {
    println!("🔧 Creating device configuration for {} ({})", name, soc);

    let config = create_default_config(soc, name);
    
    let output_path = output.unwrap_or_else(|| {
        Box::leak(format!("{}.yaml", name.to_lowercase().replace(' ', "-")).into_boxed_str())
    });

    config.to_file(output_path)?;
    
    println!("✅ Created: {}", output_path);
    println!();
    println!("Next steps:");
    println!("  1. Edit {} to match your hardware", output_path);
    println!("  2. Run: phoenix config validate {}", output_path);
    println!("  3. Run: phoenix generate dts --board {} --output device.dts", output_path);

    Ok(())
}

pub fn validate(path: &str) -> Result<()> {
    println!("🔍 Validating configuration: {}", path);

    let yaml = std::fs::read_to_string(path)?;
    DeviceConfig::validate_schema_yaml(&yaml)?;
    let config = DeviceConfig::from_str(&yaml)?;
    config.validate()?;

    println!("✅ Configuration is valid");
    println!();
    println!("Device: {} ({})", config.device.name, config.device.soc);
    println!("Memory: {} MB {}", config.hardware.memory.size_mb, config.hardware.memory.mem_type);
    println!("Storage: {} GB {}", config.hardware.storage.size_gb, config.hardware.storage.storage_type);
    
    if let Some(wifi) = &config.hardware.wifi {
        println!("WiFi: {} ({})", wifi.chip, wifi.driver);
    }

    println!("Profiles: {}", config.profiles.keys().cloned().collect::<Vec<_>>().join(", "));

    Ok(())
}

pub fn show(path: &str) -> Result<()> {
    let config = DeviceConfig::from_file(path)?;
    let yaml = serde_yaml::to_string(&config)?;
    println!("{}", yaml);
    Ok(())
}
