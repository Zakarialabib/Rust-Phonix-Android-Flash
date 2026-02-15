pub mod backup;
pub mod build;
pub mod check;
pub mod config;
pub mod detect;
pub mod doctor;
pub mod extract;
pub mod flash;
pub mod forensics;
pub mod generate;
pub mod patch;
pub mod phase;
pub mod remote;
pub mod security;
pub mod unlock;
pub mod validate;
pub mod vault;

use crate::cli::Commands;

pub async fn run(command: Commands) -> anyhow::Result<()> {
    match command {
        Commands::Detect {
            method,
            port,
            format,
        } => detect::run(&method, port.as_deref(), &format).await,
        Commands::Config { action } => config::run(action),
        Commands::Generate { target } => generate::run(target),
        Commands::Build {
            profile,
            board,
            output,
            dry_run,
        } => build::run(&profile, &board, &output, dry_run).await,
        Commands::Flash {
            target,
            device,
            image,
        } => flash::run(&target, &device, &image).await,
        Commands::Doctor => doctor::run().await,
        Commands::Backup { action } => backup::run(action).await,
        Commands::Unlock { action } => unlock::run(action).await,
        Commands::Extract { target } => extract::run(target).await,
        Commands::Forensics { action } => forensics::run(action).await,
        Commands::Check {
            profile,
            firmware,
            os,
            version,
            kernel,
            format,
        } => {
            check::run(
                &profile,
                &firmware,
                os.as_deref(),
                version.as_deref(),
                kernel.as_deref(),
                &format,
            )
            .await
        }
        Commands::Patch { action } => patch::run(action).await,
        Commands::Validate { device, format } => validate::run(device.as_deref(), &format).await,
        Commands::Security { action } => security::run(action).await,
        Commands::Remote { action } => remote::run(action).await,
        Commands::Vault { action } => vault::run(action).await,
        Commands::Gui => {
            println!("Launching Phoenix GUI...");
            println!("Run 'phoenix-gui' or use the desktop application.");
            Ok(())
        }
    }
}
