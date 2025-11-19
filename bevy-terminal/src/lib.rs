//! Bevy plugin for embedding terminal emulation in games.
//!
//! Uses render-to-texture architecture for flexible display:
//! - Terminal renders to `Image` texture
//! - Texture can be used as sprite, UI, or material
//! - Supports multiple views (tiny CRT head, fullscreen overlay)
//! - Persistent PTY lifecycle (always running in background)
//!
//! # Example
//! ```no_run
//! use bevy::prelude::*;
//! use bevy_terminal::prelude::*;
//!
//! fn main() {
//!     App::new()
//!         .add_plugins(DefaultPlugins)
//!         .add_plugins(TerminalPlugin)
//!         .run();
//! }
//!
//! fn use_terminal(terminal_texture: Res<TerminalTexture>) {
//!     // terminal_texture.handle is Handle<Image>
//!     // Use it anywhere: sprites, UI, materials
//! }
//! ```

pub mod atlas;
mod colors;
mod events;
pub mod font;
pub mod gpu_types;
pub mod gpu_prep;
pub mod render_node;
pub mod input;
pub mod pty;
pub mod renderer;
mod terminal;

pub use renderer::TerminalTexture;
pub use terminal::{TerminalPlugin, TerminalState};

/// Re-export commonly used types
pub mod prelude {
    pub use crate::events::TerminalEvent;
    pub use crate::input::TerminalInputEnabled;
    pub use crate::renderer::TerminalTexture;
    pub use crate::terminal::TerminalPlugin;
}
