use std::sync::Arc;

use image::{self, RgbaImage};
use serde::Serialize;

use crate::common::tools::columns::{self, Column};
use crate::common::tools::file;

use super::RigError;

const PNG_MAGIC: [u8; 8] = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];

const MINIMUM_PNG_LENGTH: usize = 33;

/// One sprite region within a texture atlas, in source pixels.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub struct SpriteCut {
    /// The left edge of the region.
    pub x: i32,
    /// The top edge of the region.
    pub y: i32,
    /// The width of the region.
    pub width: i32,
    /// The height of the region.
    pub height: i32,
    /// The region's declared name, which is empty when the cut list supplies none.
    pub name: String,
}

impl SpriteCut {
    /// The column mapping this parser applies, in the order it applies it.
    ///
    /// Published so a consumer can read the layout of an `.imgcut` region row
    /// from the parser's own table instead of restating it. [`SpriteCut::name`]
    /// is the row's trailing text rather than a column.
    pub const COLUMNS: &'static [Column<Self>] = columns::columns! {
        x      : 0;
        y      : 1;
        width  : 2;
        height : 3;
    };
}

/// A decoded texture atlas and the sprite regions carved out of it.
#[derive(Clone, Debug, Default)]
pub struct SpriteSheet {
    /// The decoded atlas with its color channels premultiplied by alpha, shared cheaply between clones.
    pub image_data: Option<Arc<RgbaImage>>,
    /// The declared format version of the cut list.
    pub version: i32,
    /// The file name of the atlas the cut list names.
    pub image_name: String,
    /// The sprite regions carved out of the atlas, in the order the file declares them.
    pub cuts: Vec<SpriteCut>,
}

impl SpriteSheet {
    /// Parses a texture atlas and its sprite cut list into a resolved sprite sheet.
    ///
    /// Color channels are premultiplied by alpha and fully transparent pixels
    /// zeroed. A texture that fails to decode is passed through salvage before
    /// being abandoned.
    ///
    /// # Arguments
    /// * `png` - The raw bytes of the PNG texture atlas.
    /// * `imgcut` - The raw bytes of the `.imgcut` sprite region list.
    ///
    /// # Returns
    /// A `Result` containing the resolved `SpriteSheet` on success, or a
    /// `RigError` if the atlas could not be decoded or the cut list described no
    /// usable regions.
    pub fn parse(png: impl AsRef<[u8]>, imgcut: impl AsRef<[u8]>) -> Result<Self, RigError> {
        Self::parse_inner(png.as_ref(), imgcut.as_ref())
    }

    fn parse_inner(png: &[u8], imgcut: &[u8]) -> Result<Self, RigError> {
        let mut image_opt = image::load_from_memory(png).map(|img| img.to_rgba8()).ok();

        if image_opt.is_none() && png.starts_with(&PNG_MAGIC) {
            image_opt = Self::repair_inner(png).ok();
        }

        let mut image = image_opt.ok_or(RigError::ImageDecodeFailed)?;

        for pixel in image.pixels_mut() {
            let alpha = pixel[3] as u32;

            if alpha == 0 {
                pixel[0] = 0;
                pixel[1] = 0;
                pixel[2] = 0;
                continue;
            }

            if alpha < 255 {
                pixel[0] = ((pixel[0] as u32 * alpha) / 255) as u8;
                pixel[1] = ((pixel[1] as u32 * alpha) / 255) as u8;
                pixel[2] = ((pixel[2] as u32 * alpha) / 255) as u8;
            }
        }

        let content = file::scrub(imgcut);
        let delimiter = file::detect_separator(&content);
        let lines: Vec<&str> = content.lines().filter(|line| !line.trim().is_empty()).collect();

        let mut cursor = usize::from(lines.first().is_some_and(|line| line.trim_start().starts_with('[')));

        let version = lines.get(cursor).and_then(|line| line.trim().parse().ok()).unwrap_or(0);
        cursor += 1;

        let image_name = lines.get(cursor).map(|line| line.trim().to_string()).unwrap_or_default();
        cursor += 1;

        let count = lines.get(cursor)
            .and_then(|line| line.trim().parse::<usize>().ok())
            .ok_or(RigError::NoSpriteCuts)?;
        cursor += 1;

        let declared = count.min(lines.len().saturating_sub(cursor));
        let mut cuts = Vec::with_capacity(declared);

        for index in 0..declared {
            let row: Vec<&str> = lines[cursor + index].split(delimiter).collect();
            let mut cut = SpriteCut::default();

            let trailing = columns::apply(&row, SpriteCut::COLUMNS, &mut cut);
            cut.name = row.get(trailing).map(|text| text.trim().to_string()).unwrap_or_default();

            cuts.push(cut);
        }

        if cuts.is_empty() {
            return Err(RigError::NoSpriteCuts);
        }

        Ok(Self { image_data: Some(Arc::new(image)), version, image_name, cuts })
    }

