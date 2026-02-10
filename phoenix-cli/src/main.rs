//! Phoenix CLI - Build automation for TV box liberation
//!
//! Main entry point for the phoenix command-line tool.

use clap::{Parser, Subcommand};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod commands;

/// Phoenix OS Build Tools - Transform Android TV boxes into Linux compute nodes
#[derive(Parser)]
#[command(name = "phoenix")]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    /// Enable verbose output
    #[arg(short, long, global = true)]
    verbose: bool,

    /// Configuration file path
    #[arg(short, long, global = true)]
    config: Option<String>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Detect connected devices (USB/UART)
    Detect {
        /// Detection method: usb, uart, or all
        #[arg(default_value = "all")]
        method: String,

        /// Serial port for UART detection
        #[arg(short, long)]
        port: Option<String>,

        /// Output format: text, json
        #[arg(short, long, default_value = "text")]
        format: String,
    },

    /// Manage device configurations
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },

    /// Generate files from templates
    Generate {
        #[command(subcommand)]
        target: GenerateTarget,
    },

    /// Build firmware images
    Build {
        /// Build profile (minimal, signage, server)
        #[arg(short, long, default_value = "minimal")]
        profile: String,

        /// Target board name
        #[arg(short, long)]
        board: String,

        /// Output directory
        #[arg(short, long, default_value = "./output")]
        output: String,

        /// Dry run - show build plan without executing
        #[arg(long)]
        dry_run: bool,
    },

    /// Flash image to device
    Flash {
        /// Target: sd, emmc
        #[arg(short, long)]
        target: String,

        /// Device path (e.g., /dev/sdb, COM3)
        #[arg(short, long)]
        device: String,

        /// Image file to flash
        #[arg(short, long)]
        image: String,
    },

    /// System Health Check
    Doctor,

    /// Backup device partitions
    Backup {
        #[command(subcommand)]
        action: BackupAction,
    },

    /// Unlock device bootloader
    Unlock {
        #[command(subcommand)]
        action: UnlockAction,
    },

    /// Extract artifacts from device
    Extract {
        #[command(subcommand)]
        target: ExtractTarget,
    },

    Forensics {
        #[command(subcommand)]
        action: ForensicsAction,
    },

    Check {
        #[arg(short, long)]
        profile: String,

        #[arg(short, long)]
        firmware: String,

        #[arg(long)]
        os: Option<String>,

        #[arg(long)]
        version: Option<String>,

        #[arg(long)]
        kernel: Option<String>,

        #[arg(short, long, default_value = "text")]
        format: String,
    },

    Patch {
        #[command(subcommand)]
        action: PatchAction,
    },

    /// Validate patched image on hardware (stubs)
    Validate {
        /// Serial device (optional)
        #[arg(short, long)]
        device: Option<String>,

        /// Output format: text, json
        #[arg(short, long, default_value = "text")]
        format: String,
    },

    /// Scan firmware for malware (Corejava botnet, etc.)
    Security {
        #[command(subcommand)]
        action: SecurityAction,
    },

    /// Remote control configuration
    Remote {
        #[command(subcommand)]
        action: RemoteAction,
    },

    /// Secure firmware backup/restore (Vault)
    Vault {
        #[command(subcommand)]
        action: VaultAction,
    },

    /// Launch the GUI application
    Gui,
}

#[derive(Subcommand)]
enum ConfigAction {
    /// Create a new device configuration
    Init {
        /// SoC type (s905w, s905x, rk3229)
        #[arg(short, long)]
        soc: String,

        /// Device name
        #[arg(short, long)]
        name: String,

        /// Output file path
        #[arg(short, long)]
        output: Option<String>,
    },

    /// Validate a configuration file
    Validate {
        /// Path to config file
        path: String,
    },

    /// Show configuration details
    Show {
        /// Path to config file
        path: String,
    },
}

#[derive(Subcommand)]
enum BackupAction {
    /// Backup full device or partition
    Dump {
        /// Device path or identifier
        #[arg(short, long)]
        device: String,

        /// Output file path
        #[arg(short, long)]
        output: String,

        /// Specific partition device path
        #[arg(short, long)]
        partition: Option<String>,
    },

    /// Extract range from firmware image
    Extract {
        /// Firmware image file
        #[arg(short, long)]
        firmware: String,

        /// Output file path
        #[arg(short, long)]
        output: String,

        /// Byte offset
        #[arg(long)]
        offset: u64,

        /// Byte size
        #[arg(long)]
        size: u64,
    },

    /// Verify firmware image checksum
    Verify {
        /// Firmware image file
        #[arg(short, long)]
        firmware: String,
    },
}

