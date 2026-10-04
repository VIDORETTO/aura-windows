//! Privacy policy engine (`specs/004-contexto-de-tela`, TK-003).
//!
//! [`decide`] is deterministic and does no I/O (OT-002): the host feeds it the
//! current [`Policy`], session [`Grants`] and a description of the request, and
//! must honour the returned [`Decision`] *before* any pixel or sample leaves
//! the capture module (OT-001).

use aura_core::placement::Rect;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

pub mod wildcard;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Source {
    Screen,
    Mic,
    SystemAudio,
    /// Text selected in the previous app; follows the screen exclusions.
    Selection,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "type",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum CaptureMode {
    Off,
    OnDemand,
    RecentBuffer { minutes: u32 },
    Manual,
    Continuous,
}

impl CaptureMode {
    /// Whether the source records in the background in this mode.
    pub fn records_in_background(&self) -> bool {
        matches!(
            self,
            CaptureMode::RecentBuffer { .. } | CaptureMode::Continuous
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum AgentPermission {
    Never,
    #[default]
    Ask,
    Always,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SourcePolicy {
    pub mode: CaptureMode,
    pub agent: AgentPermission,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExclusionRule {
    pub id: String,
    /// Executable name pattern (`*` and `?` wildcards, case-insensitive).
    pub process: Option<String>,
    pub title_glob: Option<String>,
    pub class: Option<String>,
    pub enabled: bool,
    pub builtin: bool,
}

impl ExclusionRule {
    pub fn matches(&self, w: &WindowInfo) -> bool {
        if !self.enabled {
            return false;
        }
        if self.process.is_none() && self.title_glob.is_none() && self.class.is_none() {
            return false;
        }
        let p = self
            .process
            .as_deref()
            .is_none_or(|g| wildcard::matches(g, &w.process));
        let t = self
            .title_glob
            .as_deref()
            .is_none_or(|g| wildcard::matches(g, &w.title));
        let c = self
            .class
            .as_deref()
            .is_none_or(|g| wildcard::matches(g, &w.class));
        p && t && c
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Policy {
    pub screen: SourcePolicy,
    pub mic: SourcePolicy,
    pub system_audio: SourcePolicy,
    pub exclusions: Vec<ExclusionRule>,
    pub paused: bool,
    /// Limits for continuous/manual recordings (004 AC-016, 005 AC-006).
    #[serde(default)]
    pub retention: Retention,
}

/// User-facing retention limits; applied whenever a segment is written.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Retention {
    pub days: u32,
    pub max_gb: u32,
    /// Manual recordings also follow the limits.
    pub apply_to_manual: bool,
}

impl Default for Retention {
    fn default() -> Self {
        Self {
            days: 7,
            max_gb: 20,
            apply_to_manual: false,
        }
    }
}

impl Default for Policy {
    fn default() -> Self {
        Self {
            screen: SourcePolicy {
                mode: CaptureMode::OnDemand,
                agent: AgentPermission::Ask,
            },
            mic: SourcePolicy {
                mode: CaptureMode::OnDemand,
                agent: AgentPermission::Ask,
            },
            system_audio: SourcePolicy {
                mode: CaptureMode::Off,
                agent: AgentPermission::Ask,
            },
            exclusions: default_exclusions(),
            paused: false,
            retention: Retention::default(),
        }
    }
}

impl Policy {
    pub fn source(&self, source: Source) -> SourcePolicy {
        match source {
            Source::Screen | Source::Selection => self.screen,
            Source::Mic => self.mic,
            Source::SystemAudio => self.system_audio,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WindowInfo {
    pub id: u64,
    pub pid: u32,
    pub process: String,
    pub title: String,
    pub class: String,
    pub rect: Rect,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "type",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum Requester {
    User,
    Agent { tool: String, conversation: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "type",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum Target {
    /// Whole monitor (area in physical pixels).
    Monitor { area: Rect },
    /// A specific window.
    Window { window: WindowInfo },
    /// Audio or recorded range; no windows involved.
    Range,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccessRequest {
    pub source: Source,
    pub requester: Requester,
    pub target: Target,
    /// Visible top-level windows on the target, front to back.
    pub visible_windows: Vec<WindowInfo>,
    /// `true` when this is background recording (buffer/continuous) rather
    /// than an explicit capture.
    pub background: bool,
}

/// Session grants given through "Permitir nesta Conversa".
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Grants {
    granted: HashSet<(String, Source)>,
}

impl Grants {
    pub fn grant(&mut self, conversation: &str, source: Source) {
        self.granted.insert((conversation.to_string(), source));
    }
    pub fn revoke_conversation(&mut self, conversation: &str) {
        self.granted.retain(|(c, _)| c != conversation);
    }
    pub fn allows(&self, conversation: &str, source: Source) -> bool {
        self.granted.contains(&(conversation.to_string(), source))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "type",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum DenyReason {
    Paused,
    SourceOff,
    Excluded { app: String },
    AgentNever,
    NotRecording,
}

impl DenyReason {
    /// Short Portuguese text for chips and tool errors (UI translates by `type`).
    pub fn code(&self) -> &'static str {
        match self {
            DenyReason::Paused => "paused",
            DenyReason::SourceOff => "off",
            DenyReason::Excluded { .. } => "excluded",
            DenyReason::AgentNever => "denied",
            DenyReason::NotRecording => "unavailable",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "type",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum Decision {
    Allow,
    /// Allowed, but these rectangles (physical pixels, relative to the
    /// captured image origin) must be covered before use.
    AllowRedacted {
        rects: Vec<Rect>,
    },
    /// The agent must ask the user first.
    Ask,
    Deny {
        reason: DenyReason,
    },
}

impl Decision {
    pub fn is_allowed(&self) -> bool {
        matches!(self, Decision::Allow | Decision::AllowRedacted { .. })
    }
}

/// Share of the target area that an excluded window must cover for the whole
/// capture to be refused instead of redacted.
pub const FULL_COVER_RATIO: f64 = 0.95;

pub fn decide(policy: &Policy, grants: &Grants, req: &AccessRequest) -> Decision {
    if policy.paused {
        return Decision::Deny {
            reason: DenyReason::Paused,
        };
    }
    let sp = policy.source(req.source);
    if sp.mode == CaptureMode::Off {
        return Decision::Deny {
            reason: DenyReason::SourceOff,
        };
    }
    if req.background && !sp.mode.records_in_background() && sp.mode != CaptureMode::Manual {
        return Decision::Deny {
            reason: DenyReason::NotRecording,
        };
    }

    let excluded = |w: &WindowInfo| policy.exclusions.iter().any(|r| r.matches(w));

    let mut rects = Vec::new();
    match &req.target {
        Target::Window { window } => {
            if excluded(window) {
                return Decision::Deny {
                    reason: DenyReason::Excluded {
                        app: display_name(window),
                    },
                };
            }
            // Excluded windows overlapping the captured window are covered.
            for w in req
                .visible_windows
                .iter()
                .filter(|w| w.id != window.id && excluded(w))
            {
                if let Some(i) = w.rect.intersect(&window.rect) {
                    rects.push(Rect::new(
                        i.x - window.rect.x,
                        i.y - window.rect.y,
                        i.w,
                        i.h,
                    ));
                }
            }
        }
        Target::Monitor { area } => {
            let area_px = area.w as f64 * area.h as f64;
            for w in req.visible_windows.iter().filter(|w| excluded(w)) {
                if let Some(i) = w.rect.intersect(area) {
                    if area_px > 0.0 && (i.w as f64 * i.h as f64) / area_px >= FULL_COVER_RATIO {
                        return Decision::Deny {
                            reason: DenyReason::Excluded {
                                app: display_name(w),
                            },
                        };
                    }
                    rects.push(Rect::new(i.x - area.x, i.y - area.y, i.w, i.h));
                }
            }
        }
        Target::Range => {}
    }

    if let Requester::Agent { conversation, .. } = &req.requester {
        match sp.agent {
            AgentPermission::Never => {
                return Decision::Deny {
                    reason: DenyReason::AgentNever,
                };
            }
            AgentPermission::Ask if !grants.allows(conversation, req.source) => {
                return Decision::Ask;
            }
            _ => {}
        }
    }

    if rects.is_empty() {
        Decision::Allow
    } else {
        Decision::AllowRedacted { rects }
    }
}

fn display_name(w: &WindowInfo) -> String {
    let name = w.process.trim_end_matches(".exe").trim_end_matches(".EXE");
    if name.is_empty() {
        w.title.clone()
    } else {
        name.to_string()
    }
}

fn rule(id: &str, process: Option<&str>, title: Option<&str>) -> ExclusionRule {
    ExclusionRule {
        id: id.to_string(),
        process: process.map(str::to_string),
        title_glob: title.map(str::to_string),
        class: None,
        enabled: true,
        builtin: true,
    }
}

/// Built-in exclusion list (password managers, private browsing, Windows
/// credential and security surfaces). Users can disable but not delete them.
pub fn default_exclusions() -> Vec<ExclusionRule> {
    vec![
        rule("keepass", Some("KeePass.exe"), None),
        rule("keepassxc", Some("KeePassXC.exe"), None),
        rule("bitwarden", Some("Bitwarden.exe"), None),
        rule("1password", Some("1Password.exe"), None),
        rule("lastpass", Some("LastPass*.exe"), None),
        rule("dashlane", Some("Dashlane*.exe"), None),
        rule("nordpass", Some("NordPass*.exe"), None),
        rule("credential-ui", Some("CredentialUIBroker.exe"), None),
        rule("uac-consent", Some("consent.exe"), None),
        rule("windows-security", Some("SecHealthUI.exe"), None),
        rule("inprivate", None, Some("*InPrivate*")),
        rule("incognito", None, Some("*Incognito*")),
        rule("anonima", None, Some("*Navegação anônima*")),
        rule("private-browsing", None, Some("*Private Browsing*")),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn win(id: u64, process: &str, rect: Rect) -> WindowInfo {
        WindowInfo {
            id,
            pid: id as u32,
            process: process.into(),
            title: String::new(),
            class: String::new(),
            rect,
        }
    }
    fn screen_req(requester: Requester, windows: Vec<WindowInfo>) -> AccessRequest {
        AccessRequest {
            source: Source::Screen,
            requester,
            target: Target::Monitor {
                area: Rect::new(0, 0, 1920, 1080),
            },
            visible_windows: windows,
            background: false,
        }
    }
    fn agent() -> Requester {
        Requester::Agent {
            tool: "screen_capture".into(),
            conversation: "c1".into(),
        }
    }

    #[test]
    fn excluded_window_on_half_the_screen_is_redacted() {
        let req = screen_req(
            Requester::User,
            vec![
                win(1, "chrome.exe", Rect::new(0, 0, 960, 1080)),
                win(2, "KeePassXC.exe", Rect::new(960, 0, 960, 1080)),
            ],
        );
        assert_eq!(
            decide(&Policy::default(), &Grants::default(), &req),
            Decision::AllowRedacted {
                rects: vec![Rect::new(960, 0, 960, 1080)]
            }
        );
    }

    #[test]
    fn excluded_target_window_or_fullscreen_excluded_is_denied() {
        let kp = win(2, "keepassxc.EXE", Rect::new(0, 0, 1920, 1080));
        let full = screen_req(Requester::User, vec![kp.clone()]);
        assert_eq!(
            decide(&Policy::default(), &Grants::default(), &full),
            Decision::Deny {
                reason: DenyReason::Excluded {
                    app: "keepassxc".into()
                }
            }
        );
        let mut target = full.clone();
        target.target = Target::Window { window: kp };
        assert!(matches!(
            decide(&Policy::default(), &Grants::default(), &target),
            Decision::Deny { .. }
        ));
    }

    #[test]
    fn pause_denies_everyone() {
        let policy = Policy {
            paused: true,
            ..Default::default()
        };
        for requester in [Requester::User, agent()] {
            assert_eq!(
                decide(&policy, &Grants::default(), &screen_req(requester, vec![])),
                Decision::Deny {
                    reason: DenyReason::Paused
                }
            );
        }
    }

    #[test]
    fn agent_permission_never_ask_always() {
        let mut policy = Policy::default();
        policy.screen.agent = AgentPermission::Never;
        assert_eq!(
            decide(&policy, &Grants::default(), &screen_req(agent(), vec![])),
            Decision::Deny {
                reason: DenyReason::AgentNever
            }
        );
        policy.screen.agent = AgentPermission::Ask;
        assert_eq!(
            decide(&policy, &Grants::default(), &screen_req(agent(), vec![])),
            Decision::Ask
        );
        let mut grants = Grants::default();
        grants.grant("c1", Source::Screen);
        assert_eq!(
            decide(&policy, &grants, &screen_req(agent(), vec![])),
            Decision::Allow
        );
        policy.screen.agent = AgentPermission::Always;
        assert_eq!(
            decide(&policy, &Grants::default(), &screen_req(agent(), vec![])),
            Decision::Allow
        );
    }

    #[test]
    fn source_off_and_background_rules() {
        let mut policy = Policy::default();
        let mut req = screen_req(Requester::User, vec![]);
        req.background = true;
        assert_eq!(
            decide(&policy, &Grants::default(), &req),
            Decision::Deny {
                reason: DenyReason::NotRecording
            }
        );
        policy.screen.mode = CaptureMode::RecentBuffer { minutes: 5 };
        assert_eq!(decide(&policy, &Grants::default(), &req), Decision::Allow);
        policy.screen.mode = CaptureMode::Off;
        assert_eq!(
            decide(&policy, &Grants::default(), &req),
            Decision::Deny {
                reason: DenyReason::SourceOff
            }
        );
    }

    #[test]
    fn private_browsing_title_is_excluded() {
        let mut w = win(3, "msedge.exe", Rect::new(0, 0, 100, 100));
        w.title = "Nova guia - Perfil 1 - [InPrivate] - Microsoft Edge".into();
        let req = screen_req(Requester::User, vec![w]);
        assert!(matches!(
            decide(&Policy::default(), &Grants::default(), &req),
            Decision::AllowRedacted { .. }
        ));
    }

    #[test]
    fn disabled_rule_does_not_match() {
        let mut policy = Policy::default();
        policy.exclusions.iter_mut().for_each(|r| r.enabled = false);
        let req = screen_req(
            Requester::User,
            vec![win(2, "KeePassXC.exe", Rect::new(0, 0, 10, 10))],
        );
        assert_eq!(decide(&policy, &Grants::default(), &req), Decision::Allow);
    }

    #[test]
    fn window_target_redaction_is_relative_to_the_window() {
        let target = win(1, "code.exe", Rect::new(100, 100, 800, 600));
        let req = AccessRequest {
            source: Source::Screen,
            requester: Requester::User,
            target: Target::Window {
                window: target.clone(),
            },
            visible_windows: vec![
                win(2, "Bitwarden.exe", Rect::new(700, 600, 400, 400)),
                target,
            ],
            background: false,
        };
        assert_eq!(
            decide(&Policy::default(), &Grants::default(), &req),
            Decision::AllowRedacted {
                rects: vec![Rect::new(600, 500, 200, 100)]
            }
        );
    }
}
