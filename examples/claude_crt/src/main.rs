//! Claude CRT Character Example
//!
//! Demonstrates:
//! - Character with CRT head showing real terminal
//! - Terminal texture rendered tiny on sprite
//! - Zoom interaction (E key to fullscreen)
//! - Persistent terminal state
//!
//! Debug Mode:
//! - Shows glyph atlas texture for verification
//! - Press 'D' to toggle debug view

use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use bevy::asset::RenderAssetUsages;
use bevy_terminal::prelude::*;

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Claude CRT Character - Terminal Demo".into(),
                        resolution: (1920, 1080).into(),
                        ..default()
                    }),
                    ..default()
                })
                .set(ImagePlugin::default_nearest()), // Crisp pixels
        )
        .add_plugins(TerminalPlugin)
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                spawn_atlas_debug_view,
                toggle_debug_view,
                toggle_terminal_size,
                update_help_text,
            ),
        )
        .run();
}

#[derive(Component)]
struct ClaudeCharacter;

#[derive(Component)]
struct HelpText;

#[derive(Component)]
struct AtlasDebugView;

#[derive(Resource)]
struct DebugState {
    atlas_spawned: bool,
    debug_visible: bool,
}

fn setup(
    mut commands: Commands,
    // terminal_texture: Res<TerminalTexture>, // TODO: Uncomment when ready
) {
    // Camera
    commands.spawn(Camera2d);

    // Initialize debug state
    commands.insert_resource(DebugState {
        atlas_spawned: false,
        debug_visible: true,
    });

    // TODO: Spawn Claude character with CRT head
    // For now, just placeholder
    info!("🎮 Claude CRT example ready");
    info!("📺 Press 'T' to toggle terminal size (not implemented yet)");
    info!("🐛 Press 'D' to toggle debug atlas view");

    // Help text
    commands.spawn((
        Text::new("Press 'D' to toggle debug atlas view\nPress 'T' to toggle terminal (WIP)"),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            left: Val::Px(10.0),
            ..default()
        },
        HelpText,
    ));
}

fn toggle_terminal_size(keyboard: Res<ButtonInput<KeyCode>>) {
    if keyboard.just_pressed(KeyCode::KeyT) {
        info!("Terminal toggle requested (not implemented)");
        // TODO: Toggle between tiny (0.05) and fullscreen (1.0)
    }
}

fn update_help_text() {
    // TODO: Update help text based on state
}

/// Spawn atlas debug view once the atlas is ready.
///
/// Creates a full-screen sprite showing the glyph atlas texture.
/// Useful for verifying that glyphs are crisp and properly aligned.
fn spawn_atlas_debug_view(
    atlas: Option<Res<bevy_terminal::atlas::GlyphAtlas>>,
    font_metrics: Option<Res<bevy_terminal::font::FontMetrics>>,
    mut debug_state: ResMut<DebugState>,
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
) {
    // Only spawn once when resources are available
    if debug_state.atlas_spawned {
        return;
    }

    let Some(atlas) = atlas else {
        return;
    };

    let Some(font_metrics) = font_metrics else {
        return;
    };

    info!("🐛 Spawning atlas debug view");

    // Create Bevy Image from atlas texture data
    let image = Image::new(
        Extent3d {
            width: atlas.atlas_width,
            height: atlas.atlas_height,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        atlas.texture_data.clone(),
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    );

    let image_handle = images.add(image);

    // Calculate scale to fit atlas on screen (assuming 1920×1080 window)
    let window_height = 1080.0;
    let scale = window_height / atlas.atlas_height as f32;

    info!(
        "📐 Atlas: {}×{} pixels, {}×{} cells, {} glyphs, scale={:.2}",
        atlas.atlas_width,
        atlas.atlas_height,
        atlas.cell_width,
        atlas.cell_height,
        atlas.uv_map.len(),
        scale
    );

    info!(
        "🔤 Font: cell={}×{:.1}, baseline={:.1}",
        font_metrics.cell_width, font_metrics.cell_height, font_metrics.baseline
    );

    // Spawn sprite showing atlas
    commands.spawn((
        Sprite {
            image: image_handle,
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 0.0).with_scale(Vec3::splat(scale)),
        AtlasDebugView,
    ));

    debug_state.atlas_spawned = true;
    info!("✅ Atlas debug view spawned");
}

/// Toggle debug view visibility with 'D' key.
fn toggle_debug_view(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut debug_state: ResMut<DebugState>,
    mut query: Query<&mut Visibility, With<AtlasDebugView>>,
) {
    if keyboard.just_pressed(KeyCode::KeyD) {
        debug_state.debug_visible = !debug_state.debug_visible;

        for mut visibility in query.iter_mut() {
            *visibility = if debug_state.debug_visible {
                Visibility::Visible
            } else {
                Visibility::Hidden
            };
        }

        info!(
            "🐛 Debug view: {}",
            if debug_state.debug_visible {
                "visible"
            } else {
                "hidden"
            }
        );
    }
}
