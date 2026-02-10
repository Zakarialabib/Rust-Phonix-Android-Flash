//! Build orchestration and recipe execution

use anyhow::{Context, Result};
use std::collections::HashMap;
use std::io::BufRead;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::thread;
use tracing::{debug, info, warn};

/// Recipe execution result
#[derive(Debug)]
pub struct RecipeResult {
    pub success: bool,
    pub artifact: Option<PathBuf>,
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
}

#[derive(Debug, Clone, Copy)]
pub enum OutputStream {
    Stdout,
    Stderr,
}

/// Recipe environment configuration
#[derive(Debug, Clone)]
pub struct RecipeEnv {
    pub board: String,
    pub profile: String,
    pub output_dir: PathBuf,
    pub cache_dir: PathBuf,
    pub extra: HashMap<String, String>,
}

impl Default for RecipeEnv {
    fn default() -> Self {
        Self {
            board: String::new(),
            profile: "minimal".to_string(),
            output_dir: PathBuf::from("./output"),
            cache_dir: dirs::cache_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join("phoenix"),
            extra: HashMap::new(),
        }
    }
}

/// Execute a build recipe script
pub fn execute_recipe(recipe_path: &Path, env: &RecipeEnv) -> Result<RecipeResult> {
    info!("Executing recipe: {:?}", recipe_path);

    // Determine shell based on platform
    let (shell, args) = if cfg!(target_os = "windows") {
        if recipe_path.extension().map_or(false, |ext| ext == "sh") {
            // Use WSL for shell scripts on Windows
            ("wsl", vec![recipe_path.to_string_lossy().to_string()])
        } else {
            // Use PowerShell for everything else
            ("powershell", vec!["-File".to_string(), recipe_path.to_string_lossy().to_string()])
        }
    } else {
        ("bash", vec!["-c".to_string(), recipe_path.to_string_lossy().to_string()])
    };

    // Build environment variables
    let mut cmd = Command::new(shell);
    cmd.args(&args)
        .env("PHOENIX_BOARD", &env.board)
        .env("PHOENIX_PROFILE", &env.profile)
        .env("PHOENIX_OUTPUT", &env.output_dir)
        .env("PHOENIX_CACHE", &env.cache_dir)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    // Add extra environment variables
    for (key, value) in &env.extra {
        cmd.env(format!("PHOENIX_{}", key.to_uppercase()), value);
    }

    debug!("Running: {:?}", cmd);

    let output = cmd.output().context("Failed to execute recipe")?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let exit_code = output.status.code().unwrap_or(-1);

    // Try to parse artifact path from JSON output
    let artifact = parse_artifact_path(&stdout);

    if !output.status.success() {
        warn!("Recipe failed with exit code {}: {}", exit_code, stderr);
    }

    Ok(RecipeResult {
        success: output.status.success(),
        artifact,
        stdout,
        stderr,
        exit_code,
    })
}

pub fn execute_recipe_streaming<F>(recipe_path: &Path, env: &RecipeEnv, mut on_line: F) -> Result<RecipeResult>
where
    F: FnMut(OutputStream, &str),
{
    info!("Executing recipe: {:?}", recipe_path);

    let (shell, args) = if cfg!(target_os = "windows") {
        if recipe_path.extension().map_or(false, |ext| ext == "sh") {
            ("wsl", vec![recipe_path.to_string_lossy().to_string()])
        } else {
            ("powershell", vec!["-File".to_string(), recipe_path.to_string_lossy().to_string()])
        }
    } else {
        ("bash", vec!["-c".to_string(), recipe_path.to_string_lossy().to_string()])
    };

    let mut cmd = Command::new(shell);
    cmd.args(&args)
        .env("PHOENIX_BOARD", &env.board)
        .env("PHOENIX_PROFILE", &env.profile)
        .env("PHOENIX_OUTPUT", &env.output_dir)
        .env("PHOENIX_CACHE", &env.cache_dir)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    for (key, value) in &env.extra {
        cmd.env(format!("PHOENIX_{}", key.to_uppercase()), value);
    }

    debug!("Running: {:?}", cmd);

    let mut child = cmd.spawn().context("Failed to execute recipe")?;
    let stdout = child.stdout.take().context("Failed to capture stdout")?;
    let stderr = child.stderr.take().context("Failed to capture stderr")?;

    let (tx, rx) = mpsc::channel::<(OutputStream, String)>();
    let tx_stdout = tx.clone();
    let tx_stderr = tx.clone();

    let stdout_handle = thread::spawn(move || {
        let reader = std::io::BufReader::new(stdout);
        for line in reader.lines().flatten() {
            let _ = tx_stdout.send((OutputStream::Stdout, line));
        }
    });

    let stderr_handle = thread::spawn(move || {
        let reader = std::io::BufReader::new(stderr);
        for line in reader.lines().flatten() {
            let _ = tx_stderr.send((OutputStream::Stderr, line));
        }
    });

    drop(tx);

    let mut stdout_buffer = String::new();
    let mut stderr_buffer = String::new();

    for (stream, line) in rx {
        match stream {
            OutputStream::Stdout => {
                stdout_buffer.push_str(&line);
                stdout_buffer.push('\n');
            }
            OutputStream::Stderr => {
                stderr_buffer.push_str(&line);
                stderr_buffer.push('\n');
            }
        }
        on_line(stream, &line);
    }

    let _ = stdout_handle.join();
    let _ = stderr_handle.join();

    let status = child.wait().context("Failed to wait for recipe")?;
    let exit_code = status.code().unwrap_or(-1);

    let artifact = parse_artifact_path(&stdout_buffer);

    if !status.success() {
        warn!("Recipe failed with exit code {}: {}", exit_code, stderr_buffer);
    }

    Ok(RecipeResult {
        success: status.success(),
        artifact,
        stdout: stdout_buffer,
        stderr: stderr_buffer,
        exit_code,
    })
}

