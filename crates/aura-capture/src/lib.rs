//! Screen capture pipeline (`specs/004-contexto-de-tela`).
//!
//! Pure, OS-independent parts live here; Windows adapters (Windows.Graphics
//! .Capture, window inventory, UI Automation, OCR, Media Foundation encoder)
//! implement the traits in [`source`] and [`segments`] inside `aura-win`.
//!
//! Every pixel leaving this crate must have passed `aura_policy::decide` and
//! [`frame::redact`] (OT-001). [`capture_with_policy`] is the entry point for
//! explicit captures.

pub mod clips;
pub mod encoder;
pub mod frame;
pub mod retention;
#[cfg(feature = "store")]
pub mod segments;
pub mod source;

use aura_policy::{AccessRequest, Decision, Grants, Policy, Requester, Source, decide};
use frame::{Frame, redact};
use source::{CaptureError, FrameSource, Target, WindowInventory};

#[derive(Debug, Clone, PartialEq)]
pub enum CaptureOutcome {
    Captured { frame: Frame, redacted: bool },
    Denied { reason: aura_policy::DenyReason },
    NeedsPermission,
}

/// Explicit capture (user or agent), policy first, redaction applied.
pub fn capture_with_policy(
    policy: &Policy,
    grants: &Grants,
    requester: Requester,
    target: &Target,
    source: &dyn FrameSource,
    inventory: &dyn WindowInventory,
) -> Result<CaptureOutcome, CaptureError> {
    let (policy_target, area) = match target {
        Target::Monitor { area, .. } => (aura_policy::Target::Monitor { area: *area }, *area),
        Target::Window { window } => {
            let info = inventory.window(*window).ok_or(CaptureError::WindowGone)?;
            let rect = info.rect;
            (aura_policy::Target::Window { window: info }, rect)
        }
    };
    let decision = decide(
        policy,
        grants,
        &AccessRequest {
            source: Source::Screen,
            requester,
            target: policy_target,
            visible_windows: inventory.visible_windows(area),
            background: false,
        },
    );
    match decision {
        Decision::Deny { reason } => Ok(CaptureOutcome::Denied { reason }),
        Decision::Ask => Ok(CaptureOutcome::NeedsPermission),
        Decision::Allow => Ok(CaptureOutcome::Captured {
            frame: source.capture(target)?,
            redacted: false,
        }),
        Decision::AllowRedacted { rects } => {
            let mut frame = source.capture(target)?;
            redact(&mut frame, &rects);
            Ok(CaptureOutcome::Captured {
                frame,
                redacted: true,
            })
        }
    }
}
