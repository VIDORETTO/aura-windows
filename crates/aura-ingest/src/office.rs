//! DOCX, PPTX and ODT/ODP → Markdown by reading their XML (AC-006 of 007).

use crate::IngestError;
use crate::model::{Block, Content, DocKind, IngestedDoc, Locator};
use quick_xml::Reader;
use quick_xml::events::Event;
use std::io::{Cursor, Read};
use std::path::Path;

fn open_zip(bytes: &[u8]) -> Result<zip::ZipArchive<Cursor<Vec<u8>>>, IngestError> {
    zip::ZipArchive::new(Cursor::new(bytes.to_vec())).map_err(|e| {
        let m = e.to_string();
        if m.to_lowercase().contains("password") || m.to_lowercase().contains("encrypt") {
            IngestError::Encrypted
        } else {
            IngestError::Corrupt(m)
        }
    })
}

fn read_entry(zip: &mut zip::ZipArchive<Cursor<Vec<u8>>>, name: &str) -> Option<String> {
    let mut f = zip.by_name(name).ok()?;
    let mut s = String::new();
    f.read_to_string(&mut s).ok()?;
    Some(s)
}

fn local(name: &str) -> &str {
    name.rsplit_once(':').map_or(name, |(_, l)| l)
}

/// Resolves a general entity reference (`amp`, `#233`, `#xE9`).
fn entity(name: &str) -> String {
    let num = |s: &str, radix| {
        u32::from_str_radix(s, radix)
            .ok()
            .and_then(char::from_u32)
            .map(String::from)
    };
    match name {
        "amp" => "&".into(),
        "lt" => "<".into(),
        "gt" => ">".into(),
        "quot" => "\"".into(),
        "apos" => "'".into(),
        n => match n.strip_prefix("#x").or_else(|| n.strip_prefix("#X")) {
            Some(hex) => num(hex, 16),
            None => n.strip_prefix('#').and_then(|d| num(d, 10)),
        }
        .unwrap_or_default(),
    }
}

fn attr(e: &quick_xml::events::BytesStart<'_>, key: &str) -> Option<String> {
    e.attributes()
        .flatten()
        .find(|a| local(a.key.as_ref()) == key)
        .map(|a| a.value.into_owned())
}

/// Word: paragraphs with heading styles, list items and tables.
pub fn docx_to_markdown(xml: &str) -> String {
    let mut r = Reader::from_str(xml);
    let mut out = String::new();
    let mut para = String::new();
    let mut style = String::new();
    let mut in_list = false;
    let mut in_text = false;
    let mut table: Vec<Vec<String>> = Vec::new();
    let mut row: Vec<String> = Vec::new();
    let mut cell = String::new();
    let mut table_depth = 0usize;
    loop {
        match r.read_event() {
            Ok(Event::Start(e)) | Ok(Event::Empty(e)) => match local(e.name().as_ref()) {
                "p" => {
                    para.clear();
                    style.clear();
                    in_list = false;
                }
                "pStyle" => style = attr(&e, "val").unwrap_or_default(),
                "numPr" => in_list = true,
                "t" => in_text = true,
                "tab" => para.push('\t'),
                "br" => para.push('\n'),
                "tbl" => {
                    table_depth += 1;
                    table.clear();
                }
                "tr" => row.clear(),
                "tc" => cell.clear(),
                _ => {}
            },
            Ok(Event::Text(t)) if in_text => {
                para.push_str(&t.xml10_content());
            }
            Ok(Event::GeneralRef(g)) if in_text => para.push_str(&entity(&g)),
            Ok(Event::End(e)) => match local(e.name().as_ref()) {
                "t" => in_text = false,
                "p" => {
                    let text = para.trim().to_string();
                    if table_depth > 0 {
                        if !cell.is_empty() && !text.is_empty() {
                            cell.push(' ');
                        }
                        cell.push_str(&text);
                    } else if !text.is_empty() {
                        let s = style.to_lowercase();
                        let prefix = if s == "title" {
                            "# ".to_string()
                        } else if let Some(n) =
                            s.strip_prefix("heading").or(s.strip_prefix("título"))
                        {
                            format!(
                                "{} ",
                                "#".repeat(n.trim().parse::<usize>().unwrap_or(1).clamp(1, 6))
                            )
                        } else if in_list || s.contains("list") {
                            "- ".to_string()
                        } else {
                            String::new()
                        };
                        if !out.is_empty() {
                            out.push_str(
                                if prefix == "- "
                                    && out.ends_with('\n')
                                    && out
                                        .trim_end()
                                        .lines()
                                        .last()
                                        .is_some_and(|l| l.starts_with("- "))
                                {
                                    ""
                                } else {
                                    "\n"
                                },
                            );
                        }
                        out.push_str(&prefix);
                        out.push_str(&text);
                        out.push('\n');
                    }
                }
                "tc" => row.push(cell.replace('|', "\\|")),
                "tr" => table.push(std::mem::take(&mut row)),
                "tbl" => {
                    table_depth = table_depth.saturating_sub(1);
                    if table_depth == 0 && !table.is_empty() {
                        let width = table.iter().map(|r| r.len()).max().unwrap_or(0);
                        let mut md = String::new();
                        for (i, r) in table.iter().enumerate() {
                            let cells: Vec<String> = (0..width)
                                .map(|c| r.get(c).cloned().unwrap_or_default())
                                .collect();
                            md.push_str(&format!("| {} |\n", cells.join(" | ")));
                            if i == 0 {
                                md.push_str(&format!("|{}|\n", vec!["---"; width].join("|")));
                            }
                        }
                        out.push('\n');
                        out.push_str(&md);
                    }
                }
                _ => {}
            },
            Ok(Event::Eof) | Err(_) => break,
            _ => {}
        }
    }
    out.trim().to_string()
}

