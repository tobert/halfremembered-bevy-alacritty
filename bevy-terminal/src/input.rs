//! Keyboard input handling for terminal emulation.
//!
//! Captures Bevy keyboard events and converts them to appropriate byte sequences
//! for the PTY. Handles:
//! - ASCII characters (a-z, 0-9, symbols)
//! - Special keys (Enter, Backspace, Tab, Escape)
//! - Arrow keys (ANSI escape sequences)
//! - Control sequences (Ctrl+C, Ctrl+D, etc.)

use bevy::prelude::*;
use std::io::Write;
use log::{error, trace};

use crate::pty::PtyResource;

/// Controls whether keyboard input is routed to the terminal.
///
/// When false, keyboard events are ignored by the terminal.
/// Use this to implement game-specific input modes (e.g., zoom controls).
#[derive(Resource, Default)]
pub struct TerminalInputEnabled {
    pub enabled: bool,
}

/// Handles keyboard input and sends it to the PTY.
///
/// System: Update
/// Runs: Every frame
///
/// Supports Shift and Ctrl modifiers for proper terminal interaction.
/// Respects TerminalInputEnabled resource to allow game-specific input modes.
pub fn handle_keyboard_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    pty: Res<PtyResource>,
    input_enabled: Option<Res<TerminalInputEnabled>>,
) {
    // Check if terminal input is enabled (defaults to true if resource not present)
    let enabled = input_enabled.map(|r| r.enabled).unwrap_or(true);
    if !enabled {
        return;
    }
    // Check modifier state
    let shift = keyboard.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]);
    let ctrl = keyboard.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight]);

    // Process all just-pressed keys this frame
    for key in keyboard.get_just_pressed() {
        if let Some(bytes) = keycode_to_bytes(*key, shift, ctrl) {
            // Write to PTY
            if let Ok(mut writer) = pty.writer.try_lock() {
                if let Err(error) = writer.write_all(&bytes) {
                    error!("❌ Failed to write to PTY: {}", error);
                } else if let Err(error) = writer.flush() {
                    error!("❌ Failed to flush PTY writer: {}", error);
                } else {
                    trace!("⌨️  Sent {} bytes to PTY", bytes.len());
                }
            }
        }
    }
}

