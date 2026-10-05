//! BYOK provider registry (003 TK-001). Credentials never live here (OT-001):
//! they are written to the credential store and read per request.

use aura_core::Secret;
use aura_core::credentials::{CredentialStore, target};
use aura_store::{Store, StoreError, now_secs};
use rusqlite::{OptionalExtension, params};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Wire {
    Responses,
    Chat,
    Anthropic,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AuthStyle {
    Bearer,
    ApiKeyHeader,
    XApiKey,
    None,
}

impl AuthStyle {
    fn parse(s: &str) -> Self {
        match s {
            "api-key" => AuthStyle::ApiKeyHeader,
            "x-api-key" => AuthStyle::XApiKey,
            "none" => AuthStyle::None,
            _ => AuthStyle::Bearer,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct Quirks {
    #[serde(default, alias = "noParallelToolCalls")]
    pub no_parallel_tool_calls: bool,
    #[serde(default, alias = "reasoningEffort")]
    pub reasoning_effort: bool,
    #[serde(default, alias = "noStreamOptions")]
    pub no_stream_options: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct Preset {
    pub id: String,
    pub name: String,
    pub wire: Wire,
    pub base_url: String,
    pub auth: String,
    pub credential_required: bool,
    #[serde(default)]
    pub headers: BTreeMap<String, String>,
    #[serde(default)]
    pub quirks: Quirks,
    #[serde(default)]
    pub transcription_models: Vec<String>,
    #[serde(default)]
    pub tts: bool,
}

#[derive(Deserialize)]
struct PresetFile {
    preset: Vec<Preset>,
}

pub fn presets() -> Vec<Preset> {
    toml::from_str::<PresetFile>(include_str!("../presets.toml"))
        .expect("valid presets.toml")
        .preset
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelSpec {
    pub id: String,
    pub display_name: Option<String>,
    pub context_window: Option<u32>,
    pub max_output: Option<u32>,
    pub supports_images: bool,
    pub supports_tools: bool,
    pub supports_reasoning: bool,
    /// True when capabilities were guessed from the name.
    pub estimated: bool,
    /// Entered or corrected by the user (003 AC-013); discovery keeps it.
    #[serde(default)]
    pub manual: bool,
    /// Reasoning efforts the model accepts, lowest first (013); empty =
    /// low/medium/high when it reasons.
    #[serde(default)]
    pub efforts: Vec<String>,
    /// Effort used when the user picks none.
    #[serde(default)]
    pub default_effort: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ProviderStatus {
    Unverified,
    Verified,
    Error,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Provider {
    pub id: String,
    pub name: String,
    pub preset: String,
    pub wire: Wire,
    pub base_url: String,
    pub auth: AuthStyle,
    pub extra_headers: BTreeMap<String, String>,
    pub models: Vec<ModelSpec>,
    /// `••••abcd` when a credential exists.
    pub credential_hint: Option<String>,
    pub status: ProviderStatus,
    pub last_error: Option<String>,
    pub quirks: Quirks,
}

/// What the UI sends. The credential goes straight to the credential store.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderDraft {
    pub id: Option<String>,
    pub name: String,
    pub preset: String,
    pub wire: Option<Wire>,
    pub base_url: Option<String>,
    #[serde(default)]
    pub extra_headers: BTreeMap<String, String>,
    #[serde(skip)]
    pub credential: Option<Secret<String>>,
}

/// Everything an upstream needs at request time.
#[derive(Clone)]
pub struct ProviderTarget {
    pub base_url: String,
    pub auth: AuthStyle,
    pub headers: Vec<(String, String)>,
    pub query: Vec<(String, String)>,
    pub credential: Option<Secret<String>>,
}

#[derive(Debug, Error)]
pub enum RegistryError {
    #[error("invalid field: {0}")]
    Invalid(&'static str),
    #[error("storage: {0}")]
    Storage(#[from] StoreError),
    #[error("credential store: {0}")]
    Credential(#[from] aura_core::credentials::CredentialError),
    #[error("provider not found")]
    NotFound,
}

impl From<rusqlite::Error> for RegistryError {
    fn from(e: rusqlite::Error) -> Self {
        RegistryError::Storage(StoreError::Sqlite(e))
    }
}

pub fn credential_target(provider_id: &str) -> String {
    target("provider", provider_id)
}

fn hint(secret: &str) -> String {
    let tail: String = secret
        .chars()
        .rev()
        .take(4)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect();
    format!("••••{tail}")
}

fn slug(name: &str) -> String {
    let s: String = name
        .to_lowercase()
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|p| !p.is_empty())
        .collect::<Vec<_>>()
        .join("-");
    let mut suffix = [0u8; 3];
    let _ = getrandom::fill(&mut suffix);
    format!(
        "{}-{}",
        if s.is_empty() { "provider".into() } else { s },
        hex::encode(suffix)
    )
}

pub struct ProviderRegistry<'a> {
    store: &'a Store,
    creds: &'a dyn CredentialStore,
}

impl<'a> ProviderRegistry<'a> {
    pub fn new(store: &'a Store, creds: &'a dyn CredentialStore) -> Self {
        Self { store, creds }
    }

    pub fn upsert(&self, draft: ProviderDraft) -> Result<Provider, RegistryError> {
        let presets = presets();
        let preset = presets.iter().find(|p| p.id == draft.preset);
        let wire = draft
            .wire
            .or(preset.map(|p| p.wire))
            .ok_or(RegistryError::Invalid("wire"))?;
        let base_url = draft
            .base_url
            .clone()
            .filter(|u| !u.trim().is_empty())
            .or(preset.map(|p| p.base_url.clone()))
            .filter(|u| u.starts_with("http://") || u.starts_with("https://"))
            .ok_or(RegistryError::Invalid("baseUrl"))?;
        if draft.name.trim().is_empty() {
            return Err(RegistryError::Invalid("name"));
        }
        let id = draft.id.clone().unwrap_or_else(|| slug(&draft.name));
        let existing = self.get(&id)?;
        // Header names are RFC 7230 tokens; values are single-line.
        let valid_header = |k: &str, v: &str| {
            !k.is_empty()
                && k.bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b"!#$%&'*+-.^_|~".contains(&b))
                && !v.contains(['\r', '\n'])
        };
        if !draft.extra_headers.iter().all(|(k, v)| valid_header(k, v)) {
            return Err(RegistryError::Invalid("extraHeaders"));
        }
        let mut headers = preset.map(|p| p.headers.clone()).unwrap_or_default();
        headers.extend(draft.extra_headers.clone());
        let credential_hint = match &draft.credential {
            Some(secret) if !secret.expose().is_empty() => {
                self.creds.put(&credential_target(&id), secret)?;
                Some(hint(secret.expose()))
            }
            _ => existing.as_ref().and_then(|e| e.credential_hint.clone()),
        };
        let provider = Provider {
            id: id.clone(),
            name: draft.name.trim().to_string(),
            preset: draft.preset.clone(),
            wire,
            base_url: base_url.trim_end_matches('/').to_string(),
            // A custom endpoint in the Anthropic format authenticates like Anthropic.
            auth: if draft.preset == "custom" && wire == Wire::Anthropic {
                AuthStyle::XApiKey
            } else {
                AuthStyle::parse(preset.map(|p| p.auth.as_str()).unwrap_or("bearer"))
            },
            extra_headers: headers,
            models: existing
                .as_ref()
                .map(|e| e.models.clone())
                .unwrap_or_default(),
            credential_hint,
            status: ProviderStatus::Unverified,
            last_error: None,
            quirks: preset.map(|p| p.quirks.clone()).unwrap_or_default(),
        };
        self.save(&provider)?;
        Ok(provider)
    }

    fn save(&self, p: &Provider) -> Result<(), RegistryError> {
        let extra =
            serde_json::json!({"headers": p.extra_headers, "auth": p.auth, "quirks": p.quirks});
        self.store.with_conn(|c| {
            c.execute(
                "INSERT INTO providers(id, name, preset, wire, base_url, extra_headers_json, models_json, credential_hint, status, last_error, updated_at)
                 VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11)
                 ON CONFLICT(id) DO UPDATE SET name=excluded.name, preset=excluded.preset, wire=excluded.wire, base_url=excluded.base_url,
                   extra_headers_json=excluded.extra_headers_json, models_json=excluded.models_json, credential_hint=excluded.credential_hint,
                   status=excluded.status, last_error=excluded.last_error, updated_at=excluded.updated_at",
                params![
                    p.id,
                    p.name,
                    p.preset,
                    serde_json::to_string(&p.wire)?.trim_matches('"'),
                    p.base_url,
                    extra.to_string(),
                    serde_json::to_string(&p.models)?,
                    p.credential_hint,
                    serde_json::to_string(&p.status)?.trim_matches('"'),
                    p.last_error,
                    now_secs()
                ],
            )?;
            Ok(())
        })?;
        Ok(())
    }

    fn row(r: &rusqlite::Row<'_>) -> rusqlite::Result<Provider> {
        let extra: serde_json::Value =
            serde_json::from_str(&r.get::<_, String>(5)?).unwrap_or_default();
        let wire: String = r.get(3)?;
        let status: String = r.get(8)?;
        Ok(Provider {
            id: r.get(0)?,
            name: r.get(1)?,
            preset: r.get(2)?,
            wire: serde_json::from_value(serde_json::Value::String(wire)).unwrap_or(Wire::Chat),
            base_url: r.get(4)?,
            auth: serde_json::from_value(extra["auth"].clone()).unwrap_or(AuthStyle::Bearer),
            extra_headers: serde_json::from_value(extra["headers"].clone()).unwrap_or_default(),
            models: serde_json::from_str(&r.get::<_, String>(6)?).unwrap_or_default(),
            credential_hint: r.get(7)?,
            status: serde_json::from_value(serde_json::Value::String(status))
                .unwrap_or(ProviderStatus::Unverified),
            last_error: r.get(9)?,
            quirks: serde_json::from_value(extra["quirks"].clone()).unwrap_or_default(),
        })
    }

    pub fn list(&self) -> Result<Vec<Provider>, RegistryError> {
        Ok(self.store.with_conn(|c| {
            let mut stmt = c.prepare(
                "SELECT id, name, preset, wire, base_url, extra_headers_json, models_json, credential_hint, status, last_error FROM providers ORDER BY name",
            )?;
            let rows = stmt.query_map([], Self::row)?;
            Ok(rows.filter_map(|r| r.ok()).collect())
        })?)
    }

    pub fn get(&self, id: &str) -> Result<Option<Provider>, RegistryError> {
        Ok(self.store.with_conn(|c| {
            Ok(c.query_row(
                "SELECT id, name, preset, wire, base_url, extra_headers_json, models_json, credential_hint, status, last_error FROM providers WHERE id = ?1",
                [id],
                Self::row,
            )
            .optional()?)
        })?)
    }

    pub fn remove(&self, id: &str) -> Result<(), RegistryError> {
        self.creds.delete(&credential_target(id))?;
        self.store.with_conn(|c| {
            c.execute("DELETE FROM providers WHERE id = ?1", [id])?;
            Ok(())
        })?;
        Ok(())
    }

    pub fn set_status(
        &self,
        id: &str,
        status: ProviderStatus,
        error: Option<String>,
    ) -> Result<(), RegistryError> {
        let mut p = self.get(id)?.ok_or(RegistryError::NotFound)?;
        p.status = status;
        p.last_error = error;
        self.save(&p)
    }

    /// Stores discovered models; user-entered ones survive and win on the same id.
    pub fn set_models(&self, id: &str, models: Vec<ModelSpec>) -> Result<(), RegistryError> {
        let mut p = self.get(id)?.ok_or(RegistryError::NotFound)?;
        let manual: Vec<ModelSpec> = p.models.iter().filter(|m| m.manual).cloned().collect();
        let mut merged: Vec<ModelSpec> = models
            .into_iter()
            .filter(|m| !manual.iter().any(|x| x.id == m.id))
            .collect();
        merged.extend(manual);
        p.models = merged;
        self.save(&p)
    }

    /// Adds or corrects one model by hand (providers without /models).
    pub fn save_model(&self, id: &str, mut spec: ModelSpec) -> Result<Provider, RegistryError> {
        spec.id = spec.id.trim().to_string();
        if spec.id.is_empty()
            || spec.id.chars().count() > 200
            || spec.id.chars().any(char::is_control)
        {
            return Err(RegistryError::Invalid("modelId"));
        }
        spec.display_name = spec
            .display_name
            .map(|n| n.trim().to_string())
            .filter(|n| !n.is_empty());
        spec.manual = true;
        spec.estimated = false;
        // Efforts from the known set, in its order, without repeats (013).
        if spec
            .efforts
            .iter()
            .any(|e| !aura_core::model_catalog::is_effort(e))
        {
            return Err(RegistryError::Invalid("efforts"));
        }
        spec.efforts = aura_core::model_catalog::EFFORTS
            .iter()
            .filter(|e| spec.efforts.iter().any(|x| x == *e))
            .map(|e| e.to_string())
            .collect();
        if let Some(d) = &spec.default_effort
            && !spec.efforts.contains(d)
        {
            return Err(RegistryError::Invalid("defaultEffort"));
        }
        let mut p = self.get(id)?.ok_or(RegistryError::NotFound)?;
        match p.models.iter_mut().find(|m| m.id == spec.id) {
            Some(existing) => *existing = spec,
            None => p.models.push(spec),
        }
        self.save(&p)?;
        Ok(p)
    }

    pub fn remove_model(&self, id: &str, model_id: &str) -> Result<Provider, RegistryError> {
        let mut p = self.get(id)?.ok_or(RegistryError::NotFound)?;
        p.models.retain(|m| m.id != model_id);
        self.save(&p)?;
        Ok(p)
    }

    /// Resolves the request target with the credential read now.
    pub fn target(&self, p: &Provider) -> Result<ProviderTarget, RegistryError> {
        let mut query = Vec::new();
        if p.preset == "azure-openai" && !p.base_url.contains("/openai/v1") {
            query.push(("api-version".to_string(), "preview".to_string()));
        }
        Ok(ProviderTarget {
            base_url: p.base_url.clone(),
            auth: p.auth,
            headers: p
                .extra_headers
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect(),
            query,
            credential: self.creds.get(&credential_target(&p.id))?,
        })
    }
}

#[cfg(test)]
mod quirks_tests {
    use super::Quirks;

    #[test]
    fn quirks_round_trip_through_camel_case() {
        let q = Quirks {
            no_parallel_tool_calls: true,
            reasoning_effort: false,
            no_stream_options: true,
        };
        let v = serde_json::to_value(&q).unwrap();
        assert_eq!(v["noParallelToolCalls"], true);
        assert_eq!(serde_json::from_value::<Quirks>(v).unwrap(), q);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aura_core::credentials::MemoryCredentialStore;

    #[test]
    fn presets_parse_and_cover_the_spec_list() {
        let ids: Vec<String> = presets().into_iter().map(|p| p.id).collect();
        for id in [
            "openai",
            "azure-openai",
            "openrouter",
            "anthropic",
            "gemini",
            "groq",
            "deepseek",
            "mistral",
            "xai",
            "together",
            "ollama",
            "lmstudio",
            "vllm",
            "custom",
        ] {
            assert!(ids.contains(&id.to_string()), "missing preset {id}");
        }
    }

    #[test]
    fn upsert_keeps_credential_out_of_the_database() {
        let store = Store::open_in_memory().unwrap();
        let creds = MemoryCredentialStore::new();
        let reg = ProviderRegistry::new(&store, &creds);
        let p = reg
            .upsert(ProviderDraft {
                id: None,
                name: "OpenRouter".into(),
                preset: "openrouter".into(),
                wire: None,
                base_url: None,
                extra_headers: Default::default(),
                credential: Some(Secret::from("sk-or-v1-secretabcd")),
            })
            .unwrap();
        assert_eq!(p.wire, Wire::Responses);
        assert_eq!(p.base_url, "https://openrouter.ai/api/v1");
        assert_eq!(p.credential_hint.as_deref(), Some("••••abcd"));
        let dump: String = store
            .with_conn(|c| {
                Ok(c.query_row("SELECT * FROM providers", [], |r| {
                    Ok((0..11)
                        .map(|i| {
                            r.get::<_, Option<String>>(i)
                                .ok()
                                .flatten()
                                .unwrap_or_default()
                        })
                        .collect::<Vec<_>>()
                        .join("|"))
                })?)
            })
            .unwrap();
        assert!(!dump.contains("secret"));
        let target = reg.target(&p).unwrap();
        assert_eq!(target.credential.unwrap().expose(), "sk-or-v1-secretabcd");
        assert!(target.headers.iter().any(|(k, _)| k == "X-Title"));
        reg.remove(&p.id).unwrap();
        assert!(creds.get(&credential_target(&p.id)).unwrap().is_none());
        assert!(reg.list().unwrap().is_empty());
    }

    #[test]
    fn manual_models_survive_discovery_and_override_guesses() {
        let store = Store::open_in_memory().unwrap();
        let creds = MemoryCredentialStore::new();
        let reg = ProviderRegistry::new(&store, &creds);
        let p = reg
            .upsert(ProviderDraft {
                id: Some("local".into()),
                name: "Local".into(),
                preset: "custom".into(),
                wire: Some(Wire::Chat),
                base_url: Some("http://127.0.0.1:9/v1".into()),
                extra_headers: Default::default(),
                credential: None,
            })
            .unwrap();
        let spec = |id: &str, images: bool| ModelSpec {
            id: id.into(),
            display_name: None,
            context_window: None,
            max_output: None,
            supports_images: images,
            supports_tools: false,
            supports_reasoning: false,
            estimated: true,
            manual: false,
            efforts: vec![],
            default_effort: None,
        };
        let saved = reg
            .save_model(
                &p.id,
                ModelSpec {
                    display_name: Some(" Meu modelo ".into()),
                    ..spec(" my-model:7b ", true)
                },
            )
            .unwrap();
        let m = saved.models.iter().find(|m| m.id == "my-model:7b").unwrap();
        assert!(m.manual && !m.estimated && m.supports_images);
        assert_eq!(m.display_name.as_deref(), Some("Meu modelo"));
        // Discovery returns a guess for the same id plus another model.
        reg.set_models(
            &p.id,
            vec![spec("my-model:7b", false), spec("other", false)],
        )
        .unwrap();
        let models = reg.get(&p.id).unwrap().unwrap().models;
        assert_eq!(models.len(), 2);
        assert!(
            models
                .iter()
                .find(|m| m.id == "my-model:7b")
                .unwrap()
                .supports_images
        );
        assert!(matches!(
            reg.save_model(&p.id, spec("  ", false)),
            Err(RegistryError::Invalid("modelId"))
        ));
        // 013 AC-003: efforts accepted by a manual model, ordered, with a
        // default among them.
        let saved = reg
            .save_model(
                &p.id,
                ModelSpec {
                    supports_reasoning: true,
                    efforts: vec!["max".into(), "low".into(), "high".into(), "low".into()],
                    default_effort: Some("high".into()),
                    ..spec("qa-reasoner", false)
                },
            )
            .unwrap();
        let m = saved.models.iter().find(|m| m.id == "qa-reasoner").unwrap();
        assert_eq!(m.efforts, ["low", "high", "max"]);
        assert_eq!(m.default_effort.as_deref(), Some("high"));
        assert!(matches!(
            reg.save_model(
                &p.id,
                ModelSpec {
                    efforts: vec!["ultra".into()],
                    ..spec("x", false)
                }
            ),
            Err(RegistryError::Invalid("efforts"))
        ));
        assert!(matches!(
            reg.save_model(
                &p.id,
                ModelSpec {
                    efforts: vec!["low".into()],
                    default_effort: Some("high".into()),
                    ..spec("x", false)
                }
            ),
            Err(RegistryError::Invalid("defaultEffort"))
        ));
        reg.remove_model(&p.id, "qa-reasoner").unwrap();
        let left = reg.remove_model(&p.id, "my-model:7b").unwrap();
        assert_eq!(
            left.models
                .iter()
                .map(|m| m.id.as_str())
                .collect::<Vec<_>>(),
            ["other"]
        );
    }

    #[test]
    fn custom_provider_format_drives_auth_and_headers_are_validated() {
        let store = Store::open_in_memory().unwrap();
        let creds = MemoryCredentialStore::new();
        let reg = ProviderRegistry::new(&store, &creds);
        let draft = |wire: Wire, headers: &[(&str, &str)]| ProviderDraft {
            id: Some("meu".into()),
            name: "Meu".into(),
            preset: "custom".into(),
            wire: Some(wire),
            base_url: Some("https://llm.example/v1".into()),
            extra_headers: headers
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            credential: None,
        };
        let anthropic = reg
            .upsert(draft(Wire::Anthropic, &[("X-Tenant", "alpha")]))
            .unwrap();
        assert_eq!(anthropic.auth, AuthStyle::XApiKey);
        assert_eq!(
            anthropic.extra_headers.get("X-Tenant").map(String::as_str),
            Some("alpha")
        );
        assert_eq!(
            reg.upsert(draft(Wire::Chat, &[])).unwrap().auth,
            AuthStyle::Bearer
        );
        assert_eq!(
            reg.upsert(draft(Wire::Responses, &[])).unwrap().auth,
            AuthStyle::Bearer
        );
        for bad in ["", "X Tenant", "X:Tenant", "Ünicode"] {
            assert!(
                matches!(
                    reg.upsert(draft(Wire::Chat, &[(bad, "v")])),
                    Err(RegistryError::Invalid("extraHeaders"))
                ),
                "{bad:?}"
            );
        }
        assert!(matches!(
            reg.upsert(draft(Wire::Chat, &[("X-Ok", "line\nbreak")])),
            Err(RegistryError::Invalid("extraHeaders"))
        ));
    }

    #[test]
    fn custom_provider_requires_valid_url() {
        let store = Store::open_in_memory().unwrap();
        let creds = MemoryCredentialStore::new();
        let reg = ProviderRegistry::new(&store, &creds);
        let draft = |url: &str| ProviderDraft {
            id: None,
            name: "Meu".into(),
            preset: "custom".into(),
            wire: Some(Wire::Chat),
            base_url: Some(url.into()),
            extra_headers: [("X-Team".to_string(), "a".to_string())].into(),
            credential: None,
        };
        assert!(matches!(
            reg.upsert(draft("")),
            Err(RegistryError::Invalid("baseUrl"))
        ));
        let p = reg.upsert(draft("http://localhost:8000/v1/")).unwrap();
        assert_eq!(p.base_url, "http://localhost:8000/v1");
        assert_eq!(p.extra_headers["X-Team"], "a");
        assert!(p.credential_hint.is_none());
    }
}
