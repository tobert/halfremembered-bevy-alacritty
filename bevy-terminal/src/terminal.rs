//! Core terminal plugin definition and terminal state management.

use alacritty_terminal::event::{Event as AlacEvent, EventListener};
use alacritty_terminal::grid::Dimensions;
use alacritty_terminal::sync::FairMutex;
use alacritty_terminal::term::{Config as AlacConfig, Term};
use alacritty_terminal::vte::ansi::Processor;
use bevy::prelude::*;
use std::sync::Arc;

use crate::pty;

/// Simple dimensions struct for MVP (hardcoded 120×30).
struct TerminalDimensions {
    cols: usize,
    rows: usize,
}

impl Dimensions for TerminalDimensions {
    fn total_lines(&self) -> usize {
        self.rows
    }

    fn screen_lines(&self) -> usize {
        self.rows
    }

    fn columns(&self) -> usize {
        self.cols
    }
}

/// Terminal grid state powered by alacritty_terminal.
///
/// Integrates alacritty's ANSI/VT parser and grid management.
/// The Term is updated by feeding bytes from the PTY.
#[derive(Resource)]
pub struct TerminalState {
    pub term: Arc<FairMutex<Term<EventProxy>>>,
    pub processor: Processor,
    pub cols: usize,
    pub rows: usize,
}

/// Event proxy for alacritty terminal events.
///
/// Currently no-op, but allows future integration with Bevy's event system.
#[derive(Clone)]
pub struct EventProxy;

impl EventListener for EventProxy {
    fn send_event(&self, _event: AlacEvent) {
        // Future: Forward to Bevy events
    }
}

impl TerminalState {
    /// Creates a new terminal state with hardcoded MVP configuration.
    ///
    /// Configuration:
    /// - Size: 120 cols × 30 rows
    /// - Colors: Tokyo Night theme
    /// - Scrollback: 10,000 lines
    pub fn new() -> Self {
        const COLS: usize = 120;
        const ROWS: usize = 30;

        let config = AlacConfig::default();
        let dimensions = TerminalDimensions {
            cols: COLS,
            rows: ROWS,
        };

        let term = Term::new(config, &dimensions, EventProxy);

        info!("📋 Terminal grid initialized: {}×{}", COLS, ROWS);

        Self {
            term: Arc::new(FairMutex::new(term)),
            processor: Processor::new(),
            cols: COLS,
            rows: ROWS,
        }
    }
}

impl Default for TerminalState {
    fn default() -> Self {
        Self::new()
    }
}

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
            // Phase 1.2: Terminal State
            .init_resource::<TerminalState>()
            // TODO: Add remaining resources
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
