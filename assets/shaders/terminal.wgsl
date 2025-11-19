// Terminal Compute Shader

struct TerminalUniforms {
    term_cols: u32,
    term_rows: u32,
    cell_width: u32,
    cell_height: u32,
    atlas_cols: u32,
    atlas_rows: u32,
};

struct TerminalCell {
    glyph_index: u32,
    fg_color: u32,
    bg_color: u32,
    flags: u32,
};

@group(0) @binding(0) var<uniform> uniforms: TerminalUniforms;
@group(0) @binding(1) var<storage, read> grid: array<TerminalCell>;
@group(0) @binding(2) var atlas_texture: texture_2d<f32>;
@group(0) @binding(3) var output_texture: texture_storage_2d<rgba8unorm, write>;

fn unpack_color(packed: u32) -> vec4<f32> {
    // Packed as 0xAABBGGRR (little endian)
    let r = f32(packed & 0xFFu) / 255.0;
    let g = f32((packed >> 8u) & 0xFFu) / 255.0;
    let b = f32((packed >> 16u) & 0xFFu) / 255.0;
    let a = f32((packed >> 24u) & 0xFFu) / 255.0;
    return vec4<f32>(r, g, b, a);
}

@compute @workgroup_size(8, 8, 1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let pixel = vec2<u32>(global_id.xy);
    
    // DEBUG: Force top-left corner to RED to verify shader is running
    if (pixel.x < 10 && pixel.y < 10) {
        textureStore(output_texture, pixel, vec4<f32>(1.0, 0.0, 0.0, 1.0));
        return;
    }

    let width = uniforms.term_cols * uniforms.cell_width;
    let height = uniforms.term_rows * uniforms.cell_height;

    if (pixel.x >= width || pixel.y >= height) {
        return;
    }

    // Identify which cell we are in
    let cell_x = pixel.x / uniforms.cell_width;
    let cell_y = pixel.y / uniforms.cell_height;
    let cell_index = cell_y * uniforms.term_cols + cell_x;

    let cell = grid[cell_index];

    // Identify pixel within cell
    let intra_x = pixel.x % uniforms.cell_width;
    let intra_y = pixel.y % uniforms.cell_height;

    // Calculate Atlas UV (in texels)
    // Assuming a simple grid layout for the atlas
    // We need to know how many columns the atlas has.
    // Glyph index 0 -> col 0, row 0
    // Glyph index 1 -> col 1, row 0
    let glyph_idx = cell.glyph_index;
    let atlas_col = glyph_idx % uniforms.atlas_cols;
    let atlas_row = glyph_idx / uniforms.atlas_cols;

    let atlas_x = atlas_col * uniforms.cell_width + intra_x;
    let atlas_y = atlas_row * uniforms.cell_height + intra_y;

    // Load glyph pixel (using 0 mip level)
    let glyph_color = textureLoad(atlas_texture, vec2<u32>(atlas_x, atlas_y), 0);
    let alpha = glyph_color.a; // Assuming alpha contains the shape

    // Blend colors
    let fg = unpack_color(cell.fg_color);
    let bg = unpack_color(cell.bg_color);

    // Simple alpha blend: result = fg * alpha + bg * (1 - alpha)
    let final_color = mix(bg, fg, alpha);

    // Write to output
    textureStore(output_texture, pixel, final_color);
}
