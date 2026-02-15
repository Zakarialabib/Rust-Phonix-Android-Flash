//! Security scanning and malware detection for Android TV box firmware
//!
//! Detects known malware patterns commonly found in cheap Chinese TV boxes,
//! including the notorious "Corejava" botnet and click-fraud malware.

use serde::{Deserialize, Serialize};
use std::path::Path;
use tracing::{debug, warn};

use crate::error::AppError;

/// Threat severity levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ThreatLevel {
    Critical,
    High,
    Medium,
    Low,
    Info,
}

impl std::fmt::Display for ThreatLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ThreatLevel::Critical => write!(f, "CRITICAL"),
            ThreatLevel::High => write!(f, "HIGH"),
            ThreatLevel::Medium => write!(f, "MEDIUM"),
            ThreatLevel::Low => write!(f, "LOW"),
            ThreatLevel::Info => write!(f, "INFO"),
        }
    }
}

/// A detected security threat
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThreatDetection {
    pub name: String,
    pub severity: ThreatLevel,
    pub path: String,
    pub description: String,
    pub remediation: String,
}

/// Known malware signatures for detection
#[derive(Debug, Clone)]
pub struct MalwareSignature {
    pub name: &'static str,
    pub paths: &'static [&'static str],
    pub severity: ThreatLevel,
    pub description: &'static str,
    pub remediation: &'static str,
}

/// Database of known malware signatures found in Chinese TV boxes
pub const KNOWN_MALWARE: &[MalwareSignature] = &[
    MalwareSignature {
        name: "Corejava Botnet",
        paths: &[
            "/data/system/Corejava",
            "/data/system/shared_prefs/openpreserve.xml",
            "/data/system/shared_prefs/clicker.xml",
        ],
        severity: ThreatLevel::Critical,
        description: "Click-fraud botnet that enrolls devices in botnets for ad fraud. \
                      Communicates with C2 servers and can install additional malware.",
        remediation: "Flash a clean custom ROM (SlimBoxTV or Aidan's ROM). \
                      Factory reset is NOT sufficient as malware persists in /system.",
    },
    MalwareSignature {
        name: "BadBox Malware",
        paths: &[
            "/system/app/Ambient",
            "/system/priv-app/Provision",
        ],
        severity: ThreatLevel::High,
        description: "Pre-installed malware that creates hidden proxy nodes and \
                      enables residential proxy fraud.",
        remediation: "Replace firmware with trusted custom ROM.",
    },
    MalwareSignature {
        name: "Suspicious OTA Server",
        paths: &[
            "/system/build.prop", // Will check content for malicious OTA URLs
        ],
        severity: ThreatLevel::Medium,
        description: "Device configured to receive OTA updates from untrusted servers \
                      that may push malware updates.",
        remediation: "Disable OTA updates or flash clean firmware.",
    },
    MalwareSignature {
        name: "Chinese Analytics SDK",
        paths: &[
            "/system/app/UMengSDK",
            "/system/app/TencentStat",
            "/data/data/com.umeng",
        ],
        severity: ThreatLevel::Low,
        description: "Chinese analytics/tracking SDKs that may collect usage data \
                      and send to servers in China.",
        remediation: "Remove or disable these packages for privacy.",
    },
    MalwareSignature {
        name: "Fake Google Services",
        paths: &[
            "/system/app/FakeGms",
            "/system/priv-app/GmsCore", // May need content check
        ],
        severity: ThreatLevel::Medium,
        description: "Fake or modified Google Mobile Services that may intercept \
                      authentication tokens or inject ads.",
        remediation: "Install official GApps package from trusted source.",
    },
];

/// Security scan result for a firmware image or device
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SecurityReport {
    pub is_infected: bool,
    pub threats: Vec<ThreatDetection>,
    pub recommendations: Vec<String>,
    pub scan_path: String,
    pub scan_type: ScanType,
}

/// Type of security scan performed
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ScanType {
    /// Scan an extracted/mounted firmware image
    Image,
    /// Scan a live device via ADB
    LiveDevice,
    /// Scan a backup archive
    Backup,
}

/// Security scanner for firmware images and devices
pub struct SecurityScanner;

