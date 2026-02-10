use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::fs;

use crate::error::AppError;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PatchContext {
    pub workspace: PathBuf,
    pub dry_run: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PatchJob {
    Dtb(DtbPatch),
    Blob(BlobPatch),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DtbPatch {
    pub base_dtb: PathBuf,
    pub output_dtb: PathBuf,
    pub overlays: Vec<DtbOverlay>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DtbOverlay {
    pub path: String,
    pub property: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BlobPatch {
    pub source_path: PathBuf,
    pub target_path: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PatchOutcome {
    pub job: PatchJobSummary,
    pub status: PatchStatus,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PatchStatus {
    Planned,
    Skipped,
    Applied,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PatchJobSummary {
    pub kind: String,
    pub description: String,
}

pub struct PatchEngine;

impl PatchEngine {
    pub async fn apply_all(
        context: &PatchContext,
        jobs: Vec<PatchJob>,
    ) -> Result<Vec<PatchOutcome>, AppError> {
        let mut outcomes = Vec::new();

        for job in jobs {
            let outcome = match job {
                PatchJob::Dtb(patch) => apply_dtb_patch(context, patch).await,
                PatchJob::Blob(patch) => apply_blob_patch(context, patch).await,
            };
            outcomes.push(outcome);
        }

        Ok(outcomes)
    }
}

async fn apply_dtb_patch(context: &PatchContext, patch: DtbPatch) -> PatchOutcome {
    let base_exists = fs::metadata(&patch.base_dtb).await.is_ok();
    let description = format!(
        "DTB overlay to {} ({} overlays)",
        patch.output_dtb.display(),
        patch.overlays.len()
    );

    if !base_exists {
        return PatchOutcome {
            job: PatchJobSummary {
                kind: "dtb".to_string(),
                description,
            },
            status: PatchStatus::Failed,
            detail: format!("Base DTB not found: {}", patch.base_dtb.display()),
        };
    }

    if context.dry_run {
        return PatchOutcome {
            job: PatchJobSummary {
                kind: "dtb".to_string(),
                description,
            },
            status: PatchStatus::Planned,
            detail: "Dry run only".to_string(),
        };
    }

    PatchOutcome {
        job: PatchJobSummary {
            kind: "dtb".to_string(),
            description,
        },
        status: PatchStatus::Skipped,
        detail: "DTB patch application not implemented yet".to_string(),
    }
}

async fn apply_blob_patch(context: &PatchContext, patch: BlobPatch) -> PatchOutcome {
    let source_exists = fs::metadata(&patch.source_path).await.is_ok();
    let description = format!(
        "Blob copy {} -> {}",
        patch.source_path.display(),
        patch.target_path.display()
    );

    if !source_exists {
        return PatchOutcome {
            job: PatchJobSummary {
                kind: "blob".to_string(),
                description,
            },
            status: PatchStatus::Failed,
            detail: format!("Source blob not found: {}", patch.source_path.display()),
        };
    }

    if context.dry_run {
        return PatchOutcome {
            job: PatchJobSummary {
                kind: "blob".to_string(),
                description,
            },
            status: PatchStatus::Planned,
            detail: "Dry run only".to_string(),
        };
    }

    PatchOutcome {
        job: PatchJobSummary {
            kind: "blob".to_string(),
            description,
        },
        status: PatchStatus::Skipped,
        detail: "Blob patch application not implemented yet".to_string(),
    }
}
