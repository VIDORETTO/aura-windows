//! Spreadsheets → per-sheet schema, 50-row sample and numeric statistics
//! (AC-005 of 007). xlsx/xlsm/xls/ods via `calamine`, csv/tsv via `csv`.

use crate::IngestError;
use crate::model::{Block, Content, DocKind, IngestedDoc, Locator, fmt_count};
use calamine::{Data, Reader, open_workbook_auto_from_rs};
use std::io::Cursor;
use std::path::Path;

pub const SAMPLE_ROWS: usize = 50;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColType {
    Number,
    Date,
    Text,
    Bool,
    Empty,
}

impl ColType {
    fn label(&self) -> &'static str {
        match self {
            ColType::Number => "número",
            ColType::Date => "data",
            ColType::Text => "texto",
            ColType::Bool => "booleano",
            ColType::Empty => "vazio",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Cell {
    Empty,
    Number(f64),
    Text(String),
    Bool(bool),
    Date(String),
}

impl Cell {
    fn ty(&self) -> ColType {
        match self {
            Cell::Empty => ColType::Empty,
            Cell::Number(_) => ColType::Number,
            Cell::Text(_) => ColType::Text,
            Cell::Bool(_) => ColType::Bool,
            Cell::Date(_) => ColType::Date,
        }
    }
    fn display(&self) -> String {
        match self {
            Cell::Empty => String::new(),
            Cell::Number(n) => format_number(*n),
            Cell::Text(t) => t.replace('|', "\\|").replace('\n', " "),
            Cell::Bool(b) => b.to_string(),
            Cell::Date(d) => d.clone(),
        }
    }
}

fn format_number(n: f64) -> String {
    if n.fract() == 0.0 && n.abs() < 1e15 {
        format!("{}", n as i64)
    } else {
        let s = format!("{n:.4}");
        s.trim_end_matches('0').trim_end_matches('.').to_string()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Sheet {
    pub name: String,
    pub rows: Vec<Vec<Cell>>,
}

fn from_calamine(d: &Data) -> Cell {
    match d {
        Data::Empty => Cell::Empty,
        Data::Float(f) => Cell::Number(*f),
        Data::Int(i) => Cell::Number(*i as f64),
        Data::Bool(b) => Cell::Bool(*b),
        Data::String(s) => {
            if s.trim().is_empty() {
                Cell::Empty
            } else {
                Cell::Text(s.clone())
            }
        }
        Data::DateTime(dt) => Cell::Date(excel_serial_to_iso(dt.as_f64())),
        Data::DateTimeIso(s) => Cell::Date(s.clone()),
        Data::DurationIso(s) => Cell::Text(s.clone()),
        Data::Error(e) => Cell::Text(format!("#{e:?}")),
    }
}

/// Excel serial date (days since 1899-12-30) → `YYYY-MM-DD[ HH:MM]`.
pub fn excel_serial_to_iso(serial: f64) -> String {
    let days = serial.floor() as i64;
    let secs = ((serial - serial.floor()) * 86_400.0).round() as i64;
    // Days from 1970-01-01: 1899-12-30 is 25 569 days before the epoch.
    let z = days - 25_569 + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + if m <= 2 { 1 } else { 0 };
    if secs == 0 {
        format!("{y:04}-{m:02}-{d:02}")
    } else {
        format!(
            "{y:04}-{m:02}-{d:02} {:02}:{:02}",
            secs / 3600,
            (secs % 3600) / 60
        )
    }
}

pub fn read_workbook(bytes: &[u8]) -> Result<Vec<Sheet>, IngestError> {
    let mut wb = open_workbook_auto_from_rs(Cursor::new(bytes.to_vec())).map_err(|e| {
        let msg = e.to_string();
        if msg.to_lowercase().contains("password") || msg.to_lowercase().contains("encrypt") {
            IngestError::Encrypted
        } else {
            IngestError::Corrupt(msg)
        }
    })?;
    let names = wb.sheet_names().to_vec();
    let mut out = Vec::new();
    for name in names {
        let range = wb
            .worksheet_range(&name)
            .map_err(|e| IngestError::Corrupt(e.to_string()))?;
        let rows = range
            .rows()
            .map(|r| r.iter().map(from_calamine).collect())
            .collect();
        out.push(Sheet { name, rows });
    }
    Ok(out)
}

pub fn read_csv(bytes: &[u8], name: &str) -> Result<Vec<Sheet>, IngestError> {
    let text = crate::text::decode(bytes);
    let first_line = text.lines().next().unwrap_or_default();
    let delim = b";\t,|"
        .iter()
        .copied()
        .max_by_key(|d| first_line.bytes().filter(|b| b == d).count())
        .unwrap_or(b',');
    let mut rdr = csv::ReaderBuilder::new()
        .delimiter(delim)
        .has_headers(false)
        .flexible(true)
        .from_reader(text.as_bytes());
    let mut rows = Vec::new();
    for rec in rdr.records() {
        let rec = rec.map_err(|e| IngestError::Corrupt(e.to_string()))?;
        rows.push(
            rec.iter()
                .map(|v| {
                    let t = v.trim();
                    if t.is_empty() {
                        Cell::Empty
                    } else if let Some(n) = parse_number(t) {
                        Cell::Number(n)
                    } else if looks_like_date(t) {
                        Cell::Date(t.to_string())
                    } else {
                        Cell::Text(t.to_string())
                    }
                })
                .collect(),
        );
    }
    Ok(vec![Sheet {
        name: name.to_string(),
        rows,
    }])
}

fn parse_number(t: &str) -> Option<f64> {
    if let Ok(n) = t.parse::<f64>() {
        return Some(n);
    }
    // Brazilian format: 1.234,56
    let br = t.replace('.', "").replace(',', ".");
    if t.contains(',')
        && br.parse::<f64>().is_ok()
        && t.chars().all(|c| c.is_ascii_digit() || ".,-".contains(c))
    {
        return br.parse().ok();
    }
    None
}

fn looks_like_date(t: &str) -> bool {
    let b = t.as_bytes();
    (b.len() == 10 && b[4] == b'-' && b[7] == b'-')
        || (b.len() == 10 && b[2] == b'/' && b[5] == b'/')
}

/// Header = first non-empty row when ≥ 60% of its cells are text and the next
/// row has at least one non-text cell (or there is no next row).
pub fn infer_header(rows: &[Vec<Cell>]) -> Option<usize> {
    let first = rows
        .iter()
        .position(|r| r.iter().any(|c| *c != Cell::Empty))?;
    let row = &rows[first];
    let non_empty: Vec<&Cell> = row.iter().filter(|c| **c != Cell::Empty).collect();
    let texts = non_empty
        .iter()
        .filter(|c| matches!(c, Cell::Text(_)))
        .count();
    if non_empty.is_empty() || (texts as f64) / (non_empty.len() as f64) < 0.6 {
        return None;
    }
    match rows.get(first + 1) {
        Some(next)
            if next
                .iter()
                .all(|c| matches!(c, Cell::Text(_) | Cell::Empty)) =>
        {
            None
        }
        _ => Some(first),
    }
}

/// Majority type (≥ 80% of non-empty cells in the first 1 000 rows).
pub fn infer_type(cells: &[&Cell]) -> ColType {
    let non_empty: Vec<&&Cell> = cells
        .iter()
        .take(1000)
        .filter(|c| ***c != Cell::Empty)
        .collect();
    if non_empty.is_empty() {
        return ColType::Empty;
    }
    for ty in [ColType::Number, ColType::Date, ColType::Bool] {
        let n = non_empty.iter().filter(|c| c.ty() == ty).count();
        if n as f64 / non_empty.len() as f64 >= 0.8 {
            return ty;
        }
    }
    ColType::Text
}

#[derive(Debug, Clone, PartialEq)]
pub struct Stats {
    pub min: f64,
    pub max: f64,
    pub mean: f64,
    pub sum: f64,
}

pub fn column_stats(values: &[f64]) -> Option<Stats> {
    if values.is_empty() {
        return None;
    }
    let sum: f64 = values.iter().sum();
    Some(Stats {
        min: values.iter().cloned().fold(f64::INFINITY, f64::min),
        max: values.iter().cloned().fold(f64::NEG_INFINITY, f64::max),
        mean: sum / values.len() as f64,
        sum,
    })
}

fn r2(v: f64) -> String {
    format!("{:.2}", (v * 100.0).round() / 100.0)
}

fn markdown_table(header: &[String], rows: &[Vec<Cell>]) -> String {
    let mut md = format!(
        "| {} |\n|{}|\n",
        header.join(" | "),
        header.iter().map(|_| "---").collect::<Vec<_>>().join("|")
    );
    for r in rows {
        let cells: Vec<String> = (0..header.len())
            .map(|i| r.get(i).map(Cell::display).unwrap_or_default())
            .collect();
        md.push_str(&format!("| {} |\n", cells.join(" | ")));
    }
    md
}

fn col_name(i: usize) -> String {
    let mut n = i + 1;
    let mut s = String::new();
    while n > 0 {
        let r = (n - 1) % 26;
        s.insert(0, (b'A' + r as u8) as char);
        n = (n - 1) / 26;
    }
    s
}

/// Builds blocks for one sheet. `rows` limits the sample to a range (1-based,
/// inclusive, counted from the first data row) for `attachment_read`.
pub fn sheet_blocks(sheet: &Sheet, range: Option<(u32, u32)>) -> Vec<Block> {
    let header_idx = infer_header(&sheet.rows);
    let width = sheet.rows.iter().map(|r| r.len()).max().unwrap_or(0);
    let header: Vec<String> = match header_idx {
        Some(i) => (0..width)
            .map(|c| match sheet.rows[i].get(c) {
                Some(Cell::Text(t)) => t.clone(),
                Some(other) if *other != Cell::Empty => other.display(),
                _ => col_name(c),
            })
            .collect(),
        None => (0..width).map(col_name).collect(),
    };
    let data: Vec<Vec<Cell>> = sheet
        .rows
        .iter()
        .skip(header_idx.map(|i| i + 1).unwrap_or(0))
        .cloned()
        .collect();
    let mut schema = format!(
        "Aba \"{}\": {} linhas × {} colunas",
        sheet.name,
        fmt_count(data.len()),
        width
    );
    let mut stats_lines = Vec::new();
    for (c, name) in header.iter().enumerate() {
        let cells: Vec<&Cell> = data.iter().filter_map(|r| r.get(c)).collect();
        let ty = infer_type(&cells);
        schema.push_str(&format!("\n- {name} ({})", ty.label()));
        if ty == ColType::Number {
            let values: Vec<f64> = cells
                .iter()
                .filter_map(|c| {
                    if let Cell::Number(n) = c {
                        Some(*n)
                    } else {
                        None
                    }
                })
                .collect();
            if let Some(s) = column_stats(&values) {
                stats_lines.push(format!(
                    "- {name}: mín {}, máx {}, média {}, soma {}",
                    r2(s.min),
                    r2(s.max),
                    r2(s.mean),
                    r2(s.sum)
                ));
            }
        }
    }
    if !stats_lines.is_empty() {
        schema.push_str("\nEstatísticas:\n");
        schema.push_str(&stats_lines.join("\n"));
    }
    let (from, to) = match range {
        Some((a, b)) => ((a.max(1) - 1) as usize, (b as usize).min(data.len())),
        None => (0, SAMPLE_ROWS.min(data.len())),
    };
    let sample = if from < to {
        &data[from..to]
    } else {
        &data[0..0]
    };
    let label = match range {
        Some(_) => format!("Linhas {}–{}:", from + 1, to),
        None if data.len() > SAMPLE_ROWS => format!(
            "Primeiras {SAMPLE_ROWS} de {} linhas:",
            fmt_count(data.len())
        ),
        None => "Dados:".to_string(),
    };
    vec![
        Block {
            locator: Locator::Sheet {
                name: sheet.name.clone(),
                rows: None,
            },
            content: Content::Text { text: schema },
        },
        Block {
            locator: Locator::Sheet {
                name: sheet.name.clone(),
                rows: Some(((from + 1) as u32, to as u32)),
            },
            content: Content::Table {
                markdown: format!("{label}\n{}", markdown_table(&header, sample)),
            },
        },
    ]
}

pub fn ingest(path: &Path, bytes: &[u8]) -> Result<IngestedDoc, IngestError> {
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    let ext = path
        .extension()
        .map(|e| e.to_string_lossy().to_lowercase())
        .unwrap_or_default();
    let sheets = if ext == "csv" || ext == "tsv" {
        read_csv(bytes, &name)?
    } else {
        read_workbook(bytes)?
    };
    let total_rows: usize = sheets
        .iter()
        .map(|s| {
            s.rows
                .len()
                .saturating_sub(infer_header(&s.rows).map(|i| i + 1).unwrap_or(0))
        })
        .sum();
    let blocks = sheets.iter().flat_map(|s| sheet_blocks(s, None)).collect();
    let tabs = if sheets.len() == 1 {
        "1 aba".to_string()
    } else {
        format!("{} abas", sheets.len())
    };
    Ok(IngestedDoc {
        kind: DocKind::Spreadsheet,
        file_name: name,
        summary: format!("{tabs} · {} linhas", fmt_count(total_rows)),
        blocks,
        warnings: vec![],
    })
}

pub fn read_sheet(
    path: &Path,
    bytes: &[u8],
    sheet: &str,
    rows: Option<(u32, u32)>,
) -> Result<Vec<Block>, IngestError> {
    let ext = path
        .extension()
        .map(|e| e.to_string_lossy().to_lowercase())
        .unwrap_or_default();
    let csv = ext == "csv" || ext == "tsv";
    let sheets = if csv {
        read_csv(bytes, sheet)?
    } else {
        read_workbook(bytes)?
    };
    // An empty name (or any name for CSV) means "the only sheet"; a wrong
    // explicit name is an error so the agent can correct itself.
    let lenient = csv || (sheet.trim().is_empty() && sheets.len() == 1);
    let s = sheets
        .iter()
        .find(|s| s.name.eq_ignore_ascii_case(sheet.trim()) || lenient)
        .ok_or_else(|| IngestError::NotFound(format!("aba {sheet}")))?;
    Ok(sheet_blocks(s, rows))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn header_and_types() {
        let rows = vec![
            vec![
                Cell::Text("Data".into()),
                Cell::Text("Produto".into()),
                Cell::Text("Valor".into()),
            ],
            vec![
                Cell::Date("2026-01-02".into()),
                Cell::Text("Café".into()),
                Cell::Number(10.5),
            ],
            vec![
                Cell::Date("2026-01-03".into()),
                Cell::Text("Pão".into()),
                Cell::Number(3.0),
            ],
        ];
        assert_eq!(infer_header(&rows), Some(0));
        let col: Vec<&Cell> = rows[1..].iter().map(|r| &r[2]).collect();
        assert_eq!(infer_type(&col), ColType::Number);
        let only_numbers = vec![vec![Cell::Number(1.0), Cell::Number(2.0)]];
        assert_eq!(infer_header(&only_numbers), None);
    }

    #[test]
    fn csv_with_semicolons_and_brazilian_numbers() {
        let text = "Nome;Valor\nAção;1.234,56\nOutro;10\n";
        let latin: Vec<u8> = encoding_rs::WINDOWS_1252.encode(text).0.into_owned();
        let d = ingest(Path::new("dados.csv"), &latin).unwrap();
        let Content::Text { text: schema } = &d.blocks[0].content else {
            panic!()
        };
        assert!(schema.contains("Valor (número)"), "{schema}");
        assert!(schema.contains("soma 1244.56"), "{schema}");
        let Content::Table { markdown } = &d.blocks[1].content else {
            panic!()
        };
        assert!(markdown.contains("| Ação | 1234.56 |"), "{markdown}");
        assert_eq!(d.summary, "1 aba · 2 linhas");
    }

    #[test]
    fn excel_dates() {
        assert_eq!(excel_serial_to_iso(45658.0), "2025-01-01");
        assert_eq!(excel_serial_to_iso(45658.5), "2025-01-01 12:00");
        assert_eq!(excel_serial_to_iso(1.0), "1899-12-31");
    }

    #[test]
    fn stats() {
        let s = column_stats(&[10.0, 999.9, 20.0]).unwrap();
        assert_eq!((s.min, s.max), (10.0, 999.9));
        assert_eq!(r2(s.mean), "343.30");
    }
}
