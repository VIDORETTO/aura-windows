//! Pure DSP: mono mix, anti-aliased resampling to 16 kHz, RMS level, WAV.

pub const TARGET_RATE: u32 = 16_000;

/// Averages interleaved channels into mono.
pub fn to_mono(interleaved: &[f32], channels: usize) -> Vec<f32> {
    if channels <= 1 {
        return interleaved.to_vec();
    }
    interleaved
        .chunks_exact(channels)
        .map(|f| f.iter().sum::<f32>() / channels as f32)
        .collect()
}

/// Streaming resampler: windowed-sinc low-pass (cutoff 0.45 × target rate)
/// followed by linear interpolation. Keeps state between chunks.
pub struct Resampler {
    from: u32,
    to: u32,
    taps: Vec<f32>,
    history: Vec<f32>,
    pos: f64,
    last: f32,
}

impl Resampler {
    pub fn new(from: u32, to: u32) -> Self {
        let taps = if to < from {
            lowpass(from, to as f32 * 0.45, 63)
        } else {
            vec![1.0]
        };
        Self {
            from,
            to,
            history: vec![0.0; taps.len().saturating_sub(1)],
            taps,
            pos: 0.0,
            last: 0.0,
        }
    }

    pub fn process(&mut self, input: &[f32]) -> Vec<f32> {
        if self.from == self.to {
            return input.to_vec();
        }
        // FIR filter with carried history.
        let mut buf = std::mem::take(&mut self.history);
        buf.extend_from_slice(input);
        let n = self.taps.len();
        let filtered: Vec<f32> = (0..input.len())
            .map(|i| {
                self.taps
                    .iter()
                    .enumerate()
                    .map(|(k, t)| t * buf[i + n - 1 - k])
                    .sum()
            })
            .collect();
        self.history = buf[buf.len() - (n - 1)..].to_vec();

        // Linear interpolation from `from` to `to`.
        let step = self.from as f64 / self.to as f64;
        let mut out = Vec::with_capacity((input.len() as f64 / step) as usize + 1);
        while self.pos < filtered.len() as f64 {
            let i = self.pos.floor() as isize;
            let frac = (self.pos - i as f64) as f32;
            let a = if i < 1 {
                self.last
            } else {
                filtered[i as usize - 1]
            };
            let b = filtered[i.max(0) as usize];
            out.push(if i < 1 {
                self.last + (b - self.last) * frac
            } else {
                a + (b - a) * frac
            });
            self.pos += step;
        }
        self.pos -= filtered.len() as f64;
        self.last = *filtered.last().unwrap_or(&self.last);
        out
    }
}

fn lowpass(rate: u32, cutoff: f32, n: usize) -> Vec<f32> {
    let fc = cutoff / rate as f32;
    let m = (n - 1) as f32;
    let mut taps: Vec<f32> = (0..n)
        .map(|i| {
            let x = i as f32 - m / 2.0;
            let sinc = if x == 0.0 {
                2.0 * fc
            } else {
                (2.0 * std::f32::consts::PI * fc * x).sin() / (std::f32::consts::PI * x)
            };
            let window = 0.54 - 0.46 * (2.0 * std::f32::consts::PI * i as f32 / m).cos();
            sinc * window
        })
        .collect();
    let sum: f32 = taps.iter().sum();
    taps.iter_mut().for_each(|t| *t /= sum);
    taps
}

/// RMS level in dBFS (−100 for silence).
pub fn rms_dbfs(samples: &[f32]) -> f32 {
    if samples.is_empty() {
        return -100.0;
    }
    let rms = (samples.iter().map(|s| s * s).sum::<f32>() / samples.len() as f32).sqrt();
    if rms <= 1e-5 {
        -100.0
    } else {
        20.0 * rms.log10()
    }
}

