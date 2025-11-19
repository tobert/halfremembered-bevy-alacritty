# Agent Debugging Guide for claude_crt

This guide is for AI coding agents (Claude, Gemini, etc.) working on this project.

## Quick Start

### Launch the App for Debugging

```bash
# Smart launcher - automatically selects best mode
./run-headless.sh

# BRP will be available at: http://localhost:15702
```

The script automatically:
1. Detects your GPU and configures Vulkan ICD
2. Uses **gamescope** for GPU-accelerated headless rendering if no display available
3. Starts app with BRP enabled on port 15702

**Why gamescope:**
- Real GPU rendering (not software emulation)
- Screenshot support via BRP works in headless mode
- Same code path as windowed mode
- No special Vulkan extensions needed

**Requirements:**
- `gamescope` installed (Arch: `pacman -S gamescope`)
- GPU with Vulkan support
- User in `video` group for DRM access

## Using BRP (Bevy Remote Protocol)

### Required: bevy_brp_mcp MCP Server

This guide assumes `bevy_brp_mcp` MCP server is installed and configured.

**If MCP tools are not available**, prompt the user:
> "The bevy_brp_mcp MCP server is not configured. Please install it to enable BRP debugging tools. See: https://github.com/atobey/bevy_brp_mcp"

### Available MCP Tools

```
mcp__bevy_brp_mcp__world_query              # Query entities by components
mcp__bevy_brp_mcp__world_get_components     # Get component data
mcp__bevy_brp_mcp__world_list_components    # List all registered components
mcp__bevy_brp_mcp__world_list_resources     # List all resources
mcp__bevy_brp_mcp__brp_extras_screenshot    # Take screenshot (window modes only)
mcp__bevy_brp_mcp__brp_extras_send_keys     # Send keyboard input
```

## Common Debugging Workflows

### 1. Inspect Terminal Rendering State

```bash
# Find terminal sprite entities
mcp__bevy_brp_mcp__world_query({
  "data": {"option": ["bevy_sprite::sprite::Sprite", "bevy_transform::components::transform::Transform"]},
  "filter": {"with": ["bevy_sprite::sprite::Sprite"]}
})

# Expected: 2 entities
# - Entity with scale 0.05 (tiny CRT)
# - Entity with scale ~0.26 (atlas debug)
```

### 2. Check Compute Shader Execution

The terminal uses a compute shader at `bevy-terminal/assets/shaders/terminal.wgsl`.

**Key components to inspect:**
- Terminal texture: 960×420 pixels (120×30 cells)
- Atlas texture: 4096×4096 pixels (255 glyphs)
- GPU buffer: 3600 cells (120×30)

**Shader uniforms:**
- `term_cols`: 120
- `term_rows`: 30
- `cell_width`: 8
- `cell_height`: 14
- `atlas_cols`, `atlas_rows`: Based on 255 glyphs

### 3. Visual Debugging with BRP Screenshots

```bash
# Headless GPU-accelerated (recommended for agents)
./run-headless.sh

# With display
cargo run -p claude_crt -- --fullscreen

# Take screenshot via BRP (works in both modes!)
mcp__bevy_brp_mcp__brp_extras_screenshot("/tmp/debug.png")

# View screenshot
Read("/tmp/debug.png")
```

**Development iteration workflow:**
1. Launch: `./run-headless.sh` (in background or tmux pane)
2. Wait for: "BRP server: http://localhost:15702"
3. Inspect: Use MCP tools to query entities/components
4. Screenshot: `mcp__bevy_brp_mcp__brp_extras_screenshot("/tmp/render.png")`
5. View: `Read("/tmp/render.png")` to see rendered output
6. Iterate: Make code changes, restart, repeat

This enables visual debugging without a physical display!

### 4. Shader Validation

```bash
# Validate WGSL syntax
naga bevy-terminal/assets/shaders/terminal.wgsl

# Expected: "Validation successful"
```

## System Architecture

### Rendering Pipeline

1. **PTY**: Spawns bash shell, captures output
2. **Terminal Grid**: 120×30 cells, stores glyphs + colors
3. **Atlas**: Pre-rendered glyphs in 4096×4096 texture
4. **Compute Shader**: Reads grid + atlas → writes terminal texture
5. **Sprite**: Displays terminal texture (can zoom 0.05 ↔ 1.0)

### Key Files

```
bevy-terminal/
├── src/
│   ├── terminal.rs       # Main plugin, grid state
│   ├── pty.rs            # PTY management
│   ├── atlas.rs          # Glyph atlas generation
│   ├── renderer.rs       # Terminal texture creation
│   ├── gpu_prep.rs       # CPU→GPU buffer sync
│   └── render_node.rs    # Compute shader dispatch
└── assets/shaders/
    └── terminal.wgsl     # Compute shader (main rendering)

examples/claude_crt/
└── src/main.rs           # Example with BRP integration
```

### Debug Logging

Terminal components use `info!()` and `warn!()` macros.
Filter with `RUST_LOG`:

```bash
RUST_LOG=bevy_terminal=debug ./run-headless.sh
```

## Known Limitations

❌ **Sprite component serialization**: Asset handles don't fully serialize
✅ **Workaround**: Use Transform queries instead

✅ **Screenshots work in headless mode** with gamescope! (`run-headless.sh`)

## Troubleshooting

### "gamescope: unrecognized option"

Gamescope version too old or not installed.
- Solution: Update gamescope to 3.14+ or install it (`pacman -S gamescope` on Arch)

### "No write access to DRM devices"

Permission denied for `/dev/dri/card*`.
- Solution: Add user to `video` group: `sudo usermod -aG video $USER` (logout/login required)

### "neither WAYLAND_DISPLAY nor DISPLAY is set"

No display available and gamescope not working.
- Solution: Let `./run-headless.sh` handle this automatically, or manually run with gamescope:
  ```bash
  gamescope -w 1920 -h 1080 --backend headless -- cargo run -p claude_crt -- --fullscreen
  ```

### Screenshots show black/blank screen

Shader or rendering pipeline issue.
- Check logs for errors in terminal.wgsl compilation
- Verify GPU buffer initialization: `bevy_terminal::gpu_prep: Initializing CPU buffer`
- Query sprite entities to ensure they exist with correct transforms

### Shader not updating terminal texture

Check logs for:
```
bevy_terminal::gpu_prep: Initializing CPU buffer with 3600 cells
bevy_terminal::renderer: Creating terminal texture: 960×420 pixels
```

Query sprite entities to verify they exist with correct transforms.

## Agent Workflow Example

```
1. Launch app:
   ./run-headless.sh

2. Wait for "BRP server: http://localhost:15702"

3. Query entities:
   mcp__bevy_brp_mcp__world_query(...)

4. Inspect transforms:
   Check scale values (0.05 for tiny, 0.26 for atlas)

5. If window available:
   mcp__bevy_brp_mcp__brp_extras_screenshot("/tmp/debug.png")

6. Read screenshot to verify rendering
```

## Questions?

Check:
- `examples/claude_crt/README.md` - BRP setup details
- `bevy-terminal/src/*.rs` - Implementation
- Logs with `RUST_LOG=debug`
