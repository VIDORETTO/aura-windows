//! Agent Skills (<https://agentskills.io>): a folder with `SKILL.md`
//! (YAML frontmatter `name` + `description`, Markdown body) plus optional
//! `scripts/`, `references/` and `assets/`.
//!
//! Importing never runs anything (OT-002): [`review_dir`]/[`review_zip`]
//! only read, [`install`] only copies after the user confirmed the review.

use serde::Serialize;
use std::collections::BTreeMap;
use std::io::Read;
use std::path::{Component, Path, PathBuf};
use thiserror::Error;

pub const MAX_NAME: usize = 64;
pub const MAX_DESCRIPTION: usize = 1024;
pub const MAX_SKILL_BYTES: u64 = 20 * 1024 * 1024;
pub const REVIEW_WARNING: &str = "Skills podem instruir o agente a executar comandos. Revise o conteúdo e os scripts antes de instalar.";

#[derive(Debug, Error, PartialEq, Eq)]
pub enum SkillError {
    #[error("SKILL.md não encontrado")]
    MissingSkillMd,
    #[error("o cabeçalho (frontmatter) do SKILL.md está ausente ou malformado")]
    BadFrontmatter,
    #[error("campo obrigatório ausente: {0}")]
    MissingField(&'static str),
    #[error(
        "nome inválido: use até 64 letras minúsculas, números e hífens (sem hífen no início, no fim ou duplo)"
    )]
    BadName,
    #[error("a descrição passa de 1024 caracteres")]
    DescriptionTooLong,
    #[error("o nome \"{name}\" não corresponde à pasta \"{dir}\"")]
    NameMismatch { name: String, dir: String },
    #[error("a Skill passa de 20 MB")]
    TooLarge,
    #[error("o arquivo .zip contém caminhos inseguros")]
    UnsafePath,
    #[error("já existe uma Skill chamada \"{0}\"")]
    AlreadyExists(String),
    #[error("io: {0}")]
    Io(String),
}

