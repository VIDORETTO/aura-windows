//! Obtains and verifies the pinned `codex-app-server` (002 TK-001, OT-002).
//!
//! Layout: `<bin_dir>/codex/<version>/` holds the extracted package plus
//! `installed.json` with the SHA-256 of the executable, which is re-checked
//! before every spawn. `AURA_CODEX_BIN` overrides everything (development and
//! tests only; a warning is logged).

use futures::StreamExt;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Pinned {
    pub version: String,
    pub asset: String,
    pub url: String,
    pub sha256: String,
    pub size: Option<u64>,
    pub executable_glob: String,
}

pub fn pinned() -> Pinned {
    toml::from_str(include_str!("../codex-version.toml")).expect("valid codex-version.toml")
}

#[derive(Debug, Error)]
pub enum BinaryError {
    #[error("download failed: {0}")]
    Download(String),
    #[error("checksum mismatch (expected {expected}, got {actual})")]
    Checksum { expected: String, actual: String },
    #[error("executable not found in package")]
    ExecutableMissing,
    #[error("installed executable was modified")]
    Tampered,
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Debug, Serialize, Deserialize)]
struct Installed {
    version: String,
    executable: PathBuf,
    sha256: String,
}

pub fn sha256_file(path: &Path) -> std::io::Result<String> {
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

fn glob_match(pattern: &str, name: &str) -> bool {
    // Minimal `*` glob, case-insensitive.
    let (p, n) = (pattern.to_ascii_lowercase(), name.to_ascii_lowercase());
    let parts: Vec<&str> = p.split('*').collect();
    let mut pos = 0usize;
    for (i, part) in parts.iter().enumerate() {
        if part.is_empty() {
            continue;
        }
        match n[pos..].find(part) {
            Some(found) if i != 0 || found == 0 => pos += found + part.len(),
            _ => return false,
        }
    }
    parts.last().is_some_and(|l| l.is_empty()) || pos == n.len()
}

fn find_executable(dir: &Path, pattern: &str) -> Option<PathBuf> {
    let mut stack = vec![dir.to_path_buf()];
    let mut found = Vec::new();
    while let Some(d) = stack.pop() {
        for entry in std::fs::read_dir(&d).ok()?.flatten() {
            let p = entry.path();
            if p.is_dir() {
                stack.push(p);
            } else if p
                .file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| glob_match(pattern, n))
            {
                found.push(p);
            }
        }
    }
    found.sort_by_key(|p| p.components().count());
    found.into_iter().next()
}

/// Returns the verified executable, downloading and installing it if needed.
pub async fn ensure(
    bin_dir: &Path,
    pin: &Pinned,
    progress: impl Fn(u64, Option<u64>) + Send,
) -> Result<PathBuf, BinaryError> {
    if let Ok(path) = std::env::var("AURA_CODEX_BIN") {
        tracing::warn!("AURA_CODEX_BIN is set; skipping download and verification");
        return Ok(PathBuf::from(path));
    }
    let target = bin_dir.join("codex").join(&pin.version);
    if let Ok(path) = verify_installed(&target) {
        return Ok(path);
    }
    let archive = bin_dir
        .join("codex")
        .join(format!("{}.download", pin.version));
    download(&pin.url, &archive, &pin.sha256, pin.size, &progress).await?;
    let staging = bin_dir
        .join("codex")
        .join(format!("{}.staging", pin.version));
    let _ = std::fs::remove_dir_all(&staging);
    std::fs::create_dir_all(&staging)?;
    extract(&archive, &staging, &pin.asset)?;
    let exe =
        find_executable(&staging, &pin.executable_glob).ok_or(BinaryError::ExecutableMissing)?;
    let rel = exe
        .strip_prefix(&staging)
        .map_err(|_| BinaryError::ExecutableMissing)?
        .to_path_buf();
    let installed = Installed {
        version: pin.version.clone(),
        sha256: sha256_file(&exe)?,
        executable: rel,
    };
    std::fs::write(
        staging.join("installed.json"),
        serde_json::to_vec_pretty(&installed).unwrap(),
    )?;
    let _ = std::fs::remove_dir_all(&target);
    std::fs::rename(&staging, &target)?;
    let _ = std::fs::remove_file(&archive);
    verify_installed(&target)
}

/// Re-checks the executable hash recorded at install time.
pub fn verify_installed(dir: &Path) -> Result<PathBuf, BinaryError> {
    let meta: Installed = serde_json::from_slice(&std::fs::read(dir.join("installed.json"))?)
        .map_err(|e| BinaryError::Io(std::io::Error::other(e)))?;
    let exe = dir.join(&meta.executable);
    if sha256_file(&exe)? != meta.sha256 {
        return Err(BinaryError::Tampered);
    }
    Ok(exe)
}

