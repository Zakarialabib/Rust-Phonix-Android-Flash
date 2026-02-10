//! File generation commands

use anyhow::Result;
use phoenix_lib::config::DeviceConfig;
use phoenix_lib::templates::{DtsContext, ExtlinuxContext, KconfigContext, TemplateEngine};
use std::path::Path;

pub fn dts(board: &str, output: &str) -> Result<()> {
    println!("🔧 Generating device tree source...");

    let config = DeviceConfig::from_file(board)?;
    let engine = TemplateEngine::new()?;

    let context = DtsContext {
        device_name: config.device.name.to_lowercase().replace(' ', "-"),
        soc: config.device.soc.clone(),
        soc_family: match config.device.soc.as_str() {
            s if s.starts_with("s905") => "meson-gxl".to_string(),
            s if s.starts_with("s912") => "meson-gxm".to_string(),
            s if s.starts_with("rk3") => format!("rk{}", &s[2..]),
            _ => "unknown".to_string(),
        },
        reference_dtb: config.boot.reference_dtb.replace(".dtb", ".dtsi"),
        memory_size_mb: config.hardware.memory.size_mb,
        has_wifi: config.hardware.wifi.is_some(),
        wifi_chip: config.hardware.wifi.as_ref().map_or(String::new(), |w| w.chip.clone()),
        has_ethernet: config.hardware.ethernet.is_some(),
        led_gpio: None,
    };

    engine.render_to_file("dts", &context, Path::new(output))?;
    
    println!("✅ Generated: {}", output);
    println!();
    println!("To compile: dtc -I dts -O dtb -o device.dtb {}", output);

    Ok(())
}

pub fn kconfig(board: &str, output: &str) -> Result<()> {
    println!("🔧 Generating kernel config fragment...");

    let config = DeviceConfig::from_file(board)?;
    let engine = TemplateEngine::new()?;

    let context = KconfigContext {
        soc: config.device.soc.clone(),
        enable_panfrost: config.device.soc.starts_with("s905"),
        enable_vdec: true,
        enable_wifi: config.hardware.wifi.is_some(),
        wifi_driver: config.hardware.wifi.as_ref().map_or(String::new(), |w| w.driver.clone()),
        enable_cec: true,
        cma_size_mb: 256,
    };

    engine.render_to_file("kconfig", &context, Path::new(output))?;
    
    println!("✅ Generated: {}", output);
    println!();
    println!("Usage: ./scripts/kconfig/merge_config.sh .config {}", output);

    Ok(())
}

pub fn extlinux(board: &str, output: &str) -> Result<()> {
    println!("🔧 Generating boot configuration...");

    let config = DeviceConfig::from_file(board)?;
    let engine = TemplateEngine::new()?;

    let console = config.boot.uart.as_ref()
        .map_or("ttyAML0,115200".to_string(), |u| format!("{},{}", u.port, u.baud));

    let context = ExtlinuxContext {
        label: config.device.name.to_uppercase().replace(' ', "_"),
        kernel_path: "/Image".to_string(),
        dtb_path: format!("/dtbs/{}", config.boot.reference_dtb),
        root_device: "/dev/mmcblk0p2".to_string(),
        console,
        extra_args: "quiet logo.nologo".to_string(),
    };

    engine.render_to_file("extlinux", &context, Path::new(output))?;
    
    println!("✅ Generated: {}", output);

    Ok(())
}