impl SecurityScanner {
    /// Scan a mounted/extracted firmware image for malware
    pub fn scan_image(image_path: &Path) -> Result<SecurityReport, AppError> {
        let mut threats = Vec::new();
        let mut recommendations = Vec::new();

        if !image_path.exists() {
            return Err(AppError::ValidationError(format!(
                "Image path does not exist: {}",
                image_path.display()
            )));
        }

        debug!("Scanning image for malware: {}", image_path.display());

        // Check for known malware signatures
        for signature in KNOWN_MALWARE {
            for malware_path in signature.paths {
                let full_path = image_path.join(malware_path.trim_start_matches('/'));
                
                if full_path.exists() {
                    warn!(
                        "Malware detected: {} at {}",
                        signature.name,
                        full_path.display()
                    );
                    
                    threats.push(ThreatDetection {
                        name: signature.name.to_string(),
                        severity: signature.severity.clone(),
                        path: malware_path.to_string(),
                        description: signature.description.to_string(),
                        remediation: signature.remediation.to_string(),
                    });
                    
                    // Only report each signature once
                    break;
                }
            }
        }

        // Check for suspicious build.prop entries
        let build_prop_path = image_path.join("system/build.prop");
        if build_prop_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&build_prop_path) {
                Self::check_build_prop(&content, &mut threats, &mut recommendations);
            }
        }

        // Generate recommendations based on findings
        if !threats.is_empty() {
            recommendations.push(
                "Flash a clean custom ROM such as SlimBoxTV or Aidan's ROM".to_string()
            );
            recommendations.push(
                "Backup any personal data before re-flashing".to_string()
            );
            
            let has_critical = threats.iter().any(|t| t.severity == ThreatLevel::Critical);
            if has_critical {
                recommendations.push(
                    "⚠️ CRITICAL: Do not connect this device to networks with sensitive data"
                        .to_string()
                );
            }
        }

        let is_infected = !threats.is_empty();

        Ok(SecurityReport {
            is_infected,
            threats,
            recommendations,
            scan_path: image_path.display().to_string(),
            scan_type: ScanType::Image,
        })
    }

    /// Scan a live device via ADB (stub - requires ADB connection)
    pub fn scan_live_device(device_serial: &str) -> Result<SecurityReport, AppError> {
        debug!("Scanning live device: {}", device_serial);
        
        if which::which("adb").is_err() {
            return Err(AppError::CommandFailed("ADB tool not found in PATH".to_string()));
        }

        // Verify device is online
        let state = std::process::Command::new("adb")
            .args(["-s", device_serial, "get-state"])
            .output()
            .map_err(|e| AppError::CommandFailed(format!("Failed to check device state: {}", e)))?;

        if !state.status.success() {
             return Err(AppError::DeviceNotFound(format!("Device {} not found or offline", device_serial)));
        }

        let mut threats = Vec::new();
        let mut recommendations = Vec::new();

        // Helper to run ADB command
        let run_adb = |args: &[&str]| -> Result<(String, bool), AppError> {
            let output = std::process::Command::new("adb")
                .args(["-s", device_serial, "shell"])
                .args(args)
                .output()
                .map_err(|e| AppError::CommandFailed(format!("Failed to run adb: {}", e)))?;
            
            Ok((String::from_utf8_lossy(&output.stdout).to_string(), output.status.success()))
        };

        // 1. Check for known malware paths using `test -e`
        let known_malware_paths = [
            "/data/system/Corejava",
            "/system/xbin/fp_check", 
            "/system/bin/rtk_fp_check",
            "/data/system/shared_prefs/openpreserve.xml",
        ];

        for path in known_malware_paths {
            // Check existence using `test -e <path>`
            // If file exists, exit code is 0 (success=true). If not, exit code is 1.
            if let Ok((_, true)) = run_adb(&["test", "-e", path]) {
                threats.push(ThreatDetection {
                    name: "Known Malware Artifact".to_string(),
                    severity: ThreatLevel::Critical,
                    path: path.to_string(),
                    description: "Found file associated with known Android TV box malware".to_string(),
                    remediation: "Flash clean firmware immediately".to_string(),
                });
            }
        }

        // 2. Check build.prop
        if let Ok((build_prop, true)) = run_adb(&["cat", "/system/build.prop"]) {
             Self::check_build_prop(&build_prop, &mut threats, &mut recommendations);
        }

        // 3. Check for suspicious packages
        if let Ok((packages, true)) = run_adb(&["pm", "list", "packages"]) {
            let suspicious_pkgs = ["com.android.system.corejava", "com.fota.update", "com.adups.fota"];
            for pkg in suspicious_pkgs {
                if packages.contains(pkg) {
                    threats.push(ThreatDetection {
                        name: "Malicious Package".to_string(),
                        severity: ThreatLevel::High,
                        path: pkg.to_string(),
                        description: "Known malicious package installed".to_string(),
                        remediation: "Uninstall package via ADB or reflash".to_string(),
                    });
                }
            }
        }

        let is_infected = !threats.is_empty();

        Ok(SecurityReport {
            is_infected,
            threats,
            recommendations,
            scan_path: format!("adb://{}", device_serial),
            scan_type: ScanType::LiveDevice,
        })
    }

    /// Check build.prop for suspicious OTA servers and other red flags
    fn check_build_prop(
        content: &str,
        threats: &mut Vec<ThreatDetection>,
        recommendations: &mut Vec<String>,
    ) {
        // Check for suspicious OTA server URLs
        let suspicious_ota_patterns = [
            "ota.tanix.co",
            "update.chinaonlinetv.com",
            "ota.cheapbox.cn",
            "update.amlogic-firmware.com",
        ];

        for line in content.lines() {
            if line.starts_with("ro.build.ota_url=") || line.starts_with("ro.ota.server_uri=") {
                for pattern in &suspicious_ota_patterns {
                    if line.contains(pattern) {
                        threats.push(ThreatDetection {
                            name: "Suspicious OTA Server".to_string(),
                            severity: ThreatLevel::Medium,
                            path: "/system/build.prop".to_string(),
                            description: format!(
                                "Device configured to receive updates from untrusted server: {}",
                                pattern
                            ),
                            remediation: "Disable OTA updates or remove this property".to_string(),
                        });
                        break;
                    }
                }
            }

            // Check for debug/insecure build flags
            if line.starts_with("ro.debuggable=1")
                || line.starts_with("ro.secure=0")
                || line.starts_with("ro.adb.secure=0")
            {
                recommendations.push(format!(
                    "Insecure build flag detected: {} - May allow unauthorized access",
                    line.split('=').next().unwrap_or("unknown")
                ));
            }
        }
    }

    /// Quick check for the most critical malware (Corejava)
    pub fn quick_check_corejava(image_path: &Path) -> bool {
        let corejava_path = image_path.join("data/system/Corejava");
        corejava_path.exists()
    }
}