#[derive(Subcommand)]
enum PatchAction {
    Plan {
        #[arg(short, long)]
        profile: String,

        #[arg(short, long)]
        firmware: String,

        #[arg(long)]
        os: Option<String>,

        #[arg(long)]
        version: Option<String>,

        #[arg(long)]
        kernel: Option<String>,

        #[arg(short, long, default_value = "text")]
        format: String,
    },
}

#[derive(Subcommand)]
enum UnlockAction {
    /// Detect unlockable devices
    Detect {
        /// Detection method: usb, uart, or all
        #[arg(short, long, default_value = "all")]
        method: String,

        /// Serial port for UART detection
        #[arg(short, long)]
        port: Option<String>,
    },

    /// Show maskrom entry instructions
    Maskrom {
        /// SoC type (s905w, rk3229, etc.)
        #[arg(short, long)]
        soc: String,
    },

    /// Show current unlock status
    Status {
        /// Status method: usb, uart, or all
        #[arg(short, long, default_value = "all")]
        method: String,
    },
}

#[derive(Subcommand)]
enum ExtractTarget {
    /// Extract WiFi firmware
    Wifi {
        /// Mounted firmware root
        #[arg(short, long)]
        mount: String,

        /// Output directory
        #[arg(short, long)]
        output: String,
    },

    /// Extract DDR timings
    Ddr {
        /// Mounted firmware root
        #[arg(short, long)]
        mount: String,

        /// Output directory
        #[arg(short, long)]
        output: String,
    },

    /// Extract DTB files
    Dtb {
        /// Mounted firmware root
        #[arg(short, long)]
        mount: String,

        /// Output directory
        #[arg(short, long)]
        output: String,
    },

    /// Extract kernel config
    Config {
        /// Mounted firmware root
        #[arg(short, long)]
        mount: String,

        /// Output directory
        #[arg(short, long)]
        output: String,
    },
}

#[derive(Subcommand)]
enum ForensicsAction {
    DeepScan {
        #[arg(short, long)]
        device: Option<String>,

        #[arg(short, long, default_value = "text")]
        format: String,
    },
}

#[derive(Subcommand)]
enum GenerateTarget {
    /// Generate device tree source
    Dts {
        /// Board configuration file
        #[arg(short, long)]
        board: String,

        /// Output file path
        #[arg(short, long)]
        output: String,
    },

    /// Generate kernel config fragment
    Kconfig {
        /// Board configuration file
        #[arg(short, long)]
        board: String,

        /// Output file path
        #[arg(short, long)]
        output: String,
    },

    /// Generate boot configuration
    Extlinux {
        /// Board configuration file
        #[arg(short, long)]
        board: String,

        /// Output file path
        #[arg(short, long)]
        output: String,
    },
}

#[derive(Subcommand)]
enum SecurityAction {
    /// Scan firmware image for known malware
    Scan {
        /// Path to firmware image or extracted root
        #[arg(short, long)]
        image: String,

        /// Output format: text, json
        #[arg(short, long, default_value = "text")]
        format: String,
    },
}

#[derive(Subcommand)]
enum RemoteAction {
    /// List available remote configurations
    List {
        /// Output format: text, json
        #[arg(short, long, default_value = "text")]
        format: String,
    },

    /// Generate remote.conf for Amlogic devices
    GenerateConf {
        /// Remote name (e.g., "X96 Mini")
        #[arg(short, long)]
        name: String,

        /// Output file (use - for stdout)
        #[arg(short, long, default_value = "-")]
        output: String,
    },

    /// Generate Android keylayout (.kl) file
    GenerateKeylayout {
        /// Remote name (e.g., "X96 Mini")
        #[arg(short, long)]
        name: String,

        /// Output file (use - for stdout)
        #[arg(short, long, default_value = "-")]
        output: String,
    },
}

#[derive(Subcommand)]
enum VaultAction {
    /// Create a new secure backup of the original firmware
    Create {
        /// Device name (e.g., COM3)
        #[arg(short, long)]
        device: Option<String>,

        /// Name for the backup
        #[arg(short, long)]
        name: String,
    },

    /// List all available backups
    List,

    /// Verify integrity of a backup
    Verify {
        /// Name of the backup to verify
        #[arg(short, long)]
        name: String,
    },

    /// Restore a backup to a device
    Restore {
        /// Name of the backup to restore
        #[arg(short, long)]
        name: String,

        /// Device name (e.g., COM3)
        #[arg(short, long)]
        device: Option<String>,
    },

