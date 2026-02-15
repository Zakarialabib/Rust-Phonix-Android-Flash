use clap::{Parser, Subcommand};

/// Phoenix OS Build Tools - Transform Android TV boxes into Linux compute nodes
#[derive(Parser)]
#[command(name = "phoenix")]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
    /// Enable verbose output
    #[arg(short, long, global = true)]
    pub verbose: bool,

    /// Configuration file path
    #[arg(short, long, global = true)]
    pub config: Option<String>,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
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
pub enum ConfigAction {
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
pub enum BackupAction {
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
pub enum PatchAction {
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
pub enum UnlockAction {
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
pub enum ExtractTarget {
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
pub enum ForensicsAction {
    DeepScan {
        #[arg(short, long)]
        device: Option<String>,

        #[arg(short, long, default_value = "text")]
        format: String,
    },
}

#[derive(Subcommand)]
pub enum GenerateTarget {
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
pub enum SecurityAction {
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
pub enum RemoteAction {
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
pub enum VaultAction {
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
