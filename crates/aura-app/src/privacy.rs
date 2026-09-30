//! Persistence of the privacy policy (modes, agent permission, exclusions,
//! pause), per-conversation grants and the access log (004 / 005).

use aura_policy::{
    AccessRequest, Decision, ExclusionRule, Grants, Policy, Requester, Source, SourcePolicy,
    default_exclusions,
};
use aura_store::settings_repo::SettingsRepo;
use aura_store::{Store, StoreError, now_secs};
use rusqlite::params;
use serde::{Deserialize, Serialize};

const POLICY_KEY: &str = "privacy.policy";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct StoredPolicy {
    screen: SourcePolicy,
    mic: SourcePolicy,
    system_audio: SourcePolicy,
    paused: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccessLogEntry {
    pub at: i64,
    pub source: String,
    pub requester: String,
    pub tool: Option<String>,
    pub conversation: Option<String>,
    pub decision: String,
    pub reason: Option<String>,
}

#[derive(Clone)]
pub struct PrivacyRepo {
    store: Store,
}

fn source_key(s: Source) -> &'static str {
    match s {
        Source::Screen => "screen",
        Source::Mic => "mic",
        Source::SystemAudio => "systemAudio",
        Source::Selection => "selection",
    }
}

fn source_from(s: &str) -> Option<Source> {
    Some(match s {
        "screen" => Source::Screen,
        "mic" => Source::Mic,
        "systemAudio" => Source::SystemAudio,
        "selection" => Source::Selection,
        _ => return None,
    })
}

impl PrivacyRepo {
    pub fn new(store: Store) -> Self {
        Self { store }
    }

    /// Loads the policy; first run seeds the built-in exclusions.
    pub fn load(&self) -> Result<Policy, StoreError> {
        let defaults = Policy::default();
        let stored: Option<StoredPolicy> = SettingsRepo::new(&self.store)
            .get_raw(POLICY_KEY)?
            .and_then(|v| serde_json::from_value(v).ok());
        let exclusions = self.exclusions()?;
        let exclusions = if exclusions.is_empty() {
            for r in default_exclusions() {
                self.upsert_exclusion(&r)?;
            }
            default_exclusions()
        } else {
            exclusions
        };
        Ok(match stored {
            Some(s) => Policy {
                screen: s.screen,
                mic: s.mic,
                system_audio: s.system_audio,
                exclusions,
                paused: s.paused,
            },
            None => Policy {
                exclusions,
                ..defaults
            },
        })
    }

    pub fn save(&self, policy: &Policy) -> Result<(), StoreError> {
        let stored = StoredPolicy {
            screen: policy.screen,
            mic: policy.mic,
            system_audio: policy.system_audio,
            paused: policy.paused,
        };
        SettingsRepo::new(&self.store)
            .set_raw(POLICY_KEY, &serde_json::to_value(stored).expect("json"))
    }

    pub fn exclusions(&self) -> Result<Vec<ExclusionRule>, StoreError> {
        self.store.with_conn(|c| {
            let mut st = c.prepare("SELECT id, process, title_glob, class, enabled, builtin FROM exclusion_rules ORDER BY builtin DESC, id")?;
            let rows = st
                .query_map([], |r| {
                    Ok(ExclusionRule {
                        id: r.get(0)?,
                        process: r.get(1)?,
                        title_glob: r.get(2)?,
                        class: r.get(3)?,
                        enabled: r.get::<_, i64>(4)? == 1,
                        builtin: r.get::<_, i64>(5)? == 1,
                    })
                })?
                .collect::<Result<Vec<_>, _>>()?;
            Ok(rows)
        })
    }

    pub fn upsert_exclusion(&self, r: &ExclusionRule) -> Result<(), StoreError> {
        self.store.with_conn(|c| {
            c.execute(
                "INSERT INTO exclusion_rules(id, process, title_glob, class, enabled, builtin) VALUES (?1, ?2, ?3, ?4, ?5, ?6)
                 ON CONFLICT(id) DO UPDATE SET process = excluded.process, title_glob = excluded.title_glob,
                   class = excluded.class, enabled = excluded.enabled",
                params![r.id, r.process, r.title_glob, r.class, r.enabled as i64, r.builtin as i64],
            )?;
            Ok(())
        })
    }

    /// Built-in rules can only be disabled, not deleted.
    pub fn remove_exclusion(&self, id: &str) -> Result<bool, StoreError> {
        self.store.with_conn(|c| {
            Ok(c.execute(
                "DELETE FROM exclusion_rules WHERE id = ?1 AND builtin = 0",
                params![id],
            )? > 0)
        })
    }