/// Converts Bevy KeyCode to terminal byte sequences.
///
/// Handles Shift and Ctrl modifiers for proper terminal interaction.
/// Returns None for keys that don't map to terminal input.
fn keycode_to_bytes(key: KeyCode, shift: bool, ctrl: bool) -> Option<Vec<u8>> {
    use KeyCode::*;

    // Ctrl sequences take precedence (Ctrl+C, Ctrl+D, etc.)
    if ctrl {
        return match key {
            KeyA => Some(vec![0x01]), // Ctrl+A (SOH - Start of Heading)
            KeyB => Some(vec![0x02]), // Ctrl+B
            KeyC => Some(vec![0x03]), // Ctrl+C (ETX - End of Text / Interrupt)
            KeyD => Some(vec![0x04]), // Ctrl+D (EOT - End of Transmission / EOF)
            KeyE => Some(vec![0x05]), // Ctrl+E
            KeyF => Some(vec![0x06]), // Ctrl+F
            KeyG => Some(vec![0x07]), // Ctrl+G (BEL - Bell)
            KeyH => Some(vec![0x08]), // Ctrl+H (Backspace)
            KeyI => Some(vec![0x09]), // Ctrl+I (Tab)
            KeyJ => Some(vec![0x0A]), // Ctrl+J (Line Feed)
            KeyK => Some(vec![0x0B]), // Ctrl+K
            KeyL => Some(vec![0x0C]), // Ctrl+L (Form Feed / Clear Screen)
            KeyM => Some(vec![0x0D]), // Ctrl+M (Carriage Return)
            KeyN => Some(vec![0x0E]), // Ctrl+N
            KeyO => Some(vec![0x0F]), // Ctrl+O
            KeyP => Some(vec![0x10]), // Ctrl+P
            KeyQ => Some(vec![0x11]), // Ctrl+Q
            KeyR => Some(vec![0x12]), // Ctrl+R
            KeyS => Some(vec![0x13]), // Ctrl+S
            KeyT => Some(vec![0x14]), // Ctrl+T
            KeyU => Some(vec![0x15]), // Ctrl+U
            KeyV => Some(vec![0x16]), // Ctrl+V
            KeyW => Some(vec![0x17]), // Ctrl+W
            KeyX => Some(vec![0x18]), // Ctrl+X
            KeyY => Some(vec![0x19]), // Ctrl+Y
            KeyZ => Some(vec![0x1A]), // Ctrl+Z (SUB - Suspend)
            _ => None,
        };
    }

    // Letters: handle Shift for uppercase
    let letter_result = match key {
        KeyA => Some(if shift { 'A' } else { 'a' }),
        KeyB => Some(if shift { 'B' } else { 'b' }),
        KeyC => Some(if shift { 'C' } else { 'c' }),
        KeyD => Some(if shift { 'D' } else { 'd' }),
        KeyE => Some(if shift { 'E' } else { 'e' }),
        KeyF => Some(if shift { 'F' } else { 'f' }),
        KeyG => Some(if shift { 'G' } else { 'g' }),
        KeyH => Some(if shift { 'H' } else { 'h' }),
        KeyI => Some(if shift { 'I' } else { 'i' }),
        KeyJ => Some(if shift { 'J' } else { 'j' }),
        KeyK => Some(if shift { 'K' } else { 'k' }),
        KeyL => Some(if shift { 'L' } else { 'l' }),
        KeyM => Some(if shift { 'M' } else { 'm' }),
        KeyN => Some(if shift { 'N' } else { 'n' }),
        KeyO => Some(if shift { 'O' } else { 'o' }),
        KeyP => Some(if shift { 'P' } else { 'p' }),
        KeyQ => Some(if shift { 'Q' } else { 'q' }),
        KeyR => Some(if shift { 'R' } else { 'r' }),
        KeyS => Some(if shift { 'S' } else { 's' }),
        KeyT => Some(if shift { 'T' } else { 't' }),
        KeyU => Some(if shift { 'U' } else { 'u' }),
        KeyV => Some(if shift { 'V' } else { 'v' }),
        KeyW => Some(if shift { 'W' } else { 'w' }),
        KeyX => Some(if shift { 'X' } else { 'x' }),
        KeyY => Some(if shift { 'Y' } else { 'y' }),
        KeyZ => Some(if shift { 'Z' } else { 'z' }),
        _ => None,
    };

    if let Some(ch) = letter_result {
        return Some(vec![ch as u8]);
    }

    // Numbers and symbols: handle Shift
    match key {
        Digit0 => Some(if shift { b")" } else { b"0" }.to_vec()),
        Digit1 => Some(if shift { b"!" } else { b"1" }.to_vec()),
        Digit2 => Some(if shift { b"@" } else { b"2" }.to_vec()),
        Digit3 => Some(if shift { b"#" } else { b"3" }.to_vec()),
        Digit4 => Some(if shift { b"$" } else { b"4" }.to_vec()),
        Digit5 => Some(if shift { b"%" } else { b"5" }.to_vec()),
        Digit6 => Some(if shift { b"^" } else { b"6" }.to_vec()),
        Digit7 => Some(if shift { b"&" } else { b"7" }.to_vec()),
        Digit8 => Some(if shift { b"*" } else { b"8" }.to_vec()),
        Digit9 => Some(if shift { b"(" } else { b"9" }.to_vec()),

        // Special characters with Shift variants
        Space => Some(b" ".to_vec()),
        Minus => Some(if shift { b"_" } else { b"-" }.to_vec()),
        Equal => Some(if shift { b"+" } else { b"=" }.to_vec()),
        BracketLeft => Some(if shift { b"{" } else { b"[" }.to_vec()),
        BracketRight => Some(if shift { b"}" } else { b"]" }.to_vec()),
        Backslash => Some(if shift { b"|" } else { b"\\" }.to_vec()),
        Semicolon => Some(if shift { b":" } else { b";" }.to_vec()),
        Quote => Some(if shift { b"\"" } else { b"'" }.to_vec()),
        Comma => Some(if shift { b"<" } else { b"," }.to_vec()),
        Period => Some(if shift { b">" } else { b"." }.to_vec()),
        Slash => Some(if shift { b"?" } else { b"/" }.to_vec()),
        Backquote => Some(if shift { b"~" } else { b"`" }.to_vec()),

        // Control keys (unaffected by modifiers in MVP)
        Enter => Some(b"\r".to_vec()),
        Tab => Some(b"\t".to_vec()),
        Backspace => Some(b"\x7f".to_vec()),
        Escape => Some(b"\x1b".to_vec()),

        // Arrow keys (ANSI escape sequences)
        ArrowUp => Some(b"\x1b[A".to_vec()),
        ArrowDown => Some(b"\x1b[B".to_vec()),
        ArrowRight => Some(b"\x1b[C".to_vec()),
        ArrowLeft => Some(b"\x1b[D".to_vec()),

        // Home/End
        Home => Some(b"\x1b[H".to_vec()),
        End => Some(b"\x1b[F".to_vec()),

        // Page Up/Down
        PageUp => Some(b"\x1b[5~".to_vec()),
        PageDown => Some(b"\x1b[6~".to_vec()),

        // Delete/Insert
        Delete => Some(b"\x1b[3~".to_vec()),
        Insert => Some(b"\x1b[2~".to_vec()),

        // Modifiers themselves and other unmapped keys
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_ascii_lowercase() {
        assert_eq!(keycode_to_bytes(KeyCode::KeyA, false, false), Some(b"a".to_vec()));
        assert_eq!(keycode_to_bytes(KeyCode::Digit1, false, false), Some(b"1".to_vec()));
        assert_eq!(keycode_to_bytes(KeyCode::Space, false, false), Some(b" ".to_vec()));
    }

    #[test]
    fn test_shift_uppercase() {
        assert_eq!(keycode_to_bytes(KeyCode::KeyA, true, false), Some(b"A".to_vec()));
        assert_eq!(keycode_to_bytes(KeyCode::KeyZ, true, false), Some(b"Z".to_vec()));
        assert_eq!(keycode_to_bytes(KeyCode::KeyM, true, false), Some(b"M".to_vec()));
    }

    #[test]
    fn test_shift_symbols() {
        assert_eq!(keycode_to_bytes(KeyCode::Digit1, true, false), Some(b"!".to_vec()));
        assert_eq!(keycode_to_bytes(KeyCode::Digit2, true, false), Some(b"@".to_vec()));
        assert_eq!(keycode_to_bytes(KeyCode::Digit3, true, false), Some(b"#".to_vec()));
        assert_eq!(keycode_to_bytes(KeyCode::Digit4, true, false), Some(b"$".to_vec()));
        assert_eq!(keycode_to_bytes(KeyCode::Digit5, true, false), Some(b"%".to_vec()));
        assert_eq!(keycode_to_bytes(KeyCode::Digit8, true, false), Some(b"*".to_vec()));
        assert_eq!(keycode_to_bytes(KeyCode::Digit9, true, false), Some(b"(".to_vec()));
        assert_eq!(keycode_to_bytes(KeyCode::Minus, true, false), Some(b"_".to_vec()));
        assert_eq!(keycode_to_bytes(KeyCode::Equal, true, false), Some(b"+".to_vec()));
    }

    #[test]
    fn test_ctrl_sequences() {
        assert_eq!(keycode_to_bytes(KeyCode::KeyC, false, true), Some(vec![0x03])); // Ctrl+C
        assert_eq!(keycode_to_bytes(KeyCode::KeyD, false, true), Some(vec![0x04])); // Ctrl+D
        assert_eq!(keycode_to_bytes(KeyCode::KeyL, false, true), Some(vec![0x0C])); // Ctrl+L
        assert_eq!(keycode_to_bytes(KeyCode::KeyZ, false, true), Some(vec![0x1A])); // Ctrl+Z
    }

    #[test]
    fn test_special_keys() {
        assert_eq!(keycode_to_bytes(KeyCode::Enter, false, false), Some(b"\r".to_vec()));
        assert_eq!(keycode_to_bytes(KeyCode::Tab, false, false), Some(b"\t".to_vec()));
        assert_eq!(keycode_to_bytes(KeyCode::Backspace, false, false), Some(b"\x7f".to_vec()));
    }

    #[test]
    fn test_arrow_keys() {
        assert_eq!(keycode_to_bytes(KeyCode::ArrowUp, false, false), Some(b"\x1b[A".to_vec()));
        assert_eq!(keycode_to_bytes(KeyCode::ArrowDown, false, false), Some(b"\x1b[B".to_vec()));
        assert_eq!(keycode_to_bytes(KeyCode::ArrowRight, false, false), Some(b"\x1b[C".to_vec()));
        assert_eq!(keycode_to_bytes(KeyCode::ArrowLeft, false, false), Some(b"\x1b[D".to_vec()));
    }

    #[test]
    fn test_unmapped_keys() {
        assert_eq!(keycode_to_bytes(KeyCode::ShiftLeft, false, false), None);
        assert_eq!(keycode_to_bytes(KeyCode::ControlLeft, false, false), None);
        assert_eq!(keycode_to_bytes(KeyCode::AltLeft, false, false), None);
    }
}
