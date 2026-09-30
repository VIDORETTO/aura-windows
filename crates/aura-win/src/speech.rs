//! Text-to-speech with Windows.Media.SpeechSynthesis (installed voices,
//! offline). Returns WAV bytes for the UI to play.

use windows::Media::SpeechSynthesis::SpeechSynthesizer;
use windows::Storage::Streams::DataReader;
use windows::core::HSTRING;

pub fn synthesize(text: &str, language: &str) -> Result<Vec<u8>, String> {
    let err = |e: windows::core::Error| e.message().to_string();
    if text.trim().is_empty() {
        return Err("nada para ler".into());
    }
    let synth = SpeechSynthesizer::new().map_err(err)?;
    // Prefer a voice for the UI language (e.g. pt-BR "Maria").
    if let Ok(voices) = SpeechSynthesizer::AllVoices() {
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
