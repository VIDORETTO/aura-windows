//! Context budget (AC-003 of 007, OT-002) and conversion to turn inputs.

use crate::model::{Content, IngestedDoc, block_tokens};
use aura_core::context::{ChipPayload, TurnInput};
use serde::{Deserialize, Serialize};

/// Share of the model context window that attachments may use.
pub const ATTACHMENT_SHARE: f64 = 0.60;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DocBudget {
    pub file_name: String,
    pub tokens: u32,
    /// Suggested cheaper representation when over budget.
    pub suggestion: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BudgetPlan {
    pub budget: u32,
    pub total: u32,
    pub reductions_needed: bool,
    pub docs: Vec<DocBudget>,
}

pub fn budget_for_window(context_window: u32) -> u32 {
    (context_window as f64 * ATTACHMENT_SHARE) as u32
}

pub fn plan(docs: &[IngestedDoc], budget: u32, per_image: u32) -> BudgetPlan {
    let items: Vec<DocBudget> = docs
        .iter()
        .map(|d| {
            let tokens = d.token_estimate(per_image);
            let suggestion = (tokens > budget / docs.len().max(1) as u32).then(|| match d.kind {
                crate::model::DocKind::Pdf => "Selecionar páginas".to_string(),
                crate::model::DocKind::Spreadsheet => {
                    "Enviar só o esquema e uma amostra menor".to_string()
                }
                crate::model::DocKind::Audio | crate::model::DocKind::Video => {
                    "Enviar só a transcrição".to_string()
                }
                _ => "Enviar um trecho".to_string(),
            });
            DocBudget {
                file_name: d.file_name.clone(),
                tokens,
                suggestion,
            }
        })
        .collect();
    let total = items.iter().map(|d| d.tokens).sum();
    BudgetPlan {
        budget,
        total,
        reductions_needed: total > budget,
        docs: items,
    }
}

/// Converts an ingested document to a chip payload; text blocks are prefixed
/// with their locator label (`[Página 3]`, `[Slide 2]`…).
pub fn to_payload(doc: &IngestedDoc) -> ChipPayload {
    let mut parts = vec![ChipPayload::Text {
        text: format!("Anexo: {} ({})", doc.file_name, doc.summary),
    }];
    for b in &doc.blocks {
        let label = match &b.locator {
            crate::model::Locator::Page { n } => Some(format!("[Página {n}]")),
            crate::model::Locator::Time { from_ms, .. } => Some(format!(
                "[{:02}:{:02}]",
                from_ms / 60_000,
                (from_ms / 1000) % 60
            )),
            _ => None,
        };
        match &b.content {
            Content::Text { text } | Content::Table { markdown: text } => {
                parts.push(ChipPayload::Text {
                    text: match &label {
                        Some(l) => format!("{l}\n{text}"),
                        None => text.clone(),
                    },
                })
            }
            Content::Image { path } => parts.push(ChipPayload::Image { path: path.clone() }),
        }
    }
    ChipPayload::Mixed { parts }
}

/// Flattens a document for a turn while respecting `max_tokens` (OT-002).
pub fn to_turn_inputs(doc: &IngestedDoc, max_tokens: u32, per_image: u32) -> Vec<TurnInput> {
    let mut used = 0u32;
    let mut out = vec![TurnInput::Text {
        text: format!("Anexo: {} ({})", doc.file_name, doc.summary),
    }];
    for b in &doc.blocks {
        let t = block_tokens(b, per_image);
        if used + t > max_tokens {
            out.push(TurnInput::Text {
                text: "[conteúdo restante omitido pelo limite de contexto; use attachment_read]"
                    .into(),
            });
            break;
        }
        used += t;
        match &b.content {
            Content::Text { text } | Content::Table { markdown: text } => {
                out.push(TurnInput::Text { text: text.clone() })
            }
            Content::Image { path } => out.push(TurnInput::LocalImage { path: path.clone() }),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Block, DocKind, Locator};

    fn doc(chars: usize) -> IngestedDoc {
        IngestedDoc {
            kind: DocKind::Pdf,
            file_name: "a.pdf".into(),
            summary: "x".into(),
            blocks: vec![Block {
                locator: Locator::Whole,
                content: Content::Text {
                    text: "a".repeat(chars),
                },
            }],
            warnings: vec![],
        }
    }

    #[test]
    fn over_budget_needs_reduction() {
        let p = plan(&[doc(160_000), doc(160_000), doc(160_000)], 60_000, 765);
        assert!(p.reductions_needed);
        assert_eq!(p.total, 120_000);
        assert_eq!(p.docs[0].suggestion.as_deref(), Some("Selecionar páginas"));
        let ok = plan(&[doc(80_000), doc(40_000)], 60_000, 765);
        assert!(!ok.reductions_needed);
    }

    #[test]
    fn turn_inputs_never_exceed_the_budget() {
        let mut d = doc(400);
        d.blocks.push(Block {
            locator: Locator::Page { n: 2 },
            content: Content::Text {
                text: "b".repeat(4000),
            },
        });
        let inputs = to_turn_inputs(&d, 500, 765);
        assert_eq!(inputs.len(), 3);
        assert!(matches!(&inputs[2], TurnInput::Text { text } if text.contains("omitido")));
        assert_eq!(budget_for_window(200_000), 120_000);
    }
}