    /// Extract data from a backup
    Extract {
        /// Name of the backup
        #[arg(short, long)]
        name: String,

        /// Partition to extract
        #[arg(short, long)]
        partition: String,

        /// Output file path
        #[arg(short, long)]
        output: String,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // Initialize logging
    let log_level = if cli.verbose { "debug" } else { "info" };
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| log_level.into()))
        .with(tracing_subscriber::fmt::layer())
        .init();

    match cli.command {
        Commands::Detect { method, port, format } => {
            commands::detect::run(&method, port.as_deref(), &format).await
        }
        Commands::Config { action } => match action {
            ConfigAction::Init { soc, name, output } => {
                commands::config::init(&soc, &name, output.as_deref())
            }
            ConfigAction::Validate { path } => {
                commands::config::validate(&path)
            }
            ConfigAction::Show { path } => {
                commands::config::show(&path)
            }
        },
        Commands::Generate { target } => match target {
            GenerateTarget::Dts { board, output } => {
                commands::generate::dts(&board, &output)
            }
            GenerateTarget::Kconfig { board, output } => {
                commands::generate::kconfig(&board, &output)
            }
            GenerateTarget::Extlinux { board, output } => {
                commands::generate::extlinux(&board, &output)
            }
        },
        Commands::Build { profile, board, output, dry_run } => {
            commands::build::run(&profile, &board, &output, dry_run).await
        }
        Commands::Flash { target, device, image } => {
            commands::flash::run(&target, &device, &image).await
        }
        Commands::Doctor => {
            commands::doctor::run().await
        }
        Commands::Backup { action } => match action {
            BackupAction::Dump { device, output, partition } => {
                commands::backup::dump(&device, &output, partition.as_deref()).await
            }
            BackupAction::Extract { firmware, output, offset, size } => {
                commands::backup::extract(&firmware, &output, offset, size).await
            }
            BackupAction::Verify { firmware } => {
                commands::backup::verify(&firmware).await
            }
        },
        Commands::Unlock { action } => match action {
            UnlockAction::Detect { method, port } => {
                commands::unlock::detect(&method, port.as_deref()).await
            }
            UnlockAction::Maskrom { soc } => {
                commands::unlock::maskrom(&soc).await
            }
            UnlockAction::Status { method } => {
                commands::unlock::status(&method).await
            }
        },
        Commands::Extract { target } => match target {
            ExtractTarget::Wifi { mount, output } => {
                commands::extract::wifi(&mount, &output).await
            }
            ExtractTarget::Ddr { mount, output } => {
                commands::extract::ddr(&mount, &output).await
            }
            ExtractTarget::Dtb { mount, output } => {
                commands::extract::dtb(&mount, &output).await
            }
            ExtractTarget::Config { mount, output } => {
                commands::extract::config(&mount, &output).await
            }
        },
        Commands::Forensics { action } => match action {
            ForensicsAction::DeepScan { device, format } => {
                commands::forensics::deep_scan(device.as_deref(), &format).await
            }
        },
        Commands::Check { profile, firmware, os, version, kernel, format } => {
            commands::check::run(
                &profile,
                &firmware,
                os.as_deref(),
                version.as_deref(),
                kernel.as_deref(),
                &format,
            )
            .await
        }
        Commands::Patch { action } => match action {
            PatchAction::Plan { profile, firmware, os, version, kernel, format } => {
                commands::patch::plan(
                    &profile,
                    &firmware,
                    os.as_deref(),
                    version.as_deref(),
                    kernel.as_deref(),
                    &format,
                )
                .await
            }
        },
        Commands::Validate { device, format } => {
            commands::validate::run(device.as_deref(), &format).await
        }
        Commands::Security { action } => match action {
            SecurityAction::Scan { image, format } => {
                commands::security::scan(&image, &format).await
            }
        },
        Commands::Remote { action } => match action {
            RemoteAction::List { format } => {
                commands::remote::list(&format).await
            }
            RemoteAction::GenerateConf { name, output } => {
                commands::remote::generate_conf(&name, &output).await
            }
            RemoteAction::GenerateKeylayout { name, output } => {
                commands::remote::generate_keylayout(&name, &output).await
            }
        },
        Commands::Vault { action } => match action {
            VaultAction::Create { device, name } => {
                commands::vault::create(device.as_deref(), &name).await
            }
            VaultAction::List => {
                commands::vault::list().await
            }
            VaultAction::Verify { name } => {
                commands::vault::verify(&name).await
            }
            VaultAction::Restore { name, device } => {
                commands::vault::restore(&name, device.as_deref()).await
            }
            VaultAction::Extract { name, partition, output } => {
                commands::vault::extract(&name, &partition, &output).await
            }
        },
        Commands::Gui => {
            println!("Launching Phoenix GUI...");
            println!("Run 'phoenix-gui' or use the desktop application.");
            Ok(())
        }
    }
}
