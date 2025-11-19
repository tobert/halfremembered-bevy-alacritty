//! Render-to-texture system.
//!
//! Renders terminal grid to Image texture.
//! Exposes Handle<Image> via TerminalTexture resource.

use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat, TextureUsages};
use log::info;

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
        pixel[0] = 255; // Red
        pixel[1] = 0;
        pixel[2] = 0;
        pixel[3] = 255;
    }

    let mut image = Image::new(
        Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        texture_data,
        TextureFormat::Rgba8Unorm,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    );
    image.texture_descriptor.usage = TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST | TextureUsages::STORAGE_BINDING;

    let handle = images.add(image);

    commands.insert_resource(TerminalTexture {
        handle,
        width,
        height,
    });

    info!("✅ Terminal texture initialized");
}

