# halfremembered-bevy-alcritty

A Bevy plugin for embedding fully functional terminal emulation in games.

## What is this?

This plugin integrates terminal emulation into Bevy 0.17 games, allowing in-game computers or terminals that run real shells, editors (nvim), and tools. Built using `alacritty_terminal` for ANSI/VT sequence handling and `portable-pty` for cross-platform PTY management.

## Architecture

- **Terminal Emulation**: alacritty_terminal for grid management and ANSI escape sequences
- **PTY Management**: portable-pty for spawning and managing shell processes
- **Rendering**: Sprite-based approach with pre-rendered glyph atlas
- **Plugin Design**: Clean Bevy plugin architecture for reusability

## Use Case

Originally created for the "Endgame of SRE" game port, where players interact with in-game computers to access terminal sessions running real shells and development tools.

## Quick Start

### Build and Run

```bash
# Build and run the example
cargo run --example claude_crt --release

# Or build standalone binary
cargo build --release --example claude_crt
./target/release/examples/claude_crt
```

**What to expect:**
- Window opens (1920×1080) with live terminal
- Shell prompt appears automatically (bash/zsh/sh)
- Type commands → see output rendered in real-time at 60 FPS
- Press **'D'** to toggle debug atlas view
- Terminal is fully interactive with proper keyboard input

### Using HalfRemembered Launcher (Windows/WSL)

For easy deployment to Windows/WSL machines:

```bash
# On dev machine (Linux)
cargo build --release --example claude_crt
halfremembered-launcher config-sync --server your-laptop

# On laptop (WSL or Windows)
~/bin/claude_crt
```

See `docs/launcher-guide.md` for complete setup instructions.

## Current Status: MVP Complete ✅

**Phase 1: Terminal Backend** (Complete)
- ✅ PTY spawning and management
- ✅ Terminal state (alacritty_terminal integration)
- ✅ PTY polling (non-blocking I/O)
- ✅ Keyboard input (full Shift/Ctrl support)

**Phase 2: Font System** (Complete)
- ✅ Font loading (Cascadia Mono 14pt)
- ✅ Glyph atlas generation (4096×4096, 255 chars)
- ✅ Anti-aliased rendering

**Phase 3: Render-to-Texture** (Complete)
- ✅ Terminal grid → texture rendering
- ✅ Tokyo Night color scheme
- ✅ Real-time updates (60 FPS)
- ✅ Alpha-blended glyph compositing

**Tests:** 15/15 passing 🎉

## Documentation

- **Technical Design**: `docs/technical-design.md` - Architecture and implementation strategy
- **Bootstrap Plan**: `docs/plan-bootstrap/` - Detailed phase-by-phase roadmap
- **Launcher Guide**: `docs/launcher-guide.md` - Windows/WSL deployment
- **Code Reviews**: `docs/reviews/` - Gemini code review feedback