/// All text runs (`a:t`) of a DrawingML part, one paragraph per line.
pub fn drawing_text(xml: &str) -> Vec<String> {
    let mut r = Reader::from_str(xml);
    let mut paras = Vec::new();
    let mut cur = String::new();
    let mut in_t = false;
    loop {
        match r.read_event() {
            Ok(Event::Start(e)) if local(e.name().as_ref()) == "t" => in_t = true,
            Ok(Event::Text(t)) if in_t => cur.push_str(&t.xml10_content()),
            Ok(Event::GeneralRef(g)) if in_t => cur.push_str(&entity(&g)),
            Ok(Event::End(e)) => match local(e.name().as_ref()) {
                "t" => in_t = false,
                "p" => {
                    if !cur.trim().is_empty() {
                        paras.push(cur.trim().to_string());
                    }
                    cur.clear();
                }
                _ => {}
            },
            Ok(Event::Eof) | Err(_) => break,
            _ => {}
        }
    }
    if !cur.trim().is_empty() {
        paras.push(cur.trim().to_string());
    }
    paras
}

pub fn ingest_docx(path: &Path, bytes: &[u8]) -> Result<IngestedDoc, IngestError> {
    let mut zip = open_zip(bytes)?;
    let xml = read_entry(&mut zip, "word/document.xml")
        .ok_or_else(|| IngestError::Corrupt("word/document.xml ausente".into()))?;
    let md = docx_to_markdown(&xml);
    let words = md.split_whitespace().count();
    Ok(IngestedDoc {
        kind: DocKind::Document,
        file_name: file_name(path),
        summary: format!("{} palavras", crate::model::fmt_count(words)),
        blocks: vec![Block {
            locator: Locator::Whole,
            content: Content::Text { text: md },
        }],
        warnings: vec![],
    })
}

