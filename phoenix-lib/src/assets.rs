use std::path::Path;
use anyhow::{Context, Result};
use tokio::fs::File;
use tokio::io::AsyncWriteExt; // for write_all if needed, but we use copy
use futures_util::StreamExt; // for next() on stream
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::error::AppError;

/// Asset Registry
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AssetRegistry {
    pub assets: HashMap<String, Asset>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Asset {
    pub name: String,
    pub url: String,
    pub sha256: Option<String>,
    pub size_bytes: Option<u64>,
}

impl AssetRegistry {
    pub async fn from_url(url: &str) -> Result<Self, AppError> {
        let response = reqwest::get(url).await.map_err(|e| AppError::NetworkError(e.to_string()))?;
        if !response.status().is_success() {
            return Err(AppError::NetworkError(format!("Failed to fetch registry: {}", response.status())));
        }
        let registry = response.json::<AssetRegistry>().await.map_err(|e| AppError::ConfigError(e.to_string()))?;
        Ok(registry)
    }

    pub fn from_file(path: &Path) -> Result<Self, AppError> {
        let content = std::fs::read_to_string(path).map_err(|e| AppError::IoError(e.to_string()))?;
        let registry = serde_json::from_str(&content).map_err(|e| AppError::ConfigError(e.to_string()))?;
        Ok(registry)
    }

    pub fn get_url(&self, key: &str) -> Option<String> {
        self.assets.get(key).map(|a| a.url.clone())
    }
}

/// Download a file from a URL to a destination path
pub async fn download_file(url: &str, destination: &Path) -> Result<()> {
    let response = reqwest::get(url).await.context("Failed to make request")?;

    if !response.status().is_success() {
        return Err(anyhow::anyhow!("Request failed with status: {}", response.status()));
    }

    let mut file = File::create(destination).await.context("Failed to create file")?;
    let mut stream = response.bytes_stream();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.context("Error while reading chunk")?;
        file.write_all(&chunk).await.context("Failed to write chunk")?;
    }
    
    file.flush().await.context("Failed to flush file")?;

    Ok(())
}
