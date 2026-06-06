use crate::geometry::Rect;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ImageFit {
    Stretch,
    Center,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ImageRef<'a> {
    pub width: u32,
    pub height: u32,
    pub pixels: &'a [u16],
}

impl<'a> ImageRef<'a> {
    pub const fn new(width: u32, height: u32, pixels: &'a [u16]) -> Self {
        Self {
            width,
            height,
            pixels,
        }
    }

    pub fn bounds_at(&self, rect: Rect, fit: ImageFit) -> Rect {
        match fit {
            ImageFit::Stretch => rect,
            ImageFit::Center => {
                let x = rect.x + rect.w.saturating_sub(self.width) as i32 / 2;
                let y = rect.y + rect.h.saturating_sub(self.height) as i32 / 2;
                Rect::new(x, y, self.width.min(rect.w), self.height.min(rect.h))
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SpriteSheet<'a> {
    pub image: ImageRef<'a>,
    pub sprite_w: u32,
    pub sprite_h: u32,
    pub columns: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ReelFrame {
    pub sprite_index: u16,
    pub duration_ms: u16,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ReelPlayer<'a> {
    pub sheet: SpriteSheet<'a>,
    pub frames: &'a [ReelFrame],
    pub repeat: bool,
    current: usize,
    elapsed_in_frame_ms: u32,
    finished: bool,
}

impl<'a> ReelPlayer<'a> {
    pub const fn new(sheet: SpriteSheet<'a>, frames: &'a [ReelFrame], repeat: bool) -> Self {
        Self {
            sheet,
            frames,
            repeat,
            current: 0,
            elapsed_in_frame_ms: 0,
            finished: false,
        }
    }

    pub fn tick(&mut self, dt_ms: u32) {
        if self.frames.is_empty() || self.finished {
            return;
        }
        self.elapsed_in_frame_ms = self.elapsed_in_frame_ms.saturating_add(dt_ms);
        loop {
            let frame = self.frames[self.current];
            let frame_ms = u32::from(frame.duration_ms).max(1);
            if self.elapsed_in_frame_ms < frame_ms {
                break;
            }
            self.elapsed_in_frame_ms -= frame_ms;
            if self.current + 1 < self.frames.len() {
                self.current += 1;
                continue;
            }
            if self.repeat {
                self.current = 0;
            } else {
                self.finished = true;
            }
            break;
        }
    }

    pub const fn is_finished(&self) -> bool {
        self.finished
    }

    pub fn restart(&mut self) {
        self.current = 0;
        self.elapsed_in_frame_ms = 0;
        self.finished = false;
    }

    pub fn current_sprite_rect(&self) -> Option<Rect> {
        let frame = self.frames.get(self.current)?;
        Some(self.sheet.sprite_rect(frame.sprite_index as u32))
    }
}

impl<'a> SpriteSheet<'a> {
    pub const fn new(image: ImageRef<'a>, sprite_w: u32, sprite_h: u32) -> Self {
        let columns = match image.width.checked_div(sprite_w) {
            Some(c) => c,
            None => 1,
        };
        Self {
            image,
            sprite_w,
            sprite_h,
            columns,
        }
    }

    pub fn sprite_rect(&self, index: u32) -> Rect {
        let columns = self.columns.max(1);
        let col = index % columns;
        let row = index / columns;
        Rect::new(
            (col * self.sprite_w) as i32,
            (row * self.sprite_h) as i32,
            self.sprite_w,
            self.sprite_h,
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ImageAtlasEntry {
    pub id: u16,
    pub rect: Rect,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ImageAtlas<'a> {
    pub image: ImageRef<'a>,
    pub entries: &'a [ImageAtlasEntry],
}

impl<'a> ImageAtlas<'a> {
    pub const fn new(image: ImageRef<'a>, entries: &'a [ImageAtlasEntry]) -> Self {
        Self { image, entries }
    }

    pub fn rect_for(&self, id: u16) -> Option<Rect> {
        self.entries
            .iter()
            .find(|entry| entry.id == id)
            .map(|e| e.rect)
    }
}

#[cfg(all(feature = "std", feature = "image-decode"))]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ImageDecodeError {
    InvalidHeader,
    Unsupported,
    InvalidData,
    Capacity,
}

#[cfg(all(feature = "std", feature = "image-decode"))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EncodedImageFormat {
    PpmAscii,
}

#[cfg(all(feature = "std", feature = "image-decode"))]
pub trait ImageDecoder {
    fn decode<const N: usize>(
        &self,
        format: EncodedImageFormat,
        data: &str,
        out_pixels: &mut heapless::Vec<u16, N>,
    ) -> Result<(u32, u32), ImageDecodeError>;
}

#[cfg(all(feature = "std", feature = "image-decode"))]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BasicImageDecoder;

#[cfg(all(feature = "std", feature = "image-decode"))]
impl ImageDecoder for BasicImageDecoder {
    fn decode<const N: usize>(
        &self,
        format: EncodedImageFormat,
        data: &str,
        out_pixels: &mut heapless::Vec<u16, N>,
    ) -> Result<(u32, u32), ImageDecodeError> {
        match format {
            EncodedImageFormat::PpmAscii => decode_ppm_ascii(data, out_pixels),
        }
    }
}

#[cfg(all(feature = "std", feature = "image-decode"))]
pub fn decode_image_with<const N: usize>(
    decoder: &impl ImageDecoder,
    format: EncodedImageFormat,
    data: &str,
    out_pixels: &mut heapless::Vec<u16, N>,
) -> Result<(u32, u32), ImageDecodeError> {
    decoder.decode(format, data, out_pixels)
}

#[cfg(all(feature = "std", feature = "image-decode"))]
pub fn decode_image_auto<const N: usize>(
    data: &str,
    out_pixels: &mut heapless::Vec<u16, N>,
) -> Result<(u32, u32), ImageDecodeError> {
    let format = if data.trim_start().starts_with("P3") {
        EncodedImageFormat::PpmAscii
    } else {
        return Err(ImageDecodeError::Unsupported);
    };
    decode_image_with(&BasicImageDecoder, format, data, out_pixels)
}

#[cfg(all(feature = "std", feature = "image-decode"))]
pub fn decode_ppm_ascii<const N: usize>(
    data: &str,
    out_pixels: &mut heapless::Vec<u16, N>,
) -> Result<(u32, u32), ImageDecodeError> {
    let mut parts = data.split_whitespace();
    if parts.next() != Some("P3") {
        return Err(ImageDecodeError::InvalidHeader);
    }
    let width: u32 = parts
        .next()
        .ok_or(ImageDecodeError::InvalidHeader)?
        .parse()
        .map_err(|_| ImageDecodeError::InvalidHeader)?;
    let height: u32 = parts
        .next()
        .ok_or(ImageDecodeError::InvalidHeader)?
        .parse()
        .map_err(|_| ImageDecodeError::InvalidHeader)?;
    let maxv: u32 = parts
        .next()
        .ok_or(ImageDecodeError::InvalidHeader)?
        .parse()
        .map_err(|_| ImageDecodeError::InvalidHeader)?;
    if maxv == 0 {
        return Err(ImageDecodeError::InvalidData);
    }
    out_pixels.clear();
    let count = width.saturating_mul(height);
    for _ in 0..count {
        let r: u32 = parts
            .next()
            .ok_or(ImageDecodeError::InvalidData)?
            .parse()
            .map_err(|_| ImageDecodeError::InvalidData)?;
        let g: u32 = parts
            .next()
            .ok_or(ImageDecodeError::InvalidData)?
            .parse()
            .map_err(|_| ImageDecodeError::InvalidData)?;
        let b: u32 = parts
            .next()
            .ok_or(ImageDecodeError::InvalidData)?
            .parse()
            .map_err(|_| ImageDecodeError::InvalidData)?;
        let r5 = ((r.saturating_mul(31)) / maxv) as u16;
        let g6 = ((g.saturating_mul(63)) / maxv) as u16;
        let b5 = ((b.saturating_mul(31)) / maxv) as u16;
        out_pixels
            .push((r5 << 11) | (g6 << 5) | b5)
            .map_err(|_| ImageDecodeError::Capacity)?;
    }
    Ok((width, height))
}
