//! End-to-end ingestion of light formats built in-test (no binary fixtures).

use aura_ingest::model::{Content, DocKind, Locator};
use aura_ingest::{Format, IngestError, NoHeavy, Selector, detect, ingest_file, read_selection};
use std::io::Write;
use std::path::Path;

fn zip_with(entries: &[(&str, &str)]) -> Vec<u8> {
    let mut buf = std::io::Cursor::new(Vec::new());
    {
        let mut z = zip::ZipWriter::new(&mut buf);
        let opts = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated);
        for (name, body) in entries {
            z.start_file(*name, opts).unwrap();
            z.write_all(body.as_bytes()).unwrap();
        }
        z.finish().unwrap();
    }
    buf.into_inner()
}

fn all_text(doc: &aura_ingest::model::IngestedDoc) -> String {
    doc.blocks
        .iter()
        .filter_map(|b| match &b.content {
            Content::Text { text } | Content::Table { markdown: text } => Some(text.clone()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("\n")
}

const DOCX: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body>
<w:p><w:pPr><w:pStyle w:val="Heading1"/></w:pPr><w:r><w:t>Relatório</w:t></w:r></w:p>
<w:p><w:r><w:t xml:space="preserve">Receita cresceu </w:t></w:r><w:r><w:t>12% &amp; margem</w:t></w:r></w:p>
<w:p><w:pPr><w:pStyle w:val="Heading2"/></w:pPr><w:r><w:t>Riscos</w:t></w:r></w:p>
<w:p><w:r><w:t>Câmbio</w:t></w:r></w:p>
<w:tbl><w:tr><w:tc><w:p><w:r><w:t>A</w:t></w:r></w:p></w:tc><w:tc><w:p><w:r><w:t>B</w:t></w:r></w:p></w:tc></w:tr>
<w:tr><w:tc><w:p><w:r><w:t>1</w:t></w:r></w:p></w:tc><w:tc><w:p><w:r><w:t>2</w:t></w:r></w:p></w:tc></w:tr></w:tbl>
</w:body></w:document>"#;

#[test]
fn docx_becomes_markdown_with_headings_and_tables() {
    let dir = tempfile::tempdir().unwrap();
    let p = dir.path().join("r.docx");
    std::fs::write(&p, zip_with(&[("word/document.xml", DOCX)])).unwrap();
    let (fmt, doc) = ingest_file(&p, &NoHeavy).unwrap();
    assert_eq!(fmt, Format::Docx);
    assert_eq!(doc.kind, DocKind::Document);
    let t = all_text(&doc);
    assert!(t.contains("# Relatório"), "{t}");
    assert!(t.contains("Receita cresceu 12% & margem"), "{t}");
    assert!(t.contains("## Riscos"), "{t}");
    assert!(t.contains("| A | B |"), "{t}");

    let sel = Selector {
        section: Some("Riscos".into()),
        ..Default::default()
    };
    let blocks = read_selection(&p, fmt, &doc, &sel, &NoHeavy).unwrap();
    assert_eq!(blocks.len(), 1);
    assert!(
        matches!(&blocks[0].content, Content::Text { text } if text.contains("Câmbio") && !text.contains("Receita"))
    );
}

fn slide(text: &str) -> String {
    format!(
        r#"<p:sld xmlns:p="p" xmlns:a="a"><p:cSld><p:spTree><p:sp><p:txBody><a:p><a:r><a:t>{text}</a:t></a:r></a:p></p:txBody></p:sp></p:spTree></p:cSld></p:sld>"#
    )
}

#[test]
fn pptx_keeps_slide_order_and_selects_slides() {
    let dir = tempfile::tempdir().unwrap();
    let p = dir.path().join("deck.pptx");
    let (s1, s2, s10) = (slide("Abertura"), slide("Mercado"), slide("Fim"));
    // slide10 must sort after slide2 (numeric, not lexical).
    std::fs::write(
        &p,
        zip_with(&[
            ("ppt/slides/slide10.xml", &s10),
            ("ppt/slides/slide1.xml", &s1),
            ("ppt/slides/slide2.xml", &s2),
        ]),
    )
    .unwrap();
    let (fmt, doc) = ingest_file(&p, &NoHeavy).unwrap();
    assert_eq!(fmt, Format::Pptx);
    let order: Vec<u32> = doc
        .blocks
        .iter()
        .filter_map(|b| {
            if let Locator::Slide { n } = b.locator {
                Some(n)
            } else {
                None
            }
        })
        .collect();
    assert_eq!(order, vec![1, 2, 10]);
    let sel = Selector {
        slides: Some("2-3".into()),
        ..Default::default()
    };
    let blocks = read_selection(&p, fmt, &doc, &sel, &NoHeavy).unwrap();
    assert_eq!(blocks.len(), 1);
    assert!(matches!(&blocks[0].content, Content::Text { text } if text.contains("Mercado")));
}

#[test]
fn xlsx_schema_sample_and_row_ranges() {
    let dir = tempfile::tempdir().unwrap();
    let p = dir.path().join("vendas.xlsx");
    let mut wb = rust_xlsxwriter::Workbook::new();
    let ws = wb.add_worksheet().set_name("Vendas").unwrap();
    ws.write_string(0, 0, "Produto").unwrap();
    ws.write_string(0, 1, "Valor").unwrap();
    for i in 1..=300u32 {
        ws.write_string(i, 0, format!("item {i}")).unwrap();
        ws.write_number(i, 1, i as f64).unwrap();
    }
    wb.save(&p).unwrap();

    let (fmt, doc) = ingest_file(&p, &NoHeavy).unwrap();
    assert_eq!(fmt, Format::Xlsx);
    assert_eq!(doc.kind, DocKind::Spreadsheet);
    let t = all_text(&doc);
    assert!(t.contains("Vendas"), "{t}");
    assert!(t.contains("Produto") && t.contains("Valor"), "{t}");
    assert!(
        !t.contains("item 300"),
        "full sheet must not be inlined: {t}"
    );

    let sel = Selector {
        sheet: Some("Vendas".into()),
        rows: Some("299-300".into()),
        ..Default::default()
    };
    let blocks = read_selection(&p, fmt, &doc, &sel, &NoHeavy).unwrap();
    let got = blocks
        .iter()
        .map(|b| match &b.content {
            Content::Text { text } | Content::Table { markdown: text } => text.clone(),
            _ => String::new(),
        })
        .collect::<String>();
    assert!(
        got.contains("item 299") && got.contains("item 300"),
        "{got}"
    );

    let bad = Selector {
        sheet: Some("Nada".into()),
        ..Default::default()
    };
    assert!(matches!(
        read_selection(&p, fmt, &doc, &bad, &NoHeavy),
        Err(IngestError::NotFound(_))
    ));
}

#[test]
fn csv_with_semicolons_and_latin1() {
    let dir = tempfile::tempdir().unwrap();
    let p = dir.path().join("c.csv");
    let (bytes, _, _) = encoding_rs::WINDOWS_1252.encode("Nome;Preço\nCafé;1.234,50\nPão;2,00\n");
    std::fs::write(&p, &bytes).unwrap();
    let (_, doc) = ingest_file(&p, &NoHeavy).unwrap();
    let t = all_text(&doc);
    assert!(t.contains("Preço") && t.contains("Café"), "{t}");
}

#[test]
fn detection_is_by_content_not_extension() {
    assert_eq!(
        detect(Path::new("x.txt"), b"%PDF-1.7\n").unwrap(),
        Format::Pdf
    );
    assert_eq!(
        detect(
            Path::new("x.bin"),
            &[
                0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A, 0, 0, 0, 0x0D
            ]
        )
        .unwrap(),
        Format::Image
    );
    assert_eq!(
        detect(Path::new("x.docx"), &[0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1]),
        Err(IngestError::Encrypted)
    );
    assert_eq!(
        detect(Path::new("main.rs"), b"fn main() {}\n").unwrap(),
        Format::Text
    );
    assert_eq!(
        detect(Path::new("x.exe"), b"MZ\x90\x00\x03\x00"),
        Err(IngestError::Unsupported)
    );
}

#[test]
fn heavy_formats_report_the_missing_worker_and_text_ingests_code() {
    let dir = tempfile::tempdir().unwrap();
    let pdf = dir.path().join("a.pdf");
    std::fs::write(&pdf, b"%PDF-1.4\n%%EOF").unwrap();
    // Broken PDF: reported as corrupt (the text layer is parsed in-process).
    assert!(matches!(
        ingest_file(&pdf, &NoHeavy),
        Err(IngestError::Corrupt(_))
    ));
    // Audio still needs the worker (decoding + ASR).
    let mp3 = dir.path().join("a.mp3");
    std::fs::write(&mp3, b"ID3\x03\x00\x00\x00\x00\x00\x00").unwrap();
    assert!(matches!(
        ingest_file(&mp3, &NoHeavy),
        Err(IngestError::Worker(_))
    ));

    let rs = dir.path().join("lib.rs");
    std::fs::write(&rs, "pub fn f() -> u8 { 1 }\n").unwrap();
    let (_, doc) = ingest_file(&rs, &NoHeavy).unwrap();
    assert_eq!(doc.kind, DocKind::Code);
    assert!(all_text(&doc).contains("```rust"));
}

#[test]
fn cache_round_trip_and_eviction() {
    let dir = tempfile::tempdir().unwrap();
    let rs = dir.path().join("a.txt");
    std::fs::write(&rs, "olá mundo").unwrap();
    let (_, doc) = ingest_file(&rs, &NoHeavy).unwrap();
    let cache = aura_ingest::cache::IngestCache::new(dir.path().join("cache"), 1);
    cache.put("h1", &doc).unwrap();
    // max_bytes = 1 evicts everything, including what was just written.
    assert!(cache.get("h1").is_none());
    let big = aura_ingest::cache::IngestCache::new(dir.path().join("cache2"), 1 << 20);
    big.put("h1", &doc).unwrap();
    assert_eq!(big.get("h1"), Some(doc));
}

#[test]
fn store_original_deduplicates_by_hash() {
    let dir = tempfile::tempdir().unwrap();
    let src = dir.path().join("Nota.TXT");
    std::fs::write(&src, "x").unwrap();
    let ws = dir.path().join("ws");
    let (h1, p1) = aura_ingest::store_original(&src, &ws).unwrap();
    let (h2, p2) = aura_ingest::store_original(&src, &ws).unwrap();
    assert_eq!((h1, &p1), (h2, &p2));
    assert!(p1.to_string_lossy().ends_with(".txt"));
    assert_eq!(aura_ingest::parse_range("3-5"), Some((3, 5)));
    assert_eq!(aura_ingest::parse_range("0"), None);
    assert_eq!(aura_ingest::parse_range("5-3"), None);
}
