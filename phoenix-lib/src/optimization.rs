//! Performance optimization and debloating for Android TV boxes
//!
//! Provides low RAM tweaks, ZRAM configuration, and debloating profiles
//! for devices with limited resources (especially 1GB RAM S905W boxes).

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Target RAM configuration for optimization
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum RamTarget {
    Ram512Mb,
    Ram1Gb,
    Ram2Gb,
    Ram4Gb,
}

impl RamTarget {
    pub fn from_mb(mb: u32) -> Self {
        match mb {
            0..=768 => RamTarget::Ram512Mb,
            769..=1536 => RamTarget::Ram1Gb,
            1537..=3072 => RamTarget::Ram2Gb,
            _ => RamTarget::Ram4Gb,
        }
    }

    /// Default ZRAM size for this RAM target (in MB)
    pub fn default_zram_size_mb(&self) -> u32 {
        match self {
            RamTarget::Ram512Mb => 256,
            RamTarget::Ram1Gb => 512,
            RamTarget::Ram2Gb => 768,
            RamTarget::Ram4Gb => 1024,
        }
    }
}

/// CPU governor preference
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum CpuGovernor {
    Performance,
    Schedutil,
    Interactive,
    Ondemand,
    Conservative,
    Powersave,
}

impl CpuGovernor {
    pub fn name(&self) -> &'static str {
        match self {
            CpuGovernor::Performance => "performance",
            CpuGovernor::Schedutil => "schedutil",
            CpuGovernor::Interactive => "interactive",
            CpuGovernor::Ondemand => "ondemand",
            CpuGovernor::Conservative => "conservative",
            CpuGovernor::Powersave => "powersave",
        }
    }
}

/// Optimization profile for a device
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OptimizationProfile {
    pub name: String,
    pub description: String,
    pub ram_target: RamTarget,

    // Memory optimization
    pub zram_enabled: bool,
    pub zram_size_mb: u32,
    pub swap_enabled: bool,
    pub low_ram_mode: bool,
    pub low_memory_killer_adj: Option<String>,

    // CPU optimization
    pub cpu_governor: CpuGovernor,
    pub thermal_throttle_temp_c: u32,

    // Package debloating
    pub debloat_packages: Vec<String>,

    // Custom init scripts
    pub init_scripts: Vec<InitScript>,

    // Build.prop modifications
    pub build_prop_mods: HashMap<String, String>,
}

/// An init.d script for post-boot optimization
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InitScript {
    pub name: String,
    pub priority: u8, // 00-99, lower runs first
    pub content: String,
}

impl OptimizationProfile {
    /// Create a profile optimized for 1GB RAM devices
    pub fn low_ram_profile() -> Self {
        let mut build_prop_mods = HashMap::new();

        // Essential low-RAM properties
        build_prop_mods.insert("ro.config.low_ram".to_string(), "true".to_string());
        build_prop_mods.insert("ro.lmk.critical_upgrade".to_string(), "true".to_string());
        build_prop_mods.insert("ro.lmk.upgrade_pressure".to_string(), "40".to_string());
        build_prop_mods.insert("ro.lmk.downgrade_pressure".to_string(), "60".to_string());
        build_prop_mods.insert("ro.sys.fw.bg_apps_limit".to_string(), "16".to_string());
        build_prop_mods.insert("ro.sys.fw.bservice_limit".to_string(), "5".to_string());

        // Disable heavy features
        build_prop_mods.insert("persist.sys.purgeable_assets".to_string(), "1".to_string());
        build_prop_mods.insert("debug.composition.type".to_string(), "gpu".to_string());

        Self {
            name: "Low RAM (1GB)".to_string(),
            description: "Optimized for devices with 1GB RAM - aggressive memory management"
                .to_string(),
            ram_target: RamTarget::Ram1Gb,
            zram_enabled: true,
            zram_size_mb: 512,
            swap_enabled: false,
            low_ram_mode: true,
            low_memory_killer_adj: Some("0,1,2,4,7,15".to_string()),
            cpu_governor: CpuGovernor::Interactive,
            thermal_throttle_temp_c: 85,
            debloat_packages: Self::default_debloat_list(),
            init_scripts: vec![Self::zram_init_script(512)],
            build_prop_mods,
        }
    }

    /// Create a balanced profile for 2GB RAM devices
    pub fn balanced_profile() -> Self {
        let mut build_prop_mods = HashMap::new();
        build_prop_mods.insert("ro.sys.fw.bg_apps_limit".to_string(), "24".to_string());

        Self {
            name: "Balanced (2GB)".to_string(),
            description: "Balanced performance for 2GB RAM devices".to_string(),
            ram_target: RamTarget::Ram2Gb,
            zram_enabled: true,
            zram_size_mb: 768,
            swap_enabled: false,
            low_ram_mode: false,
            low_memory_killer_adj: None,
            cpu_governor: CpuGovernor::Schedutil,
            thermal_throttle_temp_c: 90,
            debloat_packages: Self::minimal_debloat_list(),
            init_scripts: vec![Self::zram_init_script(768)],
            build_prop_mods,
        }
    }

    /// Create a performance profile for 4GB RAM devices
    pub fn performance_profile() -> Self {
        Self {
            name: "Performance (4GB)".to_string(),
            description: "Maximum performance for 4GB RAM devices".to_string(),
            ram_target: RamTarget::Ram4Gb,
            zram_enabled: false,
            zram_size_mb: 0,
            swap_enabled: false,
            low_ram_mode: false,
            low_memory_killer_adj: None,
            cpu_governor: CpuGovernor::Performance,
            thermal_throttle_temp_c: 95,
            debloat_packages: Self::minimal_debloat_list(),
            init_scripts: vec![],
            build_prop_mods: HashMap::new(),
        }
    }

