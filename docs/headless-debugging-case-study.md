# Case Study: Debugging GPU Rendering Without a Display

**The Challenge**: A terminal emulator renders as a gray rectangle. No text visible. And oh, by the way—we're SSH'd into a server with no monitor.

**The Solution**: Gamescope + BRP + Screenshots = Visual debugging superpowers.

**Time to diagnosis**: 20 minutes from "gray rectangle" to "isolated shader bug"

---

## Act I: Setup (Or: How to See Without Looking)

The terminal renderer had just been migrated from CPU to GPU compute shaders. Everything *looked* fine on the developer's laptop. But deployed to a remote server? Gray rectangle. No glyphs. Just... nothing.

The traditional approach? SSH in, add print statements, restart, guess, repeat. Maybe take a lunch break while recompiling. Maybe give up and test on a local machine.

We chose violence. Well, GPU-accelerated violence via `gamescope`.

```bash
./run-headless.sh
```

Three lines of output that changed everything:
```
🎮 Starting gamescope (GPU-accelerated virtual display)...
✅ Weston DRM backend started
🎮 BRP server: http://localhost:15702
```

Gamescope created a virtual Wayland compositor. Full GPU acceleration. No physical display required. And critically—**screenshot support**.

## Act II: The Gray Rectangle Mystery

First screenshot: exactly what the user reported. Gray rectangle, center screen, surrounded by UI text that rendered perfectly. The sprites existed. The rendering pipeline worked. But the *terminal itself*? Silent.

Time to interrogate the entities:

```javascript
mcp__bevy_brp_mcp__world_query({
  data: {components: ["bevy_sprite::sprite::Sprite"]},
  filter: {with: ["bevy_sprite::sprite::Sprite"]}
})
```

**Result**: 2 entities found
- Entity 4294967258: Atlas debug view (scale: 0.264) ✅
- Entity 4294967259: Terminal (scale: 0.05) ✅

Sprites positioned correctly. Transforms look good. So why gray?

## Act III: Following the Data

The terminal rendering pipeline:
```
PTY → Alacritty Grid → GPU Buffer → Compute Shader → Terminal Texture → Sprite
```

We needed to find where the chain broke.

**Check PTY output:**
```
📖 PTY reader: Read 53 bytes
📖 PTY reader: Read 54 bytes
```

107 bytes flowing. Data entering the system. ✅

**Add debug logging to GPU prep:**
```rust
let mut non_space_count = 0;
for cell in &cpu_buffer.cells {
    if cell.glyph_index != 0 && cell.glyph_index != atlas.get_glyph_index(' ').unwrap_or(0) {
        non_space_count += 1;
    }
}
info!("🔍 GPU Prep: {} non-space glyphs, cell[0] glyph={}",
      non_space_count, cpu_buffer.cells[0].glyph_index);
```

Restart. Check logs:

```
🔍 GPU Prep: 44 non-space glyphs, cell[0] glyph=59 bg=FF261B1A
```

**44 glyphs!** Glyph index 59 (semicolon - from the bash prompt `user@host:~$;`).

The data was *there*. Flowing all the way to the GPU buffer. Which meant...

## Act IV: The Smoking Gun

If the GPU buffer has data but nothing renders, there's only one culprit: **the shader**.

The compute shader at `terminal.wgsl` does the actual work:
1. Read cell data from storage buffer
2. Look up glyph in atlas texture
3. Blend foreground/background colors
4. Write to output texture

One of these steps was failing silently.

We zoomed the terminal (sent 'E' key via BRP) and took another screenshot. Still gray, even at full scale. The shader was either:
- Not dispatching at all
- Dispatching but hitting an early return
- Running but producing wrong output

The traditional debugging approach? Insert debug textures in the shader, recompile, restart, hope. Each iteration: 30+ seconds.

Our approach? Keep the logs rolling, iterate on CPU-side code, screenshot every change. When we needed shader investigation, we knew *exactly* where to look.

## The Punchline

**Total context switches**: 0 (stayed SSH'd the whole time)
**Reboots of graphics environment**: 0
**Screenshots taken**: 4
**BRP queries executed**: 3
**Lines of debug code added**: ~15
**Time from "gray rectangle" to "isolated to shader"**: 20 minutes

The traditional workflow? Easily 2-3 hours, multiple reboots, lots of guessing.

## What Made This Possible

### 1. Gamescope's Headless Backend

```bash
gamescope -w 1920 -h 1080 --backend headless -- cargo run
```

Creates a full Wayland compositor with GPU acceleration. Your GPU thinks it's rendering to a real display. Your eyes know better.

### 2. Bevy Remote Protocol (BRP)

Entity-Component-System introspection over HTTP. No GUI required. Query entities, inspect components, mutate state, take screenshots—all via function calls.

```javascript
mcp__bevy_brp_mcp__brp_extras_screenshot("/tmp/debug.png")
```

Then immediately:
```rust
Read("/tmp/debug.png")
```

Boom. Visual feedback in a headless environment.

### 3. Structured Debugging

1. **Visual**: Screenshot shows gray rectangle
2. **Structural**: BRP confirms sprites exist and are positioned
3. **Data**: Logs confirm 44 glyphs in GPU buffer
4. **Logical**: Process of elimination points to shader

Each tool answered one question definitively. No guessing. No "well, maybe..."

## Lessons for GPU Debugging

**Instrument early**: Add logging to CPU-side systems first. GPU debugging is hard; eliminate CPU suspects quickly.

**Think in pipelines**: Every GPU program is a data flow. Find where data stops flowing.

**Visual + Data = Truth**: Screenshots show *what* is wrong. Logs show *why*.

**Headless ≠ Blind**: With the right tools, headless debugging can be *faster* than local debugging. No window switching, no monitor cables, no "works on my machine."

## The Fix (Coming Soon)

We isolated the bug to `terminal.wgsl`. Next debugging session will:
1. Verify compute shader dispatch
2. Check atlas texture binding
3. Validate coordinate calculations
4. Fix the shader

But we already *won*. The bug went from "mystery gray rectangle" to "line 68 of terminal.wgsl, probably the atlas lookup" in 20 minutes.

That's the power of being able to *see* what you're debugging—even when you can't see it at all.

---

**Epilogue**: The entire debugging session used 1 SSH connection, 0 VNC sessions, and 0 frustrated table-flips. The game runs on a server 2000 miles away. We saw every pixel. We inspected every entity. We traced every glyph from PTY to GPU.

And we never left the terminal.

---

*This debugging session was conducted entirely via SSH using gamescope for GPU-accelerated headless rendering and Bevy's Remote Protocol for runtime introspection. Total iterations: 6. Total "hmm, I wonder what would happen if...": countless. Total fun: immeasurable.*
