# Claude CRT Example - Bevy Remote Protocol (BRP) Setup

This example demonstrates terminal rendering with BRP support for remote debugging and iteration.

## Running Modes

### 🪟 Windowed Mode (Default)
```bash
cargo run -p claude_crt
```

### 🖥️ Fullscreen Mode
```bash
cargo run -p claude_crt -- --fullscreen
```

### 🎮 Headless Mode with Gamescope (For Remote Debugging)

**Recommended:** Use the root-level script for GPU-accelerated headless rendering:

```bash
# From project root
./run-headless.sh
```

This provides:
- ✅ Full GPU acceleration via gamescope
- ✅ Screenshot support via BRP (works in headless!)
- ✅ Same rendering code path as windowed mode
- ✅ Automatic GPU detection and Vulkan ICD configuration

**Requirements:**
- `gamescope` installed (Arch: `pacman -S gamescope`)
- GPU with Vulkan support
- User in `video` group for DRM access

**Manual gamescope launch:**
```bash
gamescope -w 1920 -h 1080 -W 1920 -H 1080 --backend headless -- \
  cargo run -p claude_crt --quiet -- --fullscreen
```

**Alternative - True Headless (No Screenshots):**
```bash
cargo run -p claude_crt -- --headless
```
Note: Screenshot BRP method won't work in true headless mode.

## Bevy Remote Protocol (BRP)

BRP server runs on **http://localhost:15702** (all modes).

### Available via BRP:
✅ Entity/component queries
✅ Resource inspection
✅ Component mutation
✅ Window title changes
✅ Keyboard input injection
✅ Screenshots (works with gamescope headless mode!)

### Example BRP Usage

**Query sprites:**
```bash
curl -X POST http://localhost:15702 \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "world.query",
    "params": {
      "data": {"option": ["bevy_sprite::sprite::Sprite"]},
      "filter": {"with": ["bevy_sprite::sprite::Sprite"]}
    }
  }'
```

**Send keyboard input:**
```bash
curl -X POST http://localhost:15702 \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "brp_extras/send_keys",
    "params": {"keys": ["KeyH", "KeyI"]}
  }'
```

## Gamescope Setup Verification

```bash
# Check gamescope is installed
command -v gamescope

# Check gamescope version (needs 3.14+)
gamescope --help | head -1

# Check DRM devices
ls -la /dev/dri/

# Verify permissions
groups | grep video
```

### Troubleshooting

**"gamescope: unrecognized option"**
→ Gamescope version too old.
→ Solution: Update gamescope to 3.14+ (`pacman -Syu gamescope` on Arch)

**Permission denied on /dev/dri**
→ User not in `video` group.
→ Solution: `sudo usermod -aG video $USER` (logout/login required)

**Screenshots show blank/black screen**
→ Shader or rendering pipeline issue.
→ Solution: Check logs for terminal.wgsl errors, verify GPU buffer initialization

## Architecture

- **Headless mode**: No window, ScheduleRunnerPlugin @ 60fps
- **Window modes**: WinitPlugin with optional fullscreen
- **BRP**: Always enabled via `bevy_brp_extras`

## Controls (Windowed/Fullscreen)

- **E**: Toggle zoom (tiny ↔ fullscreen)
- **D**: Toggle debug atlas view
- **ESC**: Exit

## MCP Integration

Use `bevy_brp_mcp` MCP server for Claude Code integration:
- Entity queries
- Component inspection
- State mutation
- Screenshots (works with gamescope headless mode!)

See `AGENT_GUIDE.md` for full development workflow.
