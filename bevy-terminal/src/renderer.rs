//! Render-to-texture system.
//!
//! Renders terminal grid to Image texture.
//! Exposes Handle<Image> via TerminalTexture resource.

use alacritty_terminal::index::{Column, Line};
use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use log::{info, error, warn};

use crate::atlas::GlyphAtlas;
use crate::colors::TOKYO_NIGHT_BG;
use crate::terminal::TerminalState;

/// Resource exposing the terminal texture for game use.
///
/// Contains a Handle<Image> that can be used as a sprite, UI element, or material.
/// The texture updates every frame based on terminal grid state.
#[derive(Resource)]
pub struct TerminalTexture {
    /// Bevy image handle for the terminal texture
    pub handle: Handle<Image>,
    /// Texture width in pixels
    pub width: u32,
    /// Texture height in pixels
    pub height: u32,
}

/// Initialize terminal texture resource.
///
/// Creates an RGBA texture sized to fit the terminal grid with current cell dimensions.
/// Runs once at startup after atlas is ready.
pub fn initialize_terminal_texture(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    atlas: Res<GlyphAtlas>,
    term_state: Res<TerminalState>,
) {
    let width = atlas.cell_width * term_state.cols as u32;
    let height = atlas.cell_height * term_state.rows as u32;

    info!(
        "🖼️  Creating terminal texture: {}×{} pixels ({}×{} cells)",
        width, height, term_state.cols, term_state.rows
    );

    // Create RGBA texture filled with background color
    let bg = TOKYO_NIGHT_BG;
    let mut texture_data = vec![0u8; (width * height * 4) as usize];
    for pixel in texture_data.chunks_exact_mut(4) {
        pixel[0] = bg[0];
        pixel[1] = bg[1];
        pixel[2] = bg[2];
        pixel[3] = 255;
    }

    let image = Image::new(
        Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        texture_data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    );

    let handle = images.add(image);

    commands.insert_resource(TerminalTexture {
        handle,
        width,
        height,
    });

    info!("✅ Terminal texture initialized");
}

/// Render terminal grid to texture.
///
/// System: Update
/// Runs: Every frame
///
/// Reads terminal grid state and renders each cell to the texture using the glyph atlas.
/// Applies foreground and background colors from terminal cells.
pub fn render_terminal_to_texture(
    term_state: Res<TerminalState>,
    atlas: Res<GlyphAtlas>,
    terminal_texture: Res<TerminalTexture>,
    mut images: ResMut<Assets<Image>>,
) {
    let Some(image) = images.get_mut(&terminal_texture.handle) else {
        error!("🚨 Renderer: Failed to get terminal texture image!");
        return;
    };

    let Some(ref mut image_data) = image.data else {
        error!("🚨 Renderer: Terminal texture has no data");
        return;
    };

    let term = term_state.term.lock();
    let grid = term.grid();

    // Clear texture with background color
    let bg = TOKYO_NIGHT_BG;
    for pixel in image_data.chunks_exact_mut(4) {
        pixel[0] = bg[0];
        pixel[1] = bg[1];
        pixel[2] = bg[2];
        pixel[3] = 255;
    }

    // Track rendering stats
    let mut chars_rendered = 0;
    let mut chars_skipped_space = 0;
    let mut chars_missing_glyph = 0;

    // Render each cell
    for row in 0..term_state.rows {
        for col in 0..term_state.cols {
            let line = Line(row as i32);
            let column = Column(col);

            let cell = &grid[line][column];

            // Get character from cell
            let character = cell.c;

            // Skip rendering for space characters (background already cleared)
            if character == ' ' || character == '\0' {
                chars_skipped_space += 1;
                continue;
            }

            // Get glyph UV from atlas
            let Some(uv) = atlas.get_uv(character) else {
                // Character not in atlas, skip it
                if chars_missing_glyph == 0 {
                    warn!("⚠️  Renderer: Character '{}' (U+{:04X}) not in atlas", character, character as u32);
                }
                chars_missing_glyph += 1;
                continue;
            };

            chars_rendered += 1;

            // Get colors from cell
            let fg_color = convert_alacritty_color(cell.fg);
            let bg_color = convert_alacritty_color(cell.bg);

            // Calculate destination position in texture
            let dest_x = col * atlas.cell_width as usize;
            let dest_y = row * atlas.cell_height as usize;

            // Blit glyph from atlas to terminal texture
            blit_glyph(
                image_data,
                &atlas.texture_data,
                atlas.atlas_width,
                atlas.atlas_height,
                terminal_texture.width,
                dest_x as u32,
                dest_y as u32,
                atlas.cell_width,
                atlas.cell_height,
                uv,
                fg_color,
                bg_color,
            );
        }
    }

    // Log rendering stats (always log in tests to help debugging)
    #[cfg(test)]
    println!(
        "🎨 Renderer: {} chars rendered, {} spaces skipped, {} missing glyphs",
        chars_rendered, chars_skipped_space, chars_missing_glyph
    );

    // Log in production only when there's interesting activity
    #[cfg(not(test))]
    if chars_rendered > 0 || chars_missing_glyph > 0 {
        info!(
            "🎨 Renderer: {} chars rendered, {} spaces skipped, {} missing glyphs",
            chars_rendered, chars_skipped_space, chars_missing_glyph
        );
    }
}

