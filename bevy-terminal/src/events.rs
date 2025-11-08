//! Terminal events.

use bevy::prelude::*;

/// Events emitted by the terminal system
#[derive(Event, Debug)]
pub enum TerminalEvent {
    /// PTY and terminal spawned successfully
    Spawned,
    /// PTY process exited
    ProcessExited { exit_code: Option<i32> },
    /// Error occurred
    Error { message: String },
}