    pub fn grants(&self) -> Result<Grants, StoreError> {
        self.store.with_conn(|c| {
            let mut st = c.prepare("SELECT conversation, source FROM agent_grants")?;
            let rows = st
                .query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))?
                .collect::<Result<Vec<_>, _>>()?;
            let mut g = Grants::default();
            for (conv, src) in rows {
                if let Some(s) = source_from(&src) {
                    g.grant(&conv, s);
                }
            }
            Ok(g)
        })
    }

    pub fn grant(&self, conversation: &str, source: Source) -> Result<(), StoreError> {
        self.store.with_conn(|c| {
            c.execute(
                "INSERT OR REPLACE INTO agent_grants(conversation, source, granted_at) VALUES (?1, ?2, ?3)",
                params![conversation, source_key(source), now_secs()],
            )?;
            Ok(())
        })
    }

    pub fn revoke_conversation(&self, conversation: &str) -> Result<(), StoreError> {
        self.store.with_conn(|c| {
            Ok(c.execute(
                "DELETE FROM agent_grants WHERE conversation = ?1",
                params![conversation],
            )
            .map(|_| ())?)
        })
    }

    pub fn log(&self, req: &AccessRequest, decision: &Decision) -> Result<(), StoreError> {
        let (requester, tool, conversation) = match &req.requester {
            Requester::User => ("user".to_string(), None, None),
            Requester::Agent { tool, conversation } => (
                "agent".to_string(),
                Some(tool.clone()),
                Some(conversation.clone()),
            ),
        };
        let (decision, reason) = match decision {
            Decision::Allow => ("allow", None),
            Decision::AllowRedacted { rects } => (
                "allowRedacted",
                Some(format!("{} área(s) cobertas", rects.len())),
            ),
            Decision::Ask => ("ask", None),
            Decision::Deny { reason } => ("deny", Some(reason.code().to_string())),
        };
        self.store.with_conn(|c| {
            c.execute(
                "INSERT INTO access_log(at, source, requester, tool, conversation, decision, reason) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![now_secs(), source_key(req.source), requester, tool, conversation, decision, reason],
            )?;
            // Keep the log bounded (last 5000 entries).
            c.execute("DELETE FROM access_log WHERE id <= (SELECT MAX(id) - 5000 FROM access_log)", [])?;
            Ok(())
        })
    }

    pub fn access_log(&self, limit: u32) -> Result<Vec<AccessLogEntry>, StoreError> {
        self.store.with_conn(|c| {
            let mut st = c.prepare(
                "SELECT at, source, requester, tool, conversation, decision, reason FROM access_log ORDER BY id DESC LIMIT ?1",
            )?;
            let rows = st
                .query_map(params![limit], |r| {
                    Ok(AccessLogEntry {
                        at: r.get(0)?,
                        source: r.get(1)?,
                        requester: r.get(2)?,
                        tool: r.get(3)?,
                        conversation: r.get(4)?,
                        decision: r.get(5)?,
                        reason: r.get(6)?,
                    })
                })?
                .collect::<Result<Vec<_>, _>>()?;
            Ok(rows)
        })
    }

    pub fn clear_access_log(&self) -> Result<(), StoreError> {
        self.store
            .with_conn(|c| Ok(c.execute("DELETE FROM access_log", []).map(|_| ())?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aura_policy::{AgentPermission, CaptureMode, Target};

    #[test]
    fn policy_round_trip_with_seeded_exclusions() {
        let repo = PrivacyRepo::new(Store::open_in_memory().unwrap());
        let mut p = repo.load().unwrap();
        assert!(p.exclusions.iter().any(|r| r.id == "keepass" && r.builtin));
        p.screen = SourcePolicy {
            mode: CaptureMode::RecentBuffer { minutes: 10 },
            agent: AgentPermission::Always,
        };
        p.paused = true;
        repo.save(&p).unwrap();
        let again = repo.load().unwrap();
        assert_eq!(again.screen, p.screen);
        assert!(again.paused);
        assert!(!repo.remove_exclusion("keepass").unwrap());
        let custom = ExclusionRule {
            id: "bank".into(),
            process: None,
            title_glob: Some("*Banco*".into()),
            class: None,
            enabled: true,
            builtin: false,
        };
        repo.upsert_exclusion(&custom).unwrap();
        assert!(repo.remove_exclusion("bank").unwrap());
    }

    #[test]
    fn grants_and_log() {
        let repo = PrivacyRepo::new(Store::open_in_memory().unwrap());
        repo.grant("c1", Source::Screen).unwrap();
        let g = repo.grants().unwrap();
        assert!(g.allows("c1", Source::Screen));
        repo.revoke_conversation("c1").unwrap();
        assert!(!repo.grants().unwrap().allows("c1", Source::Screen));
        let req = AccessRequest {
            source: Source::Screen,
            requester: Requester::Agent {
                tool: "screen_capture".into(),
                conversation: "c1".into(),
            },
            target: Target::Range,
            visible_windows: vec![],
            background: false,
        };
        repo.log(&req, &Decision::Ask).unwrap();
        let log = repo.access_log(10).unwrap();
        assert_eq!(log[0].decision, "ask");
        assert_eq!(log[0].tool.as_deref(), Some("screen_capture"));
    }
}
