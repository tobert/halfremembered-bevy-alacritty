# HalfRemembered Launcher Quick Start

## For Windows/WSL Laptop

### Prerequisites

1. **On your development machine** (this Linux box):
   - Build the launcher server: `cd ~/src/halfremembered-launcher && cargo build --release`
   - Ensure launcher binary is in PATH

2. **On your Windows/WSL laptop**:
   - Install halfremembered-launcher client
   - Ensure `~/bin/` is in your PATH

### Build and Sync

```bash
# On development machine (Linux)
cd ~/src/halfremembered-bevy-alacritty

# Build the example (Linux binary)
cargo build --release --example claude_crt

# Start config-based sync server
halfremembered-launcher config-sync --server your-laptop-hostname
```

The launcher will:
1. ✅ Load `.hrlauncher.toml` from this directory
2. ✅ Watch `target/release/claude_crt` for changes
3. ✅ Automatically sync to connected clients (your laptop)
4. ✅ Copy to `~/bin/claude_crt` on your laptop

### Run on Windows/WSL

Once synced, on your laptop:

```bash
# From WSL or Windows Terminal
~/bin/claude_crt

# Or if ~/bin is in PATH:
claude_crt
```

**Expected behavior:**
- Window opens with terminal display (1920×1080)
- Shell prompt appears automatically
- Type commands → see output rendered in real-time
- Press 'D' → toggle debug atlas view
- Terminal updates at 60 FPS

### For Windows Native Build

If you want to build for Windows (from Linux with cross-compilation):

```bash
# Install Windows target
rustup target add x86_64-pc-windows-gnu

# Build Windows binary
cargo build --release --target x86_64-pc-windows-gnu --example claude_crt

# Sync will automatically pick up:
# target/x86_64-pc-windows-gnu/release/claude_crt.exe
```

On Windows, run directly:
```powershell
# From PowerShell or Windows Terminal
~\bin\claude_crt.exe
```

### Manual Sync (Alternative)

If you prefer manual sync without the launcher:

```bash
# Using scp
scp target/release/claude_crt your-laptop:~/bin/

# Using rsync
rsync -avz target/release/claude_crt your-laptop:~/bin/
```

### Troubleshooting

**Binary won't run on laptop:**
- Check architecture: `file ~/bin/claude_crt`
- Ensure executable: `chmod +x ~/bin/claude_crt`
- Check dependencies: `ldd ~/bin/claude_crt`

**Window doesn't open:**
- Ensure X11 forwarding (WSL): Export DISPLAY correctly
- Windows native: Run .exe directly from Windows, not WSL
- Check Bevy dependencies are installed

**Terminal doesn't show:**
- Check logs in console output
- Look for "✅ Terminal texture initialized"
- Verify PTY spawned successfully

**Performance issues:**
- Expected: 60 FPS with ~1000 blit operations/frame
- If slower: Check GPU drivers
- Use `--release` build (debug is much slower)

## Development Workflow

```bash
# Edit code on dev machine
vim bevy-terminal/src/renderer.rs

# Rebuild
cargo build --release --example claude_crt

# Launcher auto-syncs to laptop (if config-sync running)

# Test on laptop immediately
ssh laptop "~/bin/claude_crt"
```

## Config Reference

The `.hrlauncher.toml` in project root defines:
- **linux-example**: Linux binary → `~/bin/claude_crt`
- **windows-example**: Windows .exe → `~/bin/claude_crt.exe`
- **docs**: Project docs → `~/bevy-terminal-docs/`
- **fonts**: Font assets → `~/bevy-terminal-assets/fonts/`

Modify as needed for your setup!
