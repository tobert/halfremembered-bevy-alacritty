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

use bevy::prelude::*;

mod atlas;
mod colors;
mod events;
mod font;
mod input;
mod pty;
mod renderer;
mod terminal;

pub use renderer::TerminalTexture;
pub use terminal::TerminalPlugin;

/// Re-export commonly used types
pub mod prelude {
    pub use crate::events::TerminalEvent;
    pub use crate::renderer::TerminalTexture;
    pub use crate::terminal::TerminalPlugin;
}
