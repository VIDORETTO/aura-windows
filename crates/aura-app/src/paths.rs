//! On-disk layout under `%LOCALAPPDATA%\Aura` (architecture overview, "Dados locais").

use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppPaths {
    pub root: PathBuf,
}

impl AppPaths {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    /// `%LOCALAPPDATA%\Aura` on Windows, `$XDG_DATA_HOME/aura` or `~/.local/share/aura` elsewhere.
    pub fn default_root() -> PathBuf {
        if let Some(p) = std::env::var_os("AURA_HOME") {
            return PathBuf::from(p);
        }
        if let Some(p) = std::env::var_os("LOCALAPPDATA") {
            return PathBuf::from(p).join("Aura");
        }
        if let Some(p) = std::env::var_os("XDG_DATA_HOME") {
            return PathBuf::from(p).join("aura");
        }
        let home = std::env::var_os("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("."));
        home.join(".local").join("share").join("aura")
    }

    pub fn db(&self) -> PathBuf {
        self.root.join("aura.db")
    }
    pub fn codex_home(&self) -> PathBuf {
        self.root.join("codex-home")
    }
    pub fn bin(&self) -> PathBuf {
        self.root.join("bin")
    }
    pub fn workspaces(&self) -> PathBuf {
        self.root.join("workspaces")
    }
    pub fn screen_segments(&self) -> PathBuf {
        self.root.join("captures").join("screen")
    }
    pub fn audio_segments(&self) -> PathBuf {
        self.root.join("captures").join("audio")
    }
    pub fn asr_models(&self) -> PathBuf {
        self.root.join("models").join("asr")
    }
    pub fn logs(&self) -> PathBuf {
        self.root.join("logs")
    }
    pub fn skills(&self) -> PathBuf {
        self.root.join("skills")
    }
    /// Skills the agent always has (017); rewritten on every start.
    pub fn core_skills(&self) -> PathBuf {
        self.root.join("core-skills")
    }
    pub fn ingest_cache(&self) -> PathBuf {
        self.root.join("cache").join("ingest")
    }
    pub fn captures_tmp(&self) -> PathBuf {
        self.root.join("cache").join("captures")
    }

    pub fn ensure(&self) -> std::io::Result<()> {
        for d in [
            self.root.clone(),
            self.codex_home(),
            self.bin(),
            self.workspaces(),
            self.logs(),
            self.skills(),
            self.captures_tmp(),
        ] {
            std::fs::create_dir_all(d)?;
        }
        Ok(())
    }

    pub fn is_inside(&self, p: &Path) -> bool {
        p.starts_with(&self.root)
    }
}
