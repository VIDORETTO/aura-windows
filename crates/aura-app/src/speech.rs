//! Read aloud (009 TK-003): markdown → plain speakable text → platform TTS
//! (Windows.Media.SpeechSynthesis on Windows; a short tone in the fake).

/// Longest text spoken at once (the rest is cut at a sentence boundary).
pub const MAX_SPEECH_CHARS: usize = 5000;

/// An installed offline voice.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SpeechVoice {
    pub id: String,
    pub name: String,
    /// BCP-47 tag, e.g. `pt-BR`.
    pub language: String,
}

pub trait Speech: Send + Sync {
    /// Installed voices (empty when the platform cannot list them).
    fn voices(&self) -> Vec<SpeechVoice> {
        Vec::new()
    }
    /// Returns audio bytes and their MIME type (e.g. `audio/wav`). `voice`
    /// is a [`SpeechVoice::id`]; `None` picks one for `language`.
    fn synthesize(
        &self,
        text: &str,
        language: &str,
        voice: Option<&str>,
    ) -> Result<(Vec<u8>, String), String>;
}

/// Test/demo voice: 0.3 s of a quiet tone as WAV.
pub struct FakeSpeech;

impl Speech for FakeSpeech {
    fn voices(&self) -> Vec<SpeechVoice> {
        [
            ("fake-pt", "Fake Maria", "pt-BR"),
            ("fake-en", "Fake Zira", "en-US"),
        ]
        .into_iter()
        .map(|(id, name, language)| SpeechVoice {
            id: id.into(),
            name: name.into(),
            language: language.into(),
        })
        .collect()
    }

    fn synthesize(
        &self,
        text: &str,
        _language: &str,
        voice: Option<&str>,
    ) -> Result<(Vec<u8>, String), String> {
        if text.trim().is_empty() {
            return Err("nada para ler".into());
        }
        if let Some(v) = voice
            && !self.voices().iter().any(|x| x.id == v)
        {
            return Err(format!("voz não encontrada: {v}"));
        }
        let pcm = aura_audio::dsp::sine(440.0, 16_000, 0.3, 0.1);
        Ok((aura_audio::dsp::wav_bytes(&pcm, 16_000), "audio/wav".into()))
    }
}

/// Removes markdown syntax, code blocks and URLs so the voice reads prose.
pub fn speakable(markdown: &str) -> String {
    let mut out = String::new();
    let mut in_code = false;
    for line in markdown.lines() {
        let l = line.trim();
        if l.starts_with("```") || l.starts_with("~~~") {
            in_code = !in_code;
            if in_code {
                out.push_str("(bloco de código omitido)\n");
            }
            continue;
        }
        if in_code || l.starts_with('|') && l.contains("---") {
            continue;
        }
        let l = l.trim_start_matches(['#', '>', ' ']);
        let l = l
            .strip_prefix("- ")
            .or_else(|| l.strip_prefix("* "))
            .unwrap_or(l);
        let mut s = String::with_capacity(l.len());
        let mut chars = l.chars().peekable();
        while let Some(c) = chars.next() {
            match c {
                '*' | '_' | '`' | '|' => {}
                '[' => {
                    // [text](url) → text
                    let text: String = chars.by_ref().take_while(|c| *c != ']').collect();
                    if chars.peek() == Some(&'(') {
                        for c in chars.by_ref() {
                            if c == ')' {
                                break;
                            }
                        }
                    }
                    s.push_str(&text);
                }
                _ => s.push(c),
            }
        }
        let s: Vec<&str> = s
            .split_whitespace()
            .filter(|w| !w.starts_with("http://") && !w.starts_with("https://"))
            .collect();
        if !s.is_empty() {
            out.push_str(&s.join(" "));
            out.push('\n');
        }
    }
    let text = out.trim().to_string();
    if text.chars().count() <= MAX_SPEECH_CHARS {
        return text;
    }
    let cut: String = text.chars().take(MAX_SPEECH_CHARS).collect();
    match cut.rfind(['.', '!', '?', '\n']) {
        Some(i) => cut[..=i].to_string(),
        None => cut,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_markdown_code_and_links() {
        let md = "## Resumo\n\nO **erro** está em `main.rs`. Veja [a doc](https://doc.rust-lang.org).\n\n```rust\nfn main() {}\n```\n- item um\n- https://x.y";
        assert_eq!(
            speakable(md),
            "Resumo\nO erro está em main.rs. Veja a doc.\n(bloco de código omitido)\nitem um"
        );
    }

    #[test]
    fn long_text_is_cut_at_a_sentence() {
        let long = "Frase curta. ".repeat(600);
        let s = speakable(&long);
        assert!(s.chars().count() <= MAX_SPEECH_CHARS && s.ends_with('.'));
    }
}
