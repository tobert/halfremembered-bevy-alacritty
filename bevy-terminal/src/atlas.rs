//! Glyph atlas generation for high-quality rendering.
//!
//! Pre-renders all required characters to a large texture atlas.
//! Supports ASCII, box-drawing, and block element characters.

use ab_glyph::{point, Font, Glyph, ScaleFont};
use anyhow::{Context, Result};
use bevy::prelude::*;
use std::collections::HashMap;

use crate::font::FontMetrics;

/// Atlas texture size (4096×4096 for high quality).
pub const ATLAS_SIZE: u32 = 4096;

/// Character sets to pre-render in the atlas.
pub struct CharacterSets;

impl CharacterSets {
    /// ASCII printable characters (32-126).
    pub fn ascii() -> Vec<char> {
        (32..=126).map(|c| c as u8 as char).collect()
    }

    /// Box-drawing characters (U+2500-257F).
    ///
    /// These must connect perfectly for terminal UIs (borders, lines, etc.).
    pub fn box_drawing() -> Vec<char> {
        (0x2500..=0x257F).filter_map(char::from_u32).collect()
    }

    /// Block elements (U+2580-259F).
    ///
    /// Used for graphics, progress bars, and shading in terminal UIs.
    pub fn block_elements() -> Vec<char> {
        (0x2580..=0x259F).filter_map(char::from_u32).collect()
    }

    /// All characters for MVP atlas.
    pub fn all_mvp() -> Vec<char> {
        let mut chars = Vec::new();
        chars.extend(Self::ascii());
        chars.extend(Self::box_drawing());
        chars.extend(Self::block_elements());
        chars
    }
}

/// Pre-rendered glyph atlas texture.
///
/// Contains all required characters rasterized to a single RGBA texture.
/// UV coordinates allow fast lookup for rendering.
#[derive(Resource)]
pub struct GlyphAtlas {
    /// RGBA pixel data for atlas texture
    pub texture_data: Vec<u8>,
    /// Atlas width in pixels
    pub atlas_width: u32,
    /// Atlas height in pixels
    pub atlas_height: u32,
    /// Character to UV coordinate mapping
    pub uv_map: HashMap<char, Rect>,
    /// Cell width in pixels
    pub cell_width: u32,
    /// Cell height in pixels
    pub cell_height: u32,
}

impl GlyphAtlas {
    /// Generate atlas from font metrics and character set.
    ///
    /// Rasterizes all characters to a single texture and builds UV map.
    ///
    /// # Arguments
    /// * `font_metrics` - Loaded font with cell dimensions
    /// * `chars` - Characters to pre-render
    ///
    /// # Returns
    /// Atlas texture with UV coordinate map
    pub fn generate(font_metrics: &FontMetrics, chars: &[char]) -> Result<Self> {
        let atlas_width = ATLAS_SIZE;
        let atlas_height = ATLAS_SIZE;

        // Allocate RGBA texture (initialized to transparent black)
        let mut texture_data = vec![0u8; (atlas_width * atlas_height * 4) as usize];

        // Cell dimensions (rounded up for pixel alignment)
        let cell_width = font_metrics.cell_width.ceil() as u32;
        let cell_height = font_metrics.cell_height.ceil() as u32;

        // Calculate atlas layout
        let cells_per_row = atlas_width / cell_width;
        let cells_per_column = atlas_height / cell_height;
        let max_chars = (cells_per_row * cells_per_column) as usize;

        if chars.len() > max_chars {
            anyhow::bail!(
                "Atlas too small: {} characters requested, but only {} fit in {}×{} with {}×{} cells",
                chars.len(),
                max_chars,
                atlas_width,
                atlas_height,
                cell_width,
                cell_height
            );
        }

        let mut uv_map = HashMap::new();
        let scaled_font = font_metrics.font.as_scaled(font_metrics.scale);

        info!(
            "🎨 Generating glyph atlas: {} chars, {}×{} cells, {}×{} atlas",
            chars.len(),
            cell_width,
            cell_height,
            atlas_width,
            atlas_height
        );

        // Rasterize each character
        for (index, &character) in chars.iter().enumerate() {
            let column = (index as u32) % cells_per_row;
            let row = (index as u32) / cells_per_row;

            let cell_x = column * cell_width;
            let cell_y = row * cell_height;

            // Rasterize glyph to atlas
            rasterize_glyph(
                &scaled_font,
                character,
                font_metrics.baseline,
                &mut texture_data,
                atlas_width,
                cell_x,
                cell_y,
                cell_width,
                cell_height,
            );

            // Calculate UV coordinates (normalized 0.0-1.0)
            let uv = Rect {
                min: Vec2::new(
                    cell_x as f32 / atlas_width as f32,
                    cell_y as f32 / atlas_height as f32,
                ),
                max: Vec2::new(
                    (cell_x + cell_width) as f32 / atlas_width as f32,
                    (cell_y + cell_height) as f32 / atlas_height as f32,
                ),
            };
            uv_map.insert(character, uv);
        }

        info!("✅ Atlas generated: {} glyphs", uv_map.len());

        Ok(Self {
            texture_data,
            atlas_width,
            atlas_height,
            uv_map,
            cell_width,
            cell_height,
        })
    }

    /// Generate atlas with all MVP characters.
    pub fn generate_mvp(font_metrics: &FontMetrics) -> Result<Self> {
        let chars = CharacterSets::all_mvp();
        Self::generate(font_metrics, &chars)
            .context("Failed to generate MVP glyph atlas")
    }

