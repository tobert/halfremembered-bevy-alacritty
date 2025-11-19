use alacritty_terminal::vte::ansi::Color;

/// Background color used for terminal (Tokyo Night Dark)
pub const TOKYO_NIGHT_BG: [u8; 3] = [0x1a, 0x1b, 0x26];

/// Convert alacritty color to RGB array.
///
/// Handles named colors (using Tokyo Night theme) and RGB colors.
pub fn convert_alacritty_color(color: Color) -> [u8; 3] {
    match color {
        Color::Named(named) => {
            use alacritty_terminal::vte::ansi::NamedColor;
            match named {
                NamedColor::Black => [0x1a, 0x1b, 0x26],
                NamedColor::Red => [0xf7, 0x76, 0x8e],
                NamedColor::Green => [0x9e, 0xce, 0x6a],
                NamedColor::Yellow => [0xe0, 0xaf, 0x68],
                NamedColor::Blue => [0x7a, 0xa2, 0xf7],
                NamedColor::Magenta => [0xbb, 0x9a, 0xf7],
                NamedColor::Cyan => [0x7d, 0xcf, 0xff],
                NamedColor::White => [0xc0, 0xca, 0xf5],
                NamedColor::BrightBlack => [0x41, 0x4b, 0x6b],
                NamedColor::BrightRed => [0xf7, 0x76, 0x8e],
                NamedColor::BrightGreen => [0x9e, 0xce, 0x6a],
                NamedColor::BrightYellow => [0xe0, 0xaf, 0x68],
                NamedColor::BrightBlue => [0x7a, 0xa2, 0xf7],
                NamedColor::BrightMagenta => [0xbb, 0x9a, 0xf7],
                NamedColor::BrightCyan => [0x7d, 0xcf, 0xff],
                NamedColor::BrightWhite => [0xc0, 0xca, 0xf5],
                NamedColor::Foreground => [0xc0, 0xca, 0xf5],
                NamedColor::Background => TOKYO_NIGHT_BG,
                _ => [0xc0, 0xca, 0xf5], // Default to foreground
            }
        }
        Color::Spec(rgb) => [rgb.r, rgb.g, rgb.b],
        Color::Indexed(index) => {
            // 256-color palette - for MVP, just use a simple mapping
            match index {
                0 => [0x1a, 0x1b, 0x26],  // Black
                1 => [0xf7, 0x76, 0x8e],  // Red
                2 => [0x9e, 0xce, 0x6a],  // Green
                3 => [0xe0, 0xaf, 0x68],  // Yellow
                4 => [0x7a, 0xa2, 0xf7],  // Blue
                5 => [0xbb, 0x9a, 0xf7],  // Magenta
                6 => [0x7d, 0xcf, 0xff],  // Cyan
                7 => [0xc0, 0xca, 0xf5],  // White
                _ => [0xc0, 0xca, 0xf5],  // Default
            }
        }
    }
}