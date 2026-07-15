use std::collections::HashMap;
use std::sync::Arc;

use image::{self, RgbaImage};

use crate::common::tools::file;

/// A simple two-dimensional vector representing spatial coordinates or dimensions in floating-point precision.
#[derive(Clone, Copy, Debug, Default)]
pub struct ImgVec2 {
    pub x: f32,
    pub y: f32
}

/// A spatial bounding box defined by two points.
#[derive(Clone, Copy, Debug, Default)]
pub struct ImgRect {
    pub min: ImgVec2,
    pub max: ImgVec2
}

/// Represents a discrete sub-region within a master texture atlas.
#[derive(Clone, Debug)]
pub struct SpriteCut {
    pub uv_coordinates: ImgRect,
    pub original_size: ImgVec2,
    pub name: String,
}

/// The comprehensive parsed representation of a graphical texture atlas and its spatial mappings.
#[derive(Clone, Default)]
pub struct SpriteSheet {
    pub image_data: Option<Arc<RgbaImage>>,
    pub cuts_map: HashMap<usize, SpriteCut>,
}

impl SpriteSheet {
    #[inline(always)]
    pub fn parse(png: impl AsRef<[u8]>, imgcut: impl AsRef<[u8]>) -> Option<Self> {
        Self::parse_inner(png.as_ref(), imgcut.as_ref())
    }

    fn parse_inner(png: &[u8], imgcut: &[u8]) -> Option<Self> {
        let mut image_opt = image::load_from_memory(png).map(|img| img.to_rgba8()).ok();

        if image_opt.is_none() && png.len() > 33 && &png[0..8] == &[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A] {
            image_opt = Self::repair_inner(png);
        }

        let mut image = image_opt?;

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

        Some(Self { image_data: Some(Arc::new(image)), cuts_map: parsed_cuts })
    }

    /// Salvages corrupted or truncated PNG streams.
    #[inline(always)]
    pub fn repair(png: impl AsRef<[u8]>) -> Option<RgbaImage> {
        Self::repair_inner(png.as_ref())
    }

    fn repair_inner(png: &[u8]) -> Option<RgbaImage> {

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

        let orig_width = u32::from_be_bytes([sanitized[16], sanitized[17], sanitized[18], sanitized[19]]);
        let orig_height = u32::from_be_bytes([sanitized[20], sanitized[21], sanitized[22], sanitized[23]]).min(10000);

        let mut low = 1;
        let mut high = orig_height;
        let mut best_img = None;

        // Binary-search to extract the maximum surviving partial image
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

        if let Some(partial) = best_img {
            let mut full_canvas = RgbaImage::new(orig_width, orig_height);
            for y in 0..partial.height() {
                for x in 0..partial.width() {
                    full_canvas.put_pixel(x, y, *partial.get_pixel(x, y));
                }
            }
            return Some(full_canvas);
        }

        None
    }
}