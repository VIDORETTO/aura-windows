//! The ingested representation of an attachment.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum Locator {
    Whole,
    Page {
        n: u32,
    },
    Sheet {
        name: String,
        rows: Option<(u32, u32)>,
    },
    Slide {
        n: u32,
    },
    Time {
        from_ms: i64,
        to_ms: i64,
    },
    Section {
        title: String,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum Content {
    Text {
        text: String,
    },
    /// Markdown table (already rendered).
    Table {
        markdown: String,
    },
    Image {
        path: PathBuf,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Block {
    pub locator: Locator,
    pub content: Content,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum DocKind {
    Pdf,
    Spreadsheet,
    Document,
    Presentation,
    Text,
    Code,
    Audio,
    Video,
    Image,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IngestedDoc {
    pub kind: DocKind,
    pub file_name: String,
    /// Short UI summary, e.g. "12 páginas" or "3 abas · 1 240 linhas".
    pub summary: String,
    pub blocks: Vec<Block>,
    pub warnings: Vec<String>,
}

impl IngestedDoc {
    pub fn token_estimate(&self, per_image: u32) -> u32 {
        self.blocks.iter().map(|b| block_tokens(b, per_image)).sum()
    }
}

pub fn block_tokens(b: &Block, per_image: u32) -> u32 {
    match &b.content {
        Content::Text { text } => aura_core::context::estimate_text_tokens(text),
        Content::Table { markdown } => aura_core::context::estimate_text_tokens(markdown),
        Content::Image { .. } => per_image,
    }
}

/// Thousands separator used in summaries ("1 240").
pub fn fmt_count(n: usize) -> String {
    let s = n.to_string();
    let mut out = String::new();
    for (i, c) in s.chars().enumerate() {
        if i > 0 && (s.len() - i).is_multiple_of(3) {
            out.push('\u{202f}');
        }
        out.push(c);
    }
    out
}