/// Parse artifact path from recipe JSON output
fn parse_artifact_path(stdout: &str) -> Option<PathBuf> {
    // Look for JSON line with artifact field
    for line in stdout.lines().rev() {
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(line) {
            if let Some(artifact) = json.get("artifact").and_then(|v| v.as_str()) {
                return Some(PathBuf::from(artifact));
            }
        }
    }
    None
}

/// Build step definition
#[derive(Debug, Clone)]
pub struct BuildStep {
    pub name: String,
    pub recipe: PathBuf,
    pub required: bool,
}

/// Build pipeline with multiple steps
#[derive(Debug, Default)]
pub struct BuildPipeline {
    pub steps: Vec<BuildStep>,
}

impl BuildPipeline {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a build step
    pub fn add_step(&mut self, name: &str, recipe: PathBuf, required: bool) {
        self.steps.push(BuildStep {
            name: name.to_string(),
            recipe,
            required,
        });
    }

    /// Create a standard image build pipeline
    pub fn image_build(recipes_dir: &Path) -> Self {
        let mut pipeline = Self::new();
        pipeline.add_step("kernel", recipes_dir.join("kernel/build.sh"), true);
        pipeline.add_step("rootfs", recipes_dir.join("buildroot/build.sh"), true);
        pipeline.add_step("uboot", recipes_dir.join("uboot/build.sh"), true);
        pipeline.add_step("assemble", recipes_dir.join("image/assemble.sh"), true);
        pipeline
    }

    /// Execute all steps
    pub fn execute(&self, env: &RecipeEnv) -> Result<Vec<RecipeResult>> {
        let mut results = Vec::new();

        for step in &self.steps {
            info!("Running build step: {}", step.name);
            
            let result = execute_recipe(&step.recipe, env)?;
            
            if !result.success && step.required {
                anyhow::bail!(
                    "Required build step '{}' failed with exit code {}",
                    step.name,
                    result.exit_code
                );
            }

            results.push(result);
        }

        Ok(results)
    }
}

/// Check if required build tools are installed
pub fn check_prerequisites() -> Result<Vec<String>> {
    let mut missing = Vec::new();
    // Core tools required for all builds
    let tools = ["make", "git", "dtc", "mkimage"];
    
    // On Windows, we check for WSL presence instead of direct tools
    if cfg!(target_os = "windows") {
        if which::which("wsl").is_err() {
            missing.push("wsl (Windows Subsystem for Linux)".to_string());
        } else {
            // Check if tools exist inside WSL
            // This is a simple check; in reality we might want to run `wsl which make`
            let output = std::process::Command::new("wsl")
                .arg("which")
                .arg("make")
                .output();
                
            if output.map_or(true, |o| !o.status.success()) {
                 missing.push("Build tools in WSL (run scripts/setup_wsl.sh)".to_string());
            }
        }
    } else {
        // On Linux/macOS, check directly
        for tool in tools {
            if which::which(tool).is_err() {
                missing.push(tool.to_string());
            }
        }
    }

    Ok(missing)
}
