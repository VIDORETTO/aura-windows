//! Model catalog and hardware-aware recommendation (AC-001/AC-002 of 006).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModelFile {
    pub path: String,
    pub url: String,
    pub size: u64,
    pub sha256: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct ModelEntry {
    pub id: String,
    pub name: String,
    pub family: String,
    /// `onnx-parakeet`, `ggml-whisper`, … (worker engine key).
    pub engine: String,
    pub description: String,
    /// ISO 639-1 codes; `*` means multilingual (Whisper).
    pub languages: Vec<String>,
    pub speed: f32,
    pub accuracy: f32,
    pub streaming: bool,
    pub min_ram_mb: u64,
    pub gpu_recommended: bool,
    pub license: String,
    pub source_url: String,
    pub files: Vec<ModelFile>,
}

impl ModelEntry {
    pub fn size_bytes(&self) -> u64 {
        self.files.iter().map(|f| f.size).sum()
    }
    pub fn supports(&self, lang: &str) -> bool {
        let base = lang.split(['-', '_']).next().unwrap_or(lang).to_lowercase();
        self.languages.iter().any(|l| l == "*" || *l == base)
    }
}

#[derive(Deserialize)]
struct CatalogFile {
    model: Vec<ModelEntry>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Catalog {
    pub models: Vec<ModelEntry>,
}

impl Catalog {
    pub fn builtin() -> Self {
        Self::parse(include_str!("../models.toml")).expect("valid models.toml")
    }
    pub fn parse(text: &str) -> Result<Self, toml::de::Error> {
        Ok(Self {
            models: toml::from_str::<CatalogFile>(text)?.model,
        })
    }
    pub fn get(&self, id: &str) -> Option<&ModelEntry> {
        self.models.iter().find(|m| m.id == id)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Gpu {
    pub name: String,
    pub vendor: String,
    pub vram_mb: u64,
    pub dedicated: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Hardware {
    pub ram_mb: u64,
    pub cpu_cores: u32,
    pub gpus: Vec<Gpu>,
    pub npu: bool,
    /// The installed speech worker can run models on a GPU. Without it a
    /// dedicated GPU does not make large models usable (they run on CPU).
    #[serde(default)]
    pub gpu_inference: bool,
}

const CJK: &[&str] = &["zh", "ja", "ko", "yue"];

/// Rules of AC-002, each with a preference list so the recommendation still
/// works when a preferred family is not in the (verified) catalog.
pub fn recommend(catalog: &Catalog, hw: &Hardware, ui_lang: &str) -> Option<String> {
    let lang = ui_lang
        .split(['-', '_'])
        .next()
        .unwrap_or(ui_lang)
        .to_lowercase();
    let big_gpu = hw.gpu_inference && hw.gpus.iter().any(|g| g.dedicated && g.vram_mb >= 6144);
    let prefs: &[&str] = if big_gpu {
        &[
            "whisper-turbo",
            "whisper-turbo-q5",
            "parakeet-tdt-0.6b-v3",
            "whisper-small",
        ]
    } else if CJK.contains(&lang.as_str()) {
        &["sense-voice-int8", "whisper-small", "whisper-base"]
    } else if lang == "en" && hw.ram_mb < 8192 {
        &[
            "moonshine-small-streaming-en",
            "parakeet-tdt-0.6b-v2",
            "whisper-base",
        ]
    } else {
        &["parakeet-tdt-0.6b-v3", "whisper-small", "whisper-base"]
    };
    prefs
        .iter()
        .filter_map(|id| catalog.get(id))
        .find(|m| m.supports(&lang) && (hw.ram_mb == 0 || m.min_ram_mb <= hw.ram_mb))
        .or_else(|| catalog.models.iter().find(|m| m.supports(&lang)))
        .map(|m| m.id.clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builtin_catalog_is_complete_and_verified() {
        let c = Catalog::builtin();
        assert!(c.models.len() >= 6);
        for m in &c.models {
            assert!(!m.files.is_empty(), "{}", m.id);
            for f in &m.files {
                assert_eq!(f.sha256.len(), 64, "{} {}", m.id, f.path);
                assert!(f.sha256.chars().all(|c| c.is_ascii_hexdigit()));
                assert!(f.url.starts_with("https://"));
                assert!(f.size > 0);
            }
            assert!(!m.license.is_empty());
        }
        assert!(c.get("parakeet-tdt-0.6b-v3").unwrap().supports("pt-BR"));
        assert!(!c.get("parakeet-tdt-0.6b-v2").unwrap().supports("pt"));
    }

    fn hw(ram: u64, gpu_vram: Option<u64>) -> Hardware {
        Hardware {
            ram_mb: ram,
            cpu_cores: 8,
            gpus: gpu_vram
                .map(|v| {
                    vec![Gpu {
                        name: "RTX".into(),
                        vendor: "NVIDIA".into(),
                        vram_mb: v,
                        dedicated: true,
                    }]
                })
                .unwrap_or_default(),
            npu: false,
            gpu_inference: true,
        }
    }

    /// Catalog with every family named in the spec, to prove the rules.
    fn full_catalog() -> Catalog {
        let mut c = Catalog::builtin();
        for (id, langs) in [
            ("moonshine-small-streaming-en", vec!["en"]),
            ("sense-voice-int8", vec!["zh", "en", "ja", "ko", "yue"]),
        ] {
            let mut m = c.models[0].clone();
            m.id = id.into();
            m.languages = langs.into_iter().map(String::from).collect();
            m.min_ram_mb = 512;
            c.models.push(m);
        }
        c
    }

    #[test]
    fn recommendation_rules_of_ac_002() {
        let c = full_catalog();
        assert_eq!(
            recommend(&c, &hw(16384, None), "pt-BR").as_deref(),
            Some("parakeet-tdt-0.6b-v3")
        );
        assert_eq!(
            recommend(&c, &hw(16384, Some(8192)), "pt-BR").as_deref(),
            Some("whisper-turbo")
        );
        assert_eq!(
            recommend(&c, &hw(6144, None), "en").as_deref(),
            Some("moonshine-small-streaming-en")
        );
        assert_eq!(
            recommend(&c, &hw(8192, None), "ja").as_deref(),
            Some("sense-voice-int8")
        );
        assert_eq!(
            recommend(&c, &hw(8192, None), "tr").as_deref(),
            Some("whisper-small")
        );
    }

    #[test]
    fn dedicated_gpu_without_gpu_inference_keeps_the_cpu_recommendation() {
        let c = full_catalog();
        let mut cpu_only = hw(16384, Some(8192));
        cpu_only.gpu_inference = false;
        assert_eq!(
            recommend(&c, &cpu_only, "pt-BR").as_deref(),
            Some("parakeet-tdt-0.6b-v3")
        );
        assert_eq!(
            recommend(&c, &cpu_only, "en").as_deref(),
            Some("parakeet-tdt-0.6b-v3")
        );
    }

    #[test]
    fn builtin_catalog_falls_back_when_a_family_is_missing() {
        let c = Catalog::builtin();
        assert_eq!(
            recommend(&c, &hw(6144, None), "en").as_deref(),
            Some("parakeet-tdt-0.6b-v2")
        );
        assert_eq!(
            recommend(&c, &hw(8192, None), "ja").as_deref(),
            Some("whisper-small")
        );
    }
}
