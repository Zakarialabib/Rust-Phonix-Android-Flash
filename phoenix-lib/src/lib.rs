//! Phoenix OS Build Tools - Shared Library
//!
//! Core functionality for hardware detection, configuration management,
//! and build orchestration.

pub mod config;
pub mod hardware;
pub mod build;
pub mod templates;
pub mod profiles;
pub mod flash;
pub mod assets;
pub mod error;
pub mod doctor;
pub mod backup;
pub mod unlock;
pub mod extract;
pub mod compatibility;
pub mod patches;
pub mod workflow;
pub mod security;
pub mod remote_config;
pub mod optimization;
pub mod flash_amlogic;
pub mod flash_rockchip;
pub mod flash_allwinner;
pub mod archives;
pub mod vault;

pub use config::DeviceConfig;
pub use hardware::detect_devices;
pub use profiles::ProfileDatabase;
pub use error::AppError;
