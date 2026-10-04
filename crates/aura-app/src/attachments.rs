//! Attachments of a conversation (007): the original is copied into the
//! conversation workspace, ingested (cached by content hash) and exposed as a
//! Context Chip; `attachment_read` reads selected parts later.

use aura_core::context::{
    AttachmentLabel, ChipKind, ChipPayload, ContextChip, SummaryPart, SummaryUnit, TOKENS_PER_IMAGE,
};
use aura_ingest::budget::{to_payload, to_turn_inputs};
use aura_ingest::cache::IngestCache;
use aura_ingest::model::{Block, Content, DocKind, IngestedDoc};
use aura_ingest::{
    Format, HeavyIngestor, IngestError, Selector, ingest_file, read_selection, store_original,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

/// Per-attachment share of the context before the chip is reduced.
pub const DEFAULT_ATTACHMENT_TOKENS: u32 = 60_000;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AttachmentInfo {
    pub id: String,
    pub conversation: String,
    pub file_name: String,
    pub stored: PathBuf,
    pub hash: String,
    pub summary: String,
    pub kind: String,
    pub tokens: u32,
    pub warnings: Vec<String>,
}

#[derive(Clone)]
struct Entry {
    info: AttachmentInfo,
    format: FormatKey,
    doc: IngestedDoc,
}

/// `Format` is not serializable; this mirrors it for the on-disk index.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
struct FormatKey(u8);

const FORMATS: [Format; 15] = [
    Format::Pdf,
    Format::Xlsx,
    Format::Xls,
    Format::Ods,
    Format::Csv,
    Format::Docx,
    Format::Pptx,
    Format::Odt,
    Format::Odp,
    Format::Rtf,
    Format::Html,
    Format::Text,
    Format::Image,
    Format::Audio,
    Format::Video,
];

impl FormatKey {
    fn of(f: Format) -> Self {
        FormatKey(FORMATS.iter().position(|x| *x == f).unwrap_or(11) as u8)
    }
    fn format(self) -> Format {
        FORMATS
            .get(self.0 as usize)
            .copied()
            .unwrap_or(Format::Text)
    }
}

fn kind_label(k: &DocKind) -> &'static str {
    match k {
        DocKind::Pdf => "pdf",
        DocKind::Spreadsheet => "spreadsheet",
        DocKind::Document => "document",
        DocKind::Presentation => "presentation",
        DocKind::Text => "text",
        DocKind::Code => "code",
        DocKind::Audio => "audio",
        DocKind::Video => "video",
        DocKind::Image => "image",
    }
}

pub struct AttachmentService {
    heavy: Arc<dyn HeavyIngestor>,
    cache: IngestCache,
    entries: Mutex<HashMap<String, Entry>>,
}

impl AttachmentService {
    pub fn new(heavy: Arc<dyn HeavyIngestor>, cache_dir: PathBuf) -> Self {
        Self {
            heavy,
            cache: IngestCache::new(cache_dir, 512 * 1024 * 1024),
            entries: Mutex::new(HashMap::new()),
        }
    }

    /// Copies, ingests and registers a file. Blocking: call from `spawn_blocking`.
    pub fn add(
        &self,
        conversation: &str,
        workspace: &Path,
        path: &Path,
    ) -> Result<(AttachmentInfo, ContextChip), IngestError> {
        let (hash, stored) = store_original(path, workspace)?;
        let (format, doc) = match self.cache.get(&hash) {
            Some(doc) => {
                let bytes = std::fs::read(&stored)?;
                (
                    aura_ingest::detect(&stored, &bytes[..bytes.len().min(8192)])?,
                    doc,
                )
            }
            None => {
                let (f, mut doc) = ingest_file(&stored, self.heavy.as_ref())?;
                doc.file_name = path
                    .file_name()
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or(doc.file_name);
                let _ = self.cache.put(&hash, &doc);
                (f, doc)
            }
        };
        let id = format!("att_{}", &hash[..12]);
        let tokens = doc.token_estimate(TOKENS_PER_IMAGE);
        let info = AttachmentInfo {
            id: id.clone(),
            conversation: conversation.to_string(),
            file_name: doc.file_name.clone(),
            stored: stored.clone(),
            hash,
            summary: doc.summary.clone(),
            kind: kind_label(&doc.kind).into(),
            tokens,
            warnings: doc.warnings.clone(),
        };
        let chip = self.chip(&info, &doc, DEFAULT_ATTACHMENT_TOKENS);
        let entry = Entry {
            info: info.clone(),
            format: FormatKey::of(format),
            doc,
        };
        self.entries.lock().unwrap().insert(id, entry.clone());
        self.persist_index(workspace)?;
        Ok((info, chip))
    }

