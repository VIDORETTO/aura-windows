//! `MediaFileDecoder` over the Media Foundation source reader: any codec
//! Windows can play (MP3, AAC/M4A, WMA, MP4/H.264, MOV, MKV with codecs
//! installed) → 16 kHz mono PCM and evenly spread keyframes.

use crate::encoder::{enc, startup};
use aura_audio::dsp::{Resampler, TARGET_RATE, to_mono};
use aura_capture::encoder::{MediaFileDecoder, SegmentError};
use aura_capture::frame::Frame;
use std::path::Path;
use windows::Win32::Media::MediaFoundation::*;
use windows::Win32::System::Com::StructuredStorage::PROPVARIANT;
use windows::core::{GUID, HSTRING};

/// Two hours of audio at most (007 limits).
const MAX_SECONDS: usize = 2 * 3600;

fn reader(path: &Path, video_processing: bool) -> Result<IMFSourceReader, SegmentError> {
    startup()?;
    unsafe {
        let mut attrs: Option<IMFAttributes> = None;
        MFCreateAttributes(&mut attrs, 1).map_err(enc)?;
        let attrs = attrs.ok_or_else(|| SegmentError::Encoder("no attributes".into()))?;
        if video_processing {
            attrs
                .SetUINT32(&MF_SOURCE_READER_ENABLE_VIDEO_PROCESSING, 1)
                .map_err(enc)?;
        }
        MFCreateSourceReaderFromURL(&HSTRING::from(path.as_os_str()), &attrs).map_err(enc)
    }
}

pub fn decode_audio_16k(path: &Path) -> Result<Vec<f32>, SegmentError> {
    let r = reader(path, false)?;
    unsafe {
        let audio = MF_SOURCE_READER_FIRST_AUDIO_STREAM.0 as u32;
        let out = MFCreateMediaType().map_err(enc)?;
        out.SetGUID(&MF_MT_MAJOR_TYPE, &MFMediaType_Audio)
            .map_err(enc)?;
        out.SetGUID(&MF_MT_SUBTYPE, &MFAudioFormat_Float)
            .map_err(enc)?;
        r.SetCurrentMediaType(audio, None, &out).map_err(enc)?;
        let cur = r.GetCurrentMediaType(audio).map_err(enc)?;
        let channels = cur
            .GetUINT32(&MF_MT_AUDIO_NUM_CHANNELS)
            .map_err(enc)?
            .max(1) as usize;
        let rate = cur
            .GetUINT32(&MF_MT_AUDIO_SAMPLES_PER_SECOND)
            .map_err(enc)?;
        let mut interleaved: Vec<f32> = Vec::new();
        let limit = MAX_SECONDS * rate as usize * channels;
        loop {
            let mut flags = 0u32;
            let mut sample: Option<IMFSample> = None;
            r.ReadSample(audio, 0, None, Some(&mut flags), None, Some(&mut sample))
                .map_err(enc)?;
            if flags & MF_SOURCE_READERF_ENDOFSTREAM.0 as u32 != 0 {
                break;
            }
            let Some(sample) = sample else { continue };
            let buffer = sample.ConvertToContiguousBuffer().map_err(enc)?;
            let mut ptr = std::ptr::null_mut();
            let mut len = 0u32;
            buffer.Lock(&mut ptr, None, Some(&mut len)).map_err(enc)?;
            let floats = std::slice::from_raw_parts(ptr as *const f32, len as usize / 4);
            interleaved.extend_from_slice(floats);
            buffer.Unlock().map_err(enc)?;
            if interleaved.len() > limit {
                return Err(SegmentError::Encoder(
                    "áudio longo demais (máx. 2 h)".into(),
                ));
            }
        }
        let mono = to_mono(&interleaved, channels);
        Ok(if rate == TARGET_RATE {
            mono
        } else {
            Resampler::new(rate, TARGET_RATE).process(&mono)
        })
    }
}

/// Frames decoded at most per requested position (~20 s of 30 fps video).
const MAX_FRAMES_PER_SEEK: usize = 600;

