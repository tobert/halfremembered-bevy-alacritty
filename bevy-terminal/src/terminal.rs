//! Core terminal plugin definition.

use bevy::prelude::*;
use crate::pty;

/// Bevy plugin for terminal emulation.
///
/// MVP: Hardcoded configuration
/// - Font: Cascadia Mono Regular, 14pt
/// - Size: 120 cols × 30 rows
/// - Colors: Tokyo Night
/// - Shell: bash (or default shell)
///
/// PTY is spawned in Startup system and runs persistently.
/// Terminal state updates continuously in background.
/// Renders to texture exposed via `TerminalTexture` resource.
pub struct TerminalPlugin;

impl Plugin for TerminalPlugin {
    fn build(&self, app: &mut App) {
        info!("🖥️  Initializing TerminalPlugin (render-to-texture)");

        app
            // Phase 1.1: PTY Spawning
            .add_systems(Startup, pty::spawn_pty)
            // TODO: Add resources
            // .init_resource::<TerminalState>()
            // .init_resource::<TerminalTexture>()
            // .init_resource::<GlyphAtlas>()
            // TODO: Add remaining systems
            // .add_systems(Update, (
            //     poll_pty,
            //     update_terminal_grid,
            //     render_to_texture,
            //     handle_input,
            // ))
            // TODO: Add events
            // .add_event::<TerminalEvent>()
            ;

        info!("✅ TerminalPlugin initialized");
    }
}

impl Default for TerminalPlugin {
    fn default() -> Self {
        Self
    }
}
