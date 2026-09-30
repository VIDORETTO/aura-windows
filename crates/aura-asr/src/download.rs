//! Model downloads with resume (`Range`), SHA-256 verification, cancellation,
//! free-space check and atomic installation (AC-003..AC-005 of 006, OT-002).
//!
//! Layout under `models_dir`:
//! * `.downloads/<model>/<file>.part` — partial files (resumed on restart);
//! * `.staging/<model>/` — verified files before the final rename;
//! * `<model>/` — installed model with `installed.json`.

use crate::catalog::ModelEntry;
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use thiserror::Error;

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum DownloadError {
    #[error("network: {0}")]
    Network(String),
    #[error("checksum mismatch for {file}")]
    Checksum { file: String },
    #[error("not enough disk space: {needed} bytes needed")]
    InsufficientSpace { needed: u64 },
    #[error("cancelled")]
    Cancelled,
    #[error("io: {0}")]
    Io(String),
}

impl From<std::io::Error> for DownloadError {
    fn from(e: std::io::Error) -> Self {
        DownloadError::Io(e.to_string())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Progress {
    pub model_id: String,
    pub bytes_done: u64,
    pub total: u64,
    pub file: String,
}

/// Free space provider (Windows: `GetDiskFreeSpaceExW`; tests: fixed value).
pub trait DiskSpace: Send + Sync {
    fn free_bytes(&self, path: &Path) -> Option<u64>;
}

pub struct UnknownSpace;
impl DiskSpace for UnknownSpace {
    fn free_bytes(&self, _path: &Path) -> Option<u64> {
        None
    }
}

#[derive(Clone, Default)]
pub struct CancelToken(Arc<AtomicBool>);

impl CancelToken {
    pub fn cancel(&self) {
        self.0.store(true, Ordering::SeqCst);
    }
    pub fn is_cancelled(&self) -> bool {
        self.0.load(Ordering::SeqCst)
    }
}

pub struct Downloader {
    pub models_dir: PathBuf,
    pub http: reqwest::Client,
    pub space: Arc<dyn DiskSpace>,
    pub retries: u32,
    pub retry_delay: Duration,
}

#[derive(Serialize, Deserialize)]
struct Installed {
    id: String,
    files: Vec<String>,
}

impl Downloader {
    pub fn new(models_dir: PathBuf) -> Self {
        Self {
            models_dir,
            http: reqwest::Client::builder()
                .connect_timeout(Duration::from_secs(15))
                .build()
                .expect("client"),
            space: Arc::new(UnknownSpace),
            retries: 3,
            retry_delay: Duration::from_secs(2),
        }
    }

    pub fn installed_dir(&self, id: &str) -> PathBuf {
        self.models_dir.join(id)
    }

    pub fn is_installed(&self, id: &str) -> bool {
        self.installed_dir(id).join("installed.json").exists()
    }

    pub fn installed(&self) -> Vec<String> {
        std::fs::read_dir(&self.models_dir)
            .into_iter()
            .flatten()
            .flatten()
            .filter(|e| e.path().join("installed.json").exists())
            .filter_map(|e| e.file_name().to_str().map(str::to_string))
            .collect()
    }

    pub fn remove(&self, id: &str) -> Result<(), DownloadError> {
        let _ = std::fs::remove_dir_all(self.installed_dir(id));
        let _ = std::fs::remove_dir_all(self.models_dir.join(".downloads").join(id));
        Ok(())
    }

    /// Downloads and installs every file of `model`.
    pub async fn install(
        &self,
        model: &ModelEntry,
        cancel: &CancelToken,
        progress: &(dyn Fn(Progress) + Send + Sync),
    ) -> Result<PathBuf, DownloadError> {
        if self.is_installed(&model.id) {
            return Ok(self.installed_dir(&model.id));
        }
        let total = model.size_bytes();
        let part_dir = self.models_dir.join(".downloads").join(&model.id);
        std::fs::create_dir_all(&part_dir)?;
        let already: u64 = model
            .files
            .iter()
            .map(|f| {
                std::fs::metadata(part_dir.join(format!("{}.part", f.path)))
                    .map(|m| m.len())
                    .unwrap_or(0)
            })
            .sum();
        let needed = ((total - already.min(total)) as f64 * 1.2) as u64;
        if let Some(free) = self.space.free_bytes(&self.models_dir)
            && free < needed
        {
            return Err(DownloadError::InsufficientSpace { needed });
        }
        let mut done_before = 0u64;
        for f in &model.files {
            let part = part_dir.join(format!("{}.part", f.path));
            let mut attempt = 0;
            loop {
                match self
                    .fetch(&f.url, &part, f.size, cancel, &|n| {
                        progress(Progress {
                            model_id: model.id.clone(),
                            bytes_done: done_before + n,
                            total,
                            file: f.path.clone(),
                        })
                    })
                    .await
                {
                    Ok(()) => break,
                    Err(DownloadError::Cancelled) => {
                        let _ = std::fs::remove_dir_all(&part_dir);
                        return Err(DownloadError::Cancelled);
                    }
                    Err(DownloadError::Network(e)) if attempt < self.retries => {
                        attempt += 1;
                        tracing::warn!("download retry {attempt}: {e}");
                        tokio::time::sleep(self.retry_delay * attempt).await;
                    }
                    Err(e) => return Err(e),
                }
            }
            if sha256_file(&part)? != f.sha256 {
                let _ = std::fs::remove_file(&part);
                return Err(DownloadError::Checksum {
                    file: f.path.clone(),
                });
            }
            done_before += f.size;
        }
        // Atomic install.
        let staging = self.models_dir.join(".staging").join(&model.id);
        let _ = std::fs::remove_dir_all(&staging);
        std::fs::create_dir_all(&staging)?;
        for f in &model.files {
            let dest = staging.join(&f.path);
            if let Some(p) = dest.parent() {
                std::fs::create_dir_all(p)?;
            }
            std::fs::rename(part_dir.join(format!("{}.part", f.path)), dest)?;
        }
        let meta = Installed {
            id: model.id.clone(),
            files: model.files.iter().map(|f| f.path.clone()).collect(),
        };
        std::fs::write(
            staging.join("installed.json"),
            serde_json::to_vec_pretty(&meta).unwrap(),
        )?;
        let target = self.installed_dir(&model.id);
        let _ = std::fs::remove_dir_all(&target);
        std::fs::rename(&staging, &target)?;
        let _ = std::fs::remove_dir_all(&part_dir);
        Ok(target)
    }

    async fn fetch(
        &self,
        url: &str,
        part: &Path,
        size: u64,
        cancel: &CancelToken,
        progress: &(dyn Fn(u64) + Send + Sync),
    ) -> Result<(), DownloadError> {
        let have = std::fs::metadata(part).map(|m| m.len()).unwrap_or(0);
        if have == size && size > 0 {
            return Ok(());
        }
        let mut req = self.http.get(url);
        if have > 0 {
            req = req.header("Range", format!("bytes={have}-"));
        }
        let resp = req
            .send()
            .await
            .map_err(|e| DownloadError::Network(e.to_string()))?;
        let status = resp.status().as_u16();
        let append = status == 206;
        if !(200..300).contains(&status) {
            return Err(DownloadError::Network(format!("HTTP {status}")));
        }
        let mut file = if append {
            std::fs::OpenOptions::new().append(true).open(part)?
        } else {
            std::fs::File::create(part)?
        };
        let mut done = if append { have } else { 0 };
        let mut stream = resp.bytes_stream();
        while let Some(chunk) = stream.next().await {
            if cancel.is_cancelled() {
                return Err(DownloadError::Cancelled);
            }
            let chunk = chunk.map_err(|e| DownloadError::Network(e.to_string()))?;
            file.write_all(&chunk)?;
            done += chunk.len() as u64;
            progress(done);
        }
        file.flush()?;
        if size > 0 && done < size {
            return Err(DownloadError::Network(format!("incomplete: {done}/{size}")));
        }
        Ok(())
    }
}

pub fn sha256_file(path: &Path) -> Result<String, DownloadError> {
    let mut f = std::fs::File::open(path)?;
    let mut h = Sha256::new();
    let mut buf = vec![0u8; 1 << 16];
    loop {
        let n = f.read(&mut buf)?;
        if n == 0 {
            break;
        }
        h.update(&buf[..n]);
    }
    Ok(hex::encode(h.finalize()))
}
