use phoenix_lib::{
    archives::extract_archive,
    assets::download_file,
    build::{BuildPipeline, OutputStream, RecipeEnv},
    compatibility::{
        build_patch_plan, get_recommendations, resolve_firmware_target, resolve_hardware_profile,
        CompatibilityMatrix, CompatibilityReport, FirmwareRecommendation, HardwareProfile,
        PatchPlan,
    },
    config::{create_default_config, DeviceConfig},
    error::AppError,
    flash::{flash_image, preflight, FlashProgress},
    flash_allwinner::{AllwinnerDevice, AllwinnerImageHeader, AllwinnerVersion},
    flash_amlogic::{AmlogicChipInfo, AmlogicDevice},
    flash_rockchip::{RkImageHeader, RkParameter, RockchipChipInfo, RockchipDevice},
    hardware::{
        detect_devices, list_serial_ports, perform_deep_scan, DetectedDevice, ForensicsReport,
    },
    profiles::{default_profiles, DeviceProfile, ProfileDatabase},
    remote_config::{RemoteConfig, RemoteConfigDatabase},
    security::{SecurityReport, SecurityScanner},
    workflow::{Phase, PhaseStatus, WorkflowPhaseEvent},
};
use serde::{Deserialize, Serialize};
use std::{
    path::{Path, PathBuf},
    sync::Arc,
};
use tauri::{AppHandle, Emitter, Manager, State};
use tokio::{fs, sync::RwLock};
use tracing::{error, info, instrument};

/// Application state
pub struct AppState {
    pub settings_path: Arc<RwLock<Option<PathBuf>>>,
    pub settings: Arc<RwLock<AppSettings>>,
    pub profiles: Arc<RwLock<ProfileDatabase>>,
    // Amlogic Device Manager
    pub amlogic_device: Arc<tokio::sync::Mutex<Option<AmlogicDevice>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    pub tools_path: String,
    pub cache_path: String,
    pub output_path: String,
    #[serde(default = "default_language")]
    pub language: String,
    #[serde(default = "default_theme_mode")]
    pub theme_mode: String,
    #[serde(default = "default_theme_color")]
    pub theme_color: String,
    #[serde(default = "default_ui_scale")]
    pub ui_scale: String,
    #[serde(default = "default_typography")]
    pub typography: String,
}

fn default_language() -> String {
    "en".to_string()
}
fn default_theme_mode() -> String {
    "dark".to_string()
}
fn default_theme_color() -> String {
    "amber".to_string()
}
fn default_ui_scale() -> String {
    "normal".to_string()
}
fn default_typography() -> String {
    "technical".to_string()
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            tools_path: "tools".to_string(),
            cache_path: "cache".to_string(),
            output_path: "output".to_string(),
            language: default_language(),
            theme_mode: default_theme_mode(),
            theme_color: default_theme_color(),
            ui_scale: default_ui_scale(),
            typography: default_typography(),
        }
    }
}

// ─── Tauri Commands ─────────────────────────────────────────────────────────

/// Resolve device profile from VID/PID
#[tauri::command]
#[instrument(skip(state))]
async fn cmd_resolve_profile(
    state: State<'_, AppState>,
    vid: u16,
    pid: u16,
) -> Result<Option<DeviceProfile>, AppError> {
    let profiles = state.profiles.read().await;
    Ok(profiles.find(vid, pid).cloned())
}

/// Detect connected devices
#[tauri::command]
#[instrument]
async fn cmd_detect_devices() -> Result<Vec<DetectedDevice>, AppError> {
    info!("Detecting devices...");
    // Optimized: Run blocking detection in a separate thread to avoid stalling the async runtime
    tauri::async_runtime::spawn_blocking(move || detect_devices().map_err(AppError::from))
        .await
        .map_err(|e| AppError::Unknown(format!("Thread join error: {}", e)))?
}

