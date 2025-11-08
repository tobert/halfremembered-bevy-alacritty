//! Claude CRT Character Example
//!
//! Demonstrates:
//! - Character with CRT head showing real terminal
//! - Terminal texture rendered tiny on sprite
//! - Zoom interaction (E key to fullscreen)
//! - Persistent terminal state

use bevy::prelude::*;
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
        .add_systems(Update, (toggle_terminal_size, update_help_text))
        .run();
}

#[derive(Component)]
struct ClaudeCharacter;

#[derive(Component)]
struct HelpText;

fn setup(
    mut commands: Commands,
    // terminal_texture: Res<TerminalTexture>, // TODO: Uncomment when ready
) {
    // Camera
    commands.spawn(Camera2d);

    // TODO: Spawn Claude character with CRT head
    // For now, just placeholder
    info!("🎮 Claude CRT example ready");
    info!("📺 Press 'T' to toggle terminal size (not implemented yet)");

    // Help text
    commands.spawn((
        Text::new("Press 'T' to toggle terminal (WIP)"),
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