async fn download(
    url: &str,
    dest: &Path,
    expected_sha: &str,
    size_hint: Option<u64>,
    progress: &(impl Fn(u64, Option<u64>) + Send),
) -> Result<(), BinaryError> {
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let resp = reqwest::get(url)
        .await
        .map_err(|e| BinaryError::Download(e.to_string()))?;
    if !resp.status().is_success() {
        return Err(BinaryError::Download(format!("HTTP {}", resp.status())));
    }
    let total = resp.content_length().or(size_hint);
    let mut file = std::fs::File::create(dest)?;
    let mut hasher = Sha256::new();
    let mut done = 0u64;
    let mut stream = resp.bytes_stream();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| BinaryError::Download(e.to_string()))?;
        hasher.update(&chunk);
        file.write_all(&chunk)?;
        done += chunk.len() as u64;
        progress(done, total);
    }
    file.flush()?;
    let actual = hex::encode(hasher.finalize());
    if !actual.eq_ignore_ascii_case(expected_sha) {
        let _ = std::fs::remove_file(dest);
        return Err(BinaryError::Checksum {
            expected: expected_sha.to_string(),
            actual,
        });
    }
    Ok(())
}

fn extract(archive: &Path, dest: &Path, asset: &str) -> Result<(), BinaryError> {
    let file = std::fs::File::open(archive)?;
    if asset.ends_with(".tar.zst") {
        let decoder = zstd::stream::read::Decoder::new(file)?;
        tar::Archive::new(decoder).unpack(dest)?;
    } else if asset.ends_with(".zst") {
        let mut decoder = zstd::stream::read::Decoder::new(file)?;
        let name = asset.trim_end_matches(".zst");
        let mut out = std::fs::File::create(dest.join(name))?;
        std::io::copy(&mut decoder, &mut out)?;
    } else {
        std::fs::copy(archive, dest.join(asset))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{Router, routing::get};

    fn package(exe_content: &[u8]) -> Vec<u8> {
        let mut tar_bytes = Vec::new();
        {
            let mut builder = tar::Builder::new(&mut tar_bytes);
            let mut header = tar::Header::new_gnu();
            header.set_size(exe_content.len() as u64);
            header.set_mode(0o755);
            header.set_cksum();
            builder
                .append_data(&mut header, "bin/codex-app-server.exe", exe_content)
                .unwrap();
            builder.finish().unwrap();
        }
        zstd::encode_all(&tar_bytes[..], 3).unwrap()
    }

    async fn serve(bytes: Vec<u8>) -> String {
        let app = Router::new().route("/pkg.tar.zst", get(move || async move { bytes.clone() }));
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
        format!("http://{addr}/pkg.tar.zst")
    }

    fn pin(url: String, sha: String) -> Pinned {
        Pinned {
            version: "v-test".into(),
            asset: "pkg.tar.zst".into(),
            url,
            sha256: sha,
            size: None,
            executable_glob: "codex-app-server*.exe".into(),
        }
    }

    #[tokio::test]
    async fn downloads_verifies_extracts_and_detects_tampering() {
        let pkg = package(b"MZ fake exe");
        let sha = hex::encode(Sha256::digest(&pkg));
        let url = serve(pkg).await;
        let dir = tempfile::tempdir().unwrap();
        let exe = ensure(dir.path(), &pin(url, sha), |_, _| {}).await.unwrap();
        assert!(exe.ends_with("bin/codex-app-server.exe"));
        assert_eq!(std::fs::read(&exe).unwrap(), b"MZ fake exe");
        std::fs::write(&exe, b"evil").unwrap();
        let target = dir.path().join("codex").join("v-test");
        assert!(matches!(
            verify_installed(&target),
            Err(BinaryError::Tampered)
        ));
    }

    #[tokio::test]
    async fn wrong_checksum_installs_nothing() {
        let url = serve(package(b"MZ")).await;
        let dir = tempfile::tempdir().unwrap();
        let err = ensure(dir.path(), &pin(url, "00".repeat(32)), |_, _| {})
            .await
            .unwrap_err();
        assert!(matches!(err, BinaryError::Checksum { .. }));
        assert!(!dir.path().join("codex").join("v-test").exists());
    }

    #[test]
    fn pinned_file_parses() {
        let p = pinned();
        assert_eq!(p.sha256.len(), 64);
        assert!(p.url.contains(&p.version));
    }

    #[test]
    fn glob() {
        assert!(glob_match(
            "codex-app-server*.exe",
            "codex-app-server-x86_64.exe"
        ));
        assert!(glob_match("codex-app-server*.exe", "Codex-App-Server.EXE"));
        assert!(!glob_match("codex-app-server*.exe", "codex.exe"));
    }
}
