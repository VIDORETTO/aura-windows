//! Plain text, Markdown, data formats and source code (AC-006 of 007).

use crate::model::{Block, Content, DocKind, IngestedDoc, Locator};
use std::path::Path;

pub const MAX_TEXT_CHARS: usize = 1_000_000;

/// Decodes bytes with BOM/UTF-8 detection and a legacy fallback
/// (Windows-1252, ISO-8859-x…) guessed by `chardetng`.
pub fn decode(bytes: &[u8]) -> String {
    if let Some((enc, bom_len)) = encoding_rs::Encoding::for_bom(bytes) {
        return enc
            .decode_without_bom_handling(&bytes[bom_len..])
            .0
            .into_owned();
    }
    if let Ok(s) = std::str::from_utf8(bytes) {
        return s.to_string();
    }
    let mut det = chardetng::EncodingDetector::new(chardetng::Iso2022JpDetection::Deny);
    det.feed(bytes, true);
    det.guess(None, chardetng::Utf8Detection::Allow)
        .decode(bytes)
        .0
        .into_owned()
}

pub fn language_for(ext: &str) -> Option<&'static str> {
    Some(match ext {
        "rs" => "rust",
        "py" => "python",
        "js" | "mjs" | "cjs" => "javascript",
        "ts" => "typescript",
        "tsx" => "tsx",
        "jsx" => "jsx",
        "java" => "java",
        "kt" => "kotlin",
        "cs" => "csharp",
        "cpp" | "cc" | "hpp" | "h" => "cpp",
        "c" => "c",
        "go" => "go",
        "rb" => "ruby",
        "php" => "php",
        "swift" => "swift",
        "sql" => "sql",
        "sh" | "bash" => "bash",
        "ps1" => "powershell",
        "html" | "htm" => "html",
        "css" => "css",
        "scss" => "scss",
        "json" => "json",
        "yaml" | "yml" => "yaml",
        "toml" => "toml",
        "xml" => "xml",
        "ini" | "cfg" => "ini",
        _ => return None,
    })
}

pub fn ingest(path: &Path, bytes: &[u8]) -> IngestedDoc {
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    let ext = path
        .extension()
        .map(|e| e.to_string_lossy().to_lowercase())
        .unwrap_or_default();
    let mut text = decode(bytes);
    let mut warnings = Vec::new();
    if text.chars().count() > MAX_TEXT_CHARS {
        text = text.chars().take(MAX_TEXT_CHARS).collect();
        warnings.push(
            "Arquivo muito grande: apenas o início foi incluído; use leitura por seção.".into(),
        );
    }
    let lines = text.lines().count();
    let (kind, content) = match (ext.as_str(), language_for(&ext)) {
        ("md" | "markdown" | "txt" | "log" | "csv", _) => (DocKind::Text, text.clone()),
        ("html" | "htm", _) => (DocKind::Document, html_to_text(&text)),
        (_, Some(lang)) => (
            DocKind::Code,
            format!("```{lang}\n{}\n```", text.trim_end()),
        ),
        _ => (DocKind::Text, text.clone()),
    };
    IngestedDoc {
        kind,
        file_name: name,
        summary: format!("{} linhas", crate::model::fmt_count(lines)),
        blocks: vec![Block {
            locator: Locator::Whole,
            content: Content::Text { text: content },
        }],
        warnings,
    }
}

/// Very small HTML → text/Markdown conversion (headings, paragraphs, lists,
/// line breaks); scripts and styles are removed.
pub fn html_to_text(html: &str) -> String {
    let mut out = String::new();
    let mut i = 0;
    let bytes = html.as_bytes();
    let lower = html.to_lowercase();
    while i < bytes.len() {
        if bytes[i] == b'<' {
            let end = match html[i..].find('>') {
                Some(e) => i + e,
                None => break,
            };
            let tag = lower[i + 1..end].trim().to_string();
            let name: String = tag
                .trim_start_matches('/')
                .chars()
                .take_while(|c| c.is_ascii_alphanumeric())
                .collect();
            if !tag.starts_with('/') && (name == "script" || name == "style") {
                let close = format!("</{name}");
                i = lower[end..]
                    .find(&close)
                    .map(|p| end + p)
                    .unwrap_or(bytes.len());
                i = html[i..]
                    .find('>')
                    .map(|p| i + p + 1)
                    .unwrap_or(bytes.len());
                continue;
            }
            match (tag.starts_with('/'), name.as_str()) {
                (false, "h1") => out.push_str("\n# "),
                (false, "h2") => out.push_str("\n## "),
                (false, "h3") => out.push_str("\n### "),
                (false, "li") => out.push_str("\n- "),
                (_, "p" | "div" | "br" | "tr" | "h1" | "h2" | "h3" | "ul" | "ol") => out.push('\n'),
                (false, "td" | "th") => out.push_str(" | "),
                _ => {}
            }
            i = end + 1;
        } else {
            let next = html[i..].find('<').map(|p| i + p).unwrap_or(bytes.len());
            out.push_str(&unescape(&html[i..next]));
            i = next;
        }
    }
    let collapsed: Vec<&str> = out.lines().map(str::trim_end).collect();
    let mut result = String::new();
    let mut blank = false;
    for l in collapsed {
        if l.trim().is_empty() {
            if !blank && !result.is_empty() {
                result.push('\n');
            }
            blank = true;
        } else {
            result.push_str(l.trim_start_matches(' '));
            result.push('\n');
            blank = false;
        }
    }
    result.trim().to_string()
}

fn unescape(s: &str) -> String {
    s.replace("&nbsp;", " ")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&amp;", "&")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn latin1_and_bom_are_decoded() {
        let latin1 = [b'a', 0xE7, 0xE3, b'o']; // "ação" in Windows-1252
        assert_eq!(decode(&latin1), "ação");
        let bom = [0xEF, 0xBB, 0xBF, b'o', b'k'];
        assert_eq!(decode(&bom), "ok");
    }

    #[test]
    fn code_gets_a_fenced_block_with_language() {
        let d = ingest(Path::new("main.rs"), b"fn main() {}\n");
        let Content::Text { text } = &d.blocks[0].content else {
            panic!()
        };
        assert_eq!(text, "```rust\nfn main() {}\n```");
        assert_eq!(d.kind, DocKind::Code);
    }

    #[test]
    fn html_structure_is_kept() {
        let t = html_to_text(
            "<html><style>x{}</style><h1>Título</h1><p>Olá &amp; bem-vindo</p><ul><li>um</li><li>dois</li></ul><script>alert(1)</script></html>",
        );
        assert_eq!(t, "# Título\n\nOlá & bem-vindo\n\n- um\n- dois");
    }
}
