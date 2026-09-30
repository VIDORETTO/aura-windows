//! Local logging with secret redaction and size-based rotation (AC-017, 001).

use regex::Regex;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock};
use tracing_subscriber::EnvFilter;
use tracing_subscriber::fmt::MakeWriter;
use tracing_subscriber::prelude::*;

pub const MAX_LOG_BYTES: u64 = 10 * 1024 * 1024;
pub const MAX_LOG_FILES: usize = 5;
pub const LOG_FILE_NAME: &str = "aura.log";

fn patterns() -> &'static [(Regex, &'static str)] {
    static P: OnceLock<Vec<(Regex, &'static str)>> = OnceLock::new();
    P.get_or_init(|| {
        vec![
            (Regex::new(r"(?i)\bBearer\s+[A-Za-z0-9._~+/=-]+").unwrap(), "Bearer [REDACTED]"),
            (
                Regex::new(r#"(?i)\b(api_key|apikey|access_token|refresh_token|id_token|token|secret|password|authorization|credential)(\s*[=:]\s*)("[^"]*"|Bearer \[REDACTED\]|\S+)"#).unwrap(),
                "$1$2[REDACTED]",
            ),
            (Regex::new(r"\bsk-[A-Za-z0-9_-]{10,}").unwrap(), "[REDACTED]"),
            (Regex::new(r"\beyJ[A-Za-z0-9_-]{8,}\.[A-Za-z0-9._-]+").unwrap(), "[REDACTED]"),
            (Regex::new(r"\boaiapp_[A-Za-z0-9_-]+").unwrap(), "oaiapp_[REDACTED]"),
            (Regex::new(r"(?i)(id_token_hint=)[^&\s]+").unwrap(), "$1[REDACTED]"),
        ]
    })
}

/// Replaces anything that looks like a credential with `[REDACTED]`.
pub fn redact(input: &str) -> String {
    let mut out = input.to_string();
    for (re, replacement) in patterns() {
        out = re.replace_all(&out, *replacement).into_owned();
    }
    out
}

/// Returns how many secret-looking matches exist in `input` (used by the
/// diagnostics export as a final safety net).
pub fn count_secret_matches(input: &str) -> usize {
    patterns()
        .iter()
        .map(|(re, _)| re.find_iter(input).count())
        .sum()
}

/// Appends to `dir/aura.log`, rotating to `aura.log.1..N` when the file would
/// exceed `max_bytes`. Keeps at most `max_files` files in total.
pub struct SizeRotatingWriter {
    dir: PathBuf,
    max_bytes: u64,
    max_files: usize,
    file: File,
    size: u64,
}

impl SizeRotatingWriter {
    pub fn open(dir: &Path, max_bytes: u64, max_files: usize) -> io::Result<Self> {
        fs::create_dir_all(dir)?;
        let path = dir.join(LOG_FILE_NAME);
        let file = OpenOptions::new().create(true).append(true).open(&path)?;
        let size = file.metadata()?.len();
        Ok(Self {
            dir: dir.to_path_buf(),
            max_bytes,
            max_files: max_files.max(1),
            file,
            size,
        })
    }

    fn rotate(&mut self) -> io::Result<()> {
        self.file.flush()?;
        let base = self.dir.join(LOG_FILE_NAME);
        let oldest = self
            .dir
            .join(format!("{LOG_FILE_NAME}.{}", self.max_files - 1));
        let _ = fs::remove_file(&oldest);
        for i in (1..self.max_files - 1).rev() {
            let from = self.dir.join(format!("{LOG_FILE_NAME}.{i}"));
            if from.exists() {
                fs::rename(&from, self.dir.join(format!("{LOG_FILE_NAME}.{}", i + 1)))?;
            }
        }
        if self.max_files > 1 {
            fs::rename(&base, self.dir.join(format!("{LOG_FILE_NAME}.1")))?;
        } else {
            fs::remove_file(&base)?;
        }
        self.file = OpenOptions::new().create(true).append(true).open(&base)?;
        self.size = 0;
        Ok(())
    }
}

impl Write for SizeRotatingWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let text = String::from_utf8_lossy(buf);
        let clean = redact(&text);
        let bytes = clean.as_bytes();
        if self.size > 0 && self.size + bytes.len() as u64 > self.max_bytes {
            self.rotate()?;
        }
        self.file.write_all(bytes)?;
        self.size += bytes.len() as u64;
        // Report the original length so callers do not retry.
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        self.file.flush()
    }
}

