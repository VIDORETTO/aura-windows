//! Attachment ingestion (`specs/007-anexos-multimodais`).
//!
//! Light formats (text, code, HTML, spreadsheets, DOCX/PPTX/ODF, RTF) are
//! handled here; PDF, audio and video go through `aura-worker` (Windows build)
//! behind [`HeavyIngestor`].

pub mod budget;
pub mod cache;
pub mod model;
pub mod office;
pub mod pdf;
pub mod sheets;
pub mod text;

use model::{Block, Content, DocKind, IngestedDoc, Locator};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use thiserror::Error;

pub const MAX_BYTES: u64 = 200 * 1024 * 1024;
pub const MAX_PAGES: u32 = 500;
pub const MAX_MEDIA_SECONDS: u64 = 2 * 3600;
/// Bump when an extractor changes output; invalidates the cache.
pub const EXTRACTOR_VERSION: u32 = 1;

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum IngestError {
    #[error("unsupported format")]
    Unsupported,
    #[error("the file is password protected")]
    Encrypted,
    #[error("the file is corrupted: {0}")]
    Corrupt(String),
    #[error("the file is larger than {limit} bytes")]
    TooLarge { limit: u64 },
    #[error("more than {limit} pages")]
    TooManyPages { limit: u32 },
    #[error("a speech model is needed to transcribe this file")]
    NeedsAsr,
    #[error("processing timed out")]
    Timeout,
    #[error("not found: {0}")]
    NotFound(String),
    #[error("io: {0}")]
    Io(String),
    #[error("{0}")]
    Worker(String),
}

