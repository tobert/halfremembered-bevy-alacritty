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

use bevy::window::{WindowMode, MonitorSelection};
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use bevy::asset::RenderAssetUsages;
use bevy_terminal::prelude::*;
use clap::Parser;


#[derive(Parser, Debug, Clone, Resource)]
#[command(author, version, about, long_about = None)]
struct Args {

    /// Run in fullscreen mode
    #[arg(long)]
    fullscreen: bool,
}

fn main() {
    let args = Args::parse();

    let mut app = App::new();

    let mode = if args.fullscreen {
        WindowMode::BorderlessFullscreen(MonitorSelection::Current)
    } else {
        WindowMode::Windowed
    };

    app.add_plugins(
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Claude CRT Character - Terminal Demo".into(),
                    resolution: (1920, 1080).into(),
                    mode,
                    ..default()
                }),
                ..default()
            })
            .set(ImagePlugin::default_nearest())
    );

    app.add_plugins(bevy_brp_extras::BrpExtrasPlugin)
        .insert_resource(args)
        .add_plugins(TerminalPlugin)
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                spawn_terminal_view,
                spawn_atlas_debug_view,
                toggle_debug_view,
                handle_zoom_input,
                animate_zoom_transition,
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

#[derive(Component)]
struct TerminalSprite;

/// Current zoom state of the terminal
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ZoomState {
    Tiny,       // 0.05 scale - tiny CRT on character
    Fullscreen, // 1.0 scale - zoomed in for interaction
}

/// Terminal interaction state
#[derive(Resource)]
struct TerminalState {
    zoom: ZoomState,
    target_scale: f32,
    current_scale: f32,
    transition_speed: f32,
}

impl Default for TerminalState {
    fn default() -> Self {
        Self {
            zoom: ZoomState::Tiny,
            target_scale: 0.05,
            current_scale: 0.05,
            transition_speed: 8.0, // Smooth but responsive
        }
    }
}

#[derive(Resource)]
struct DebugState {
    atlas_spawned: bool,
    debug_visible: bool,
    terminal_spawned: bool,
}

fn setup(
    mut commands: Commands,
) {
    // Camera
    commands.spawn(Camera2d);

    // Initialize debug state
    commands.insert_resource(DebugState {
        atlas_spawned: false,
        debug_visible: false, // Start with debug off
        terminal_spawned: false,
    });

    // Initialize terminal interaction state
    commands.insert_resource(TerminalState::default());

    // Disable terminal input initially (enabled when zoomed in)
    commands.insert_resource(TerminalInputEnabled { enabled: false });

    info!("🎮 Claude CRT example ready");
    info!("📺 Terminal will appear when ready (tiny CRT)");
    info!("⌨️  Press 'E' near terminal to zoom in");
    info!("🐛 Press 'D' to toggle debug atlas view");

    // Help text
    commands.spawn((
        Text::new(
            "Terminal Loading...\n\n\
             Press 'E' to zoom in\n\
             Press 'ESC' to zoom out\n\
             Press 'D' to toggle debug atlas view"
        ),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            left: Val::Px(10.0),
            ..default()
        },
        HelpText,
    ));
}

/// Handle zoom input (E to zoom in, ESC to zoom out).
fn handle_zoom_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut terminal_state: ResMut<TerminalState>,
    mut input_enabled: ResMut<TerminalInputEnabled>,
) {
    if keyboard.just_pressed(KeyCode::KeyE) && terminal_state.zoom == ZoomState::Tiny {
        info!("🔍 Zooming in to fullscreen");
        terminal_state.zoom = ZoomState::Fullscreen;
        terminal_state.target_scale = 1.0;
        input_enabled.enabled = true; // Enable terminal input when zoomed in
    }

    if keyboard.just_pressed(KeyCode::Escape) && terminal_state.zoom == ZoomState::Fullscreen {
        info!("🔍 Zooming out to tiny");
        terminal_state.zoom = ZoomState::Tiny;
        terminal_state.target_scale = 0.05;
        input_enabled.enabled = false; // Disable terminal input when zoomed out
    }
}

/// Animate smooth zoom transitions.
fn animate_zoom_transition(
    time: Res<Time>,
    mut terminal_state: ResMut<TerminalState>,
    mut query: Query<&mut Transform, With<TerminalSprite>>,
) {
    // Lerp current scale toward target
    let delta = terminal_state.target_scale - terminal_state.current_scale;
    if delta.abs() > 0.001 {
        terminal_state.current_scale += delta * terminal_state.transition_speed * time.delta_secs();

        // Update sprite transform
        for mut transform in query.iter_mut() {
            transform.scale = Vec3::splat(terminal_state.current_scale);
        }
    }
}

/// Update help text based on current state.
fn update_help_text(
    terminal_state: Res<TerminalState>,
    mut query: Query<&mut Text, With<HelpText>>,
) {
    if !terminal_state.is_changed() {
        return;
    }

    for mut text in query.iter_mut() {
        **text = match terminal_state.zoom {
            ZoomState::Tiny => {
                "Tiny CRT Mode\n\n\
                 Press 'E' to zoom in\n\
                 Press 'D' to toggle debug atlas view"
                    .to_string()
            }
            ZoomState::Fullscreen => {
                "Fullscreen Terminal Mode\n\n\
                 Type commands here!\n\
                 Press 'ESC' to zoom out\n\
                 Press 'D' to toggle debug atlas view"
                    .to_string()
            }
        };
    }
}

/// Spawn terminal view once the terminal texture is ready.
///
/// Creates a tiny sprite showing the live terminal output (0.05 scale).
/// Use E key to zoom to fullscreen.
fn spawn_terminal_view(
    terminal_texture: Option<Res<TerminalTexture>>,
    terminal_state: Res<TerminalState>,
    mut debug_state: ResMut<DebugState>,
    mut commands: Commands,
) {
    // Only spawn once when resource is available
    if debug_state.terminal_spawned {
        return;
    }

    let Some(terminal_texture) = terminal_texture else {
        return;
    };

    info!("📺 Spawning tiny CRT terminal");

    let scale = terminal_state.current_scale;

    info!(
        "📐 Terminal texture: {}×{} pixels, tiny scale={:.3}",
        terminal_texture.width, terminal_texture.height, scale
    );

    // Spawn tiny sprite showing terminal (CRT on character's head)
    commands.spawn((
        Sprite {
            image: terminal_texture.handle.clone(),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 1.0).with_scale(Vec3::splat(scale)),
        TerminalSprite,
    ));

    debug_state.terminal_spawned = true;
    info!("✅ Tiny CRT terminal spawned - press 'E' to zoom in!");
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
