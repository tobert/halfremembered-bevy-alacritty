//! Core terminal plugin definition and terminal state management.

use alacritty_terminal::event::{Event as AlacEvent, EventListener};
use alacritty_terminal::grid::Dimensions;
use alacritty_terminal::index::{Column, Line};
use alacritty_terminal::sync::FairMutex;
use alacritty_terminal::term::{Config as AlacConfig, Term};
use alacritty_terminal::vte::ansi::Processor;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use bevy::asset::{RenderAssetUsages, load_internal_asset, uuid_handle};
use bevy::prelude::*;
use std::sync::Arc;
use log::info;

use crate::atlas::GlyphAtlas;
use crate::font::FontMetrics;
use crate::input;
use crate::pty;
use crate::renderer;
use crate::gpu_prep;
use crate::render_node;

pub const TERMINAL_SHADER_HANDLE: Handle<Shader> = uuid_handle!("be77e7aa-0000-0000-0000-000000000001");

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

    /// Process bytes from PTY through VTE parser into terminal grid.
    ///
    /// Handles locking internally for clean API.
    pub fn process_bytes(&mut self, bytes: &[u8]) {
        let mut term = self.term.lock();
        self.processor.advance(&mut *term, bytes);
    }

    /// Extract visible text from terminal grid for testing/debugging.
    ///
    /// Returns a String containing all visible characters in the terminal,
    /// with newlines separating rows. Useful for verifying VTE parsing.
    pub fn get_visible_text(&self) -> String {
        let term = self.term.lock();
        let mut result = String::new();

        for row in 0..self.rows {
            for col in 0..self.cols {
                let line = Line::from(row as i32);
                let column = Column(col);
                let cell = &term.grid()[line][column];

                // Get the character from the cell
                let c = cell.c;
                if c != ' ' && c != '\0' {
                    result.push(c);
                } else {
                    result.push(' ');
                }
            }
            result.push('\n');
        }

        result
    }

    /// Get a compact summary of non-empty lines for debugging.
    ///
    /// Returns only lines that contain non-whitespace characters,
    /// with line numbers. Useful for quick inspection in tests.
    pub fn get_content_summary(&self) -> Vec<(usize, String)> {
        let full_text = self.get_visible_text();
        full_text
            .lines()
            .enumerate()
            .filter_map(|(idx, line)| {
                let trimmed = line.trim();
                if !trimmed.is_empty() {
                    Some((idx, trimmed.to_string()))
                } else {
                    None
                }
            })
            .collect()
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

        load_internal_asset!(
            app,
            TERMINAL_SHADER_HANDLE,
            "../assets/shaders/terminal.wgsl",
            Shader::from_wgsl
        );

        app
            // Phase 1.1: PTY Spawning
            .add_systems(Startup, pty::spawn_pty)
            // Phase 1.2: Terminal State
            .init_resource::<TerminalState>()
            // Phase 1.3-1.4: PTY Polling and Input
            .add_systems(Update, (
                pty::poll_pty,
                input::handle_keyboard_input,
            ))
            // Phase 2: Font and Atlas
            .add_systems(Startup, initialize_font_and_atlas)
            // Phase 3: Render to Texture
            .add_systems(Startup, renderer::initialize_terminal_texture.after(initialize_font_and_atlas))
            // .add_systems(Update, renderer::render_terminal_to_texture) // CPU Renderer disabled
            
            // Phase 3.5: GPU Rendering
            .init_resource::<gpu_prep::TerminalCpuBuffer>()
            .add_systems(Update, gpu_prep::prepare_terminal_cpu_buffer.after(pty::poll_pty))
            .add_plugins(render_node::TerminalComputePlugin)
            ;

        info!("✅ TerminalPlugin initialized");
    }
}

impl Default for TerminalPlugin {
    fn default() -> Self {
        Self
    }
}

/// Startup system to initialize font metrics and glyph atlas.
///
/// Loads Cascadia Mono and generates the full glyph atlas with
/// ASCII, box-drawing, and block element characters.
fn initialize_font_and_atlas(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
) {
    info!("🔤 Loading font and generating glyph atlas...");

    let font_metrics = FontMetrics::load_cascadia_mono()
        .expect("Failed to load Cascadia Mono font");

    let mut atlas = GlyphAtlas::generate_mvp(&font_metrics)
        .expect("Failed to generate glyph atlas");

    // Create GPU texture for atlas
    let atlas_image = Image::new(
        Extent3d {
            width: atlas.atlas_width,
            height: atlas.atlas_height,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        atlas.texture_data.clone(),
        TextureFormat::Rgba8Unorm,
        RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD,
    );
    
    let atlas_handle = images.add(atlas_image);
    atlas.texture_handle = Some(atlas_handle);

    info!(
        "✅ Font and atlas ready: {}×{} cells, {} glyphs",
        atlas.cell_width, atlas.cell_height, atlas.uv_map.len()
    );

    commands.insert_resource(font_metrics);
    commands.insert_resource(atlas);
}
