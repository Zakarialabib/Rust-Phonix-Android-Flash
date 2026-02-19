use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Phase {
    Detect,
    Backup,
    Unlock,
    Extract,
    Build,
    Flash,
    Validate,
    Check,
    PatchPlan,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PhaseStatus {
    Started,
    Completed,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowPhaseEvent {
    pub phase: Phase,
    pub status: PhaseStatus,
    pub detail: Option<String>,
}

impl WorkflowPhaseEvent {
    pub fn new(phase: Phase, status: PhaseStatus, detail: Option<String>) -> Self {
        Self {
            phase,
            status,
            detail,
        }
    }
}