impl From<std::io::Error> for SkillError {
    fn from(e: std::io::Error) -> Self {
        SkillError::Io(e.to_string())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillManifest {
    pub name: String,
    pub description: String,
    /// Other frontmatter keys (`license`, `compatibility`, `allowed-tools`…).
    pub extra: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillFile {
    pub path: String,
    pub bytes: u64,
    pub is_script: bool,
}

/// Everything the user must see before confirming an import (AC-002).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillReview {
    pub manifest: SkillManifest,
    pub skill_md: String,
    pub files: Vec<SkillFile>,
    pub warning: &'static str,
}

impl SkillReview {
    pub fn scripts(&self) -> impl Iterator<Item = &SkillFile> {
        self.files.iter().filter(|f| f.is_script)
    }
}

pub fn validate_name(name: &str) -> Result<(), SkillError> {
    let ok = !name.is_empty()
        && name.len() <= MAX_NAME
        && name
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
        && !name.starts_with('-')
        && !name.ends_with('-')
        && !name.contains("--");
    if ok { Ok(()) } else { Err(SkillError::BadName) }
}

/// Parses the YAML frontmatter subset used by skills: `key: value`, quoted
/// values and `|`/`>` block scalars. Nested maps are kept as raw text.
pub fn parse_frontmatter(skill_md: &str) -> Result<(BTreeMap<String, String>, String), SkillError> {
    let text = skill_md.strip_prefix('\u{feff}').unwrap_or(skill_md);
    let rest = text.strip_prefix("---").ok_or(SkillError::BadFrontmatter)?;
    let rest = rest
        .strip_prefix("\r\n")
        .or_else(|| rest.strip_prefix('\n'))
        .ok_or(SkillError::BadFrontmatter)?;
    let end = rest.find("\n---").ok_or(SkillError::BadFrontmatter)?;
    let (yaml, body) = (&rest[..end], &rest[end + 4..]);
    let body = body.trim_start_matches(['\r', '\n', '-']).to_string();

    let mut map = BTreeMap::new();
    let lines: Vec<&str> = yaml.lines().collect();
    let mut i = 0;
    while i < lines.len() {
        let line = lines[i].trim_end_matches('\r');
        i += 1;
        if line.trim().is_empty()
            || line.trim_start().starts_with('#')
            || line.starts_with([' ', '\t'])
        {
            continue;
        }
        let (key, value) = line.split_once(':').ok_or(SkillError::BadFrontmatter)?;
        let value = value.trim();
        let mut block = Vec::new();
        while i < lines.len() && (lines[i].starts_with([' ', '\t']) || lines[i].trim().is_empty()) {
            block.push(lines[i].trim());
            i += 1;
        }
        let value = match value {
            "|" | "|-" => block.join("\n").trim().to_string(),
            ">" | ">-" | "" => block
                .join(" ")
                .split_whitespace()
                .collect::<Vec<_>>()
                .join(" "),
            v => unquote(v),
        };
        map.insert(key.trim().to_string(), value);
    }
    Ok((map, body))
}

fn unquote(v: &str) -> String {
    let b = v.as_bytes();
    if b.len() >= 2
        && ((b[0] == b'"' && b[b.len() - 1] == b'"') || (b[0] == b'\'' && b[b.len() - 1] == b'\''))
    {
        let inner = &v[1..v.len() - 1];
        if b[0] == b'"' {
            inner.replace("\\\"", "\"").replace("\\n", "\n")
        } else {
            inner.replace("''", "'")
        }
    } else {
        v.to_string()
    }
}

pub fn parse_manifest(skill_md: &str) -> Result<SkillManifest, SkillError> {
    let (mut map, _) = parse_frontmatter(skill_md)?;
    let name = map
        .remove("name")
        .filter(|s| !s.is_empty())
        .ok_or(SkillError::MissingField("name"))?;
    let description = map
        .remove("description")
        .filter(|s| !s.is_empty())
        .ok_or(SkillError::MissingField("description"))?;
    validate_name(&name)?;
    if description.chars().count() > MAX_DESCRIPTION {
        return Err(SkillError::DescriptionTooLong);
    }
    Ok(SkillManifest {
        name,
        description,
        extra: map,
    })
}

fn is_script(path: &str) -> bool {
    let lower = path.to_lowercase();
    lower.starts_with("scripts/")
        || [
            ".ps1", ".bat", ".cmd", ".sh", ".py", ".js", ".mjs", ".ts", ".exe", ".vbs",
        ]
        .iter()
        .any(|e| lower.ends_with(e))
}

/// Reviews a skill folder without executing anything.
pub fn review_dir(dir: &Path) -> Result<SkillReview, SkillError> {
    let skill_md =
        std::fs::read_to_string(dir.join("SKILL.md")).map_err(|_| SkillError::MissingSkillMd)?;
    let manifest = parse_manifest(&skill_md)?;
    if let Some(dir_name) = dir.file_name().map(|n| n.to_string_lossy().into_owned())
        && dir_name != manifest.name
    {
        return Err(SkillError::NameMismatch {
            name: manifest.name,
            dir: dir_name,
        });
    }
    let mut files = Vec::new();
    let mut total = 0;
    walk(dir, dir, &mut files, &mut total)?;
    files.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(SkillReview {
        manifest,
        skill_md,
        files,
        warning: REVIEW_WARNING,
    })
}

fn walk(
    root: &Path,
    dir: &Path,
    out: &mut Vec<SkillFile>,
    total: &mut u64,
) -> Result<(), SkillError> {
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let ft = entry.file_type()?;
        if ft.is_symlink() {
            return Err(SkillError::UnsafePath);
        }
        if ft.is_dir() {
            walk(root, &entry.path(), out, total)?;
        } else {
            let bytes = entry.metadata()?.len();
            *total += bytes;
            if *total > MAX_SKILL_BYTES {
                return Err(SkillError::TooLarge);
            }
            let rel = entry
                .path()
                .strip_prefix(root)
                .expect("under root")
                .to_string_lossy()
                .replace('\\', "/");
            out.push(SkillFile {
                is_script: is_script(&rel),
                path: rel,
                bytes,
            });
        }
    }
    Ok(())
}

/// A zip may hold the skill at its root or inside a single top folder.
struct ZipLayout {
    prefix: String,
}

fn zip_layout(
    zip: &mut zip::ZipArchive<std::io::Cursor<Vec<u8>>>,
) -> Result<ZipLayout, SkillError> {
    let names: Vec<String> = zip.file_names().map(str::to_string).collect();
    if names.iter().any(|n| n == "SKILL.md") {
        return Ok(ZipLayout {
            prefix: String::new(),
        });
    }
    names
        .iter()
        .filter_map(|n| {
            n.strip_suffix("SKILL.md")
                .filter(|p| p.matches('/').count() == 1)
        })
        .next()
        .map(|p| ZipLayout {
            prefix: p.to_string(),
        })
        .ok_or(SkillError::MissingSkillMd)
}

fn open_zip(bytes: Vec<u8>) -> Result<zip::ZipArchive<std::io::Cursor<Vec<u8>>>, SkillError> {
    zip::ZipArchive::new(std::io::Cursor::new(bytes)).map_err(|e| SkillError::Io(e.to_string()))
}

/// Safe relative path of a zip entry under `prefix` (zip-slip protection).
fn safe_rel(name: &str, prefix: &str) -> Result<Option<PathBuf>, SkillError> {
    let Some(rel) = name.strip_prefix(prefix) else {
        return Ok(None);
    };
    if rel.is_empty() || rel.ends_with('/') {
        return Ok(None);
    }
    let p = PathBuf::from(rel);
    if p.components().any(|c| !matches!(c, Component::Normal(_)))
        || rel.contains('\\')
        || rel.contains(':')
    {
        return Err(SkillError::UnsafePath);
    }
    Ok(Some(p))
}

pub fn review_zip(bytes: Vec<u8>) -> Result<SkillReview, SkillError> {
    let mut zip = open_zip(bytes)?;
    let layout = zip_layout(&mut zip)?;
    let mut skill_md = String::new();
    zip.by_name(&format!("{}SKILL.md", layout.prefix))
        .map_err(|_| SkillError::MissingSkillMd)?
        .read_to_string(&mut skill_md)?;
    let manifest = parse_manifest(&skill_md)?;
    let mut files = Vec::new();
    let mut total = 0u64;
    for i in 0..zip.len() {
        let f = zip.by_index(i).map_err(|e| SkillError::Io(e.to_string()))?;
        if let Some(rel) = safe_rel(f.name(), &layout.prefix)? {
            total += f.size();
            if total > MAX_SKILL_BYTES {
                return Err(SkillError::TooLarge);
            }
            let rel = rel.to_string_lossy().replace('\\', "/");
            files.push(SkillFile {
                is_script: is_script(&rel),
                path: rel,
                bytes: f.size(),
            });
        }
    }
    files.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(SkillReview {
        manifest,
        skill_md,
        files,
        warning: REVIEW_WARNING,
    })
}

pub enum SkillSource {
    Dir(PathBuf),
    Zip(Vec<u8>),
}

/// Copies a reviewed skill into `skills_root/<name>` (atomic via staging).
pub fn install(
    source: &SkillSource,
    skills_root: &Path,
    overwrite: bool,
) -> Result<PathBuf, SkillError> {
    let review = match source {
        SkillSource::Dir(d) => review_dir(d)?,
        SkillSource::Zip(b) => review_zip(b.clone())?,
    };
    let dest = skills_root.join(&review.manifest.name);
    if dest.exists() && !overwrite {
        return Err(SkillError::AlreadyExists(review.manifest.name));
    }
    std::fs::create_dir_all(skills_root)?;
    let staging = skills_root.join(format!(".staging-{}", review.manifest.name));
    let _ = std::fs::remove_dir_all(&staging);
    match source {
        SkillSource::Dir(d) => {
            for f in &review.files {
                let to = staging.join(&f.path);
                std::fs::create_dir_all(to.parent().expect("parent"))?;
                std::fs::copy(d.join(&f.path), to)?;
            }
        }
        SkillSource::Zip(b) => {
            let mut zip = open_zip(b.clone())?;
            let layout = zip_layout(&mut zip)?;
            for i in 0..zip.len() {
                let mut f = zip.by_index(i).map_err(|e| SkillError::Io(e.to_string()))?;
                if let Some(rel) = safe_rel(f.name(), &layout.prefix)? {
                    let to = staging.join(rel);
                    std::fs::create_dir_all(to.parent().expect("parent"))?;
                    let mut out = std::fs::File::create(to)?;
                    std::io::copy(&mut f, &mut out)?;
                }
            }
        }
    }
    if dest.exists() {
        std::fs::remove_dir_all(&dest)?;
    }
    std::fs::rename(&staging, &dest)?;
    Ok(dest)
}

/// Creates a new skill from the editor (AC-003).
pub fn create(
    skills_root: &Path,
    name: &str,
    description: &str,
    body: &str,
) -> Result<PathBuf, SkillError> {
    validate_name(name)?;
    let description = description.trim();
    if description.is_empty() {
        return Err(SkillError::MissingField("description"));
    }
    if description.chars().count() > MAX_DESCRIPTION {
        return Err(SkillError::DescriptionTooLong);
    }
    let dir = skills_root.join(name);
    if dir.exists() {
        return Err(SkillError::AlreadyExists(name.into()));
    }
    std::fs::create_dir_all(&dir)?;
    let desc = serde_json::to_string(description).expect("json string is valid YAML");
    std::fs::write(
        dir.join("SKILL.md"),
        format!(
            "---\nname: {name}\ndescription: {desc}\n---\n\n{}\n",
            body.trim()
        ),
    )?;
    Ok(dir)
}

/// Where a skill comes from, shown in the list (AC-001).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum SkillOrigin {
    Aura,
    User,
    System,
}