/// Thread-safe `MakeWriter` over a shared rotating writer.
#[derive(Clone)]
pub struct SharedWriter(Arc<Mutex<SizeRotatingWriter>>);

pub struct SharedGuard<'a>(std::sync::MutexGuard<'a, SizeRotatingWriter>);

impl Write for SharedGuard<'_> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.0.write(buf)
    }
    fn flush(&mut self) -> io::Result<()> {
        self.0.flush()
    }
}

impl<'a> MakeWriter<'a> for SharedWriter {
    type Writer = SharedGuard<'a>;
    fn make_writer(&'a self) -> Self::Writer {
        SharedGuard(self.0.lock().unwrap_or_else(|e| e.into_inner()))
    }
}

/// Installs the global subscriber writing redacted logs to `dir`.
/// `RUST_LOG`/`AURA_LOG` control the filter (default `info`).
pub fn init_logging(dir: &Path) -> io::Result<()> {
    let writer = SharedWriter(Arc::new(Mutex::new(SizeRotatingWriter::open(
        dir,
        MAX_LOG_BYTES,
        MAX_LOG_FILES,
    )?)));
    let filter = EnvFilter::try_from_env("AURA_LOG")
        .or_else(|_| EnvFilter::try_from_default_env())
        .unwrap_or_else(|_| EnvFilter::new("info"));
    let layer = tracing_subscriber::fmt::layer()
        .with_ansi(false)
        .with_target(true)
        .with_writer(writer);
    let _ = tracing_subscriber::registry()
        .with(filter)
        .with(layer)
        .try_init();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn credentials_are_redacted() {
        let line =
            r#"falha api_key=sk-live-XYZ123456789 Authorization: Bearer abc.def.ghi token="x y""#;
        let out = redact(line);
        assert!(!out.contains("sk-live"));
        assert!(!out.contains("abc.def.ghi"));
        assert!(out.contains("api_key=[REDACTED]"));
        assert!(out.contains("token=[REDACTED]"));
    }

    #[test]
    fn jwt_and_issued_client_ids_are_redacted() {
        let out = redact("id eyJhbGciOiJSUzI1NiJ9.eyJzdWIiOiIxIn0.sig client oaiapp_123abc");
        assert_eq!(out, "id [REDACTED] client oaiapp_[REDACTED]");
        assert_eq!(count_secret_matches(&out), 0);
    }

    #[test]
    fn rotation_keeps_at_most_five_files_of_bounded_size() {
        let dir = tempfile::tempdir().unwrap();
        let mut w = SizeRotatingWriter::open(dir.path(), 1024, 5).unwrap();
        let line = "x".repeat(99) + "\n";
        for _ in 0..600 {
            w.write_all(line.as_bytes()).unwrap();
        }
        w.flush().unwrap();
        let files: Vec<_> = fs::read_dir(dir.path()).unwrap().collect();
        assert_eq!(files.len(), 5);
        for f in files {
            assert!(f.unwrap().metadata().unwrap().len() <= 1024);
        }
    }

    #[test]
    fn writer_redacts_before_touching_disk() {
        let dir = tempfile::tempdir().unwrap();
        let mut w = SizeRotatingWriter::open(dir.path(), 1024 * 1024, 5).unwrap();
        w.write_all(b"api_key=sk-live-123456789012\n").unwrap();
        w.flush().unwrap();
        let content = fs::read_to_string(dir.path().join(LOG_FILE_NAME)).unwrap();
        assert_eq!(content, "api_key=[REDACTED]\n");
    }
}
