//! Custom vocabulary (AC-011 of 006) and energy-based VAD trimming (AC-007).

use regex::Regex;

/// Replaces case-insensitive whole-word occurrences of each term with the
/// canonical spelling. Parts of words are untouched ("aurora" stays).
pub fn apply_vocabulary(text: &str, terms: &[String]) -> String {
    let mut out = text.to_string();
    for term in terms.iter().filter(|t| !t.trim().is_empty()).take(200) {
        let pattern = format!(
            r"(?i)(^|[^\p{{L}}\p{{N}}]){}([^\p{{L}}\p{{N}}]|$)",
            regex::escape(term.trim())
        );
        if let Ok(re) = Regex::new(&pattern) {
            // Loop because adjacent matches share a boundary character.
            loop {
                let next = re
                    .replace_all(&out, |c: &regex::Captures| {
                        format!("{}{}{}", &c[1], term.trim(), &c[2])
                    })
                    .into_owned();
                if next == out {
                    break;
                }
                out = next;
            }
        }
    }
    out
}

/// Initial prompt that biases Whisper towards the terms.
pub fn prompt_for_whisper(terms: &[String]) -> String {
    terms
        .iter()
        .map(|t| t.trim())
        .filter(|t| !t.is_empty())
        .take(200)
        .collect::<Vec<_>>()
        .join(", ")
}

/// Removes leading/trailing silence from 16 kHz audio. Returns `None` when
/// less than `min_speech_ms` of speech is present ("Não ouvi nada").
pub fn trim_silence(samples: &[f32], threshold_dbfs: f32, min_speech_ms: u32) -> Option<Vec<f32>> {
    const FRAME: usize = 480; // 30 ms
    let frames: Vec<bool> = samples
        .chunks(FRAME)
        .map(|f| aura_audio::dsp::rms_dbfs(f) > threshold_dbfs)
        .collect();
    let speech_frames = frames.iter().filter(|v| **v).count();
    if (speech_frames * 30) < min_speech_ms as usize {
        return None;
    }
    let first = frames.iter().position(|v| *v)?;
    let last = frames.iter().rposition(|v| *v)?;
    // Keep 150 ms of margin on each side.
    let start = (first.saturating_sub(5)) * FRAME;
    let end = ((last + 6) * FRAME).min(samples.len());
    Some(samples[start..end].to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;
    use aura_audio::dsp::sine;

    #[test]
    fn vocabulary_whole_words_only() {
        let terms = vec![
            "Aura".to_string(),
            "Codex".to_string(),
            "Parakeet".to_string(),
        ];
        assert_eq!(
            apply_vocabulary("a aura usa o codex e o parakeet, não aurora", &terms),
            "a Aura usa o Codex e o Parakeet, não aurora"
        );
        assert_eq!(prompt_for_whisper(&terms), "Aura, Codex, Parakeet");
    }

    #[test]
    fn silence_only_is_rejected_and_speech_is_trimmed() {
        let silence = vec![0.0f32; 16_000];
        assert!(trim_silence(&silence, -45.0, 300).is_none());
        let mut audio = vec![0.0f32; 16_000];
        audio.extend(sine(300.0, 16_000, 1.0, 0.3));
        audio.extend(vec![0.0f32; 16_000]);
        let trimmed = trim_silence(&audio, -45.0, 300).unwrap();
        assert!(
            trimmed.len() > 16_000 && trimmed.len() < 16_000 + 16 * 400,
            "{}",
            trimmed.len()
        );
        let short = [
            vec![0.0; 8000],
            sine(300.0, 16_000, 0.1, 0.3),
            vec![0.0; 8000],
        ]
        .concat();
        assert!(trim_silence(&short, -45.0, 300).is_none());
    }
}
