//! Ingestion cache keyed by content hash + extractor version, LRU-trimmed
//! (AC-010 of 007).

use crate::EXTRACTOR_VERSION;
use crate::model::IngestedDoc;
use std::path::{Path, PathBuf};

pub struct IngestCache {
    dir: PathBuf,
    max_bytes: u64,
}

impl IngestCache {
    pub fn new(dir: PathBuf, max_bytes: u64) -> Self {
        Self { dir, max_bytes }
    }

    fn path(&self, hash: &str) -> PathBuf {
        self.dir.join(format!("{hash}-v{EXTRACTOR_VERSION}.json"))
    }

    pub fn get(&self, hash: &str) -> Option<IngestedDoc> {
        let p = self.path(hash);
        let bytes = std::fs::read(&p).ok()?;
        // Touch for LRU.
        let _ = std::fs::File::options()
            .append(true)
            .open(&p)
            .and_then(|f| f.set_modified(std::time::SystemTime::now()));
        serde_json::from_slice(&bytes).ok()
    }

    pub fn put(&self, hash: &str, doc: &IngestedDoc) -> std::io::Result<()> {
        std::fs::create_dir_all(&self.dir)?;
        std::fs::write(self.path(hash), serde_json::to_vec(doc).expect("json"))?;
        self.evict()
    }

    /// Removes least recently used entries until under `max_bytes`.
    pub fn evict(&self) -> std::io::Result<()> {
        let mut entries: Vec<(std::time::SystemTime, u64, PathBuf)> = std::fs::read_dir(&self.dir)?
            .flatten()
            .filter_map(|e| {
                let m = e.metadata().ok()?;
                Some((m.modified().ok()?, m.len(), e.path()))
            })
            .collect();
        let mut total: u64 = entries.iter().map(|e| e.1).sum();
        entries.sort_by_key(|e| e.0);
        for (_, len, path) in entries {
            if total <= self.max_bytes {
                break;
            }
            let _ = std::fs::remove_file(path);
            total -= len;
        }
        Ok(())
    }

    pub fn dir(&self) -> &Path {
        &self.dir
    }
}
