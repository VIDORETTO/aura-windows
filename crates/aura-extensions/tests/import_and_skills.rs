use aura_core::credentials::{CredentialStore, MemoryCredentialStore};
use aura_extensions::import::{ExternalApp, Roots, detect};
use aura_extensions::mcp_config::{EnvValue, McpServersRepo, Transport};
use aura_extensions::skills::{self, SkillError, SkillSource};
use std::io::Write;

#[test]
fn detects_servers_from_three_apps_and_moves_secrets_to_the_vault() {
    let dir = tempfile::tempdir().unwrap();
    let roots = Roots {
        home: dir.path().join("home"),
        appdata: dir.path().join("appdata"),
    };
    let claude = roots.appdata.join("Claude");
    std::fs::create_dir_all(&claude).unwrap();
    std::fs::write(
        claude.join("claude_desktop_config.json"),
        r#"{"mcpServers":{"github":{"command":"npx","args":["-y","@modelcontextprotocol/server-github"],"env":{"GITHUB_PERSONAL_ACCESS_TOKEN":"ghp_secret","LOG":"1"}}}}"#,
    )
    .unwrap();
    let vscode = roots.appdata.join("Code").join("User");
    std::fs::create_dir_all(&vscode).unwrap();
    std::fs::write(
        vscode.join("mcp.json"),
        "{\n  // comentário\n  \"servers\": {\n    \"docs\": {\"type\": \"http\", \"url\": \"https://example.com/mcp\", \"headers\": {\"Authorization\": \"Bearer tok_123\"}},\n  },\n}",
    )
    .unwrap();
    let codex = roots.home.join(".codex");
    std::fs::create_dir_all(&codex).unwrap();
    std::fs::write(codex.join("config.toml"), "model = \"x\"\n[mcp_servers.github]\ncommand = \"docker\"\nargs = [\"run\", \"gh\"]\ndisabled_tools = [\"a\"]\n").unwrap();

    let found = detect(&roots);
    assert_eq!(found.len(), 3, "{found:?}");
    let names: Vec<_> = found
        .iter()
        .map(|d| (d.app, d.spec.name.as_str()))
        .collect();
    assert_eq!(
        names,
        vec![
            (ExternalApp::ClaudeDesktop, "github"),
            (ExternalApp::VsCode, "docs"),
            (ExternalApp::CodexCli, "github-codex")
        ]
    );

    // The UI-facing JSON never carries secret values.
    let json = serde_json::to_string(&found).unwrap();
    assert!(
        !json.contains("ghp_secret") && !json.contains("tok_123"),
        "{json}"
    );

    let vault = MemoryCredentialStore::default();
    let repo = McpServersRepo::new(aura_store::Store::open_in_memory().unwrap());
    for d in &found {
        repo.save(&d.spec, &d.secrets, d.bearer.as_ref(), &vault)
            .unwrap();
    }
    let gh = &found[0].spec;
    let Transport::Stdio { env, .. } = &gh.transport else {
        panic!()
    };
    assert_eq!(env["GITHUB_PERSONAL_ACCESS_TOKEN"], EnvValue::Secret);
    assert_eq!(
        vault
            .get("Aura/mcp/github/GITHUB_PERSONAL_ACCESS_TOKEN")
            .unwrap()
            .unwrap()
            .expose(),
        "ghp_secret"
    );
    assert_eq!(
        vault
            .get("Aura/mcp/docs/BEARER_TOKEN")
            .unwrap()
            .unwrap()
            .expose(),
        "tok_123"
    );
    assert_eq!(found[2].spec.disabled_tools, vec!["a".to_string()]);
    let toml = toml::to_string(&found[0].spec.to_toml()).unwrap();
    assert!(!toml.contains("ghp_secret"));
}

fn write_skill(root: &std::path::Path, name: &str, md: &str) -> std::path::PathBuf {
    let d = root.join(name);
    std::fs::create_dir_all(d.join("scripts")).unwrap();
    std::fs::write(d.join("SKILL.md"), md).unwrap();
    std::fs::write(d.join("scripts").join("run.ps1"), "Write-Host oi").unwrap();
    d
}