    /// Salvages a corrupted or truncated PNG stream into the largest decodable image.
    ///
    /// Chunk lengths and checksums are rewritten to match the bytes present, and
    /// the declared height is binary-searched downwards for the tallest prefix
    /// the decoder accepts. That prefix is composited onto a canvas of the
    /// original dimensions so sprite coordinates still resolve.
    ///
    /// Costs a full decode attempt per search step.
    ///
    /// # Arguments
    /// * `png` - The raw bytes of the damaged PNG stream.
    ///
    /// # Returns
    /// A `Result` containing the recovered image on success, or a `RigError` if
    /// the stream was too short to carry a header or no prefix of it decoded.
    pub fn repair(png: impl AsRef<[u8]>) -> Result<RgbaImage, RigError> {
        Self::repair_inner(png.as_ref())
    }

    fn repair_inner(png: &[u8]) -> Result<RgbaImage, RigError> {
        if png.len() < MINIMUM_PNG_LENGTH {
            return Err(RigError::TruncatedHeader);
        }

        fn calculate_crc32(chunk_type: &[u8], chunk_data: &[u8]) -> u32 {
            let mut crc_value = 0xFFFFFFFFu32;
            for &byte in chunk_type.iter().chain(chunk_data.iter()) {
                crc_value ^= byte as u32;
                for _ in 0..8 {
                    crc_value = if (crc_value & 1) != 0 {
                        (crc_value >> 1) ^ 0xEDB88320
                    } else {
                        crc_value >> 1
                    };
                }
            }
            crc_value ^ 0xFFFFFFFFu32
        }

        fn sanitize_png_chunks(bytes: &[u8]) -> Vec<u8> {
            let mut fixed = Vec::with_capacity(bytes.len() + 12);
            fixed.extend_from_slice(&bytes[0..8]);

            let mut read_offset = 8;
            let mut found_iend = false;

            while read_offset + 8 <= bytes.len() {
                let len_bytes = [bytes[read_offset], bytes[read_offset+1], bytes[read_offset+2], bytes[read_offset+3]];
                let chunk_len = u32::from_be_bytes(len_bytes) as usize;
                let chunk_type = &bytes[read_offset+4..read_offset+8];

                if read_offset + 8 + chunk_len + 4 > bytes.len() {
                    let avail = bytes.len() - (read_offset + 8);
                    fixed.extend_from_slice(&(avail as u32).to_be_bytes());
                    fixed.extend_from_slice(chunk_type);
                    fixed.extend_from_slice(&bytes[read_offset+8 .. read_offset+8+avail]);

                    let true_crc = calculate_crc32(chunk_type, &bytes[read_offset+8 .. read_offset+8+avail]);
                    fixed.extend_from_slice(&true_crc.to_be_bytes());
                    break;
                }

                let chunk_data = &bytes[read_offset+8 .. read_offset+8+chunk_len];
                let true_crc = calculate_crc32(chunk_type, chunk_data);

                fixed.extend_from_slice(&len_bytes);
                fixed.extend_from_slice(chunk_type);
                fixed.extend_from_slice(chunk_data);
                fixed.extend_from_slice(&true_crc.to_be_bytes());

                if chunk_type == b"IEND" {
                    found_iend = true;
                    break;
                }

                read_offset += 8 + chunk_len + 4;
            }

            if !found_iend {
                fixed.extend_from_slice(&[0, 0, 0, 0, b'I', b'E', b'N', b'D', 0xAE, 0x42, 0x60, 0x82]);
            }

            fixed
        }

        fn patch_png_height(sanitized: &[u8], new_height: u32) -> Vec<u8> {
            let mut patched = sanitized.to_vec();
            patched[20..24].copy_from_slice(&new_height.to_be_bytes());

            let crc = calculate_crc32(&patched[12..29], &[]);
            patched[29..33].copy_from_slice(&crc.to_be_bytes());

            patched
        }

        let sanitized = sanitize_png_chunks(png);

        if sanitized.len() < MINIMUM_PNG_LENGTH {
            return Err(RigError::TruncatedHeader);
        }

        let (Some(width_bytes), Some(height_bytes)) = (
            sanitized.get(16..20).and_then(|slice| <[u8; 4]>::try_from(slice).ok()),
            sanitized.get(20..24).and_then(|slice| <[u8; 4]>::try_from(slice).ok()),
        ) else {
            return Err(RigError::TruncatedHeader);
        };

        let orig_width = u32::from_be_bytes(width_bytes);
        let orig_height = u32::from_be_bytes(height_bytes).min(10000);

        let mut low = 1;
        let mut high = orig_height;
        let mut best_img = None;

        while low <= high {
            let mid = low + (high - low) / 2;
            let test_buffer = patch_png_height(&sanitized, mid);

            if let Ok(img) = image::load_from_memory(&test_buffer) {
                best_img = Some(img.to_rgba8());
                low = mid + 1;
            } else {
                high = mid - 1;
            }
        }

        let partial = best_img.ok_or(RigError::ImageDecodeFailed)?;

        let mut full_canvas = RgbaImage::new(orig_width, orig_height);
        for y in 0..partial.height() {
            for x in 0..partial.width() {
                full_canvas.put_pixel(x, y, *partial.get_pixel(x, y));
            }
        }

        Ok(full_canvas)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cut_columns_map_one_field_each() {
        columns::assert_one_field_per_column(SpriteCut::COLUMNS);
    }
}
