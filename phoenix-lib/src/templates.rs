//! Template engine for generating device-specific files

use anyhow::{Context, Result};
use handlebars::Handlebars;
use serde::Serialize;
use std::path::Path;

/// Template context for device tree generation
#[derive(Debug, Serialize)]
pub struct DtsContext {
    pub device_name: String,
    pub soc: String,
    pub soc_family: String,
    pub reference_dtb: String,
    pub memory_size_mb: u32,
    pub has_wifi: bool,
    pub wifi_chip: String,
    pub has_ethernet: bool,
    pub led_gpio: Option<String>,
}

/// Template context for kernel config fragment
#[derive(Debug, Serialize)]
pub struct KconfigContext {
    pub soc: String,
    pub enable_panfrost: bool,
    pub enable_vdec: bool,
    pub enable_wifi: bool,
    pub wifi_driver: String,
    pub enable_cec: bool,
    pub cma_size_mb: u32,
}

/// Template context for extlinux.conf
#[derive(Debug, Serialize)]
pub struct ExtlinuxContext {
    pub label: String,
    pub kernel_path: String,
    pub dtb_path: String,
    pub root_device: String,
    pub console: String,
    pub extra_args: String,
}

/// Template engine wrapper
pub struct TemplateEngine {
    handlebars: Handlebars<'static>,
}

impl TemplateEngine {
    /// Create a new template engine with built-in templates
    pub fn new() -> Result<Self> {
        let mut handlebars = Handlebars::new();
        handlebars.set_strict_mode(true);

        // Register built-in templates
        handlebars.register_template_string("dts", include_str!("../templates/device.dts.hbs"))?;
        handlebars.register_template_string("kconfig", include_str!("../templates/kconfig.hbs"))?;
        handlebars
            .register_template_string("extlinux", include_str!("../templates/extlinux.conf.hbs"))?;

        Ok(Self { handlebars })
    }

    /// Load additional templates from directory
    pub fn load_templates_from_dir(&mut self, dir: &Path) -> Result<()> {
        if !dir.exists() {
            return Ok(());
        }

        for entry in walkdir::WalkDir::new(dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().is_some_and(|ext| ext == "hbs"))
        {
            let name = entry
                .path()
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("");
            let content = std::fs::read_to_string(entry.path())?;
            self.handlebars.register_template_string(name, &content)?;
        }

        Ok(())
    }

    /// Render a template with context
    pub fn render<T: Serialize>(&self, template_name: &str, context: &T) -> Result<String> {
        self.handlebars
            .render(template_name, context)
            .with_context(|| format!("Failed to render template: {}", template_name))
    }

    /// Render and write to file
    pub fn render_to_file<T: Serialize>(
        &self,
        template_name: &str,
        context: &T,
        output: &Path,
    ) -> Result<()> {
        let content = self.render(template_name, context)?;

        // Ensure parent directory exists
        if let Some(parent) = output.parent() {
            std::fs::create_dir_all(parent)?;
        }

        std::fs::write(output, content)?;
        Ok(())
    }
}

impl Default for TemplateEngine {
    fn default() -> Self {
        Self::new().expect("Failed to create template engine")
    }
}
