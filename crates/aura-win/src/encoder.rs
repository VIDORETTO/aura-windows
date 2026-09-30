//! H.264 video segments with Media Foundation (ADR 0004): each segment is a
//! self-contained fragmented MP4 written to memory (`IStream` on HGLOBAL),
//! never to disk in plaintext; the recorder seals it before writing.
//!
//! Hardware encoders are preferred (`MF_READWRITE_ENABLE_HARDWARE_TRANSFORMS`);
//! the sink writer inserts the RGB32→NV12 converter itself.

use aura_capture::encoder::{SegmentError, VideoEncoder};
use aura_capture::frame::Frame;
use aura_core::placement::Rect;
use std::sync::OnceLock;
use windows::Win32::Foundation::HGLOBAL;
use windows::Win32::Media::MediaFoundation::*;
use windows::Win32::System::Com::StructuredStorage::{CreateStreamOnHGlobal, GetHGlobalFromStream};
use windows::Win32::System::Com::{
    COINIT_MULTITHREADED, CoInitializeEx, IStream, STATFLAG_NONAME, STATSTG,
};
use windows::Win32::System::Memory::{GlobalLock, GlobalUnlock};
use windows::core::PCWSTR;

pub(crate) fn enc(e: windows::core::Error) -> SegmentError {
    SegmentError::Encoder(e.message().to_string())
}

pub(crate) fn startup() -> Result<(), SegmentError> {
    static STARTED: OnceLock<Result<(), String>> = OnceLock::new();
    STARTED
        .get_or_init(|| unsafe {
            let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
            MFStartup(MF_VERSION, MFSTARTUP_LITE).map_err(|e| e.message().to_string())
        })
        .clone()
        .map_err(SegmentError::Encoder)
}

struct Segment {
    writer: IMFSinkWriter,
    stream: IStream,
    index: u32,
    width: u32,
    height: u32,
    frames: i64,
}

// SAFETY: Media Foundation objects created after `CoInitializeEx(MTA)` are
// free-threaded; the encoder is used by one recorder thread at a time.
unsafe impl Send for Segment {}

pub struct MfH264Encoder {
    fps: u32,
    frames_per_segment: usize,
    bitrate: u32,
    current: Option<Segment>,
    /// Segments closed but not yet handed out (resolution change + full).
    ready: std::collections::VecDeque<Vec<u8>>,
}

impl MfH264Encoder {
    /// `bitrate` in bits/s; ~1.5 Mbps is plenty for 1 fps screen content.
    pub fn new(fps: u32, frames_per_segment: usize, bitrate: u32) -> Result<Self, SegmentError> {
        startup()?;
        Ok(Self {
            fps: fps.max(1),
            frames_per_segment: frames_per_segment.max(1),
            bitrate,
            current: None,
            ready: Default::default(),
        })
    }

    /// Probes whether an H.264 encoder can be created (settings "encoder ok").
    pub fn available() -> bool {
        Self::new(1, 1, 1_000_000)
            .and_then(|mut e| e.open(64, 64).map(|_| ()))
            .is_ok()
    }