pub fn ingest_pptx(path: &Path, bytes: &[u8]) -> Result<IngestedDoc, IngestError> {
    let mut zip = open_zip(bytes)?;
    let mut slides: Vec<(u32, String)> = zip
        .file_names()
        .filter_map(|n| {
            let num = n
                .strip_prefix("ppt/slides/slide")?
                .strip_suffix(".xml")?
                .parse()
                .ok()?;
            Some((num, n.to_string()))
        })
        .collect();
    slides.sort();
    let mut blocks = Vec::new();
    for (n, name) in &slides {
        let xml = read_entry(&mut zip, name).unwrap_or_default();
        let paras = drawing_text(&xml);
        let title = paras.first().cloned().unwrap_or_default();
        let mut text = format!("## Slide {n} — {title}");
        for p in paras.iter().skip(1) {
            text.push('\n');
            text.push_str(p);
        }
        let notes = read_entry(&mut zip, &format!("ppt/notesSlides/notesSlide{n}.xml"))
            .map(|x| {
                drawing_text(&x)
                    .into_iter()
                    .filter(|p| p.parse::<u32>().is_err())
                    .collect::<Vec<_>>()
                    .join(" ")
            })
            .unwrap_or_default();
        if !notes.trim().is_empty() {
            text.push_str(&format!("\n> Notas: {}", notes.trim()));
        }
        blocks.push(Block {
            locator: Locator::Slide { n: *n },
            content: Content::Text { text },
        });
    }
    Ok(IngestedDoc {
        kind: DocKind::Presentation,
        file_name: file_name(path),
        summary: format!("{} slides", slides.len()),
        blocks,
        warnings: vec![],
    })
}

/// OpenDocument text/presentation: headings (`text:h`), paragraphs, list items.
pub fn ingest_odf(path: &Path, bytes: &[u8]) -> Result<IngestedDoc, IngestError> {
    let mut zip = open_zip(bytes)?;
    let xml = read_entry(&mut zip, "content.xml")
        .ok_or_else(|| IngestError::Corrupt("content.xml ausente".into()))?;
    let mut r = Reader::from_str(&xml);
    let mut out = String::new();
    let mut cur = String::new();
    let mut level: Option<usize> = None;
    let mut in_item = 0usize;
    loop {
        match r.read_event() {
            Ok(Event::Start(e)) => match local(e.name().as_ref()) {
                "h" => {
                    level = Some(
                        attr(&e, "outline-level")
                            .and_then(|v| v.parse().ok())
                            .unwrap_or(1),
                    )
                }
                "list-item" => in_item += 1,
                _ => {}
            },
            Ok(Event::Text(t)) => cur.push_str(&t.xml10_content()),
            Ok(Event::GeneralRef(g)) => cur.push_str(&entity(&g)),
            Ok(Event::End(e)) => match local(e.name().as_ref()) {
                "h" | "p" => {
                    let t = cur.trim().to_string();
                    cur.clear();
                    if !t.is_empty() {
                        let prefix = match (level.take(), in_item > 0) {
                            (Some(l), _) => format!("\n{} ", "#".repeat(l.clamp(1, 6))),
                            (None, true) => "- ".into(),
                            (None, false) => "\n".into(),
                        };
                        out.push_str(&prefix);
                        out.push_str(&t);
                        out.push('\n');
                    }
                }
                "list-item" => in_item = in_item.saturating_sub(1),
                _ => {}
            },
            Ok(Event::Eof) | Err(_) => break,
            _ => {}
        }
    }
    let md = out.trim().to_string();
    let kind = if path
        .extension()
        .is_some_and(|e| e.eq_ignore_ascii_case("odp"))
    {
        DocKind::Presentation
    } else {
        DocKind::Document
    };
    Ok(IngestedDoc {
        kind,
        file_name: file_name(path),
        summary: format!(
            "{} palavras",
            crate::model::fmt_count(md.split_whitespace().count())
        ),
        blocks: vec![Block {
            locator: Locator::Whole,
            content: Content::Text { text: md },
        }],
        warnings: vec![],
    })
}

