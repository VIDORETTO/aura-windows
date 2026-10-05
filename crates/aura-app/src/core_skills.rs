//! Skills the agent always has (017): how to create skills, quick commands
//! and MCP servers with Aura's tools. They live outside the user's skills
//! root, are rewritten on every start and never appear in the catalog the
//! user toggles.

use std::path::Path;

/// `(name, SKILL.md)` of every core skill.
pub const CORE_SKILLS: &[(&str, &str)] = &[
    (
        "aura-criar-skill",
        include_str!("../core-skills/aura-criar-skill/SKILL.md"),
    ),
    (
        "aura-criar-comando-rapido",
        include_str!("../core-skills/aura-criar-comando-rapido/SKILL.md"),
    ),
    (
        "aura-ensaio",
        include_str!("../core-skills/aura-ensaio/SKILL.md"),
    ),
    (
        "aura-estudo",
        include_str!("../core-skills/aura-estudo/SKILL.md"),
    ),
    (
        "aura-carreira",
        include_str!("../core-skills/aura-carreira/SKILL.md"),
    ),
    (
        "aura-documentos",
        include_str!("../core-skills/aura-documentos/SKILL.md"),
    ),
    (
        "aura-criar-receita",
        include_str!("../core-skills/aura-criar-receita/SKILL.md"),
    ),
    (
        "aura-criar-receita",
        include_str!("../core-skills/aura-criar-receita/SKILL.md"),
    ),
    (
        "aura-configurar",
        include_str!("../core-skills/aura-configurar/SKILL.md"),
    ),
    (
        "aura-preparo",
        include_str!("../core-skills/aura-preparo/SKILL.md"),
    ),
    (
        "aura-adicionar-servidor-mcp",
        include_str!("../core-skills/aura-adicionar-servidor-mcp/SKILL.md"),
    ),
];

/// Names starting with this prefix are reserved for core skills.
pub const RESERVED_PREFIX: &str = "aura-";

/// Writes the core skills under `root`, replacing older copies and removing
/// folders of core skills that no longer exist.
pub fn install(root: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(root)?;
    for entry in std::fs::read_dir(root)?.flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        if entry.path().is_dir() && !CORE_SKILLS.iter().any(|(n, _)| *n == name) {
            std::fs::remove_dir_all(entry.path())?;
        }
    }
    for (name, md) in CORE_SKILLS {
        let dir = root.join(name);
        std::fs::create_dir_all(&dir)?;
        let path = dir.join("SKILL.md");
        if std::fs::read_to_string(&path).ok().as_deref() != Some(*md) {
            std::fs::write(path, md)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn core_skills_are_valid_and_reinstalled() {
        for (name, md) in CORE_SKILLS {
            let m = aura_extensions::skills::parse_manifest(md).unwrap();
            assert_eq!(&m.name, name);
            assert!(name.starts_with(RESERVED_PREFIX));
        }
        let root = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(root.path().join("antiga")).unwrap();
        std::fs::write(root.path().join("aura-criar-skill/SKILL.md"), "x").ok();
        install(root.path()).unwrap();
        assert!(!root.path().join("antiga").exists());
        for (name, md) in CORE_SKILLS {
            let on_disk = std::fs::read_to_string(root.path().join(name).join("SKILL.md")).unwrap();
            assert_eq!(&on_disk, md);
        }
    }
}