    /// Generate build.prop modifications as a string
    pub fn generate_build_prop_patch(&self) -> String {
        let mut output = String::new();
        output.push_str("# Phoenix Optimization Patch\n");
        output.push_str(&format!("# Profile: {}\n\n", self.name));

        for (key, value) in &self.build_prop_mods {
            output.push_str(&format!("{}={}\n", key, value));
        }

        output
    }

    /// Generate init.d script content
    pub fn generate_init_scripts(&self) -> Vec<(String, String)> {
        self.init_scripts
            .iter()
            .map(|script| {
                let filename = format!("{:02}-{}", script.priority, script.name);
                (filename, script.content.clone())
            })
            .collect()
    }

    /// Get the list of packages to remove
    pub fn get_debloat_list(&self) -> &[String] {
        &self.debloat_packages
    }

    /// Generate a debloat script
    pub fn generate_debloat_script(&self) -> String {
        let mut output = String::new();
        output.push_str("#!/system/bin/sh\n");
        output.push_str("# Phoenix Debloat Script\n");
        output.push_str(&format!("# Profile: {}\n\n", self.name));

        for package in &self.debloat_packages {
            output.push_str(&format!(
                "pm uninstall -k --user 0 {} 2>/dev/null\n",
                package
            ));
            output.push_str(&format!(
                "pm disable-user --user 0 {} 2>/dev/null\n",
                package
            ));
        }

        output
    }

    // Helper functions
    fn default_debloat_list() -> Vec<String> {
        vec![
            // Common bloatware on Chinese TV boxes
            "com.android.email".to_string(),
            "com.android.calendar".to_string(),
            "com.android.calculator2".to_string(),
            "com.android.deskclock".to_string(),
            "com.android.wallpaper".to_string(),
            "com.android.soundrecorder".to_string(),
            "com.android.musicfx".to_string(),
            "com.android.gallery3d".to_string(),
            "com.android.providers.calendar".to_string(),
            // Chinese apps often pre-installed
            "com.tencent.qqmusic".to_string(),
            "com.qiyi.video".to_string(),
            "com.youku.phone".to_string(),
            "com.letv.android.client".to_string(),
            "com.sohu.sohuvideo".to_string(),
            "com.pplive.androidphone".to_string(),
            "com.taobao.taobao".to_string(),
            "com.alibaba.aliexpresshd".to_string(),
            // Tracking/analytics
            "com.umeng".to_string(),
            "cn.jpush.android".to_string(),
        ]
    }

    fn minimal_debloat_list() -> Vec<String> {
        vec![
            "com.android.email".to_string(),
            "com.android.calendar".to_string(),
            "com.android.soundrecorder".to_string(),
        ]
    }

    fn zram_init_script(size_mb: u32) -> InitScript {
        InitScript {
            name: "zram".to_string(),
            priority: 10,
            content: format!(
                r#"#!/system/bin/sh
# Enable ZRAM swap
ZRAM_SIZE_MB={}
ZRAM_DEV=/dev/block/zram0

if [ -e $ZRAM_DEV ]; then
    echo $(($ZRAM_SIZE_MB * 1024 * 1024)) > /sys/block/zram0/disksize
    mkswap $ZRAM_DEV
    swapon $ZRAM_DEV
    echo "ZRAM enabled: ${{ZRAM_SIZE_MB}}MB"
fi
"#,
                size_mb
            ),
        }
    }
}

/// Optimization profile database
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OptimizationDatabase {
    pub profiles: Vec<OptimizationProfile>,
}

impl OptimizationDatabase {
    /// Get default optimization profiles
    pub fn default_profiles() -> Self {
        Self {
            profiles: vec![
                OptimizationProfile::low_ram_profile(),
                OptimizationProfile::balanced_profile(),
                OptimizationProfile::performance_profile(),
            ],
        }
    }

    /// Find a profile by RAM size
    pub fn find_by_ram(&self, ram_mb: u32) -> Option<&OptimizationProfile> {
        let target = RamTarget::from_mb(ram_mb);
        self.profiles.iter().find(|p| p.ram_target == target)
    }

    /// Find a profile by name
    pub fn find_by_name(&self, name: &str) -> Option<&OptimizationProfile> {
        let lower = name.to_lowercase();
        self.profiles
            .iter()
            .find(|p| p.name.to_lowercase().contains(&lower))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_low_ram_profile() {
        let profile = OptimizationProfile::low_ram_profile();
        assert!(profile.low_ram_mode);
        assert!(profile.zram_enabled);
        assert!(profile.build_prop_mods.contains_key("ro.config.low_ram"));
    }

    #[test]
    fn test_generate_build_prop() {
        let profile = OptimizationProfile::low_ram_profile();
        let patch = profile.generate_build_prop_patch();
        assert!(patch.contains("ro.config.low_ram=true"));
    }

    #[test]
    fn test_generate_debloat_script() {
        let profile = OptimizationProfile::low_ram_profile();
        let script = profile.generate_debloat_script();
        assert!(script.contains("pm uninstall"));
        assert!(script.contains("com.android.email"));
    }

    #[test]
    fn test_find_by_ram() {
        let db = OptimizationDatabase::default_profiles();
        let profile = db.find_by_ram(1024);
        assert!(profile.is_some());
        assert!(profile.unwrap().low_ram_mode);
    }
}
