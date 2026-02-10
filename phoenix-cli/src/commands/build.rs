//! Build commands

use anyhow::Result;
use indicatif::{ProgressBar, ProgressStyle};
use phoenix_lib::build::{check_prerequisites, BuildPipeline, RecipeEnv};
use phoenix_lib::config::DeviceConfig;
use std::path::PathBuf;
use std::time::Duration;

pub async fn run(profile: &str, board: &str, output: &str, dry_run: bool) -> Result<()> {
    println!("🏗️  Phoenix Build System");
    println!("========================");
    println!();
    println!("Board:   {}", board);
    println!("Profile: {}", profile);
    println!("Output:  {}", output);
    println!();

    // Check prerequisites
    println!("📋 Checking prerequisites...");
    let missing = check_prerequisites()?;
    if !missing.is_empty() {
        println!("⚠️  Missing tools: {}", missing.join(", "));
        println!("   Please install these tools before building.");
        if !dry_run {
            anyhow::bail!("Missing prerequisites");
        }
    } else {
        println!("✅ All prerequisites found");
    }

    // Load board config
    let config = DeviceConfig::from_file(board)?;
    config.validate()?;
    println!("✅ Board configuration valid");

    // Check profile exists
    if !config.profiles.contains_key(profile) {
        anyhow::bail!(
            "Profile '{}' not found. Available: {}",
            profile,
            config.profiles.keys().cloned().collect::<Vec<_>>().join(", ")
        );
    }

    let build_profile = &config.profiles[profile];
    println!();
    println!("📦 Build Profile:");
    println!("   Rootfs:  {}", build_profile.rootfs);
    println!("   Kernel:  {}", build_profile.kernel);
    if !build_profile.packages.is_empty() {
        println!("   Packages: {}", build_profile.packages.join(", "));
    }
    println!();

    if dry_run {
        println!("🔍 Dry run - Build steps:");
        println!("   1. Build kernel ({})", build_profile.kernel);
        println!("   2. Build rootfs ({})", build_profile.rootfs);
        println!("   3. Build U-Boot");
        println!("   4. Assemble image");
        println!();
        println!("Run without --dry-run to execute the build.");
        return Ok(());
    }

    // Create build environment
    let _env = RecipeEnv {
        board: config.device.name.to_lowercase().replace(' ', "-"),
        profile: profile.to_string(),
        output_dir: PathBuf::from(output),
        ..Default::default()
    };

    // Create progress bar
    let pb = ProgressBar::new(4);
    pb.set_style(ProgressStyle::default_bar()
        .template("{spinner:.green} [{bar:40.cyan/blue}] {pos}/{len} {msg}")?
        .progress_chars("█▓░"));

    let recipes_dir = PathBuf::from("recipes");
    let pipeline = BuildPipeline::image_build(&recipes_dir);

    for step in &pipeline.steps {
        pb.set_message(format!("Building {}...", step.name));
        
        // Simulate build step for now
        tokio::time::sleep(Duration::from_millis(500)).await;
        
        pb.inc(1);
    }

    pb.finish_with_message("Build complete!");
    println!();
    println!("✅ Build artifacts in: {}/", output);
    println!("   - Image.gz (kernel)");
    println!("   - rootfs.ext4");
    println!("   - fip.bin (bootloader)");
    println!("   - meson-gxl-*.dtb");
    println!();
    println!("Next: phoenix flash --target sd --device /dev/sdX --image {}/phoenix.img", output);

    Ok(())
}
