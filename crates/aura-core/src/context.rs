//! Context Chips and turn inputs (OT-005 of `specs/002-conversa-agente-codex`).
//!
//! A [`ContextChip`] is anything that will be sent with the next Turn. The host
//! owns the pending chips in a [`ContextTray`]; the UI only lists and removes
//! them, so file contents never travel over IPC.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use thiserror::Error;
use uuid::Uuid;

pub const MAX_IMAGES_PER_TURN: usize = 10;
pub const MAX_IMAGE_BYTES: u64 = 20 * 1024 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ChipKind {
    Screen,
    Region,
    Selection,
    Audio,
    Clip,
    File,
    Image,
    Skill,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "type",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum ChipPayload {
    /// An image on disk sent as `localImage`.
    Image { path: PathBuf },
    /// Several images (keyframes of a clip) followed by optional text.
    Images {
        paths: Vec<PathBuf>,
        caption: Option<String>,
    },
    /// Plain text (selection, transcript, ingested document).
    Text { text: String },
    /// A skill invocation.
    Skill { name: String, path: PathBuf },
    /// Composite payload (e.g. PDF text + page images).
    Mixed { parts: Vec<ChipPayload> },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SummaryUnit {
    Line,
    Word,
    Page,
    Sheet,
    Slide,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum SummaryPart {
    Count { amount: u64, unit: SummaryUnit },
    Image,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AttachmentLabel {
    pub file_name: String,
    pub parts: Vec<SummaryPart>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContextChip {
    pub id: String,
    pub kind: ChipKind,
    pub label: String,
    /// Optional display metadata; legacy chips keep their original label.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attachment_label: Option<AttachmentLabel>,
    pub preview_path: Option<PathBuf>,
    pub payload: ChipPayload,
    /// Estimated model tokens this chip will consume.
    pub token_estimate: u32,
    /// When set, the chip is shown but will not be sent (privacy policy, model limits).
    pub blocked_reason: Option<String>,
    /// Folder of files kept with the chip (a Clip's frames and audio); moved
    /// into the conversation workspace when the chip is sent.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub files_dir: Option<PathBuf>,
}

impl ContextChip {
    pub fn new(kind: ChipKind, label: impl Into<String>, payload: ChipPayload) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            kind,
            label: label.into(),
            attachment_label: None,
            preview_path: None,
            token_estimate: estimate_payload_tokens(&payload),
            payload,
            blocked_reason: None,
            files_dir: None,
        }
    }

    pub fn blocked(kind: ChipKind, label: impl Into<String>, reason: impl Into<String>) -> Self {
        let mut chip = Self::new(
            kind,
            label,
            ChipPayload::Text {
                text: String::new(),
            },
        );
        chip.blocked_reason = Some(reason.into());
        chip.token_estimate = 0;
        chip
    }

    fn image_count(&self) -> usize {
        count_images(&self.payload)
    }
}

/// Heuristic used everywhere in the UI: ~4 characters per token, 765 per image.
pub const TOKENS_PER_IMAGE: u32 = 765;

pub fn estimate_text_tokens(text: &str) -> u32 {
    (text.chars().count() as u32).div_ceil(4)
}

fn estimate_payload_tokens(p: &ChipPayload) -> u32 {
    match p {
        ChipPayload::Image { .. } => TOKENS_PER_IMAGE,
        ChipPayload::Images { paths, caption } => {
            paths.len() as u32 * TOKENS_PER_IMAGE
                + caption.as_deref().map_or(0, estimate_text_tokens)
        }
        ChipPayload::Text { text } => estimate_text_tokens(text),
        ChipPayload::Skill { .. } => 0,
        ChipPayload::Mixed { parts } => parts.iter().map(estimate_payload_tokens).sum(),
    }
}

fn count_images(p: &ChipPayload) -> usize {
    match p {
        ChipPayload::Image { .. } => 1,
        ChipPayload::Images { paths, .. } => paths.len(),
        ChipPayload::Mixed { parts } => parts.iter().map(count_images).sum(),
        _ => 0,
    }
}

/// Model-facing input items. Mirrors the subset of Codex `UserInput` we use.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "type",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum TurnInput {
    Text { text: String },
    LocalImage { path: PathBuf },
    Skill { name: String, path: PathBuf },
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ChipError {
    #[error("too many images in one turn (max {max})")]
    TooManyImages { max: usize },
    #[error("chip not found")]
    NotFound,
    #[error("the current model does not accept images")]
    ModelWithoutImage,
}

/// Pending chips for the next turn.
#[derive(Debug, Default)]
pub struct ContextTray {
    chips: Vec<ContextChip>,
}

impl ContextTray {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn list(&self) -> &[ContextChip] {
        &self.chips
    }

    pub fn add(&mut self, chip: ContextChip) -> Result<&ContextChip, ChipError> {
        let images: usize = self.chips.iter().map(ContextChip::image_count).sum();
        if images + chip.image_count() > MAX_IMAGES_PER_TURN {
            return Err(ChipError::TooManyImages {
                max: MAX_IMAGES_PER_TURN,
            });
        }
        self.chips.push(chip);
        Ok(self.chips.last().expect("just pushed"))
    }

    /// Replaces a chip in place (used when background ingestion finishes).
    pub fn replace(&mut self, chip: ContextChip) -> Result<(), ChipError> {
        let slot = self
            .chips
            .iter_mut()
            .find(|c| c.id == chip.id)
            .ok_or(ChipError::NotFound)?;
        *slot = chip;
        Ok(())
    }

    pub fn remove(&mut self, id: &str) -> Result<ContextChip, ChipError> {
        let pos = self
            .chips
            .iter()
            .position(|c| c.id == id)
            .ok_or(ChipError::NotFound)?;
        Ok(self.chips.remove(pos))
    }

    pub fn clear(&mut self) {
        self.chips.clear();
    }

    pub fn has_images(&self) -> bool {
        self.chips
            .iter()
            .any(|c| c.blocked_reason.is_none() && c.image_count() > 0)
    }

    pub fn token_estimate(&self) -> u32 {
        self.chips
            .iter()
            .filter(|c| c.blocked_reason.is_none())
            .map(|c| c.token_estimate)
            .sum()
    }

    /// Builds the turn inputs and empties the tray atomically. Blocked chips are
    /// dropped. `text` goes first, then chips in insertion order.
    pub fn drain_for_turn(
        &mut self,
        text: &str,
        model_accepts_images: bool,
    ) -> Result<Vec<TurnInput>, ChipError> {
        if !model_accepts_images && self.has_images() {
            return Err(ChipError::ModelWithoutImage);
        }
        let chips = std::mem::take(&mut self.chips);
        let mut inputs = Vec::new();
        let mut skills = Vec::new();
        let mut prefix = String::new();
        for chip in chips.iter().filter(|c| c.blocked_reason.is_none()) {
            if let ChipPayload::Skill { name, .. } = &chip.payload {
                prefix.push_str(&format!("${name} "));
            }
        }
        let body = format!("{prefix}{text}");
        if !body.trim().is_empty() {
            inputs.push(TurnInput::Text {
                text: body.trim_end().to_string(),
            });
        }
        for chip in chips.into_iter().filter(|c| c.blocked_reason.is_none()) {
            push_payload(chip.payload, &mut inputs, &mut skills);
        }
        inputs.extend(skills);
        Ok(inputs)
    }
}

fn push_payload(p: ChipPayload, out: &mut Vec<TurnInput>, skills: &mut Vec<TurnInput>) {
    match p {
        ChipPayload::Image { path } => out.push(TurnInput::LocalImage { path }),
        ChipPayload::Images { paths, caption } => {
            out.extend(paths.into_iter().map(|path| TurnInput::LocalImage { path }));
            if let Some(text) = caption.filter(|c| !c.is_empty()) {
                out.push(TurnInput::Text { text });
            }
        }
        ChipPayload::Text { text } if !text.is_empty() => out.push(TurnInput::Text { text }),
        ChipPayload::Text { .. } => {}
        ChipPayload::Skill { name, path } => skills.push(TurnInput::Skill { name, path }),
        ChipPayload::Mixed { parts } => {
            for part in parts {
                push_payload(part, out, skills);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn image(path: &str) -> ContextChip {
        ContextChip::new(
            ChipKind::Image,
            "Imagem",
            ChipPayload::Image { path: path.into() },
        )
    }

    #[test]
    fn removed_chip_is_never_sent() {
        let mut tray = ContextTray::new();
        tray.add(image("a.png")).unwrap();
        let id = tray.add(image("b.png")).unwrap().id.clone();
        tray.remove(&id).unwrap();
        let inputs = tray.drain_for_turn("o que é isso?", true).unwrap();
        assert_eq!(
            inputs,
            vec![
                TurnInput::Text {
                    text: "o que é isso?".into()
                },
                TurnInput::LocalImage {
                    path: "a.png".into()
                },
            ]
        );
        assert!(tray.list().is_empty());
    }

    #[test]
    fn model_without_images_blocks_the_turn_and_keeps_chips() {
        let mut tray = ContextTray::new();
        tray.add(image("a.png")).unwrap();
        assert_eq!(
            tray.drain_for_turn("oi", false),
            Err(ChipError::ModelWithoutImage)
        );
        assert_eq!(tray.list().len(), 1);
    }

    #[test]
    fn at_most_ten_images_per_turn() {
        let mut tray = ContextTray::new();
        for i in 0..10 {
            tray.add(image(&format!("{i}.png"))).unwrap();
        }
        assert_eq!(
            tray.add(image("x.png")).unwrap_err(),
            ChipError::TooManyImages { max: 10 }
        );
    }

    #[test]
    fn blocked_chips_are_shown_but_not_sent() {
        let mut tray = ContextTray::new();
        tray.add(ContextChip::blocked(
            ChipKind::Screen,
            "Tela",
            "Janela excluída: Bitwarden",
        ))
        .unwrap();
        let inputs = tray.drain_for_turn("oi", true).unwrap();
        assert_eq!(inputs, vec![TurnInput::Text { text: "oi".into() }]);
    }

    #[test]
    fn skill_chip_prefixes_text_and_adds_skill_item() {
        let mut tray = ContextTray::new();
        tray.add(ContextChip::new(
            ChipKind::Skill,
            "Skill: revisar-contrato",
            ChipPayload::Skill {
                name: "revisar-contrato".into(),
                path: "s/SKILL.md".into(),
            },
        ))
        .unwrap();
        let inputs = tray.drain_for_turn("contrato.pdf anexado", true).unwrap();
        assert_eq!(
            inputs,
            vec![
                TurnInput::Text {
                    text: "$revisar-contrato contrato.pdf anexado".into()
                },
                TurnInput::Skill {
                    name: "revisar-contrato".into(),
                    path: "s/SKILL.md".into()
                },
            ]
        );
    }

    #[test]
    fn token_estimate_counts_text_and_images() {
        let mut tray = ContextTray::new();
        tray.add(image("a.png")).unwrap();
        tray.add(ContextChip::new(
            ChipKind::Selection,
            "Seleção",
            ChipPayload::Text {
                text: "abcdefgh".into(),
            },
        ))
        .unwrap();
        assert_eq!(tray.token_estimate(), 765 + 2);
    }
}