impl From<std::io::Error> for IngestError {
    fn from(e: std::io::Error) -> Self {
        IngestError::Io(e.to_string())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    Pdf,
    Xlsx,
    Xls,
    Ods,
    Csv,
    Docx,
    Pptx,
    Odt,
    Odp,
    Rtf,
    Html,
    Text,
    Image,
    Audio,
    Video,
}

/// Detects by content first (OT-001 of 007), extension only for text-like
/// files and to disambiguate zip containers.
pub fn detect(path: &Path, head: &[u8]) -> Result<Format, IngestError> {
    let ext = path
        .extension()
        .map(|e| e.to_string_lossy().to_lowercase())
        .unwrap_or_default();
    if head.starts_with(b"%PDF") {
        return Ok(Format::Pdf);
    }
    if head.starts_with(b"{\\rtf") {
        return Ok(Format::Rtf);
    }
    if head.starts_with(&[0xD0, 0xCF, 0x11, 0xE0]) {
        // OLE2: legacy .xls/.doc/.ppt (or password-protected OOXML).
        return match ext.as_str() {
            "xls" => Ok(Format::Xls),
            "xlsx" | "docx" | "pptx" => Err(IngestError::Encrypted),
            _ => Err(IngestError::Unsupported),
        };
    }
    if head.starts_with(b"PK\x03\x04") {
        return match ext.as_str() {
            "xlsx" | "xlsm" => Ok(Format::Xlsx),
            "docx" => Ok(Format::Docx),
            "pptx" => Ok(Format::Pptx),
            "ods" => Ok(Format::Ods),
            "odt" => Ok(Format::Odt),
            "odp" => Ok(Format::Odp),
            _ => Err(IngestError::Unsupported),
        };
    }
    if let Some(kind) = infer::get(head) {
        match kind.matcher_type() {
            infer::MatcherType::Image => return Ok(Format::Image),
            infer::MatcherType::Audio => return Ok(Format::Audio),
            infer::MatcherType::Video => return Ok(Format::Video),
            _ => {}
        }
    }
    match ext.as_str() {
        "csv" | "tsv" => Ok(Format::Csv),
        "html" | "htm" => Ok(Format::Html),
        "mp3" | "m4a" | "wav" | "ogg" | "opus" | "flac" => Ok(Format::Audio),
        "mp4" | "mov" | "mkv" | "webm" | "avi" => Ok(Format::Video),
        _ if looks_textual(head) => Ok(Format::Text),
        _ => Err(IngestError::Unsupported),
    }
}

fn looks_textual(head: &[u8]) -> bool {
    !head.is_empty() && !head.contains(&0) && head.iter().filter(|b| **b < 0x09).count() == 0
}

/// Heavy formats handled out of process (PDF, audio, video).
pub trait HeavyIngestor: Send + Sync {
    fn ingest(&self, format: Format, path: &Path) -> Result<IngestedDoc, IngestError>;
    fn read(
        &self,
        format: Format,
        path: &Path,
        selector: &Selector,
    ) -> Result<Vec<Block>, IngestError>;
}

/// No worker available (non-Windows build, tests).
pub struct NoHeavy;

impl HeavyIngestor for NoHeavy {
    fn ingest(&self, _f: Format, _p: &Path) -> Result<IngestedDoc, IngestError> {
        Err(IngestError::Worker(
            "este formato requer o processador do Aura para Windows".into(),
        ))
    }
    fn read(&self, _f: Format, _p: &Path, _s: &Selector) -> Result<Vec<Block>, IngestError> {
        Err(IngestError::Worker(
            "este formato requer o processador do Aura para Windows".into(),
        ))
    }
}

pub fn sha256_hex(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}

/// Ingests one file (light formats inline, heavy via `heavy`).
pub fn ingest_file(
    path: &Path,
    heavy: &dyn HeavyIngestor,
) -> Result<(Format, IngestedDoc), IngestError> {
    let meta = std::fs::metadata(path)?;
    if meta.len() > MAX_BYTES {
        return Err(IngestError::TooLarge { limit: MAX_BYTES });
    }
    let bytes = std::fs::read(path)?;
    let format = detect(path, &bytes[..bytes.len().min(8192)])?;
    let doc = match format {
        Format::Xlsx | Format::Xls | Format::Ods | Format::Csv => sheets::ingest(path, &bytes)?,
        Format::Docx => office::ingest_docx(path, &bytes)?,
        Format::Pptx => office::ingest_pptx(path, &bytes)?,
        Format::Odt | Format::Odp => office::ingest_odf(path, &bytes)?,
        Format::Rtf => {
            let t = office::rtf_to_text(&text::decode(&bytes));
            IngestedDoc {
                kind: DocKind::Document,
                file_name: path
                    .file_name()
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or_default(),
                summary: format!(
                    "{} palavras",
                    model::fmt_count(t.split_whitespace().count())
                ),
                blocks: vec![Block {
                    locator: Locator::Whole,
                    content: Content::Text { text: t },
                }],
                warnings: vec![],
            }
        }
        Format::Html | Format::Text => text::ingest(path, &bytes),
        Format::Image => IngestedDoc {
            kind: DocKind::Image,
            file_name: path
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_default(),
            summary: "imagem".into(),
            blocks: vec![Block {
                locator: Locator::Whole,
                content: Content::Image {
                    path: path.to_path_buf(),
                },
            }],
            warnings: vec![],
        },
        // Text layer in-process; scanned PDFs go to the worker (OCR) when available.
        Format::Pdf => {
            let doc = pdf::ingest(path, &bytes)?;
            if doc.blocks.is_empty() {
                heavy.ingest(format, path).unwrap_or(doc)
            } else {
                doc
            }
        }
        Format::Audio | Format::Video => heavy.ingest(format, path)?,
    };
    Ok((format, doc))
}

/// `attachment_read` selector (007 TK-006).
#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Selector {
    pub pages: Option<String>,
    pub sheet: Option<String>,
    pub rows: Option<String>,
    pub slides: Option<String>,
    pub time: Option<String>,
    pub section: Option<String>,
}

/// Parses "3-5" / "7" (1-based, inclusive).
pub fn parse_range(s: &str) -> Option<(u32, u32)> {
    let s = s.trim();
    match s.split_once(['-', '–']) {
        Some((a, b)) => {
            let (a, b) = (a.trim().parse().ok()?, b.trim().parse().ok()?);
            (a >= 1 && b >= a).then_some((a, b))
        }
        None => s.parse().ok().filter(|n| *n >= 1).map(|n| (n, n)),
    }
}

