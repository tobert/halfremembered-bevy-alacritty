use bevy::prelude::*;
use crate::gpu_types::GpuTerminalCell;
use crate::terminal::TerminalState;
use crate::atlas::GlyphAtlas;
use crate::colors::convert_alacritty_color;
use alacritty_terminal::index::{Column, Line};

/// Resource holding the CPU-side buffer of terminal cells.
///
/// This is updated every frame from the Alacritty grid and then uploaded to the GPU.
#[derive(Resource, Default)]
pub struct TerminalCpuBuffer {
    pub cells: Vec<GpuTerminalCell>,
}

/// Updates the CPU buffer from the terminal grid.
pub fn prepare_terminal_cpu_buffer(
    term_state: Res<TerminalState>,
    atlas: Res<GlyphAtlas>,
    mut cpu_buffer: ResMut<TerminalCpuBuffer>,
) {
    let term = term_state.term.lock();
    let grid = term.grid();
    let rows = term_state.rows;
    let cols = term_state.cols;

    // Resize buffer if needed
    let total_cells = rows * cols;
    if cpu_buffer.cells.len() != total_cells {
        cpu_buffer.cells.resize(total_cells, GpuTerminalCell {
            glyph_index: 0, // Default to 0 (usually space or null)
            fg_color: 0,
            bg_color: 0,
            flags: 0,
        });
    }

    // Fill buffer
    for row in 0..rows {
        for col in 0..cols {
            let line = Line(row as i32);
            let column = Column(col);
            let cell = &grid[line][column];

            // Map char to atlas index
            // If char is not in atlas, use index 0 (or a dedicated 'missing' glyph if we had one)
            // Space (' ') is usually in atlas, null ('\0') might not be.
            let glyph_index = if cell.c == '\0' || cell.c == ' ' {
                // Optimization: space is often index 0 or we can skip rendering it in shader?
                // For now, let's try to find it, or default to something invisible.
                // Actually, we should just draw space.
                 atlas.get_glyph_index(' ').unwrap_or(0)
            } else {
                atlas.get_glyph_index(cell.c).unwrap_or_else(|| {
                     // Fallback for missing glyphs
                     atlas.get_glyph_index('?').unwrap_or(0)
                })
            };

            // Pack colors (RGBA u32)
            // We need to convert Alacritty colors to u32.
            let fg = pack_color(convert_alacritty_color(cell.fg));
            let bg = pack_color(convert_alacritty_color(cell.bg));

            let index = row * cols + col;
            cpu_buffer.cells[index] = GpuTerminalCell {
                glyph_index,
                fg_color: fg,
                bg_color: bg,
                flags: 0,
            };
        }
    }
}

// Helper: Pack [u8; 3] rgb into u32 (0xFFBBGGRR for little endian / GPU)
// We assume alpha is 255.
fn pack_color(rgb: [u8; 3]) -> u32 {
    let r = rgb[0] as u32;
    let g = rgb[1] as u32;
    let b = rgb[2] as u32;
    let a = 255u32;
    
    // Little endian: R is lowest byte
    r | (g << 8) | (b << 16) | (a << 24)
}