/// Detect Amlogic Device (WorldCup Mode)
#[tauri::command]
#[instrument(skip(state))]
async fn cmd_amlogic_detect(state: State<'_, AppState>) -> Result<AmlogicChipInfo, AppError> {
    let mut device_guard = state.amlogic_device.lock().await;

    // If we don't have a handle, try to detect one
    if device_guard.is_none() {
        info!("Attempting to detect Amlogic device...");
        let device = tauri::async_runtime::spawn_blocking(move || AmlogicDevice::detect())
            .await
            .map_err(|e| AppError::Unknown(format!("Thread join error: {}", e)))??;
        *device_guard = Some(device);
    }

    // Identify the chip
    if let Some(mut device) = device_guard.take() {
        info!("Identifying Amlogic device...");
        let (device, result) = tauri::async_runtime::spawn_blocking(move || {
            let res = device.identify();
            (device, res)
        })
        .await
        .map_err(|e| AppError::Unknown(format!("Thread join error: {}", e)))?;

        *device_guard = Some(device);
        result
    } else {
        Err(AppError::DeviceNotFound(
            "Failed to open device after detection".to_string(),
        ))
    }
}

/// Flash Amlogic Image
#[tauri::command]
#[instrument(skip(app, state))]
async fn cmd_amlogic_flash_image(
    app: AppHandle,
    state: State<'_, AppState>,
    image_path: String,
) -> Result<(), AppError> {
    let mut device_guard = state.amlogic_device.lock().await;

    if let Some(mut device) = device_guard.take() {
        let app_handle = app.clone();

        // Create progress callback
        let progress_cb = Box::new(move |progress: FlashProgress| {
            let _ = app_handle.emit("amlogic:progress", progress);
        });

        info!("Starting Amlogic flash: {}", image_path);

        let (device, result) = tauri::async_runtime::spawn_blocking(move || {
            let res = device.flash_image(Path::new(&image_path), Some(progress_cb));
            (device, res)
        })
        .await
        .map_err(|e| AppError::Unknown(format!("Thread join error: {}", e)))?;

        *device_guard = Some(device);
        result
    } else {
        Err(AppError::DeviceNotFound(
            "No Amlogic device connected".to_string(),
        ))
    }
}

/// Extract Amlogic Image
#[tauri::command]
#[instrument]
async fn cmd_amlogic_extract_image(image_path: String, output_dir: String) -> Result<(), AppError> {
    info!("Extracting Amlogic image: {}", image_path);
    tauri::async_runtime::spawn_blocking(move || {
        let header = phoenix_lib::flash_amlogic::AmlogicImageHeader::parse(Path::new(&image_path))?;
        header.extract_to(Path::new(&image_path), Path::new(&output_dir))
    })
    .await
    .map_err(|e| AppError::Unknown(format!("Thread join error: {}", e)))?
}

// ─── Rockchip Commands ──────────────────────────────────────────────────────

#[tauri::command]
#[instrument]
async fn cmd_rockchip_detect() -> Result<RockchipChipInfo, AppError> {
    info!("Detecting Rockchip device...");
    tauri::async_runtime::spawn_blocking(move || {
        let mut device = RockchipDevice::detect()?;
        device.read_chip_info()
    })
    .await
    .map_err(|e| AppError::Unknown(format!("Thread join error: {}", e)))?
}

#[tauri::command]
#[instrument]
async fn cmd_rockchip_parse_image(image_path: String) -> Result<RkImageHeader, AppError> {
    tauri::async_runtime::spawn_blocking(move || RkImageHeader::parse(Path::new(&image_path)))
        .await
        .map_err(|e| AppError::Unknown(format!("Thread join error: {}", e)))?
}

#[tauri::command]
#[instrument]
async fn cmd_rockchip_extract_image(
    image_path: String,
    output_dir: String,
) -> Result<(), AppError> {
    info!("Extracting Rockchip image: {}", image_path);
    tauri::async_runtime::spawn_blocking(move || {
        let header = RkImageHeader::parse(Path::new(&image_path))?;
        header.extract_to(Path::new(&image_path), Path::new(&output_dir))
    })
    .await
    .map_err(|e| AppError::Unknown(format!("Thread join error: {}", e)))?
}

#[tauri::command]
#[instrument(skip(content))]
async fn cmd_rockchip_parse_parameter(content: String) -> Result<RkParameter, AppError> {
    // This parses string content, CPU bound, but likely fast enough.
    // However for consistency let's spawn blocking if content is huge.
    // Assuming content can be large (parameter file):
    tauri::async_runtime::spawn_blocking(move || RkParameter::parse(&content))
        .await
        .map_err(|e| AppError::Unknown(format!("Thread join error: {}", e)))?
}

