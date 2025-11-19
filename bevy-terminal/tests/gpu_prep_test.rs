use bevy::prelude::*;
use bevy_terminal::TerminalState;
use bevy_terminal::atlas::GlyphAtlas;
use bevy_terminal::font::FontMetrics;
// Import the preparation logic. We need to expose it in lib.rs first.
use bevy_terminal::gpu_prep::{prepare_terminal_cpu_buffer, TerminalCpuBuffer};

#[test]
fn test_gpu_prep_system() {
    println!("\n🧪 Testing GPU Prep System: Grid → GpuBuffer\n");

    // 1. Setup Dependencies
    let font_metrics = FontMetrics::load_cascadia_mono().expect("Font load failed");
    let chars: Vec<char> = (32..=126).map(|c| c as u8 as char).collect();
    let atlas = GlyphAtlas::generate(&font_metrics, &chars).expect("Atlas failed");
    
    // 2. Setup Terminal State
    let mut term_state = TerminalState::new();
    let test_str = "GPU_TEST";
    term_state.process_bytes(test_str.as_bytes());

    // 3. Setup App
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.insert_resource(term_state);
    app.insert_resource(atlas);
    app.insert_resource(TerminalCpuBuffer::default());

    // 4. Run System
    app.add_systems(Update, prepare_terminal_cpu_buffer);
    app.update();

    // 5. Verify Buffer Content
    let buffer = app.world().resource::<TerminalCpuBuffer>();
    let cells = &buffer.cells;

    assert_eq!(cells.len(), 120 * 30, "Buffer size mismatch");

    // Verify first few characters
    let atlas_ref = app.world().resource::<GlyphAtlas>();
    
    for (i, ch) in test_str.chars().enumerate() {
        let cell = cells[i];
        let expected_index = atlas_ref.get_glyph_index(ch).expect("Char missing from atlas");
        
        println!("Cell {}: char='{}' index={} (expected {}) fg={:X} bg={:X}",
            i, ch, cell.glyph_index, expected_index, cell.fg_color, cell.bg_color);

        assert_eq!(cell.glyph_index, expected_index, "Wrong glyph index for char '{}'", ch);
        // Default colors (Tokyo Night)
        // FG: 0xC0CAF5 -> 0xFFF5CAC0 (little endian)
        assert_eq!(cell.fg_color, 0xFFF5CAC0, "Default FG color mismatch");
        // BG: 0x1A1B26 -> 0xFF261B1A
        assert_eq!(cell.bg_color, 0xFF261B1A, "Default BG color mismatch");
    }
    
    // Verify empty space (index 8 should be space)
    let space_cell = cells[test_str.len()];
    let space_index = atlas_ref.get_glyph_index(' ').unwrap();
    assert_eq!(space_cell.glyph_index, space_index, "Space glyph mismatch");

    println!("\n✅ TEST PASSED: GPU Prep system populates buffer correctly!");
}