impl SecurityReport {
    /// Get a summary of the security scan
    pub fn summary(&self) -> String {
        if self.is_infected {
            let critical_count = self.threats.iter()
                .filter(|t| t.severity == ThreatLevel::Critical)
                .count();
            let high_count = self.threats.iter()
                .filter(|t| t.severity == ThreatLevel::High)
                .count();
            
            format!(
                "⚠️ INFECTED: {} threats detected ({} critical, {} high)",
                self.threats.len(),
                critical_count,
                high_count
            )
        } else {
            "✅ CLEAN: No known malware detected".to_string()
        }
    }

    /// Check if any critical threats were found
    pub fn has_critical_threats(&self) -> bool {
        self.threats.iter().any(|t| t.severity == ThreatLevel::Critical)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_clean_image() {
        let dir = tempdir().unwrap();
        let report = SecurityScanner::scan_image(dir.path()).unwrap();
        assert!(!report.is_infected);
        assert!(report.threats.is_empty());
    }

    #[test]
    fn test_corejava_detection() {
        let dir = tempdir().unwrap();
        let corejava_path = dir.path().join("data/system/Corejava");
        fs::create_dir_all(&corejava_path).unwrap();
        
        let report = SecurityScanner::scan_image(dir.path()).unwrap();
        assert!(report.is_infected);
        assert!(report.threats.iter().any(|t| t.name == "Corejava Botnet"));
    }

    #[test]
    fn test_quick_check_corejava() {
        let dir = tempdir().unwrap();
        assert!(!SecurityScanner::quick_check_corejava(dir.path()));
        
        let corejava_path = dir.path().join("data/system/Corejava");
        fs::create_dir_all(&corejava_path).unwrap();
        assert!(SecurityScanner::quick_check_corejava(dir.path()));
    }

    #[test]
    fn test_scan_live_device_offline() {
        // This test assumes ADB is installed but the device is not connected.
        if which::which("adb").is_ok() {
            let result = SecurityScanner::scan_live_device("non_existent_device_12345");
            // Expect DeviceNotFound because the device is surely not there
            assert!(matches!(result, Err(AppError::DeviceNotFound(_))), "Expected DeviceNotFound, got {:?}", result);
        }
    }
}