// ─── Allwinner Commands ─────────────────────────────────────────────────────

#[tauri::command]
#[instrument]
async fn cmd_allwinner_detect() -> Result<AllwinnerVersion, AppError> {
    info!("Detecting Allwinner device...");
    tauri::async_runtime::spawn_blocking(move || {
        let mut device = AllwinnerDevice::detect()?;
        device.get_version()
    })
    .await
    .map_err(|e| AppError::Unknown(format!("Thread join error: {}", e)))?
}

#[tauri::command]
#[instrument]
async fn cmd_allwinner_parse_image(image_path: String) -> Result<AllwinnerImageHeader, AppError> {
    tauri::async_runtime::spawn_blocking(move || AllwinnerImageHeader::parse(Path::new(&image_path)))
        .await
        .map_err(|e| AppError::Unknown(format!("Thread join error: {}", e)))?
}

#[tauri::command]
#[instrument(skip(app))]
async fn cmd_allwinner_flash_image(app: AppHandle, image_path: String) -> Result<(), AppError> {
    info!("Starting Allwinner flash: {}", image_path);
    // We detect inside the blocking thread to avoid stalling the async runtime

    let cb = {
        let app_handle = app.clone();
        move |progress: FlashProgress| {
            let _ = emit_progress(
                &app_handle,
                "flash",
                progress.percent as u32,
                &progress.operation,
                Some(format!(
                    "Partition: {:?} | {}/{} bytes",
                    progress.partition, progress.bytes_transferred, progress.total_bytes
                )),
            );
        }
    };

    // Run blocking flash in thread
    tauri::async_runtime::spawn_blocking(move || {
        // Re-detect inside thread or pass handle? Passing handle is tricky with Send.
        // Usually safer to create new instance in thread if lightweight, or pass Arc<Mutex>.
        // Assuming AllwinnerDevice::detect() is cheap and stateless:
        let device = AllwinnerDevice::detect()?;
        device.flash_image(Path::new(&image_path), Some(Box::new(cb)))
    })
    .await
    .map_err(|e| AppError::Unknown(format!("Thread join error: {}", e)))??;

    Ok(())
}

/// List serial ports
#[tauri::command]
#[instrument]
async fn cmd_list_serial_ports() -> Result<Vec<String>, AppError> {
    tauri::async_runtime::spawn_blocking(move || list_serial_ports().map_err(AppError::from))
        .await
        .map_err(|e| AppError::Unknown(format!("Thread join error: {}", e)))?
}

#[tauri::command]
#[instrument]
async fn cmd_flash_image(image_path: String, target_device: String) -> Result<(), AppError> {
    info!("Generic flash image: {} to {}", image_path, target_device);
    tauri::async_runtime::spawn_blocking(move || {
        preflight(Path::new(&image_path), &target_device)?;
        flash_image(Path::new(&image_path), &target_device)
    })
    .await
    .map_err(|e| AppError::Unknown(format!("Thread join error: {}", e)))?
}

#[tauri::command]
#[instrument(skip(state))]
async fn cmd_download_assets(
    state: State<'_, AppState>,
    profile: DeviceProfile,
) -> Result<String, AppError> {
    info!("Downloading assets for: {}", profile.soc);
    let settings = state.settings.read().await.clone();
    let base_url = std::env::var("PHOENIX_ASSETS_BASE_URL")
        .map_err(|_| AppError::AssetBaseUrlMissing("PHOENIX_ASSETS_BASE_URL".to_string()))?;
    let filename = format!("{}.tar.gz", profile.soc);
    let destination_dir = PathBuf::from(settings.cache_path)
        .join("assets")
        .join(&profile.soc);
    fs::create_dir_all(&destination_dir)
        .await
        .map_err(AppError::from)?;
    let destination = destination_dir.join(&filename);
    let url = format!("{}/{}", base_url.trim_end_matches('/'), filename);
    download_file(&url, &destination)
        .await
        .map_err(AppError::from)?;
    Ok(destination.to_string_lossy().to_string())
}

/// Create new device config
#[tauri::command]
#[instrument]
async fn cmd_create_config(soc: String, name: String) -> Result<DeviceConfig, AppError> {
    Ok(create_default_config(&soc, &name))
}

