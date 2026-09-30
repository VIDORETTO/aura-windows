//! PDF text layer (007 TK-001) in pure Rust: one block per page, so the agent
//! can read "páginas 3–5" later. Scanned pages (no text layer) are reported;
//! OCR/rendering stays in the worker (`HeavyIngestor`).

use crate::model::{Block, Content, DocKind, IngestedDoc, Locator, fmt_count};
use crate::{IngestError, MAX_PAGES};
use std::path::Path;

/// Decompressed content cap per page (defends against zip-bomb streams).
const MAX_PAGE_STREAM: usize = 32 * 1024 * 1024;

fn load(bytes: &[u8]) -> Result<lopdf::Document, IngestError> {
    let doc = lopdf::Document::load_mem(bytes).map_err(|e| {
        let m = e.to_string();
        let l = m.to_lowercase();
        if l.contains("decrypt") || l.contains("password") || l.contains("encrypt") {
            IngestError::Encrypted
        } else {
            IngestError::Corrupt(m)
        }
    })?;
    if doc.is_encrypted() {
        return Err(IngestError::Encrypted);
    }
    Ok(doc)
}

fn clean(text: &str) -> String {
    let lines: Vec<&str> = text.lines().map(str::trim_end).collect();
    let mut out = String::new();
    let mut blank = 0;
    for l in lines {
        if l.trim().is_empty() {
            blank += 1;
            if blank > 1 {
                continue;
            }
        } else {
            blank = 0;
        }
        out.push_str(l);
        out.push('\n');
    }
    out.trim().to_string()
}

pub struct PdfText {
    pub pages: Vec<(u32, String)>,
    pub total_pages: u32,
}

pub fn extract(bytes: &[u8], range: Option<(u32, u32)>) -> Result<PdfText, IngestError> {
    let doc = load(bytes)?;
    let numbers: Vec<u32> = doc.get_pages().keys().copied().collect();
    let total = numbers.len() as u32;
    if total > MAX_PAGES {
        return Err(IngestError::TooManyPages { limit: MAX_PAGES });
    }
    let wanted = numbers
        .into_iter()
        .filter(|n| range.is_none_or(|(a, b)| *n >= a && *n <= b));
    let pages = wanted
        .map(|n| {
            let text = doc
                .extract_text_with_limit(&[n], MAX_PAGE_STREAM)
                .unwrap_or_default();
            (n, clean(&text))
        })
        .collect();
    Ok(PdfText {
        pages,
        total_pages: total,
    })
}

pub fn to_blocks(t: &PdfText) -> Vec<Block> {
    t.pages
        .iter()
        .filter(|(_, text)| !text.is_empty())
        .map(|(n, text)| Block {
            locator: Locator::Page { n: *n },
            content: Content::Text { text: text.clone() },
        })
        .collect()
}

pub fn ingest(path: &Path, bytes: &[u8]) -> Result<IngestedDoc, IngestError> {
    let t = extract(bytes, None)?;
    let empty: Vec<u32> = t
        .pages
        .iter()
        .filter(|(_, s)| s.is_empty())
        .map(|(n, _)| *n)
        .collect();
    let mut warnings = Vec::new();
    if !empty.is_empty() {
        warnings.push(if empty.len() as u32 == t.total_pages {
            "o PDF não tem camada de texto (escaneado); o texto depende de OCR".to_string()
        } else {
            format!("{} página(s) sem texto (imagens/escaneadas)", empty.len())
        });
    }
    let words: usize = t
        .pages
        .iter()
        .map(|(_, s)| s.split_whitespace().count())
        .sum();
    Ok(IngestedDoc {
        kind: DocKind::Pdf,
        file_name: path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default(),
        summary: format!(
            "{} {} · {} palavras",
            t.total_pages,
            if t.total_pages == 1 {
                "página"
            } else {
                "páginas"
            },
            fmt_count(words)
        ),
        blocks: to_blocks(&t),
        warnings,
    })
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use lopdf::content::{Content as PdfContent, Operation};
    use lopdf::{Document, Object, Stream, dictionary};

    /// Builds a text PDF with one page per string (empty string = no text).
    pub fn make_pdf(pages: &[&str]) -> Vec<u8> {
        let mut doc = Document::with_version("1.5");
        let pages_id = doc.new_object_id();
        let font_id = doc.add_object(
            dictionary! { "Type" => "Font", "Subtype" => "Type1", "BaseFont" => "Helvetica" },
        );
        let resources_id =
            doc.add_object(dictionary! { "Font" => dictionary! { "F1" => font_id } });
        let mut kids = Vec::new();
        for text in pages {
            let mut ops = vec![];
            if !text.is_empty() {
                ops = vec![
                    Operation::new("BT", vec![]),
                    Operation::new("Tf", vec!["F1".into(), 12.into()]),
                    Operation::new("Td", vec![72.into(), 700.into()]),
                    Operation::new("Tj", vec![Object::string_literal(*text)]),
                    Operation::new("ET", vec![]),
                ];
            }
            let content = PdfContent { operations: ops };
            let content_id = doc.add_object(Stream::new(dictionary! {}, content.encode().unwrap()));
            let page_id = doc.add_object(
                dictionary! { "Type" => "Page", "Parent" => pages_id, "Contents" => content_id },
            );
            kids.push(page_id.into());
        }
        let count = kids.len() as i64;
        doc.objects.insert(
            pages_id,
            Object::Dictionary(dictionary! {
                "Type" => "Pages", "Kids" => kids, "Count" => count, "Resources" => resources_id,
                "MediaBox" => vec![0.into(), 0.into(), 595.into(), 842.into()],
            }),
        );
        let catalog_id = doc.add_object(dictionary! { "Type" => "Catalog", "Pages" => pages_id });
        doc.trailer.set("Root", catalog_id);
        let mut out = Vec::new();
        doc.save_to(&mut out).unwrap();
        out
    }

    #[test]
    fn text_per_page_with_scanned_page_warning() {
        let bytes = make_pdf(&["Relatorio anual", "", "Receita cresceu 12 por cento"]);
        let doc = ingest(Path::new("r.pdf"), &bytes).unwrap();
        assert_eq!(doc.kind, DocKind::Pdf);
        assert!(doc.summary.starts_with("3 páginas"), "{}", doc.summary);
        let pages: Vec<u32> = doc
            .blocks
            .iter()
            .map(|b| {
                if let Locator::Page { n } = b.locator {
                    n
                } else {
                    0
                }
            })
            .collect();
        assert_eq!(pages, vec![1, 3]);
        assert!(
            matches!(&doc.blocks[1].content, Content::Text { text } if text.contains("Receita cresceu"))
        );
        assert_eq!(doc.warnings.len(), 1);

        let sel = extract(&bytes, Some((3, 3))).unwrap();
        assert_eq!(sel.pages.len(), 1);
        assert!(sel.pages[0].1.contains("Receita"));
    }

    #[test]
    fn garbage_is_corrupt() {
        assert!(matches!(
            ingest(Path::new("x.pdf"), b"%PDF-1.4\nnot really"),
            Err(IngestError::Corrupt(_))
        ));
    }
}
