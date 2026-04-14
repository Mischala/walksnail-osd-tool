use std::collections::HashMap;

use image::{
    imageops::{resize, FilterType},
    RgbaImage,
};

use crate::{
    font::{self, CharacterSize},
    osd::{self, OsdOptions},
};

#[must_use]
#[allow(clippy::cast_precision_loss)]
pub fn get_character_size(width: u32, height: u32) -> CharacterSize {
    let is_4_3 = (width as f32 / height as f32) < 1.5;
    match height {
        540 => CharacterSize::Race,
        720 => CharacterSize::Small,
        1080 => {
            if is_4_3 {
                CharacterSize::Small
            } else {
                CharacterSize::Large
            }
        }
        1440 => {
            if is_4_3 {
                CharacterSize::Large
            } else {
                CharacterSize::XLarge
            }
        }
        2160 => CharacterSize::Ultra,
        _ => CharacterSize::Large,
    }
}

/// Compute the scaled glyph for a given character index, or return None if the
/// character doesn't exist in the font.
#[inline]
fn get_scaled_glyph(
    font: &font::FontFile,
    character_index: u16,
    base_character_size: &CharacterSize,
    scaled_width: u32,
    scaled_height: u32,
) -> Option<RgbaImage> {
    font.get_character(character_index as usize).map(|character_image| {
        if scaled_width != base_character_size.width() || scaled_height != base_character_size.height() {
            resize(character_image, scaled_width, scaled_height, FilterType::Lanczos3)
        } else {
            character_image.clone()
        }
    })
}

#[inline]
pub fn fast_overlay_rgba(bottom: &mut RgbaImage, top: &RgbaImage, x: i64, y: i64) {
    let bottom_width = bottom.width() as i64;
    let bottom_height = bottom.height() as i64;
    let top_width = top.width() as i64;
    let top_height = top.height() as i64;

    if x >= bottom_width || y >= bottom_height || x + top_width <= 0 || y + top_height <= 0 {
        return;
    }

    let start_x = 0.max(-x);
    let start_y = 0.max(-y);
    let end_x = top_width.min(bottom_width - x);
    let end_y = top_height.min(bottom_height - y);

    let bottom_stride = (bottom_width as usize) * 4;
    let top_stride = (top_width as usize) * 4;

    let bottom_buf = bottom.as_mut();
    let top_buf = top.as_ref();

    for dy in start_y..end_y {
        let bottom_y = y + dy;
        let top_start = (dy as usize * top_stride) + (start_x as usize) * 4;
        let bottom_start = (bottom_y as usize * bottom_stride) + ((x + start_x) as usize) * 4;
        let len = ((end_x - start_x) as usize) * 4;

        let top_slice = &top_buf[top_start..top_start + len];
        let bottom_slice = &mut bottom_buf[bottom_start..bottom_start + len];

        for i in (0..len).step_by(4) {
            let alpha = top_slice[i + 3];
            if alpha == 0 {
                continue;
            } else if alpha == 255 {
                bottom_slice[i] = top_slice[i];
                bottom_slice[i + 1] = top_slice[i + 1];
                bottom_slice[i + 2] = top_slice[i + 2];
                bottom_slice[i + 3] = 255;
            } else {
                let a = alpha as u16;
                let inv_a = 255 - a;
                bottom_slice[i] = ((top_slice[i] as u16 * a + bottom_slice[i] as u16 * inv_a) / 255) as u8;
                bottom_slice[i + 1] = ((top_slice[i + 1] as u16 * a + bottom_slice[i + 1] as u16 * inv_a) / 255) as u8;
                bottom_slice[i + 2] = ((top_slice[i + 2] as u16 * a + bottom_slice[i + 2] as u16 * inv_a) / 255) as u8;
            }
        }
    }
}

/// Overlay OSD glyphs onto a frame image (single-use, no caching).
/// Used by the OSD preview path where only a single frame is rendered.
#[inline]
#[allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]
pub fn overlay_osd(
    image: &mut RgbaImage,
    osd_frame: &osd::Frame,
    font: &font::FontFile,
    osd_options: &OsdOptions,
    offset: (i32, i32),
) {
    let base_character_size = get_character_size(image.width(), image.height());
    let scale_factor = osd_options.scale / 100.0;
    let scaled_width = (base_character_size.width() as f32 * scale_factor).round() as u32;
    let scaled_height = (base_character_size.height() as f32 * scale_factor).round() as u32;

    for character in &osd_frame.glyphs {
        if character.index == 0 || osd_options.get_mask(&character.grid_position) {
            continue;
        }
        if let Some(scaled_image) =
            get_scaled_glyph(font, character.index, &base_character_size, scaled_width, scaled_height)
        {
            let grid_position = &character.grid_position;
            #[allow(clippy::cast_possible_wrap, clippy::semicolon_if_nothing_returned)]
            fast_overlay_rgba(
                image,
                &scaled_image,
                (grid_position.x as i32 * scaled_width as i32 + osd_options.position.x + offset.0).into(),
                (grid_position.y as i32 * scaled_height as i32 + osd_options.position.y + offset.1).into(),
            );
        }
    }
}

/// Overlay OSD glyphs onto a frame image with a glyph cache.
/// The cache persists across frames so each unique glyph index is resized only once.
#[inline]
#[allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::implicit_hasher
)]
pub fn overlay_osd_cached(
    image: &mut RgbaImage,
    osd_frame: &osd::Frame,
    font: &font::FontFile,
    osd_options: &OsdOptions,
    offset: (i32, i32),
    glyph_cache: &mut HashMap<u16, RgbaImage>,
) {
    let base_character_size = get_character_size(image.width(), image.height());
    let scale_factor = osd_options.scale / 100.0;
    let scaled_width = (base_character_size.width() as f32 * scale_factor).round() as u32;
    let scaled_height = (base_character_size.height() as f32 * scale_factor).round() as u32;

    for character in &osd_frame.glyphs {
        if character.index == 0 || osd_options.get_mask(&character.grid_position) {
            continue;
        }

        let scaled_image = glyph_cache.entry(character.index).or_insert_with(|| {
            get_scaled_glyph(font, character.index, &base_character_size, scaled_width, scaled_height)
                .unwrap_or_else(|| RgbaImage::new(scaled_width, scaled_height))
        });

        let grid_position = &character.grid_position;
        #[allow(clippy::cast_possible_wrap)]
        fast_overlay_rgba(
            image,
            scaled_image,
            (grid_position.x as i32 * scaled_width as i32 + osd_options.position.x + offset.0).into(),
            (grid_position.y as i32 * scaled_height as i32 + osd_options.position.y + offset.1).into(),
        );
    }
}
