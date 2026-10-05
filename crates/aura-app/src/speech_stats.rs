//! Speech coaching (045), private to the user: how much of the meeting they
//! spoke, their pace, filler words and longest turn. Computed locally from the
//! transcript; nothing is judged about other people.

use crate::meeting::Utterance;
use serde::Serialize;

/// Filler words to count (lowercase, whole words), Portuguese and English.
const FILLERS: &[&str] = &[
    "né",
    "tipo",
    "então",
    "assim",
    "aham",
    "hum",
    "hmm",
    "ahn",
    "eh",
    "éh",
    "uh",
    "um",
    "uhm",
    "like",
    "basically",
    "actually",
    "literally",
    "sabe",
];

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SpeechStats {
    /// Share of the speaking time that was the user's, 0..=100.
    pub talk_percent: u32,
    pub you_seconds: u32,
    pub them_seconds: u32,
    /// Words per minute while the user was speaking.
    pub words_per_minute: u32,
    pub filler_count: u32,
    /// Fillers per 100 words.
    pub filler_per_100: f32,
    pub longest_turn_seconds: u32,
    pub questions_asked: u32,
}

fn words(text: &str) -> Vec<String> {
    text.split(|c: char| !c.is_alphanumeric() && c != '\'' && c != '’')
        .filter(|w| !w.is_empty())
        .map(|w| w.to_lowercase())
        .collect()
}

/// Statistics over a meeting's utterances (notes are ignored).
pub fn compute(utterances: &[Utterance]) -> SpeechStats {
    let dur = |u: &Utterance| (u.t1 - u.t0).max(0);
    let you: Vec<&Utterance> = utterances.iter().filter(|u| u.speaker == "you").collect();
    let them_ms: i64 = utterances
        .iter()
        .filter(|u| u.speaker == "them")
        .map(dur)
        .sum();
    let you_ms: i64 = you.iter().map(|u| dur(u)).sum();
    let your_words: Vec<String> = you.iter().flat_map(|u| words(&u.text)).collect();
    let fillers = your_words
        .iter()
        .filter(|w| FILLERS.contains(&w.as_str()))
        .count() as u32;
    let total_ms = you_ms + them_ms;
    let wpm = if you_ms >= 5_000 {
        (your_words.len() as f64 / (you_ms as f64 / 60_000.0)).round() as u32
    } else {
        0
    };
    SpeechStats {
        talk_percent: if total_ms == 0 {
            0
        } else {
            ((you_ms * 100) as f64 / total_ms as f64).round() as u32
        },
        you_seconds: (you_ms / 1000) as u32,
        them_seconds: (them_ms / 1000) as u32,
        words_per_minute: wpm,
        filler_count: fillers,
        filler_per_100: if your_words.is_empty() {
            0.0
        } else {
            (fillers as f32 * 1000.0 / your_words.len() as f32).round() / 10.0
        },
        longest_turn_seconds: (you.iter().map(|u| dur(u)).max().unwrap_or(0) / 1000) as u32,
        questions_asked: you.iter().map(|u| u.text.matches('?').count() as u32).sum(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn u(who: &str, t0: i64, t1: i64, text: &str) -> Utterance {
        Utterance {
            id: t0,
            meeting_id: "m".into(),
            t0,
            t1,
            speaker: who.into(),
            text: text.into(),
        }
    }

    #[test]
    fn talk_time_pace_fillers_and_questions() {
        let lines = vec![
            u("them", 0, 30_000, "Explicação longa do cliente"),
            // 20 s, 39 words (counted apart) with 5 fillers (então, né, tipo, assim, né) → 117 wpm.
            u(
                "you",
                30_000,
                50_000,
                "então né tipo assim eu acho que dá para fazer isso na próxima semana se todo mundo concordar com o prazo e com o escopo do projeto né bom e qual é o orçamento disponível para essa etapa agora?",
            ),
            u("note", 51_000, 51_000, "ligar para o Bruno"),
        ];
        let s = compute(&lines);
        assert_eq!((s.you_seconds, s.them_seconds), (20, 30));
        assert_eq!(s.talk_percent, 40);
        assert_eq!(s.filler_count, 5);
        assert_eq!(s.longest_turn_seconds, 20);
        assert_eq!(s.questions_asked, 1);
        assert_eq!(s.words_per_minute, 117);
        assert!(
            s.filler_per_100 > 12.0 && s.filler_per_100 < 13.5,
            "{}",
            s.filler_per_100
        );
    }

    #[test]
    fn empty_or_one_sided_meetings_do_not_divide_by_zero() {
        assert_eq!(compute(&[]).talk_percent, 0);
        let only_them = compute(&[u("them", 0, 10_000, "oi")]);
        assert_eq!(
            (
                only_them.talk_percent,
                only_them.words_per_minute,
                only_them.filler_per_100
            ),
            (0, 0, 0.0)
        );
        // Too little speech for a pace.
        assert_eq!(
            compute(&[u("you", 0, 2_000, "oi tudo bem")]).words_per_minute,
            0
        );
    }

    #[test]
    fn only_whole_words_count_as_fillers() {
        let s = compute(&[u("you", 0, 10_000, "um tipográfico assimétrico umbral Uh")]);
        assert_eq!(s.filler_count, 2, "um + uh only");
    }
}
