use serde::{Deserialize, Serialize};
use which::which;

#[derive(Debug, Serialize, Deserialize)]
pub struct DoctorReport {
    pub required_tools: Vec<ToolStatus>,
    pub optional_tools: Vec<ToolStatus>,
    pub permissions: bool,
    pub internet: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ToolStatus {
    pub name: String,
    pub path: Option<String>,
    pub found: bool,
}

pub async fn check_system() -> DoctorReport {
    let required = vec!["git", "dd", "tar", "gzip"];
    let optional = vec!["docker", "adb", "fastboot", "rkdeveloptool", "update"];

    let mut report = DoctorReport {
        required_tools: Vec::new(),
        optional_tools: Vec::new(),
        permissions: false,
        internet: false,
    };

    for tool in required {
        let (found, path) = match which(tool) {
            Ok(p) => (true, Some(p.to_string_lossy().to_string())),
            Err(_) => (false, None),
        };
        report.required_tools.push(ToolStatus { name: tool.to_string(), path, found });
    }

    for tool in optional {
        let (found, path) = match which(tool) {
            Ok(p) => (true, Some(p.to_string_lossy().to_string())),
            Err(_) => (false, None),
        };
        report.optional_tools.push(ToolStatus { name: tool.to_string(), path, found });
    }

    // Check permissions
    report.permissions = std::fs::write(".phoenix_test_write", "test").is_ok();
    if report.permissions {
        std::fs::remove_file(".phoenix_test_write").ok();
    }

    // Check connectivity
    report.internet = reqwest::get("https://github.com").await.is_ok();

    report
}
