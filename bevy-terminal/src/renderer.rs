//! Render-to-texture system.
//!
//! Renders terminal grid to Image texture.
//! Exposes Handle<Image> via TerminalTexture resource.

use bevy::prelude::*;

/// Resource exposing the terminal texture for game use
#[derive(Resource)]
pub struct TerminalTexture {
    pub handle: Handle<Image>,
    pub width: u32,
    pub height: u32,
}