pub fn origin_of(path: &Path, aura_root: &Path, user_root: &Path) -> SkillOrigin {
    if path.starts_with(aura_root) {
        SkillOrigin::Aura
    } else if path.starts_with(user_root) {
        SkillOrigin::User
    } else {
        SkillOrigin::System
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frontmatter_variants() {
        let md = "---\nname: revisar-contrato\ndescription: >\n  Revisa contratos\n  em PT-BR\nlicense: 'MIT'\nmetadata:\n  author: x\n---\n# Corpo\n";
        let (m, body) = parse_frontmatter(md).unwrap();
        assert_eq!(m["description"], "Revisa contratos em PT-BR");
        assert_eq!(m["license"], "MIT");
        assert_eq!(body, "# Corpo\n");
        let man = parse_manifest(md).unwrap();
        assert_eq!(man.name, "revisar-contrato");
        assert!(man.extra.contains_key("metadata"));
    }

    #[test]
    fn invalid_manifests() {
        assert_eq!(
            parse_manifest("# sem cabeçalho"),
            Err(SkillError::BadFrontmatter)
        );
        assert_eq!(
            parse_manifest("---\nname: a\n---\n"),
            Err(SkillError::MissingField("description"))
        );
        assert_eq!(
            parse_manifest("---\nname: Revisar\ndescription: x\n---\n"),
            Err(SkillError::BadName)
        );
        for bad in ["-a", "a-", "a--b", "", &"a".repeat(65)] {
            assert_eq!(validate_name(bad), Err(SkillError::BadName), "{bad}");
        }
        let long = format!("---\nname: a\ndescription: {}\n---\n", "x".repeat(1025));
        assert_eq!(parse_manifest(&long), Err(SkillError::DescriptionTooLong));
    }

    #[test]
    fn zip_slip_is_rejected() {
        assert_eq!(safe_rel("../evil", ""), Err(SkillError::UnsafePath));
        assert_eq!(safe_rel("s/../../evil", "s/"), Err(SkillError::UnsafePath));
        assert_eq!(safe_rel("C:/x", ""), Err(SkillError::UnsafePath));
        assert_eq!(
            safe_rel("s/scripts/a.py", "s/"),
            Ok(Some(PathBuf::from("scripts/a.py")))
        );
    }
}