    /// Get UV coordinates for a character.
    ///
    /// Returns None if character is not in atlas.
    pub fn get_uv(&self, character: char) -> Option<&Rect> {
        self.uv_map.get(&character)
    }
}

/// Rasterize a single glyph to the atlas texture.
///
/// Renders the glyph with anti-aliasing and writes to the RGBA buffer.
fn rasterize_glyph<F: Font>(
    scaled_font: &impl ScaleFont<F>,
    character: char,
    baseline: f32,
    texture_data: &mut [u8],
    atlas_width: u32,
    cell_x: u32,
    cell_y: u32,
    cell_width: u32,
    cell_height: u32,
) {
    // Get glyph outline
    let glyph_id = scaled_font.font().glyph_id(character);
    let glyph = Glyph {
        id: glyph_id,
        scale: scaled_font.scale(),
        position: point(0.0, 0.0),
    };

    let outlined = match scaled_font.outline_glyph(glyph) {
        Some(outlined) => outlined,
        None => {
            // Glyph has no outline (e.g., space character)
            return;
        }
    };

    let bounds = outlined.px_bounds();

    // Center glyph horizontally in cell
    let glyph_width = bounds.width();
    let horizontal_offset = ((cell_width as f32 - glyph_width) / 2.0).max(0.0);

    // Position glyph on baseline
    let vertical_offset = baseline - bounds.min.y;

    // Rasterize glyph
    outlined.draw(|glyph_x, glyph_y, coverage| {
        // Calculate pixel position in cell
        let pixel_x = horizontal_offset + glyph_x as f32;
        let pixel_y = vertical_offset + glyph_y as f32;

        // Convert to atlas coordinates
        let atlas_x = cell_x as f32 + pixel_x;
        let atlas_y = cell_y as f32 + pixel_y;

        // Bounds check
        if atlas_x < 0.0
            || atlas_y < 0.0
            || atlas_x >= atlas_width as f32
            || atlas_y >= atlas_width as f32
        {
            return;
        }

        let atlas_x = atlas_x as u32;
        let atlas_y = atlas_y as u32;

        // Check if pixel is within cell bounds
        if atlas_x >= cell_x + cell_width || atlas_y >= cell_y + cell_height {
            return;
        }

        // Calculate pixel index in RGBA buffer
        let pixel_index = ((atlas_y * atlas_width + atlas_x) * 4) as usize;

        if pixel_index + 3 < texture_data.len() {
            // Write white glyph with alpha (coverage determines transparency)
            let alpha = (coverage * 255.0) as u8;
            texture_data[pixel_index] = 255; // R
            texture_data[pixel_index + 1] = 255; // G
            texture_data[pixel_index + 2] = 255; // B
            texture_data[pixel_index + 3] = alpha; // A
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::font::FontMetrics;

    #[test]
    fn test_character_sets() {
        let ascii = CharacterSets::ascii();
        assert_eq!(ascii.len(), 95); // 32-126 inclusive
        assert_eq!(ascii[0], ' ');
        assert_eq!(ascii[ascii.len() - 1], '~');

        let box_drawing = CharacterSets::box_drawing();
        assert!(!box_drawing.is_empty());

        let blocks = CharacterSets::block_elements();
        assert!(!blocks.is_empty());

        let all = CharacterSets::all_mvp();
        assert_eq!(all.len(), ascii.len() + box_drawing.len() + blocks.len());
    }

    #[test]
    fn test_generate_atlas() {
        let font_metrics = FontMetrics::load_cascadia_mono()
            .expect("Should load font");

        let chars = vec!['A', 'B', 'C', '1', '2', '3'];
        let atlas = GlyphAtlas::generate(&font_metrics, &chars)
            .expect("Should generate atlas");

        // Verify atlas properties
        assert_eq!(atlas.atlas_width, ATLAS_SIZE);
        assert_eq!(atlas.atlas_height, ATLAS_SIZE);
        assert_eq!(atlas.texture_data.len(), (ATLAS_SIZE * ATLAS_SIZE * 4) as usize);
        assert_eq!(atlas.uv_map.len(), 6);

        // Verify UV coordinates are valid (0.0-1.0 range)
        for &ch in &chars {
            let uv = atlas.get_uv(ch).expect("Character should be in atlas");
            assert!(uv.min.x >= 0.0 && uv.min.x <= 1.0);
            assert!(uv.min.y >= 0.0 && uv.min.y <= 1.0);
            assert!(uv.max.x >= 0.0 && uv.max.x <= 1.0);
            assert!(uv.max.y >= 0.0 && uv.max.y <= 1.0);
            assert!(uv.max.x > uv.min.x);
            assert!(uv.max.y > uv.min.y);
        }
    }

    #[test]
    fn test_generate_mvp_atlas() {
        let font_metrics = FontMetrics::load_cascadia_mono()
            .expect("Should load font");

        let atlas = GlyphAtlas::generate_mvp(&font_metrics)
            .expect("Should generate MVP atlas");

        // Should contain ASCII + box-drawing + blocks
        let expected_count = 95 + 128 + 32; // ASCII + box-drawing + blocks
        assert_eq!(atlas.uv_map.len(), expected_count);

        // Verify key characters are present
        assert!(atlas.get_uv('A').is_some());
        assert!(atlas.get_uv('z').is_some());
        assert!(atlas.get_uv('0').is_some());
        assert!(atlas.get_uv('@').is_some());
        assert!(atlas.get_uv(' ').is_some());
    }
}
