//! Font loading and metrics calculation.
//!
//! Loads Cascadia Mono and calculates cell dimensions for terminal rendering.
//! Uses ab_glyph for font parsing and rasterization.

use anyhow::{Context, Result};
use ab_glyph::{Font, FontVec, PxScale, ScaleFont};
use bevy::prelude::*;
use log::info;

/// Font size in points for terminal text (MVP: hardcoded 14pt).
pub const FONT_SIZE: f32 = 14.0;

/// Font metrics and loaded font data.
///
/// This resource holds the parsed font and calculated dimensions for
/// terminal cell rendering. All glyphs render to the same cell size
/// for proper terminal grid alignment.
#[derive(Resource)]
pub struct FontMetrics {
    /// Parsed font (Cascadia Mono)
    pub font: FontVec,
    /// Width of each terminal cell in pixels
    pub cell_width: f32,
    /// Height of each terminal cell in pixels
    pub cell_height: f32,
    /// Font scale for rendering at 14pt
    pub scale: PxScale,
    /// Baseline offset from top of cell
    pub baseline: f32,
}

impl FontMetrics {
    /// Load font from bytes and calculate metrics.
    ///
    /// Cell dimensions are calculated from the 'M' character, which is
    /// standard practice for monospace fonts.
    ///
    /// # Arguments
    /// * `font_bytes` - TTF/OTF font file contents
    /// * `font_size` - Font size in points (14.0 for MVP)
    ///
    /// # Returns
    /// Loaded font with calculated cell dimensions
    pub fn load(font_bytes: &[u8], font_size: f32) -> Result<Self> {
        // Parse font with ab_glyph (must own the data, so convert to Vec)
        let font = FontVec::try_from_vec(font_bytes.to_vec())
            .context("Failed to parse font file - invalid TTF/OTF format")?;

        let scale = PxScale::from(font_size);
        let scaled_font = font.as_scaled(scale);

        // Calculate cell width from 'M' character (widest in monospace fonts)
        let glyph_id = font.glyph_id('M');
        let cell_width = scaled_font.h_advance(glyph_id);

        // Calculate cell height from font vertical metrics
        let ascent = scaled_font.ascent();
        let descent = scaled_font.descent();
        let cell_height = ascent - descent;

        // Baseline is distance from top of cell to baseline
        let baseline = ascent;

        info!(
            "📐 Font metrics: cell={}×{:.1}, baseline={:.1}, ascent={:.1}, descent={:.1}",
            cell_width, cell_height, baseline, ascent, descent
        );

        Ok(Self {
            font,
            cell_width,
            cell_height,
            scale,
            baseline,
        })
    }

    /// Load Cascadia Mono from embedded bytes.
    ///
    /// This is the MVP font path - uses include_bytes!() for simplicity.
    pub fn load_cascadia_mono() -> Result<Self> {
        const CASCADIA_MONO: &[u8] = include_bytes!(
            "../assets/fonts/CascadiaMono-Regular.ttf"
        );

        Self::load(CASCADIA_MONO, FONT_SIZE)
            .context("Failed to load Cascadia Mono font")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_cascadia_mono() {
        let metrics = FontMetrics::load_cascadia_mono()
            .expect("Should load Cascadia Mono");

        // Verify reasonable dimensions (14pt monospace font)
        assert!(metrics.cell_width > 5.0 && metrics.cell_width < 15.0);
        assert!(metrics.cell_height > 10.0 && metrics.cell_height < 25.0);
        assert!(metrics.baseline > 0.0);
        assert_eq!(metrics.scale.x, FONT_SIZE);
        assert_eq!(metrics.scale.y, FONT_SIZE);
    }

    #[test]
    fn test_font_is_monospace() {
        let metrics = FontMetrics::load_cascadia_mono()
            .expect("Should load font");

        let scaled_font = metrics.font.as_scaled(metrics.scale);

        // Verify monospace: all ASCII chars should have same width
        let m_width = scaled_font.h_advance(metrics.font.glyph_id('M'));
        let i_width = scaled_font.h_advance(metrics.font.glyph_id('i'));
        let at_width = scaled_font.h_advance(metrics.font.glyph_id('@'));

        assert_eq!(m_width, i_width, "Font should be monospace (M vs i)");
        assert_eq!(m_width, at_width, "Font should be monospace (M vs @)");
    }
}