/// 16-bit PCM mono WAV bytes.
pub fn wav_bytes(samples: &[f32], rate: u32) -> Vec<u8> {
    let data_len = (samples.len() * 2) as u32;
    let mut out = Vec::with_capacity(44 + data_len as usize);
    out.extend_from_slice(b"RIFF");
    out.extend_from_slice(&(36 + data_len).to_le_bytes());
    out.extend_from_slice(b"WAVEfmt ");
    out.extend_from_slice(&16u32.to_le_bytes());
    out.extend_from_slice(&1u16.to_le_bytes()); // PCM
    out.extend_from_slice(&1u16.to_le_bytes()); // mono
    out.extend_from_slice(&rate.to_le_bytes());
    out.extend_from_slice(&(rate * 2).to_le_bytes());
    out.extend_from_slice(&2u16.to_le_bytes());
    out.extend_from_slice(&16u16.to_le_bytes());
    out.extend_from_slice(b"data");
    out.extend_from_slice(&data_len.to_le_bytes());
    for s in samples {
        out.extend_from_slice(&((s.clamp(-1.0, 1.0) * i16::MAX as f32) as i16).to_le_bytes());
    }
    out
}

/// Goertzel magnitude of `freq` in `samples` (tests and diagnostics).
pub fn tone_power(samples: &[f32], rate: u32, freq: f32) -> f32 {
    let k = 2.0 * (2.0 * std::f32::consts::PI * freq / rate as f32).cos();
    let (mut s1, mut s2) = (0.0f32, 0.0f32);
    for x in samples {
        let s0 = x + k * s1 - s2;
        s2 = s1;
        s1 = s0;
    }
    (s1 * s1 + s2 * s2 - k * s1 * s2).sqrt() / samples.len().max(1) as f32
}

pub fn sine(freq: f32, rate: u32, secs: f32, amplitude: f32) -> Vec<f32> {
    (0..(rate as f32 * secs) as usize)
        .map(|i| amplitude * (2.0 * std::f32::consts::PI * freq * i as f32 / rate as f32).sin())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resampling_48k_stereo_keeps_a_1khz_tone() {
        let mono = sine(1000.0, 48_000, 1.0, 0.5);
        let stereo: Vec<f32> = mono.iter().flat_map(|s| [*s, *s]).collect();
        let mut r = Resampler::new(48_000, 16_000);
        let mut out = Vec::new();
        for chunk in to_mono(&stereo, 2).chunks(480) {
            out.extend(r.process(chunk));
        }
        assert!((15_900..=16_100).contains(&out.len()), "{}", out.len());
        let steady = &out[1000..];
        assert!(tone_power(steady, 16_000, 1000.0) > 10.0 * tone_power(steady, 16_000, 3000.0));
        assert!(tone_power(steady, 16_000, 1000.0) > 10.0 * tone_power(steady, 16_000, 500.0));
    }

    #[test]
    fn aliasing_is_attenuated() {
        // 10 kHz is above the 8 kHz Nyquist of 16 kHz output.
        let mut r = Resampler::new(48_000, 16_000);
        let out = r.process(&sine(10_000.0, 48_000, 0.5, 0.5));
        assert!(rms_dbfs(&out[200..]) < -20.0);
    }

    #[test]
    fn level_of_a_minus_12_dbfs_sine() {
        // Peak 0.355 ≈ −9 dBFS peak ⇒ RMS ≈ −12 dBFS.
        let s = sine(440.0, 16_000, 0.5, 0.355);
        assert!((rms_dbfs(&s) + 12.0).abs() < 0.5, "{}", rms_dbfs(&s));
        assert_eq!(rms_dbfs(&[0.0; 100]), -100.0);
    }

    #[test]
    fn wav_header() {
        let w = wav_bytes(&[0.0, 0.5], 16_000);
        assert_eq!(&w[..4], b"RIFF");
        assert_eq!(&w[8..12], b"WAVE");
        assert_eq!(w.len(), 44 + 4);
    }
}
