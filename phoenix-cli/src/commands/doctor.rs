use anyhow::Result;
use colored::Colorize;
use which::which;

pub async fn run() -> Result<()> {
    println!("{}", "Phoenix Doctor - System Health Check".bold().green());
    println!("{}", "====================================");

    let required_tools = vec!["git", "dd", "tar", "gzip"];
    let optional_tools = vec!["docker", "adb", "fastboot", "rkdeveloptool", "update"];

    println!("\n{}", "Checking Required Tools:".bold());
    for tool in required_tools {
        match which(tool) {
            Ok(path) => println!("  ✅ {} found at {}", tool, path.display()),
            Err(_) => println!("  ❌ {} not found (Required)", tool.red()),
        }
    }

    println!("\n{}", "Checking Optional Tools:".bold());
    for tool in optional_tools {
        match which(tool) {
            Ok(path) => println!("  ✅ {} found at {}", tool, path.display()),
            Err(_) => println!("  ⚠️  {} not found (Optional)", tool.yellow()),
        }
    }

    // Check permissions
    println!("\n{}", "Checking Permissions:".bold());
    match std::fs::write(".phoenix_test_write", "test") {
        Ok(_) => {
            println!("  ✅ Write permission in current directory");
            std::fs::remove_file(".phoenix_test_write").ok();
        }
        Err(_) => println!("{}", "  ❌ No write permission in current directory".red()),
    }

    // Check connectivity
    println!("\n{}", "Checking Connectivity:".bold());
    match reqwest::get("https://github.com").await {
        Ok(_) => println!("  ✅ Internet access available"),
        Err(_) => println!("{}", "  ❌ No internet access".red()),
    }

    Ok(())
}
