//! Terminal color schemes.
//!
//! MVP: Hardcoded Tokyo Night theme

use bevy::prelude::*;

/// Tokyo Night background color as RGB bytes.
pub const TOKYO_NIGHT_BG: [u8; 3] = [0x1a, 0x1b, 0x26];

/// Tokyo Night color scheme (hardcoded for MVP)
#[derive(Debug, Clone)]
pub struct TokyoNightColors {
    pub background: Color,
    pub foreground: Color,
    // Standard ANSI colors (0-7)
    pub black: Color,
    pub red: Color,
    pub green: Color,
    pub yellow: Color,
    pub blue: Color,
    pub magenta: Color,
    pub cyan: Color,
    pub white: Color,
    // Bright ANSI colors (8-15)
    pub bright_black: Color,
    pub bright_red: Color,
    pub bright_green: Color,
    pub bright_yellow: Color,
    pub bright_blue: Color,
    pub bright_magenta: Color,
    pub bright_cyan: Color,
    pub bright_white: Color,
}

impl Default for TokyoNightColors {
    fn default() -> Self {
        Self {
            background: Color::srgb_u8(0x16, 0x16, 0x1e),
            foreground: Color::srgb_u8(0xc0, 0xca, 0xf5),
            black: Color::srgb_u8(0x15, 0x16, 0x1e),
            red: Color::srgb_u8(0xf7, 0x76, 0x8e),
            green: Color::srgb_u8(0x9e, 0xce, 0x6a),
            yellow: Color::srgb_u8(0xe0, 0xaf, 0x68),
            blue: Color::srgb_u8(0x7a, 0xa2, 0xf7),
            magenta: Color::srgb_u8(0xbb, 0x9a, 0xf7),
            cyan: Color::srgb_u8(0x7d, 0xcf, 0xff),
            white: Color::srgb_u8(0xa9, 0xb1, 0xd6),
            bright_black: Color::srgb_u8(0x41, 0x48, 0x68),
            bright_red: Color::srgb_u8(0xf7, 0x76, 0x8e),
            bright_green: Color::srgb_u8(0x9e, 0xce, 0x6a),
            bright_yellow: Color::srgb_u8(0xe0, 0xaf, 0x68),
            bright_blue: Color::srgb_u8(0x7a, 0xa2, 0xf7),
            bright_magenta: Color::srgb_u8(0xbb, 0x9a, 0xf7),
            bright_cyan: Color::srgb_u8(0x7d, 0xcf, 0xff),
            bright_white: Color::srgb_u8(0xc0, 0xca, 0xf5),
        }
    }
}