/// Load device config from file
#[tauri::command]
#[instrument]
async fn cmd_load_config(path: String) -> Result<DeviceConfig, AppError> {
    DeviceConfig::from_file(&path).map_err(AppError::from)
}

/// Save device config to file
#[tauri::command]
#[instrument]
async fn cmd_save_config(config: DeviceConfig, path: String) -> Result<(), AppError> {
    config.to_file(&path).map_err(AppError::from)
}

/// Validate device config
#[tauri::command]
#[instrument]
async fn cmd_validate_config(config: DeviceConfig) -> Result<(), AppError> {
    config.validate()
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct PatchPlanResponse {
    report: CompatibilityReport,
    plan: PatchPlan,
}

#[tauri::command]
#[instrument(skip(app))]
async fn cmd_check_compatibility(
    app: AppHandle,
    profile: String,
    firmware: String,
    os: Option<String>,
    version: Option<String>,
    kernel: Option<String>,
) -> Result<CompatibilityReport, AppError> {
    info!("Checking compatibility for {}", firmware);
    app.emit(
        "workflow:phase",
        WorkflowPhaseEvent::new(Phase::Check, PhaseStatus::Started, None),
    )
    .map_err(|e| AppError::Unknown(e.to_string()))?;

    let yaml = fs::read_to_string(&profile).await.map_err(AppError::from)?;

    let report = tauri::async_runtime::spawn_blocking(move || {
        DeviceConfig::validate_schema_yaml(&yaml)?;
        let config = DeviceConfig::from_str(&yaml).map_err(AppError::from)?;
        config.validate()?;

        let hardware = resolve_hardware_profile(&config);
        let firmware_target = resolve_firmware_target(
            Path::new(&firmware),
            os.as_deref(),
            version.as_deref(),
            kernel.as_deref(),
        )?;

        let matrix = CompatibilityMatrix::default_matrix();
        Ok(matrix.evaluate(hardware, firmware_target))
    })
    .await
    .map_err(|e| AppError::Unknown(format!("Thread join error: {}", e)))??;

    app.emit(
        "workflow:phase",
        WorkflowPhaseEvent::new(Phase::Check, PhaseStatus::Completed, None),
    )
    .map_err(|e| AppError::Unknown(e.to_string()))?;

    Ok(report)
}

#[tauri::command]
#[instrument]
async fn cmd_security_scan(image_path: String) -> Result<SecurityReport, AppError> {
    info!("Starting security scan: {}", image_path);
    tauri::async_runtime::spawn_blocking(move || SecurityScanner::scan_image(Path::new(&image_path)))
        .await
        .map_err(|e| AppError::Unknown(format!("Thread join error: {}", e)))?
}

#[tauri::command]
#[instrument(skip(app))]
async fn cmd_plan_patches(
    app: AppHandle,
    profile: String,
    firmware: String,
    os: Option<String>,
    version: Option<String>,
    kernel: Option<String>,
) -> Result<PatchPlanResponse, AppError> {
    info!("Planning patches for {}", firmware);
    app.emit(
        "workflow:phase",
        WorkflowPhaseEvent::new(Phase::PatchPlan, PhaseStatus::Started, None),
    )
    .map_err(|e| AppError::Unknown(e.to_string()))?;

    let yaml = fs::read_to_string(&profile).await.map_err(AppError::from)?;

    let (report, plan) = tauri::async_runtime::spawn_blocking(move || {
        DeviceConfig::validate_schema_yaml(&yaml)?;
        let config = DeviceConfig::from_str(&yaml).map_err(AppError::from)?;
        config.validate()?;

        let hardware = resolve_hardware_profile(&config);
        let firmware_target = resolve_firmware_target(
            Path::new(&firmware),
            os.as_deref(),
            version.as_deref(),
            kernel.as_deref(),
        )?;

        let matrix = CompatibilityMatrix::default_matrix();
        let report = matrix.evaluate(hardware, firmware_target);
        let plan = build_patch_plan(&report);
        Ok((report, plan))
    })
    .await
    .map_err(|e| AppError::Unknown(format!("Thread join error: {}", e)))??;

    app.emit(
        "workflow:phase",
        WorkflowPhaseEvent::new(Phase::PatchPlan, PhaseStatus::Completed, None),
    )
    .map_err(|e| AppError::Unknown(e.to_string()))?;

    Ok(PatchPlanResponse { report, plan })
}

/// Build status for progress tracking
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BuildProgress {
    pub step: String,
    pub progress: u32,
    pub message: String,
    pub log_line: Option<String>,
}

/// Start a build
#[tauri::command]
#[instrument(skip(app, state))]
async fn cmd_start_build(
    app: AppHandle,
    state: State<'_, AppState>, // Corrected from app.state usage
    profile: String,
    board: String,
    output_dir: String,
) -> Result<(), AppError> {
    info!("Starting build for board: {}", board);
    let settings = state.settings.read().await.clone();
    let recipes_dir = PathBuf::from(&settings.tools_path).join("recipes");

    fs::metadata(&recipes_dir)
        .await
        .map_err(|e| AppError::ValidationError(format!("Recipes directory missing: {}", e)))?;

    let env = RecipeEnv {
        board: board.clone(),
        profile: profile.clone(),
        output_dir: PathBuf::from(output_dir),
        cache_dir: PathBuf::from(settings.cache_path),
        extra: Default::default(),
    };

    let pipeline = BuildPipeline::image_build(&recipes_dir);
    let app_handle = app.clone();
    let total_steps = pipeline.steps.len() as u32;

    tauri::async_runtime::spawn_blocking(move || -> Result<(), AppError> {
        for (i, step) in pipeline.steps.iter().enumerate() {
            let progress = if total_steps == 0 {
                0
            } else {
                (i as u32 * 100) / total_steps
            };
            emit_progress(
                &app_handle,
                &step.name,
                progress,
                &format!("Starting {}...", step.name),
                None,
            )?;

            let result =
                phoenix_lib::build::execute_recipe_streaming(&step.recipe, &env, |stream, line| {
                    let line_progress = progress;
                    let log_line = match stream {
                        OutputStream::Stdout => Some(line.to_string()),
                        OutputStream::Stderr => Some(line.to_string()),
                    };
                    let _ =
                        emit_progress(&app_handle, &step.name, line_progress, &step.name, log_line);
                });

            match result {
                Ok(result) => {
                    if !result.success {
                        emit_progress(
                            &app_handle,
                            &step.name,
                            progress,
                            &format!("Failed: {}", step.name),
                            Some(result.stderr),
                        )?;
                        return Err(AppError::BuildFailed(format!("Step {} failed", step.name)));
                    }
                    let next_progress = if total_steps == 0 {
                        100
                    } else {
                        ((i as u32 + 1) * 100) / total_steps
                    };
                    emit_progress(
                        &app_handle,
                        &step.name,
                        next_progress,
                        &format!("Completed {}", step.name),
                        Some(result.stdout),
                    )?;
                }
                Err(e) => {
                    emit_progress(
                        &app_handle,
                        &step.name,
                        progress,
                        &format!("Error: {}", step.name),
                        Some(e.to_string()),
                    )?;
                    return Err(AppError::BuildFailed(e.to_string()));
                }
            }
        }

        emit_progress(
            &app_handle,
            "complete",
            100,
            "Build complete!",
            Some("Build finished successfully.".to_string()),
        )?;
        Ok(())
    })
    .await
    .map_err(|e| AppError::Unknown(format!("Build thread error: {}", e)))??;

    Ok(())
}

fn emit_progress(
    app: &AppHandle,
    step: &str,
    progress: u32,
    message: &str,
    log: Option<String>,
) -> Result<(), AppError> {
    app.emit(
        "build-progress",
        BuildProgress {
            step: step.to_string(),
            progress,
            message: message.to_string(),
            log_line: log,
        },
    )
    .map_err(|e| AppError::Unknown(e.to_string()))
}

/// Get system info
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemInfo {
    pub os: String,
    pub arch: String,
    pub rust_available: bool,
    pub has_usb_access: bool,
}

#[tauri::command]
#[instrument]
async fn cmd_get_system_info() -> Result<SystemInfo, AppError> {
    Ok(SystemInfo {
        os: std::env::consts::OS.to_string(),
        arch: std::env::consts::ARCH.to_string(),
        rust_available: which::which("rustc").is_ok(),
        has_usb_access: true,
    })
}

/// Load application settings
#[tauri::command]
#[instrument(skip(app, state))]
async fn cmd_get_settings(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<AppSettings, AppError> {
    let path = resolve_settings_path(&app, &state).await?;
    match fs::read_to_string(&path).await {
        Ok(contents) => {
            let loaded: AppSettings = serde_json::from_str(&contents)
                .map_err(|e| AppError::SettingsLoadFailed(e.to_string()))?;
            let mut current = state.settings.write().await;
            *current = loaded.clone();
            Ok(loaded)
        }
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
            let settings = state.settings.read().await;
            Ok(settings.clone())
        }
        Err(err) => Err(AppError::SettingsLoadFailed(err.to_string())),
    }
}

/// Save application settings
#[tauri::command]
#[instrument(skip(app, state))]
async fn cmd_save_settings(
    app: AppHandle,
    state: State<'_, AppState>,
    settings: AppSettings,
) -> Result<(), AppError> {
    let path = resolve_settings_path(&app, &state).await?;
    let payload = serde_json::to_string_pretty(&settings)
        .map_err(|e| AppError::SettingsSaveFailed(e.to_string()))?;
    fs::write(&path, payload)
        .await
        .map_err(|e| AppError::SettingsSaveFailed(e.to_string()))?;

    let mut current = state.settings.write().await;
    *current = settings;
    Ok(())
}

async fn resolve_settings_path(app: &AppHandle, state: &AppState) -> Result<PathBuf, AppError> {
    if let Some(path) = state.settings_path.read().await.clone() {
        return Ok(path);
    }
    let config_dir = app
        .path()
        .app_config_dir()
        .map_err(|e| AppError::IoError(e.to_string()))?;
    fs::create_dir_all(&config_dir)
        .await
        .map_err(AppError::from)?;
    let path = config_dir.join("settings.json");
    let mut stored = state.settings_path.write().await;
    *stored = Some(path.clone());
    Ok(path)
}

#[tauri::command]
#[instrument]
async fn cmd_forensics_deep_scan(device: Option<String>) -> Result<ForensicsReport, AppError> {
    info!("Starting forensics deep scan");
    tauri::async_runtime::spawn_blocking(move || perform_deep_scan(device.as_deref()))
        .await
        .map_err(|e| AppError::Unknown(format!("Thread join error: {}", e)))?
}

#[tauri::command]
#[instrument]
async fn cmd_list_remote_configs() -> Result<Vec<RemoteConfig>, AppError> {
    let db = RemoteConfigDatabase::default_database();
    Ok(db.remotes)
}

#[tauri::command]
#[instrument]
async fn cmd_generate_remote_conf(name: String) -> Result<String, AppError> {
    let db = RemoteConfigDatabase::default_database();
    if let Some(config) = db.find_by_name(&name) {
        Ok(config.generate_remote_conf())
    } else {
        Err(AppError::NotFound(format!(
            "Remote config not found: {}",
            name
        )))
    }
}

#[tauri::command]
#[instrument]
async fn cmd_extract_archive(archive_path: String, output_dir: String) -> Result<(), AppError> {
    info!("Extracting archive: {}", archive_path);
    tauri::async_runtime::spawn_blocking(move || {
        extract_archive(Path::new(&archive_path), Path::new(&output_dir))
    })
    .await
    .map_err(|e| AppError::Unknown(format!("Thread join error: {}", e)))?
}

#[tauri::command]
#[instrument]
async fn cmd_get_firmware_recommendations(
    profile: HardwareProfile,
) -> Result<Vec<FirmwareRecommendation>, AppError> {
    Ok(get_recommendations(&profile))
}

pub fn run() {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Load profiles from file or use defaults
    let profiles = ProfileDatabase::from_file("profiles.toml").unwrap_or_else(|_| {
        error!("Failed to load profiles.toml, using defaults");
        default_profiles()
    });

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .manage(AppState {
            settings_path: Arc::new(RwLock::new(None)),
            settings: Arc::new(RwLock::new(AppSettings::default())),
            profiles: Arc::new(RwLock::new(profiles)),
            amlogic_device: Arc::new(tokio::sync::Mutex::new(None)),
        })
        .invoke_handler(tauri::generate_handler![
            cmd_resolve_profile,
            cmd_detect_devices,
            cmd_amlogic_detect,
            cmd_amlogic_flash_image,
            cmd_amlogic_extract_image,
            cmd_list_serial_ports,
            cmd_flash_image,
            cmd_download_assets,
            cmd_create_config,
            cmd_load_config,
            cmd_save_config,
            cmd_validate_config,
            cmd_check_compatibility,
            cmd_plan_patches,
            cmd_start_build,
            cmd_get_system_info,
            cmd_get_settings,
            cmd_save_settings,
            cmd_forensics_deep_scan,
            cmd_security_scan,
            cmd_list_remote_configs,
            cmd_generate_remote_conf,
            cmd_extract_archive,
            cmd_get_firmware_recommendations,
            cmd_rockchip_detect,
            cmd_rockchip_parse_image,
            cmd_rockchip_extract_image,
            cmd_rockchip_parse_parameter,
            cmd_allwinner_detect,
            cmd_allwinner_parse_image,
            cmd_allwinner_flash_image,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod performance_test {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use std::time::{Duration, Instant};
    use tempfile::tempdir;
    use tokio::time::sleep;

    #[tokio::test()]
    async fn test_blocking_extract_performance() {
        // 1. Setup: Create a large dummy archive (tar.gz)
        let dir = tempdir().unwrap();
        let archive_path = dir.path().join("test_archive.tar.gz");
        let output_dir = dir.path().join("output");
        std::fs::create_dir(&output_dir).unwrap();

        // Create a large file (20MB)
        let large_file_path = dir.path().join("large_file.bin");
        {
            let mut f = File::create(&large_file_path).unwrap();
            let chunk = vec![0u8; 1024 * 1024]; // 1MB
            for _ in 0..20 {
                f.write_all(&chunk).unwrap();
            }
        }

        // Create tar.gz
        {
            let tar_gz = File::create(&archive_path).unwrap();
            let enc = flate2::write::GzEncoder::new(tar_gz, flate2::Compression::default());
            let mut tar = tar::Builder::new(enc);
            tar.append_path_with_name(&large_file_path, "large_file.bin").unwrap();
            // finish() writes EOF blocks and returns inner writer (encoder) in recent versions if into_inner is used?
            // Actually tar::Builder::finish() does not return inner.
            // Correct way:
            let enc = tar.into_inner().unwrap();
            enc.finish().unwrap();
        }

        // 2. Measure Event Loop Latency
        let (tx, mut rx) = tokio::sync::mpsc::channel(100);

        // Spawn a monitoring task that should run frequently
        let monitor_handle = tokio::spawn(async move {
            let mut last_tick = Instant::now();
            loop {
                // Sleep for a short duration
                sleep(Duration::from_millis(10)).await;
                let now = Instant::now();
                let elapsed = now.duration_since(last_tick);
                // We expect elapsed to be close to 10ms. If it's much larger, we were blocked.
                if tx.send(elapsed).await.is_err() {
                    break;
                }
                last_tick = now;
            }
        });

        // 3. Run Extraction
        let archive_str = archive_path.to_string_lossy().to_string();
        let output_str = output_dir.to_string_lossy().to_string();

        // Wait for monitor to start ticking
        sleep(Duration::from_millis(50)).await;

        info!("Starting extraction test...");
        // Call the command
        cmd_extract_archive(archive_str, output_str).await.expect("Extraction failed");
        info!("Extraction complete.");

        // Wait for one more tick to capture the latency
        sleep(Duration::from_millis(20)).await;

        // Stop monitor
        monitor_handle.abort();

        // 4. Analyze Latency
        let mut max_latency = Duration::from_millis(0);
        let mut count = 0;
        while let Ok(latency) = rx.try_recv() {
            count += 1;
            if latency > max_latency {
                max_latency = latency;
            }
        }

        println!("Test stats: {} ticks monitored. Max latency: {:?}", count, max_latency);

        assert!(max_latency < Duration::from_millis(100),
            "Event loop blocked for too long! Max latency: {:?}. Expected < 100ms",
            max_latency
        );
    }
}
