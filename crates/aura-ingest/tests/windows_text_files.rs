//! Text files as Windows tools actually write them (042 round 2).
use aura_ingest::{Format, IngestError, NoHeavy, ingest_file, model::Content};

fn utf16le_with_bom(text: &str) -> Vec<u8> {
    let mut bytes = vec![0xFF, 0xFE];
    for unit in text.encode_utf16() {
        bytes.extend_from_slice(&unit.to_le_bytes());
    }
    bytes
}

fn text_of(path: &std::path::Path) -> Result<(Format, String), IngestError> {
    let (format, doc) = ingest_file(path, &NoHeavy)?;
    let text = doc
        .blocks
        .iter()
        .map(|block| match &block.content {
            Content::Text { text } => text.clone(),
            Content::Table { markdown } => markdown.clone(),
            Content::Image { .. } => String::new(),
        })
        .collect::<Vec<_>>()
        .join("\n");
    Ok((format, text))
}

/// PowerShell 5.1 `>`/`Out-File` and Notepad's "Unicode" save UTF-16LE + BOM.
#[test]
fn utf16_text_written_by_powershell_is_read() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("saida.txt");
    std::fs::write(&path, utf16le_with_bom("Relatório de ação: 42 itens\r\n")).unwrap();
    let (format, text) = text_of(&path).expect("UTF-16 text is a text file");
    assert_eq!(format, Format::Text);
    assert!(text.contains("Relatório de ação: 42 itens"), "{text:?}");
}

#[test]
fn utf16_csv_exported_by_excel_is_read() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("vendas.csv");
    std::fs::write(&path, utf16le_with_bom("Produto\tTotal\r\nCafé\t10\r\n")).unwrap();
    let (format, text) = text_of(&path).expect("Excel 'Unicode text' export");
    assert_eq!(format, Format::Csv);
    assert!(text.contains("Café") && text.contains("10"), "{text:?}");
}

#[test]
fn an_empty_file_is_reported_as_empty_text_not_unsupported() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("notas.txt");
    std::fs::write(&path, b"").unwrap();
    let result = text_of(&path);
    assert!(
        !matches!(result, Err(IngestError::Unsupported)),
        "an empty .txt is not an unsupported format: {result:?}"
    );
}
