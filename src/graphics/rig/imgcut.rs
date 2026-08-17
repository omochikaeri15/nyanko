use std::collections::HashMap;
use std::sync::Arc;

use image::{self, RgbaImage};

use crate::common::tools::file;

use super::RigError;

const PNG_MAGIC: [u8; 8] = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];

const MINIMUM_PNG_LENGTH: usize = 33;

/// A two-dimensional coordinate or dimension.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ImgVec2 {
    /// The horizontal component.
    pub x: f32,
    /// The vertical component.
    pub y: f32
}

/// A spatial bounding box defined by two points.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ImgRect {
    /// The upper-left corner of the region.
    pub min: ImgVec2,
    /// The lower-right corner of the region.
    pub max: ImgVec2
}

/// One sprite region within a texture atlas.
#[derive(Clone, Debug, PartialEq)]
pub struct SpriteCut {
    /// The region's bounds expressed as normalized texture coordinates from zero to one.
    pub uv_coordinates: ImgRect,
    /// The region's dimensions in source pixels, before normalization.
    pub original_size: ImgVec2,
    /// The region's declared name, which is empty when the cut list supplies none.
    pub name: String,
}

/// A decoded texture atlas and the sprite regions carved out of it.
#[derive(Clone, Debug, Default)]
pub struct SpriteSheet {
    /// The decoded atlas with its color channels premultiplied by alpha, shared cheaply between clones.
    pub image_data: Option<Arc<RgbaImage>>,
    /// The sprite regions carved out of the atlas, keyed by the index the model addresses them through.
    pub cuts_map: HashMap<usize, SpriteCut>,
}

impl SpriteSheet {
    /// Parses a texture atlas and its sprite cut list into a resolved sprite sheet.
    ///
    /// Color channels are premultiplied by alpha and fully transparent pixels
    /// zeroed, and sprite regions are normalized against the decoded image
    /// dimensions. A texture that fails to decode is passed through salvage
    /// before being abandoned.
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

        let image_width = image.width() as f32;
        let image_height = image.height() as f32;

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

        let mut sprite_count = 0;
        let mut data_start_index = 0;
        let mut found_header = false;

        for (index, line) in lines.iter().enumerate() {
            if line.contains(delimiter) {
                if found_header { break; }
                continue;
            }

            let Ok(count_val) = line.trim().parse::<usize>() else { continue; };

            if count_val > 0 && count_val < 10000 {
                sprite_count = count_val;
                data_start_index = index + 1;
                found_header = true;
            }
        }

        if !found_header || sprite_count == 0 {
            data_start_index = 0;
            sprite_count = lines.len();
        }

        let mut parsed_cuts = HashMap::new();

        for current_cut_index in 0..sprite_count {
            let line_index = data_start_index + current_cut_index;
            if line_index >= lines.len() { break; }

            let line = lines[line_index];
            let parts: Vec<&str> = line.split(delimiter).collect();

            if parts.len() < 4 { continue; }

            let Ok(cut_x) = parts[0].trim().parse::<f32>() else { continue; };
            let Ok(cut_y) = parts[1].trim().parse::<f32>() else { continue; };
            let Ok(cut_width) = parts[2].trim().parse::<f32>() else { continue; };
            let Ok(cut_height) = parts[3].trim().parse::<f32>() else { continue; };

            let uv_min = ImgVec2 { x: cut_x / image_width, y: cut_y / image_height };
            let uv_max = ImgVec2 { x: (cut_x + cut_width) / image_width, y: (cut_y + cut_height) / image_height };
            let cut_name = if parts.len() > 4 { parts[4].trim().to_string() } else { String::new() };

            parsed_cuts.insert(current_cut_index, SpriteCut {
                uv_coordinates: ImgRect { min: uv_min, max: uv_max },
                original_size: ImgVec2 { x: cut_width, y: cut_height },
                name: cut_name,
            });
        }

        if parsed_cuts.is_empty() {
            return Err(RigError::NoSpriteCuts);
        }

        Ok(Self { image_data: Some(Arc::new(image)), cuts_map: parsed_cuts })
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