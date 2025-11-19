use bevy::prelude::*;
use bytemuck::{Pod, Zeroable};

/// Represents a single cell in the terminal grid for GPU consumption.
///
/// This struct must match the alignment requirements of WGSL (16-byte alignment is safest for arrays of structs,
/// though standard u32 arrays can be tighter).
/// We will pack it into 16 bytes:
/// - u32 glyph_index
/// - u32 fg_color (0xAABBGGRR)
/// - u32 bg_color (0xAABBGGRR)
/// - u32 flags (unused for now, padding)
#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable, Debug)]
pub struct GpuTerminalCell {
    pub glyph_index: u32,
    pub fg_color: u32,
    pub bg_color: u32,
    pub flags: u32,
}

/// Uniforms for the terminal renderer.
#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable, Debug, Resource)]
pub struct TerminalUniforms {
    pub term_cols: u32,
    pub term_rows: u32,
    pub cell_width: u32,
    pub cell_height: u32,
    // Atlas info
    pub atlas_cols: u32,
    pub atlas_rows: u32,
    pub _padding: [u32; 2], // Ensure 16-byte alignment
}