pub fn decode_video_frames(path: &Path, max: usize) -> Result<Vec<(i64, Frame)>, SegmentError> {
    let r = reader(path, true)?;
    unsafe {
        let video = MF_SOURCE_READER_FIRST_VIDEO_STREAM.0 as u32;
        let out = MFCreateMediaType().map_err(enc)?;
        out.SetGUID(&MF_MT_MAJOR_TYPE, &MFMediaType_Video)
            .map_err(enc)?;
        out.SetGUID(&MF_MT_SUBTYPE, &MFVideoFormat_RGB32)
            .map_err(enc)?;
        r.SetCurrentMediaType(video, None, &out).map_err(enc)?;
        let cur = r.GetCurrentMediaType(video).map_err(enc)?;
        let size = cur.GetUINT64(&MF_MT_FRAME_SIZE).map_err(enc)?;
        let (width, height) = ((size >> 32) as u32, (size & 0xffff_ffff) as u32);
        let stride = cur
            .GetUINT32(&MF_MT_DEFAULT_STRIDE)
            .map(|s| s as i32)
            .unwrap_or(-((width * 4) as i32));
        let duration: u64 = r
            .GetPresentationAttribute(MF_SOURCE_READER_MEDIASOURCE.0 as u32, &MF_PD_DURATION)
            .ok()
            .and_then(|pv| u64::try_from(&pv).ok())
            .unwrap_or(0);
        let max = max.max(1);
        let mut frames = Vec::new();
        'positions: for i in 0..max {
            // Evenly spread, skipping the very first (often black) frame.
            let target = (duration * (2 * i as u64 + 1)) / (2 * max as u64);
            if duration > 0 {
                let _ = r.SetCurrentPosition(&GUID::zeroed(), &PROPVARIANT::from(target as i64));
            }
            // The seek lands on the keyframe before `target`: decode forward
            // to it (bounded, for very sparse keyframes).
            let mut ts = 0i64;
            let mut sample: Option<IMFSample> = None;
            for _ in 0..MAX_FRAMES_PER_SEEK {
                let mut flags = 0u32;
                sample = None;
                r.ReadSample(
                    video,
                    0,
                    None,
                    Some(&mut flags),
                    Some(&mut ts),
                    Some(&mut sample),
                )
                .map_err(enc)?;
                if flags & MF_SOURCE_READERF_ENDOFSTREAM.0 as u32 != 0 {
                    break 'positions;
                }
                if sample.is_some() && (duration == 0 || ts as u64 >= target) {
                    break;
                }
            }
            let Some(sample) = sample else { continue };
            let buffer = sample.ConvertToContiguousBuffer().map_err(enc)?;
            let mut ptr = std::ptr::null_mut();
            let mut len = 0u32;
            buffer.Lock(&mut ptr, None, Some(&mut len)).map_err(enc)?;
            let data = std::slice::from_raw_parts(ptr, len as usize);
            let row = (width * 4) as usize;
            let mut bgra = Vec::with_capacity(row * height as usize);
            for y in 0..height as usize {
                let src_y = if stride < 0 {
                    height as usize - 1 - y
                } else {
                    y
                };
                let start = src_y * stride.unsigned_abs() as usize;
                if start + row <= data.len() {
                    bgra.extend_from_slice(&data[start..start + row]);
                }
            }
            buffer.Unlock().map_err(enc)?;
            if bgra.len() == row * height as usize {
                for px in bgra.as_chunks_mut::<4>().0 {
                    px[3] = 0xff;
                }
                frames.push((
                    ts / 10_000,
                    Frame {
                        width,
                        height,
                        bgra,
                        captured_at_ms: 0,
                    },
                ));
            }
            if duration == 0 {
                break; // not seekable: one frame is all we can do cheaply
            }
        }
        Ok(frames)
    }
}

#[derive(Default)]
pub struct MfMediaDecoder;

impl MediaFileDecoder for MfMediaDecoder {
    fn audio_16k(&self, path: &Path) -> Result<Vec<f32>, String> {
        decode_audio_16k(path).map_err(|e| e.to_string())
    }
    fn video_frames(&self, path: &Path, max: usize) -> Result<Vec<(i64, Frame)>, String> {
        decode_video_frames(path, max).map_err(|e| e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::encoder::MfH264Encoder;
    use aura_capture::encoder::VideoEncoder;

    /// 5 s at 10 fps, one solid color per second.
    fn five_second_clip() -> std::path::PathBuf {
        let mut enc = MfH264Encoder::new(10, 50, 1_000_000).unwrap();
        let mut bytes = None;
        for i in 0..50u8 {
            let f = Frame::solid(320, 180, [i / 10 * 50, 100, 200, 255]);
            if let Some(b) = enc.push(&f).unwrap() {
                bytes = Some(b);
            }
        }
        let bytes = bytes.or_else(|| enc.flush().unwrap()).unwrap();
        let path = std::env::temp_dir().join(format!("aura-media-{}.mp4", std::process::id()));
        std::fs::write(&path, bytes).unwrap();
        path
    }

    #[test]
    fn video_frames_are_spread_over_the_clip() {
        let path = five_second_clip();
        let frames = decode_video_frames(&path, 8).unwrap();
        let _ = std::fs::remove_file(&path);
        let ts: Vec<i64> = frames.iter().map(|(t, _)| *t).collect();
        // Positions at 1/16, 3/16 … 15/16 of 5 s: 312 ms … 4687 ms (±1 frame).
        assert_eq!(ts.len(), 8, "{ts:?}");
        assert!(ts.windows(2).all(|w| w[0] < w[1]), "{ts:?}");
        assert!(ts[0] >= 200 && ts[7] >= 4500, "{ts:?}");
    }
}
