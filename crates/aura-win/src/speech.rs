//! Text-to-speech with Windows.Media.SpeechSynthesis (installed voices,
//! offline). Returns WAV bytes for the UI to play.

use windows::Media::SpeechSynthesis::SpeechSynthesizer;
use windows::Storage::Streams::DataReader;
use windows::core::HSTRING;

/// Installed voices as `(id, display name, language)`.
pub fn voices() -> Vec<(String, String, String)> {
    let Ok(all) = SpeechSynthesizer::AllVoices() else {
        return Vec::new();
    };
    all.into_iter()
        .map(|v| {
            (
                v.Id().map(|s| s.to_string()).unwrap_or_default(),
                v.DisplayName().map(|s| s.to_string()).unwrap_or_default(),
                v.Language().map(|s| s.to_string()).unwrap_or_default(),
            )
        })
        .filter(|(id, _, _)| !id.is_empty())
        .collect()
}

pub fn synthesize(text: &str, language: &str, voice: Option<&str>) -> Result<Vec<u8>, String> {
    let err = |e: windows::core::Error| e.message().to_string();
    if text.trim().is_empty() {
        return Err("nada para ler".into());
    }
    let synth = SpeechSynthesizer::new().map_err(err)?;
    if let Some(id) = voice {
        let all = SpeechSynthesizer::AllVoices().map_err(err)?;
        let v = all
            .into_iter()
            .find(|v| v.Id().map(|s| s == id).unwrap_or(false))
            .ok_or_else(|| format!("voz não encontrada: {id}"))?;
        synth.SetVoice(&v).map_err(err)?;
    } else if let Ok(voices) = SpeechSynthesizer::AllVoices() {
        // Prefer a voice for the UI language (e.g. pt-BR "Maria").
        let lang = language.to_lowercase();
        let prefix = lang.split('-').next().unwrap_or("").to_string();
        let mut chosen = None;
        for v in voices {
            let tag = v
                .Language()
                .map(|l| l.to_string().to_lowercase())
                .unwrap_or_default();
            if tag == lang {
                chosen = Some(v);
                break;
            }
            if chosen.is_none() && tag.starts_with(&prefix) {
                chosen = Some(v);
            }
        }
        if let Some(v) = chosen {
            let _ = synth.SetVoice(&v);
        }
    }
    let stream = synth
        .SynthesizeTextToStreamAsync(&HSTRING::from(text))
        .map_err(err)?
        .join()
        .map_err(err)?;
    let size = stream.Size().map_err(err)? as u32;
    let input = stream.GetInputStreamAt(0).map_err(err)?;
    let reader = DataReader::CreateDataReader(&input).map_err(err)?;
    reader.LoadAsync(size).map_err(err)?.join().map_err(err)?;
    let mut buf = vec![0u8; size as usize];
    reader.ReadBytes(&mut buf).map_err(err)?;
    Ok(buf)
}
