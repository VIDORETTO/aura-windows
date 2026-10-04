//! Codex memories under `CODEX_HOME/memories` (008 AC-015): the user reviews
//! the remembered facts, edits them or forgets them. `memory_summary.md` is
//! what new conversations receive; `MEMORY.md` is Codex's searchable
//! registry. Each change also leaves a note in `extensions/ad_hoc/notes/`,
//! the input Codex's consolidation applies, so removed facts stay removed.

use serde::Serialize;
use std::path::Path;

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryView {
    /// `memory_summary.md` (given to new conversations).
    pub summary: String,
    /// `MEMORY.md` (registry the agent searches).
    pub registry: String,
    /// Bullet items of the summary, in order.
    pub facts: Vec<String>,
}

const SUMMARY: &str = "memory_summary.md";
const REGISTRY: &str = "MEMORY.md";

fn bullet(line: &str) -> Option<&str> {
    let t = line.trim_start();
    t.strip_prefix("- ")
        .or_else(|| t.strip_prefix("* "))
        .map(str::trim)
        .filter(|s| !s.is_empty())
}

pub fn read(dir: &Path) -> std::io::Result<MemoryView> {
    let get = |name: &str| match std::fs::read_to_string(dir.join(name)) {
        Ok(s) => Ok(s),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(String::new()),
        Err(e) => Err(e),
    };
    let summary = get(SUMMARY)?;
    let registry = get(REGISTRY)?;
    let facts = summary
        .lines()
        .filter_map(bullet)
        .map(str::to_string)
        .collect();
    Ok(MemoryView {
        summary,
        registry,
        facts,
    })
}

fn note(dir: &Path, slug: &str, text: &str) -> std::io::Result<()> {
    let notes = dir.join("extensions").join("ad_hoc").join("notes");
    std::fs::create_dir_all(&notes)?;
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    std::fs::write(notes.join(format!("{stamp}-{slug}.md")), text)
}

/// Removes every bullet equal to `fact` from both files.
pub fn forget_fact(dir: &Path, fact: &str) -> std::io::Result<()> {
    for name in [SUMMARY, REGISTRY] {
        let path = dir.join(name);
        let Ok(text) = std::fs::read_to_string(&path) else {
            continue;
        };
        let kept: String = text
            .split_inclusive('\n')
            .filter(|line| bullet(line.trim_end_matches(['\r', '\n'])) != Some(fact.trim()))
            .collect();
        std::fs::write(path, kept)?;
    }
    note(
        dir,
        "aura-forget",
        &format!(
            "The user removed this memory in Aura Settings. Forget: {}\nDo not add it back.\n",
            fact.trim()
        ),
    )
}

/// Replaces both files with the user's edited text.
pub fn save(dir: &Path, summary: &str, registry: &str) -> std::io::Result<()> {
    std::fs::create_dir_all(dir)?;
    std::fs::write(dir.join(SUMMARY), summary)?;
    std::fs::write(dir.join(REGISTRY), registry)?;
    note(
        dir,
        "aura-edit",
        "The user edited memory_summary.md and MEMORY.md in Aura Settings. Treat them as authoritative; do not restore removed or changed facts.\n",
    )
}

/// Deletes everything Codex remembered (summaries, registry, raw notes).
pub fn forget_all(dir: &Path) -> std::io::Result<()> {
    match std::fs::remove_dir_all(dir) {
        Err(e) if e.kind() != std::io::ErrorKind::NotFound => Err(e),
        _ => Ok(()),
    }
}