/// Parses "01:00-02:30" / "75-90" (seconds) into milliseconds.
pub fn parse_time_range(s: &str) -> Option<(i64, i64)> {
    let part = |p: &str| -> Option<i64> {
        let mut total = 0i64;
        for x in p.trim().split(':') {
            total = total * 60 + x.trim().parse::<i64>().ok()?;
        }
        Some(total * 1000)
    };
    let (a, b) = s.split_once(['-', '–'])?;
    let (a, b) = (part(a)?, part(b)?);
    (b >= a).then_some((a, b))
}

/// Selective read of an already-ingested light document.
pub fn read_selection(
    path: &Path,
    format: Format,
    doc: &IngestedDoc,
    selector: &Selector,
    heavy: &dyn HeavyIngestor,
) -> Result<Vec<Block>, IngestError> {
    match format {
        Format::Xlsx | Format::Xls | Format::Ods | Format::Csv => {
            let bytes = std::fs::read(path)?;
            let sheet = selector.sheet.clone().unwrap_or_default();
            let rows = selector.rows.as_deref().and_then(parse_range);
            sheets::read_sheet(path, &bytes, &sheet, rows)
        }
        Format::Pdf => {
            let bytes = std::fs::read(path)?;
            let range = selector.pages.as_deref().and_then(parse_range);
            let blocks = pdf::to_blocks(&pdf::extract(&bytes, range)?);
            if blocks.is_empty() {
                heavy.read(format, path, selector)
            } else {
                Ok(blocks)
            }
        }
        Format::Audio | Format::Video => {
            // Transcripts are cached in the document: filter by time locally.
            let timed = doc
                .blocks
                .iter()
                .any(|b| matches!(b.locator, Locator::Time { .. }));
            match selector.time.as_deref().and_then(parse_time_range) {
                Some((a, b)) if timed => Ok(doc
                    .blocks
                    .iter()
                    .filter(|bl| matches!(bl.locator, Locator::Time { from_ms, to_ms } if to_ms >= a && from_ms <= b))
                    .cloned()
                    .collect()),
                None if timed => Ok(doc.blocks.clone()),
                _ => heavy.read(format, path, selector),
            }
        }
        _ => {
            if let Some((a, b)) = selector.slides.as_deref().and_then(parse_range) {
                return Ok(doc
                    .blocks
                    .iter()
                    .filter(|bl| matches!(bl.locator, Locator::Slide { n } if n >= a && n <= b))
                    .cloned()
                    .collect());
            }
            if let Some(title) = &selector.section {
                return Ok(section(doc, title));
            }
            Ok(doc.blocks.clone())
        }
    }
}

fn section(doc: &IngestedDoc, title: &str) -> Vec<Block> {
    let mut out = Vec::new();
    for b in &doc.blocks {
        if let Content::Text { text } = &b.content {
            let mut capture = false;
            let mut level = 0;
            let mut buf = String::new();
            for line in text.lines() {
                let hashes = line.chars().take_while(|c| *c == '#').count();
                if hashes > 0 {
                    if capture && hashes <= level {
                        break;
                    }
                    if line[hashes..].trim().eq_ignore_ascii_case(title.trim()) {
                        capture = true;
                        level = hashes;
                    }
                }
                if capture {
                    buf.push_str(line);
                    buf.push('\n');
                }
            }
            if !buf.is_empty() {
                out.push(Block {
                    locator: Locator::Section {
                        title: title.into(),
                    },
                    content: Content::Text {
                        text: buf.trim().into(),
                    },
                });
            }
        }
    }
    out
}

/// Copies the original into the conversation workspace as `<hash>.<ext>`.
pub fn store_original(path: &Path, workspace: &Path) -> Result<(String, PathBuf), IngestError> {
    let bytes = std::fs::read(path)?;
    let hash = sha256_hex(&bytes);
    let ext = path
        .extension()
        .map(|e| e.to_string_lossy().to_lowercase())
        .unwrap_or_else(|| "bin".into());
    let dir = workspace.join("attachments");
    std::fs::create_dir_all(&dir)?;
    let dest = dir.join(format!("{}.{ext}", &hash[..16]));
    if !dest.exists() {
        std::fs::write(&dest, &bytes)?;
    }
    Ok((hash, dest))
}
