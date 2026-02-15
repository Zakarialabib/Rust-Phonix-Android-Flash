use anyhow::{Context, Result};
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::path::Path;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

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

impl Asset {
    /// Download the asset to a destination path and verify its checksum
    pub async fn download(&self, destination: &Path) -> Result<()> {
        download_file(&self.url, destination, self.sha256.as_deref()).await
    }
}

impl AssetRegistry {
    pub async fn from_url(url: &str) -> Result<Self, AppError> {
        let response = reqwest::get(url)
            .await
            .map_err(|e| AppError::NetworkError(e.to_string()))?;
        if !response.status().is_success() {
            return Err(AppError::NetworkError(format!(
                "Failed to fetch registry: {}",
                response.status()
            )));
        }
        let registry = response
            .json::<AssetRegistry>()
            .await
            .map_err(|e| AppError::ConfigError(e.to_string()))?;
        Ok(registry)
    }

    pub fn from_file(path: &Path) -> Result<Self, AppError> {
        let content =
            std::fs::read_to_string(path).map_err(|e| AppError::IoError(e.to_string()))?;
        let registry =
            serde_json::from_str(&content).map_err(|e| AppError::ConfigError(e.to_string()))?;
        Ok(registry)
    }

    pub fn get_url(&self, key: &str) -> Option<String> {
        self.assets.get(key).map(|a| a.url.clone())
    }
}

/// Download a file from a URL to a destination path with optional SHA256 verification
pub async fn download_file(
    url: &str,
    destination: &Path,
    expected_sha256: Option<&str>,
) -> Result<()> {
    let response = reqwest::get(url).await.context("Failed to make request")?;

    if !response.status().is_success() {
        return Err(anyhow::anyhow!(
            "Request failed with status: {}",
            response.status()
        ));
    }

    let mut file = File::create(destination)
        .await
        .context("Failed to create file")?;
    let mut stream = response.bytes_stream();
    let mut hasher = Sha256::new();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.context("Error while reading chunk")?;
        file.write_all(&chunk)
            .await
            .context("Failed to write chunk")?;
        if expected_sha256.is_some() {
            hasher.update(&chunk);
        }
    }

    file.flush().await.context("Failed to flush file")?;

    if let Some(expected) = expected_sha256 {
        let actual = format!("{:x}", hasher.finalize());
        if actual != expected {
            // Close file before removal
            drop(file);
            let _ = tokio::fs::remove_file(destination).await;
            return Err(anyhow::anyhow!(
                "Checksum mismatch for {}: expected {}, got {}",
                url,
                expected,
                actual
            ));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;

    #[tokio::test]
    async fn test_download_file_with_valid_hash() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let url = format!("http://{}", addr);

        let data = b"hello world";
        // sha256 of "hello world"
        let expected_hash = "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9";

        tokio::spawn(async move {
            if let Ok((mut socket, _)) = listener.accept().await {
                let mut buf = [0u8; 1024];
                let _ = socket.read(&mut buf).await;
                let response = format!("HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n", data.len());
                let _ = socket.write_all(response.as_bytes()).await;
                let _ = socket.write_all(data).await;
            }
        });

        let dir = tempdir().unwrap();
        let dest = dir.path().join("test_valid.txt");

        let result = download_file(&url, &dest, Some(expected_hash)).await;
        assert!(result.is_ok(), "Download failed: {:?}", result.err());
        assert!(dest.exists());
        let content = std::fs::read_to_string(dest).unwrap();
        assert_eq!(content, "hello world");
    }

    #[tokio::test]
    async fn test_download_file_with_invalid_hash() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let url = format!("http://{}", addr);

        let data = b"hello world";
        let wrong_hash = "wronghash";

        tokio::spawn(async move {
            if let Ok((mut socket, _)) = listener.accept().await {
                let mut buf = [0u8; 1024];
                let _ = socket.read(&mut buf).await;
                let response = format!("HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n", data.len());
                let _ = socket.write_all(response.as_bytes()).await;
                let _ = socket.write_all(data).await;
            }
        });

        let dir = tempdir().unwrap();
        let dest = dir.path().join("test_invalid.txt");

        let result = download_file(&url, &dest, Some(wrong_hash)).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Checksum mismatch"));
        assert!(!dest.exists()); // Should have been deleted
    }

    #[tokio::test]
    async fn test_download_file_no_hash() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let url = format!("http://{}", addr);

        let data = b"hello world";

        tokio::spawn(async move {
            if let Ok((mut socket, _)) = listener.accept().await {
                let mut buf = [0u8; 1024];
                let _ = socket.read(&mut buf).await;
                let response = format!("HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n", data.len());
                let _ = socket.write_all(response.as_bytes()).await;
                let _ = socket.write_all(data).await;
            }
        });

        let dir = tempdir().unwrap();
        let dest = dir.path().join("test_no_hash.txt");

        let result = download_file(&url, &dest, None).await;
        assert!(result.is_ok());
        assert!(dest.exists());
        let content = std::fs::read_to_string(dest).unwrap();
        assert_eq!(content, "hello world");
    }
}