/// Convert alacritty color to RGB array.
///
/// Handles named colors (using Tokyo Night theme) and RGB colors.
fn convert_alacritty_color(color: alacritty_terminal::vte::ansi::Color) -> [u8; 3] {
    use alacritty_terminal::vte::ansi::Color;

    match color {
        Color::Named(named) => {
            use alacritty_terminal::vte::ansi::NamedColor;
            match named {
                NamedColor::Black => [0x1a, 0x1b, 0x26],
                NamedColor::Red => [0xf7, 0x76, 0x8e],
                NamedColor::Green => [0x9e, 0xce, 0x6a],
                NamedColor::Yellow => [0xe0, 0xaf, 0x68],
                NamedColor::Blue => [0x7a, 0xa2, 0xf7],
                NamedColor::Magenta => [0xbb, 0x9a, 0xf7],
                NamedColor::Cyan => [0x7d, 0xcf, 0xff],
                NamedColor::White => [0xc0, 0xca, 0xf5],
                NamedColor::BrightBlack => [0x41, 0x4b, 0x6b],
                NamedColor::BrightRed => [0xf7, 0x76, 0x8e],
                NamedColor::BrightGreen => [0x9e, 0xce, 0x6a],
                NamedColor::BrightYellow => [0xe0, 0xaf, 0x68],
                NamedColor::BrightBlue => [0x7a, 0xa2, 0xf7],
                NamedColor::BrightMagenta => [0xbb, 0x9a, 0xf7],
                NamedColor::BrightCyan => [0x7d, 0xcf, 0xff],
                NamedColor::BrightWhite => [0xc0, 0xca, 0xf5],
                NamedColor::Foreground => [0xc0, 0xca, 0xf5],
                NamedColor::Background => TOKYO_NIGHT_BG,
                _ => [0xc0, 0xca, 0xf5], // Default to foreground
            }
        }
        Color::Spec(rgb) => [rgb.r, rgb.g, rgb.b],
        Color::Indexed(index) => {
            // 256-color palette - for MVP, just use a simple mapping
            // Could be enhanced with full xterm-256 palette
            match index {
                0 => [0x1a, 0x1b, 0x26],  // Black
                1 => [0xf7, 0x76, 0x8e],  // Red
                2 => [0x9e, 0xce, 0x6a],  // Green
                3 => [0xe0, 0xaf, 0x68],  // Yellow
                4 => [0x7a, 0xa2, 0xf7],  // Blue
                5 => [0xbb, 0x9a, 0xf7],  // Magenta
                6 => [0x7d, 0xcf, 0xff],  // Cyan
                7 => [0xc0, 0xca, 0xf5],  // White
                _ => [0xc0, 0xca, 0xf5],  // Default
            }
        }
    }
}

/// Blit a glyph from atlas to destination texture.
///
/// Composites the glyph with foreground color over background color using alpha blending.
#[allow(clippy::too_many_arguments)]
fn blit_glyph(
    dest_data: &mut [u8],
    atlas_data: &[u8],
    atlas_width: u32,
    atlas_height: u32,
    dest_width: u32,
    dest_x: u32,
    dest_y: u32,
    cell_width: u32,
    cell_height: u32,
    uv: &Rect,
    fg_color: [u8; 3],
    bg_color: [u8; 3],
) {
    // Calculate atlas source rectangle in pixels
    let atlas_src_x = (uv.min.x * atlas_width as f32) as u32;
    let atlas_src_y = (uv.min.y * atlas_height as f32) as u32; // Fixed per Gemini review

    // Blit each pixel
    let mut bounds_failures = 0;
    for y in 0..cell_height {
        for x in 0..cell_width {
            let atlas_pixel_x = atlas_src_x + x;
            let atlas_pixel_y = atlas_src_y + y;

            let dest_pixel_x = dest_x + x;
            let dest_pixel_y = dest_y + y;

            // Atlas pixel index (RGBA)
            let atlas_idx = ((atlas_pixel_y * atlas_width + atlas_pixel_x) * 4) as usize;

            // Destination pixel index (RGBA)
            let dest_idx = ((dest_pixel_y * dest_width + dest_pixel_x) * 4) as usize;

            // Bounds check
            if atlas_idx + 3 >= atlas_data.len() || dest_idx + 3 >= dest_data.len() {
                if bounds_failures == 0 {
                    error!(
                        "🚨 Blit bounds error: atlas_idx={}, atlas_len={}, dest_idx={}, dest_len={}",
                        atlas_idx, atlas_data.len(), dest_idx, dest_data.len()
                    );
                }
                bounds_failures += 1;
                continue;
            }

            // Get alpha from atlas (white glyph with alpha)
            let alpha = atlas_data[atlas_idx + 3];

            if alpha == 0 {
                // Fully transparent - use background color
                dest_data[dest_idx] = bg_color[0];
                dest_data[dest_idx + 1] = bg_color[1];
                dest_data[dest_idx + 2] = bg_color[2];
                dest_data[dest_idx + 3] = 255;
            } else if alpha == 255 {
                // Fully opaque - use foreground color
                dest_data[dest_idx] = fg_color[0];
                dest_data[dest_idx + 1] = fg_color[1];
                dest_data[dest_idx + 2] = fg_color[2];
                dest_data[dest_idx + 3] = 255;
            } else {
                // Alpha blend foreground over background
                let alpha_f = alpha as f32 / 255.0;
                let inv_alpha = 1.0 - alpha_f;

                dest_data[dest_idx] =
                    (fg_color[0] as f32 * alpha_f + bg_color[0] as f32 * inv_alpha) as u8;
                dest_data[dest_idx + 1] =
                    (fg_color[1] as f32 * alpha_f + bg_color[1] as f32 * inv_alpha) as u8;
                dest_data[dest_idx + 2] =
                    (fg_color[2] as f32 * alpha_f + bg_color[2] as f32 * inv_alpha) as u8;
                dest_data[dest_idx + 3] = 255;
            }
        }
    }
}