    fn open(&mut self, width: u32, height: u32) -> Result<Segment, SegmentError> {
        unsafe {
            let stream: IStream = CreateStreamOnHGlobal(HGLOBAL::default(), true).map_err(enc)?;
            let bytestream = MFCreateMFByteStreamOnStream(&stream).map_err(enc)?;
            let mut attrs: Option<IMFAttributes> = None;
            MFCreateAttributes(&mut attrs, 3).map_err(enc)?;
            let attrs = attrs.ok_or_else(|| SegmentError::Encoder("no attributes".into()))?;
            attrs
                .SetGUID(
                    &MF_TRANSCODE_CONTAINERTYPE,
                    &MFTranscodeContainerType_FMPEG4,
                )
                .map_err(enc)?;
            attrs
                .SetUINT32(&MF_READWRITE_ENABLE_HARDWARE_TRANSFORMS, 1)
                .map_err(enc)?;
            attrs
                .SetUINT32(&MF_SINK_WRITER_DISABLE_THROTTLING, 1)
                .map_err(enc)?;
            let writer =
                MFCreateSinkWriterFromURL(PCWSTR::null(), &bytestream, &attrs).map_err(enc)?;

            let size = ((width as u64) << 32) | height as u64;
            let rate = ((self.fps as u64) << 32) | 1;
            let out = MFCreateMediaType().map_err(enc)?;
            out.SetGUID(&MF_MT_MAJOR_TYPE, &MFMediaType_Video)
                .map_err(enc)?;
            out.SetGUID(&MF_MT_SUBTYPE, &MFVideoFormat_H264)
                .map_err(enc)?;
            out.SetUINT32(&MF_MT_AVG_BITRATE, self.bitrate)
                .map_err(enc)?;
            out.SetUINT32(&MF_MT_INTERLACE_MODE, MFVideoInterlace_Progressive.0 as u32)
                .map_err(enc)?;
            out.SetUINT64(&MF_MT_FRAME_SIZE, size).map_err(enc)?;
            out.SetUINT64(&MF_MT_FRAME_RATE, rate).map_err(enc)?;
            out.SetUINT64(&MF_MT_PIXEL_ASPECT_RATIO, (1u64 << 32) | 1)
                .map_err(enc)?;
            let index = writer.AddStream(&out).map_err(enc)?;

            let input = MFCreateMediaType().map_err(enc)?;
            input
                .SetGUID(&MF_MT_MAJOR_TYPE, &MFMediaType_Video)
                .map_err(enc)?;
            input
                .SetGUID(&MF_MT_SUBTYPE, &MFVideoFormat_RGB32)
                .map_err(enc)?;
            input
                .SetUINT32(&MF_MT_INTERLACE_MODE, MFVideoInterlace_Progressive.0 as u32)
                .map_err(enc)?;
            input.SetUINT64(&MF_MT_FRAME_SIZE, size).map_err(enc)?;
            input.SetUINT64(&MF_MT_FRAME_RATE, rate).map_err(enc)?;
            input
                .SetUINT64(&MF_MT_PIXEL_ASPECT_RATIO, (1u64 << 32) | 1)
                .map_err(enc)?;
            // Positive stride = top-down rows, matching `Frame`.
            input
                .SetUINT32(&MF_MT_DEFAULT_STRIDE, width * 4)
                .map_err(enc)?;
            writer.SetInputMediaType(index, &input, None).map_err(enc)?;
            writer.BeginWriting().map_err(enc)?;
            Ok(Segment {
                writer,
                stream,
                index,
                width,
                height,
                frames: 0,
            })
        }
    }

    fn write(&self, seg: &mut Segment, frame: &Frame) -> Result<(), SegmentError> {
        let len = (seg.width * seg.height * 4) as usize;
        let hns_per_frame = 10_000_000i64 / self.fps as i64;
        unsafe {
            let buffer = MFCreateMemoryBuffer(len as u32).map_err(enc)?;
            let mut ptr = std::ptr::null_mut();
            buffer.Lock(&mut ptr, None, None).map_err(enc)?;
            std::ptr::copy_nonoverlapping(frame.bgra.as_ptr(), ptr, len);
            buffer.Unlock().map_err(enc)?;
            buffer.SetCurrentLength(len as u32).map_err(enc)?;
            let sample = MFCreateSample().map_err(enc)?;
            sample.AddBuffer(&buffer).map_err(enc)?;
            sample
                .SetSampleTime(seg.frames * hns_per_frame)
                .map_err(enc)?;
            sample.SetSampleDuration(hns_per_frame).map_err(enc)?;
            seg.writer.WriteSample(seg.index, &sample).map_err(enc)?;
        }
        seg.frames += 1;
        Ok(())
    }

    fn finish(seg: Segment) -> Result<Vec<u8>, SegmentError> {
        unsafe {
            seg.writer.Finalize().map_err(enc)?;
            let mut stat = STATSTG::default();
            seg.stream.Stat(&mut stat, STATFLAG_NONAME).map_err(enc)?;
            let size = stat.cbSize as usize;
            let hg = GetHGlobalFromStream(&seg.stream).map_err(enc)?;
            let ptr = GlobalLock(hg) as *const u8;
            if ptr.is_null() {
                return Err(SegmentError::Encoder("GlobalLock failed".into()));
            }
            let bytes = std::slice::from_raw_parts(ptr, size).to_vec();
            let _ = GlobalUnlock(hg);
            Ok(bytes)
        }
    }
}

/// H.264 needs even dimensions.
fn even(frame: &Frame) -> std::borrow::Cow<'_, Frame> {
    let (w, h) = (frame.width & !1, frame.height & !1);
    if w == frame.width && h == frame.height {
        std::borrow::Cow::Borrowed(frame)
    } else {
        std::borrow::Cow::Owned(
            frame
                .crop(Rect::new(0, 0, w as i32, h as i32))
                .unwrap_or_else(|| frame.clone()),
        )
    }
}