/// RTF: strips control words and groups, keeps text and paragraph breaks.
pub fn rtf_to_text(rtf: &str) -> String {
    let mut out = String::new();
    let mut chars = rtf.chars().peekable();
    let mut skip_depth: Option<usize> = None;
    let mut depth = 0usize;
    while let Some(c) = chars.next() {
        match c {
            '{' => {
                depth += 1;
                if chars.peek() == Some(&'\\') {
                    let rest: String = chars.clone().take(12).collect();
                    if rest.starts_with("\\*")
                        || rest.starts_with("\\fonttbl")
                        || rest.starts_with("\\colortbl")
                        || rest.starts_with("\\stylesheet")
                        || rest.starts_with("\\info")
                    {
                        skip_depth.get_or_insert(depth);
                    }
                }
            }
            '}' => {
                if skip_depth == Some(depth) {
                    skip_depth = None;
                }
                depth = depth.saturating_sub(1);
            }
            '\\' => {
                let mut word = String::new();
                while let Some(&n) = chars.peek() {
                    if n.is_ascii_alphabetic() {
                        word.push(n);
                        chars.next();
                    } else {
                        break;
                    }
                }
                if word.is_empty() {
                    if let Some(n) = chars.next() {
                        if n == '\'' {
                            let hex: String = chars.by_ref().take(2).collect();
                            if let Ok(b) = u8::from_str_radix(&hex, 16)
                                && skip_depth.is_none()
                            {
                                out.push_str(&encoding_rs::WINDOWS_1252.decode(&[b]).0);
                            }
                        } else if skip_depth.is_none() && "{}\\".contains(n) {
                            out.push(n);
                        }
                    }
                    continue;
                }
                let mut num = String::new();
                while let Some(&n) = chars.peek() {
                    if n.is_ascii_digit() || (n == '-' && num.is_empty()) {
                        num.push(n);
                        chars.next();
                    } else {
                        break;
                    }
                }
                if chars.peek() == Some(&' ') {
                    chars.next();
                }
                if skip_depth.is_none() {
                    match word.as_str() {
                        "par" | "line" => out.push('\n'),
                        "tab" => out.push('\t'),
                        "u" => {
                            if let Ok(code) = num.parse::<i32>() {
                                if let Some(ch) = char::from_u32(if code < 0 {
                                    (code + 65536) as u32
                                } else {
                                    code as u32
                                }) {
                                    out.push(ch);
                                }
                                chars.next(); // replacement char
                            }
                        }
                        _ => {}
                    }
                }
            }
            '\r' | '\n' => {}
            other if skip_depth.is_none() && depth > 0 => out.push(other),
            _ => {}
        }
    }
    out.lines()
        .map(str::trim)
        .collect::<Vec<_>>()
        .join("\n")
        .trim()
        .to_string()
}

fn file_name(p: &Path) -> String {
    p.file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn docx_headings_lists_and_table() {
        let xml = r#"<w:document xmlns:w="w"><w:body>
          <w:p><w:pPr><w:pStyle w:val="Title"/></w:pPr><w:r><w:t>Relatório</w:t></w:r></w:p>
          <w:p><w:pPr><w:pStyle w:val="Heading2"/></w:pPr><w:r><w:t>Resumo</w:t></w:r></w:p>
          <w:p><w:pPr><w:numPr/></w:pPr><w:r><w:t>item um</w:t></w:r></w:p>
          <w:p><w:pPr><w:numPr/></w:pPr><w:r><w:t>item dois &amp; três</w:t></w:r></w:p>
          <w:tbl><w:tr><w:tc><w:p><w:r><w:t>A</w:t></w:r></w:p></w:tc><w:tc><w:p><w:r><w:t>B</w:t></w:r></w:p></w:tc></w:tr>
                 <w:tr><w:tc><w:p><w:r><w:t>1</w:t></w:r></w:p></w:tc><w:tc><w:p><w:r><w:t>2</w:t></w:r></w:p></w:tc></w:tr></w:tbl>
        </w:body></w:document>"#;
        assert_eq!(
            docx_to_markdown(xml),
            "# Relatório\n\n## Resumo\n\n- item um\n- item dois & três\n\n| A | B |\n|---|---|\n| 1 | 2 |"
        );
    }

    #[test]
    fn rtf_text() {
        let rtf = r"{\rtf1\ansi{\fonttbl{\f0 Arial;}}\f0 Ol\'e1 {\b mundo}\par Linha 2\u231?o}";
        assert_eq!(rtf_to_text(rtf), "Olá mundo\nLinha 2ço");
    }
}
