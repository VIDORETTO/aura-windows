//! Redacted diagnostics package (010 TK-006): state and logs for support,
//! never secrets or conversation content (AC-011).

use aura_core::logging::redact;
use serde_json::{Value, json};
use std::io::Write;
use std::path::{Path, PathBuf};

/// Last bytes of each log file included in the package.
const LOG_TAIL: u64 = 2 * 1024 * 1024;

pub struct DiagnosticsInput {
    pub summary: Value,
    pub settings: Value,
    pub privacy: Value,
    pub providers: Value,
    pub mcp: Value,
    pub logs_dir: PathBuf,
}

fn tail(path: &Path) -> std::io::Result<String> {
    use std::io::{Read, Seek, SeekFrom};
    let mut f = std::fs::File::open(path)?;
    let len = f.metadata()?.len();
    if len > LOG_TAIL {
        f.seek(SeekFrom::Start(len - LOG_TAIL))?;
    }
    let mut buf = Vec::new();
    f.read_to_end(&mut buf)?;
    Ok(String::from_utf8_lossy(&buf).into_owned())
}

/// Writes the zip and returns its path.
pub fn export(input: &DiagnosticsInput, dest: &Path) -> std::io::Result<PathBuf> {
    if let Some(dir) = dest.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let file = std::fs::File::create(dest)?;
    let mut zip = zip::ZipWriter::new(file);
    let opts = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);
    let mut put = |name: &str, text: &str| -> std::io::Result<()> {
        zip.start_file(name, opts).map_err(std::io::Error::other)?;
        // Everything passes the secret redactor, JSON included.
        zip.write_all(redact(text).as_bytes())
    };
    put(
        "README.txt",
        "Pacote de diagnóstico do Aura. Segredos e conteúdo de conversas não são incluídos; logs passaram por redação.\n",
    )?;
    for (name, v) in [
        ("summary.json", &input.summary),
        ("settings.json", &input.settings),
        ("privacy.json", &input.privacy),
        ("providers.json", &input.providers),
        ("mcp.json", &input.mcp),
    ] {
        put(name, &serde_json::to_string_pretty(v).unwrap_or_default())?;
    }
    if let Ok(rd) = std::fs::read_dir(&input.logs_dir) {
        let mut logs: Vec<PathBuf> = rd
            .flatten()
            .map(|e| e.path())
            .filter(|p| p.is_file())
            .collect();
        logs.sort();
        for p in logs.iter().rev().take(5) {
            let name = format!(
                "logs/{}",
                p.file_name().unwrap_or_default().to_string_lossy()
            );
            put(&name, &tail(p)?)?;
        }
    }
    zip.finish().map_err(std::io::Error::other)?;
    Ok(dest.to_path_buf())
}

/// Settings without free text the user wrote (personal instructions).
pub fn settings_view(s: &aura_core::settings::Settings) -> Value {
    let mut v = serde_json::to_value(s).unwrap_or(Value::Null);
    if let Some(o) = v.as_object_mut() {
        let n = s.personal_instructions.chars().count();
        o.insert(
            "personalInstructions".into(),
            json!(format!("<{n} caracteres>")),
        );
        o.insert(
            "asrVocabulary".into(),
            json!(format!("<{} termos>", s.asr_vocabulary.len())),
        );
    }
    v
}