    fn chip(&self, info: &AttachmentInfo, doc: &IngestedDoc, max_tokens: u32) -> ContextChip {
        let kind = if doc.kind == DocKind::Image {
            ChipKind::Image
        } else {
            ChipKind::File
        };
        let label = format!("{} · {}", info.file_name, info.summary);
        let payload = if doc.token_estimate(TOKENS_PER_IMAGE) <= max_tokens {
            to_payload(doc)
        } else {
            // Over budget: send what fits plus a pointer to attachment_read.
            let parts = to_turn_inputs(doc, max_tokens, TOKENS_PER_IMAGE)
                .into_iter()
                .map(|t| match t {
                    aura_core::context::TurnInput::Text { text } => ChipPayload::Text { text },
                    aura_core::context::TurnInput::LocalImage { path } => {
                        ChipPayload::Image { path }
                    }
                    aura_core::context::TurnInput::Skill { name, path } => {
                        ChipPayload::Skill { name, path }
                    }
                })
                .chain(std::iter::once(ChipPayload::Text {
                    text: format!("(id do anexo para attachment_read: {})", info.id),
                }))
                .collect();
            ChipPayload::Mixed { parts }
        };
        let mut chip = ContextChip::new(kind, label, payload);
        chip.attachment_label = attachment_label(&info.file_name, &info.summary);
        if doc.kind == DocKind::Image {
            chip.preview_path = Some(info.stored.clone());
        }
        chip
    }

    fn index_path(workspace: &Path) -> PathBuf {
        workspace.join("attachments").join("index.json")
    }

    fn persist_index(&self, workspace: &Path) -> std::io::Result<()> {
        let entries = self.entries.lock().unwrap();
        let list: Vec<(&AttachmentInfo, FormatKey)> = entries
            .values()
            .filter(|e| e.info.stored.starts_with(workspace))
            .map(|e| (&e.info, e.format))
            .collect();
        std::fs::write(
            Self::index_path(workspace),
            serde_json::to_vec_pretty(&list).expect("json"),
        )
    }

    /// Reloads attachments of a reopened conversation.
    pub fn load_workspace(&self, workspace: &Path) {
        let Ok(bytes) = std::fs::read(Self::index_path(workspace)) else {
            return;
        };
        let Ok(list) = serde_json::from_slice::<Vec<(AttachmentInfo, FormatKey)>>(&bytes) else {
            return;
        };
        let mut entries = self.entries.lock().unwrap();
        for (info, format) in list {
            if let Some(doc) = self.cache.get(&info.hash) {
                entries.insert(info.id.clone(), Entry { info, format, doc });
            }
        }
    }

    pub fn list(&self, conversation: &str) -> Vec<AttachmentInfo> {
        let mut v: Vec<_> = self
            .entries
            .lock()
            .unwrap()
            .values()
            .filter(|e| e.info.conversation == conversation)
            .map(|e| e.info.clone())
            .collect();
        v.sort_by(|a, b| a.file_name.cmp(&b.file_name));
        v
    }

    /// Selective read for the `attachment_read` tool (blocking).
    pub fn read(
        &self,
        conversation: &str,
        id: &str,
        selector: &Selector,
    ) -> Result<Vec<Block>, IngestError> {
        let entry = self
            .entries
            .lock()
            .unwrap()
            .get(id)
            .filter(|e| e.info.conversation == conversation)
            .cloned()
            .ok_or_else(|| IngestError::NotFound(format!("anexo {id}")))?;
        read_selection(
            &entry.info.stored,
            entry.format.format(),
            &entry.doc,
            selector,
            self.heavy.as_ref(),
        )
    }

    pub fn forget_conversation(&self, conversation: &str) {
        self.entries
            .lock()
            .unwrap()
            .retain(|_, e| e.info.conversation != conversation);
    }
}

/// Parse only the sealed summaries generated by the ingestors, never filenames
/// or uploaded content. Old/unrecognized summaries retain their legacy label.
fn attachment_label(file_name: &str, summary: &str) -> Option<AttachmentLabel> {
    let parts = summary
        .split(" · ")
        .map(|part| {
            if part == "imagem" {
                return Some(SummaryPart::Image);
            }
            let (amount, noun) = part.rsplit_once(' ')?;
            let unit = match noun {
                "linha" | "linhas" => SummaryUnit::Line,
                "palavra" | "palavras" => SummaryUnit::Word,
                "página" | "páginas" => SummaryUnit::Page,
                "aba" | "abas" => SummaryUnit::Sheet,
                "slide" | "slides" => SummaryUnit::Slide,
                _ => return None,
            };
            let amount = amount.replace('\u{202f}', "").parse().ok()?;
            Some(SummaryPart::Count { amount, unit })
        })
        .collect::<Option<Vec<_>>>()?;
    Some(AttachmentLabel {
        file_name: file_name.into(),
        parts,
    })
}

/// Renders blocks as tool text (images become paths the agent can view).
pub fn blocks_to_text(blocks: &[Block]) -> String {
    blocks
        .iter()
        .map(|b| match &b.content {
            Content::Text { text } | Content::Table { markdown: text } => text.clone(),
            Content::Image { path } => format!("[imagem: {}]", path.display()),
        })
        .collect::<Vec<_>>()
        .join("\n\n")
}
