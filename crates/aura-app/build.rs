use std::collections::BTreeMap;
use std::path::Path;

fn messages(path: &Path) -> BTreeMap<String, String> {
    println!("cargo:rerun-if-changed={}", path.display());
    let source = std::fs::read_to_string(path).expect("canonical UI dictionary must be readable");
    let mut entries = BTreeMap::new();
    for line in source
        .lines()
        .map(str::trim)
        .filter(|line| line.starts_with('"'))
    {
        // Dictionary entries are one-line JSON string properties, with a TS
        // trailing comma. Parse the property, not arbitrary TypeScript code.
        let property = line.strip_suffix(',').unwrap_or(line);
        let parsed: BTreeMap<String, String> = serde_json::from_str(&format!("{{{property}}}"))
            .unwrap_or_else(|e| panic!("invalid dictionary entry in {}: {e}", path.display()));
        for (key, value) in parsed {
            assert!(
                entries.insert(key, value).is_none(),
                "duplicate message key"
            );
        }
    }
    assert!(!entries.is_empty(), "dictionary cannot be empty");
    entries
}

fn main() {
    let root = std::path::PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").unwrap());
    let pt = messages(&root.join("../../apps/desktop/src/i18n/pt-BR.ts"));
    let en = messages(&root.join("../../apps/desktop/src/i18n/en.ts"));
    assert!(pt.keys().eq(en.keys()), "dictionary keys must have parity");
    let mut generated = String::new();
    for (name, entries) in [("PT_BR", pt), ("EN", en)] {
        generated.push_str(&format!("const {name}: &[(&str, &str)] = &[\n"));
        for (key, value) in entries {
            generated.push_str(&format!("    ({key:?}, {value:?}),\n"));
        }
        generated.push_str("];\n");
    }
    let out = std::path::PathBuf::from(std::env::var_os("OUT_DIR").unwrap());
    std::fs::write(out.join("messages.rs"), generated).expect("write generated UI messages");
}
