//! Phoenix OS Build Tools - Shared Library
//!
//! Core functionality for hardware detection, configuration management,
//! and build orchestration.

pub mod archives;
pub mod assets;
pub mod backup;
pub mod build;
pub mod compatibility;
pub mod config;
pub mod doctor;
pub mod error;
pub mod extract;
pub mod flash;
pub mod flash_allwinner;
pub mod flash_amlogic;
pub mod flash_rockchip;
pub mod hardware;
pub mod optimization;
pub mod patches;
pub mod profiles;
pub mod remote_config;
pub mod security;
pub mod templates;
pub mod unlock;
pub mod vault;
pub mod workflow;

pub use config::DeviceConfig;
pub use error::AppError;
pub use hardware::detect_devices;
pub use profiles::ProfileDatabase;
