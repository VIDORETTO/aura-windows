//! Transcript formatting for audio Recortes (AC-007 of 005).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TimedText {
    pub start_ms: i64,
    pub end_ms: i64,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Line {
    pub at_ms: i64,
    pub speaker: &'static str,
    pub text: String,
}

/// Merges mic ("Você") and system ("Sistema") segments ordered by start time.
/// On ties the system line comes first (it usually prompted the reply).
pub fn interleave(mic: &[TimedText], system: &[TimedText]) -> Vec<Line> {
    let mut lines: Vec<(i64, u8, Line)> = Vec::new();
    for s in system.iter().filter(|s| !s.text.trim().is_empty()) {
        lines.push((
            s.start_ms,
            0,
            Line {
                at_ms: s.start_ms,
                speaker: "Sistema",
                text: s.text.trim().into(),
            },
        ));
    }
    for s in mic.iter().filter(|s| !s.text.trim().is_empty()) {
        lines.push((
            s.start_ms,
            1,
            Line {
                at_ms: s.start_ms,
                speaker: "Você",
                text: s.text.trim().into(),
            },
        ));
    }
    lines.sort_by_key(|(t, order, _)| (*t, *order));
    lines.into_iter().map(|(_, _, l)| l).collect()
}

fn mmss(ms: i64) -> String {
    let s = ms.max(0) / 1000;
    format!("{:02}:{:02}", s / 60, s % 60)
}

/// Text block attached to the turn, e.g.
/// `Transcrição (últimos 2:00, Microfone+Sistema)\n[00:12] Você: …`.
pub fn format_transcript(lines: &[Line], duration_ms: i64, sources_label: &str) -> String {
    let total = duration_ms.max(0) / 1000;
    let mut out = format!(
        "Transcrição (últimos {}:{:02}, {sources_label})",
        total / 60,
        total % 60
    );
    if lines.is_empty() {
        out.push_str("\n(nenhuma fala detectada)");
    }
    for l in lines {
        out.push_str(&format!("\n[{}] {}: {}", mmss(l.at_ms), l.speaker, l.text));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn t(start: i64, text: &str) -> TimedText {
        TimedText {
            start_ms: start,
            end_ms: start + 500,
            text: text.into(),
        }
    }

    #[test]
    fn interleaves_by_start_time() {
        let lines = interleave(
            &[t(500, "oi")],
            &[t(200, "bem-vindo"), t(1000, "tudo bem?")],
        );
        let text = format_transcript(&lines, 120_000, "Microfone+Sistema");
        assert_eq!(
            text,
            "Transcrição (últimos 2:00, Microfone+Sistema)\n[00:00] Sistema: bem-vindo\n[00:00] Você: oi\n[00:01] Sistema: tudo bem?"
        );
    }

    #[test]
    fn empty_transcript_is_explicit() {
        assert!(format_transcript(&[], 60_000, "Microfone").ends_with("(nenhuma fala detectada)"));
    }
}
