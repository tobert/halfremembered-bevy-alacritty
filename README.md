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

## Documentation

See `docs/technical-design.md` for detailed technical architecture, implementation strategy, and font rendering guidelines.
