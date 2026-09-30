//! In-memory BGRA frames plus the pure pixel operations the pipeline needs:
//! crop, redaction fill, downscale and PNG encoding.

use aura_core::placement::Rect;

#[derive(Clone, PartialEq, Eq)]
pub struct Frame {
    pub width: u32,
    pub height: u32,
    /// BGRA8, row-major, no padding.
    pub bgra: Vec<u8>,
    /// Milliseconds since the Unix epoch.
    pub captured_at_ms: i64,
}

impl std::fmt::Debug for Frame {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Frame({}x{} @ {})",
            self.width, self.height, self.captured_at_ms
        )
    }
}

impl Frame {
    pub fn solid(width: u32, height: u32, bgra: [u8; 4]) -> Self {
        let mut data = Vec::with_capacity((width * height * 4) as usize);
        for _ in 0..width * height {
            data.extend_from_slice(&bgra);
        }
        Self {
            width,
            height,
            bgra: data,
            captured_at_ms: 0,
        }
    }

    pub fn pixel(&self, x: u32, y: u32) -> [u8; 4] {
        let i = ((y * self.width + x) * 4) as usize;
        [
            self.bgra[i],
            self.bgra[i + 1],
            self.bgra[i + 2],
            self.bgra[i + 3],
        ]
    }

    pub fn fill_rect(&mut self, rect: Rect, bgra: [u8; 4]) {
        let bounds = Rect::new(0, 0, self.width as i32, self.height as i32);
        let Some(r) = rect.intersect(&bounds) else {
            return;
        };
        for y in r.y..r.bottom() {
            for x in r.x..r.right() {
                let i = ((y as u32 * self.width + x as u32) * 4) as usize;
                self.bgra[i..i + 4].copy_from_slice(&bgra);
            }
        }
    }

    /// Crops to the intersection of `rect` with the frame.
    pub fn crop(&self, rect: Rect) -> Option<Frame> {
        let bounds = Rect::new(0, 0, self.width as i32, self.height as i32);
        let r = rect.intersect(&bounds)?;
        let mut out = Vec::with_capacity((r.w * r.h * 4) as usize);
        for y in r.y..r.bottom() {
            let start = ((y as u32 * self.width + r.x as u32) * 4) as usize;
            out.extend_from_slice(&self.bgra[start..start + (r.w * 4) as usize]);
        }
        Some(Frame {
            width: r.w as u32,
            height: r.h as u32,
            bgra: out,
            captured_at_ms: self.captured_at_ms,
        })
    }

    /// Box-filter downscale so the longest side is at most `max_side`.
    pub fn downscale(&self, max_side: u32) -> Frame {
        let longest = self.width.max(self.height);
        if longest <= max_side || longest == 0 {
            return self.clone();
        }
        let scale = max_side as f64 / longest as f64;
        let nw = ((self.width as f64 * scale).round() as u32).max(1);
        let nh = ((self.height as f64 * scale).round() as u32).max(1);
        let mut out = vec![0u8; (nw * nh * 4) as usize];
        for ny in 0..nh {
            let y0 = (ny as u64 * self.height as u64 / nh as u64) as u32;
            let y1 = (((ny + 1) as u64 * self.height as u64 / nh as u64) as u32).max(y0 + 1);
            for nx in 0..nw {
                let x0 = (nx as u64 * self.width as u64 / nw as u64) as u32;
                let x1 = (((nx + 1) as u64 * self.width as u64 / nw as u64) as u32).max(x0 + 1);
                let mut acc = [0u64; 4];
                let mut n = 0u64;
                for y in y0..y1.min(self.height) {
                    for x in x0..x1.min(self.width) {
                        let p = self.pixel(x, y);
                        for c in 0..4 {
                            acc[c] += p[c] as u64;
                        }
                        n += 1;
                    }
                }
                let i = ((ny * nw + nx) * 4) as usize;
                for c in 0..4 {
                    out[i + c] = (acc[c] / n.max(1)) as u8;
                }
            }
        }
        Frame {
            width: nw,
            height: nh,
            bgra: out,
            captured_at_ms: self.captured_at_ms,
        }
    }

    /// Encodes as 8-bit RGBA PNG.
    pub fn to_png(&self) -> Vec<u8> {
        let mut rgba = self.bgra.clone();
        for px in rgba.as_chunks_mut::<4>().0 {
            px.swap(0, 2);
        }
        let mut out = Vec::new();
        {
            let mut enc = png::Encoder::new(&mut out, self.width, self.height);
            enc.set_color(png::ColorType::Rgba);
            enc.set_depth(png::BitDepth::Eight);
            enc.set_compression(png::Compression::Fast);
            let mut w = enc.write_header().expect("png header");
            w.write_image_data(&rgba).expect("png data");
        }
        out
    }
}

/// Redaction fill: `--surface-strong` dark tone with a lock glyph-ish stripe.
pub const REDACTION_BGRA: [u8; 4] = [0x2c, 0x26, 0x26, 0xff];

/// Covers every rectangle (OT-001 of 004: callers must redact before use).
pub fn redact(frame: &mut Frame, rects: &[Rect]) {
    for r in rects {
        frame.fill_rect(*r, REDACTION_BGRA);
        // A lighter horizontal band marks the area as intentionally hidden.
        let band = Rect::new(r.x, r.y + r.h / 2 - 2, r.w, 4.min(r.h));
        frame.fill_rect(band, [0x70, 0x66, 0x66, 0xff]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crop_returns_the_intersection() {
        let mut f = Frame::solid(100, 100, [0, 0, 0, 255]);
        f.fill_rect(Rect::new(0, 0, 50, 50), [0, 0, 255, 255]);
        let c = f.crop(Rect::new(0, 0, 50, 50)).unwrap();
        assert_eq!((c.width, c.height), (50, 50));
        assert!(c.bgra.chunks(4).all(|p| p == [0, 0, 255, 255]));
        let edge = f.crop(Rect::new(90, 90, 50, 50)).unwrap();
        assert_eq!((edge.width, edge.height), (10, 10));
        assert!(f.crop(Rect::new(200, 200, 5, 5)).is_none());
    }

    #[test]
    fn redaction_covers_the_right_half_only() {
        let mut f = Frame::solid(200, 100, [0, 255, 0, 255]);
        redact(&mut f, &[Rect::new(100, 0, 100, 100)]);
        assert_eq!(f.pixel(10, 10), [0, 255, 0, 255]);
        assert_ne!(f.pixel(150, 10), [0, 255, 0, 255]);
        assert!(!(100..200).any(|x| f.pixel(x, 90) == [0, 255, 0, 255]));
    }

    #[test]
    fn downscale_keeps_aspect_and_average_color() {
        let f = Frame::solid(4000, 2000, [10, 20, 30, 255]);
        let d = f.downscale(2048);
        assert_eq!((d.width, d.height), (2048, 1024));
        assert_eq!(d.pixel(100, 100), [10, 20, 30, 255]);
    }

    #[test]
    fn png_signature() {
        let png = Frame::solid(2, 2, [1, 2, 3, 255]).to_png();
        assert_eq!(&png[..8], b"\x89PNG\r\n\x1a\n");
    }
}