impl VideoEncoder for MfH264Encoder {
    fn push(&mut self, frame: &Frame) -> Result<Option<Vec<u8>>, SegmentError> {
        let frame = even(frame);
        // Resolution change (monitor switch, DPI change): close the segment.
        if self
            .current
            .as_ref()
            .is_some_and(|s| s.width != frame.width || s.height != frame.height)
            && let Some(seg) = self.current.take()
            && seg.frames > 0
        {
            self.ready.push_back(Self::finish(seg)?);
        }
        if self.current.is_none() {
            self.current = Some(self.open(frame.width, frame.height)?);
        }
        let mut seg = self.current.take().expect("segment");
        self.write(&mut seg, &frame)?;
        if seg.frames as usize >= self.frames_per_segment {
            self.ready.push_back(Self::finish(seg)?);
        } else {
            self.current = Some(seg);
        }
        Ok(self.ready.pop_front())
    }

    fn flush(&mut self) -> Result<Option<Vec<u8>>, SegmentError> {
        if let Some(seg) = self.current.take()
            && seg.frames > 0
        {
            self.ready.push_back(Self::finish(seg)?);
        }
        Ok(self.ready.pop_front())
    }

    fn format(&self) -> &'static str {
        "mp4"
    }
}

/// Decodes an MP4 segment (as produced above) into RGB32 frames with the MF
/// source reader; the video processor converts NV12/H.264 to RGB32.
pub fn decode_frames(mp4: &[u8], max: usize) -> Result<Vec<(i64, Frame)>, SegmentError> {
    use windows::Win32::UI::Shell::SHCreateMemStream;
    startup()?;
    unsafe {
        let stream = SHCreateMemStream(Some(mp4))
            .ok_or_else(|| SegmentError::Encoder("SHCreateMemStream failed".into()))?;
        let bytestream = MFCreateMFByteStreamOnStream(&stream).map_err(enc)?;
        let mut attrs: Option<IMFAttributes> = None;
        MFCreateAttributes(&mut attrs, 1).map_err(enc)?;
        let attrs = attrs.ok_or_else(|| SegmentError::Encoder("no attributes".into()))?;
        attrs
            .SetUINT32(&MF_SOURCE_READER_ENABLE_VIDEO_PROCESSING, 1)
            .map_err(enc)?;
        let reader = MFCreateSourceReaderFromByteStream(&bytestream, &attrs).map_err(enc)?;
        let video = MF_SOURCE_READER_FIRST_VIDEO_STREAM.0 as u32;
        let out = MFCreateMediaType().map_err(enc)?;
        out.SetGUID(&MF_MT_MAJOR_TYPE, &MFMediaType_Video)
            .map_err(enc)?;
        out.SetGUID(&MF_MT_SUBTYPE, &MFVideoFormat_RGB32)
            .map_err(enc)?;
        reader.SetCurrentMediaType(video, None, &out).map_err(enc)?;
        let current = reader.GetCurrentMediaType(video).map_err(enc)?;
        let size = current.GetUINT64(&MF_MT_FRAME_SIZE).map_err(enc)?;
        let (width, height) = ((size >> 32) as u32, (size & 0xffff_ffff) as u32);
        // Negative (or missing) stride means bottom-up rows.
        let stride = current
            .GetUINT32(&MF_MT_DEFAULT_STRIDE)
            .map(|s| s as i32)
            .unwrap_or(-((width * 4) as i32));

        let mut frames = Vec::new();
        loop {
            let mut flags = 0u32;
            let mut ts = 0i64;
            let mut sample: Option<IMFSample> = None;
            reader
                .ReadSample(
                    video,
                    0,
                    None,
                    Some(&mut flags),
                    Some(&mut ts),
                    Some(&mut sample),
                )
                .map_err(enc)?;
            if flags & MF_SOURCE_READERF_ENDOFSTREAM.0 as u32 != 0 {
                break;
            }
            let Some(sample) = sample else { continue };
            let buffer = sample.ConvertToContiguousBuffer().map_err(enc)?;
            let mut ptr = std::ptr::null_mut();
            let mut len = 0u32;
            buffer.Lock(&mut ptr, None, Some(&mut len)).map_err(enc)?;
            let row = (width * 4) as usize;
            let data = std::slice::from_raw_parts(ptr, len as usize);
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
            if frames.len() >= max.max(1) * 4 {
                break;
            }
        }
        Ok(frames)
    }
}

/// `VideoCodec` for the host: H.264 fMP4 segments via Media Foundation.
pub struct MfCodec {
    pub bitrate: u32,
}

impl aura_capture::encoder::VideoCodec for MfCodec {
    fn encoder(&self) -> Result<Box<dyn VideoEncoder>, String> {
        MfH264Encoder::new(1, aura_capture::encoder::FRAMES_PER_SEGMENT, self.bitrate)
            .map(|e| Box::new(e) as Box<dyn VideoEncoder>)
            .map_err(|e| e.to_string())
    }
    fn keyframes(&self, segment: &[u8], max: usize) -> Result<Vec<(i64, Frame)>, String> {
        decode_frames(segment, max).map_err(|e| e.to_string())
    }
}