#[test]
fn skill_review_lists_scripts_and_install_copies_without_running() {
    let dir = tempfile::tempdir().unwrap();
    let src = write_skill(
        dir.path(),
        "revisar-contrato",
        "---\nname: revisar-contrato\ndescription: Revisa contratos\n---\nPasso 1\n",
    );
    let review = skills::review_dir(&src).unwrap();
    assert_eq!(review.manifest.name, "revisar-contrato");
    assert_eq!(
        review
            .scripts()
            .map(|f| f.path.as_str())
            .collect::<Vec<_>>(),
        vec!["scripts/run.ps1"]
    );
    assert!(review.warning.contains("executar comandos"));

    let root = dir.path().join("installed");
    let dest = skills::install(&SkillSource::Dir(src.clone()), &root, false).unwrap();
    assert!(dest.join("scripts/run.ps1").exists());
    assert_eq!(
        skills::install(&SkillSource::Dir(src), &root, false),
        Err(SkillError::AlreadyExists("revisar-contrato".into()))
    );

    let bad = write_skill(
        dir.path(),
        "outra",
        "---\nname: diferente\ndescription: x\n---\n",
    );
    assert!(matches!(
        skills::review_dir(&bad),
        Err(SkillError::NameMismatch { .. })
    ));
    let nodesc = write_skill(dir.path(), "sem-desc", "---\nname: sem-desc\n---\n");
    assert_eq!(
        skills::review_dir(&nodesc),
        Err(SkillError::MissingField("description"))
    );
}

#[test]
fn skill_zip_with_top_folder_and_zip_slip() {
    let mut buf = std::io::Cursor::new(Vec::new());
    {
        let mut z = zip::ZipWriter::new(&mut buf);
        let o = zip::write::SimpleFileOptions::default();
        z.start_file("resumir/SKILL.md", o).unwrap();
        z.write_all(b"---\nname: resumir\ndescription: \"Resume: textos\"\n---\nCorpo")
            .unwrap();
        z.start_file("resumir/references/guia.md", o).unwrap();
        z.write_all(b"guia").unwrap();
        z.finish().unwrap();
    }
    let bytes = buf.into_inner();
    let review = skills::review_zip(bytes.clone()).unwrap();
    assert_eq!(review.manifest.description, "Resume: textos");
    assert_eq!(review.files.len(), 2);
    let dir = tempfile::tempdir().unwrap();
    let dest = skills::install(&SkillSource::Zip(bytes), dir.path(), false).unwrap();
    assert_eq!(
        std::fs::read_to_string(dest.join("references/guia.md")).unwrap(),
        "guia"
    );

    let mut evil = std::io::Cursor::new(Vec::new());
    {
        let mut z = zip::ZipWriter::new(&mut evil);
        let o = zip::write::SimpleFileOptions::default();
        z.start_file("SKILL.md", o).unwrap();
        z.write_all(b"---\nname: x\ndescription: y\n---\n").unwrap();
        z.start_file("../../fora.txt", o).unwrap();
        z.write_all(b"x").unwrap();
        z.finish().unwrap();
    }
    assert_eq!(
        skills::review_zip(evil.into_inner()).err(),
        Some(SkillError::UnsafePath)
    );
}

#[test]
fn create_skill_from_editor() {
    let dir = tempfile::tempdir().unwrap();
    let d = skills::create(
        dir.path(),
        "revisar-contrato",
        "Revisa: cláusulas \"críticas\"",
        "Leia o contrato.",
    )
    .unwrap();
    let review = skills::review_dir(&d).unwrap();
    assert_eq!(
        review.manifest.description,
        "Revisa: cláusulas \"críticas\""
    );
    assert!(review.skill_md.contains("Leia o contrato."));
    assert_eq!(
        skills::create(dir.path(), "Nome Ruim", "x", ""),
        Err(SkillError::BadName)
    );
}
