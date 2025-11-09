//! Test for terminal renderer - verifies texture generation from grid content.
//!
//! This test creates a minimal Bevy app, puts text in the terminal grid,
//! and verifies the renderer produces non-blank texture data.

use bevy::prelude::*;
use bevy::asset::AssetPlugin;
use bevy_terminal::{TerminalState, TerminalTexture};
use bevy_terminal::atlas::GlyphAtlas;
use bevy_terminal::font::FontMetrics;

#[test]
fn test_renderer_produces_texture() {
    println!("\n🧪 Testing Renderer: Grid → Texture\n");

    // 1. Load font and generate atlas
    let font_metrics = FontMetrics::load_cascadia_mono()
        .expect("Failed to load font");

    println!("✅ Font loaded: {}×{} cells", font_metrics.cell_width, font_metrics.cell_height);

    // ASCII printable characters (same as prod)
    let chars: Vec<char> = (32..=126).map(|c| c as u8 as char)
        .chain(std::iter::once(' '))
        .collect();

    let atlas = GlyphAtlas::generate(&font_metrics, &chars)
        .expect("Failed to generate atlas");

    println!("✅ Atlas generated: {}×{} with {} glyphs",
        atlas.atlas_width, atlas.atlas_height, atlas.uv_map.len());

    // 2. Create terminal state and add some text
    let mut term_state = TerminalState::new();

    // Simulate PTY output with a simple string
    let test_text = "Hello, World!\n";
    term_state.process_bytes(test_text.as_bytes());

    println!("✅ Terminal state created and populated");

    // Verify grid has content
    let content = term_state.get_content_summary();
    println!("📋 Grid content: {} non-empty lines", content.len());
    for (line_num, text) in &content {
        println!("  Line {}: {}", line_num, text);
    }

    assert!(
        !content.is_empty(),
        "Terminal grid should contain test text"
    );

    // 3. Create a minimal Bevy app to test rendering
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(AssetPlugin::default());
    app.add_plugins(ImagePlugin::default());

    // Insert resources
    app.insert_resource(term_state);
    app.insert_resource(atlas);

    // Create terminal texture manually (since we're not running full plugin)
    let texture_width = app.world().resource::<GlyphAtlas>().cell_width * 120;
    let texture_height = app.world().resource::<GlyphAtlas>().cell_height * 30;

    let mut texture_data = vec![0u8; (texture_width * texture_height * 4) as usize];
    // Fill with a known background color (dark blue)
    for pixel in texture_data.chunks_exact_mut(4) {
        pixel[0] = 0x1a; // Tokyo Night BG
        pixel[1] = 0x1b;
        pixel[2] = 0x26;
        pixel[3] = 255;
    }

    let image = Image::new(
        bevy::render::render_resource::Extent3d {
            width: texture_width,
            height: texture_height,
            depth_or_array_layers: 1,
        },
        bevy::render::render_resource::TextureDimension::D2,
        texture_data,
        bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb,
        bevy::asset::RenderAssetUsages::MAIN_WORLD,
    );

    let handle = app.world_mut().resource_mut::<Assets<Image>>().add(image);

    app.insert_resource(TerminalTexture {
        handle: handle.clone(),
        width: texture_width,
        height: texture_height,
    });

    println!("✅ Bevy app created with terminal texture: {}×{}", texture_width, texture_height);

    // 4. Add renderer system and run one update
    // First, let's verify what's in the grid before rendering
    {
        let term_state = app.world().resource::<TerminalState>();
        let full_grid = term_state.get_visible_text();
        println!("🔍 Full grid before rendering:");
        for (i, line) in full_grid.lines().take(3).enumerate() {
            let trimmed = line.trim_end();
            if !trimmed.is_empty() {
                println!("  Row {}: '{}'", i, trimmed);
            }
        }
    }

    println!("🎨 Running renderer system...");
    app.add_systems(Update, bevy_terminal::renderer::render_terminal_to_texture);
    app.update();

    // 5. Verify texture contains rendered glyphs
    let images = app.world().resource::<Assets<Image>>();
    let texture = images.get(&handle).expect("Texture should exist");
    let texture_data = texture.data.as_ref().expect("Texture should have data");

    println!("📊 Analyzing texture data...");

    // Check if texture has any non-background pixels
    let bg_color = [0x1a, 0x1b, 0x26, 255];
    let mut non_bg_pixels = 0;
    let mut sample_colors: std::collections::HashSet<[u8; 4]> = std::collections::HashSet::new();

    for pixel in texture_data.chunks_exact(4) {
        let rgba = [pixel[0], pixel[1], pixel[2], pixel[3]];
        if rgba != bg_color {
            non_bg_pixels += 1;
            if sample_colors.len() < 10 {
                sample_colors.insert(rgba);
            }
        }
    }

    println!("📈 Non-background pixels: {} / {}", non_bg_pixels, texture_data.len() / 4);
    println!("🎨 Sample colors found: {:?}", sample_colors);

    if non_bg_pixels == 0 {
        println!("\n❌ RENDERER FAILED: Texture is completely blank!");
        println!("   Grid has content but renderer produced no visible pixels.");

        // Print first few rows of texture for debugging
        println!("\n🔍 First row of texture (first 120 pixels):");
        for i in 0..120.min(texture_data.len() / 4) {
            let idx = i * 4;
            let rgba = [
                texture_data[idx],
                texture_data[idx + 1],
                texture_data[idx + 2],
                texture_data[idx + 3]
            ];
            if rgba != bg_color {
                print!("█");
            } else {
                print!("·");
            }
        }
        println!();

        panic!("Renderer should produce non-background pixels when grid has content");
    }

    println!("\n✅ TEST PASSED: Renderer produced {} non-background pixels!", non_bg_pixels);
    println!("   The rendering pipeline works correctly!");
}
