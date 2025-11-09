# Next Steps: Windows PTY Debugging

**Date**: 2025-11-18
**Status**: Fix implemented (Background thread + Handle ownership), ready for verification

## Current State

✅ **Completed:**
- Phase 1-2: PTY + Terminal backend + Font/Glyph Atlas
- Phase 3: Render-to-texture implementation
- Phase 4: Zoom interaction demo
- Windows MSVC build setup
- **FIXED:** Windows PTY blocking I/O hang (implemented background reader thread)
- **FIXED:** Windows PTY immediate EOF (implemented persistent handle ownership)

## The Fix Implementation

We refactored `PtyResource` to:
1.  **Own `MasterPty` and `SlavePty`:** Kept these alive in the resource to prevent the PTY session from closing (which caused immediate EOF).
2.  **Background Thread:** Moved `reader.read()` to a dedicated thread that sends data via `mpsc::channel`. This prevents the blocking `read()` call on Windows from freezing the main Bevy loop.

## Verification Needed

Please verify on Windows:
1.  `cargo build-windows` (or build locally on Windows)
2.  Run `claude_crt.exe`
3.  Ensure no immediate "PTY EOF" logs.
4.  Ensure terminal accepts input and shows output (e.g. type `dir`).


## The Windows PTY Problem

**Symptoms:**
1. Spammy PTY error messages in console (exact error not yet captured)
2. Application hangs on second launch
3. Small dark rectangle displays (terminal at 0.05 scale) but behavior unclear

**What We Need:**

### 1. Capture Full Diagnostic Output

Run the Windows binary with console output visible:

```powershell
# From Windows PowerShell (NOT WSL)
cd C:\Users\YourUser\bin
.\claude_crt.exe
```

**Capture these log lines:**
```
🔧 Initializing PTY system...
🔧 Opening PTY pair (120×30 cells)...
🐚 Spawning shell: C:\Windows\System32\cmd.exe
🔧 Spawning child process...
🔧 Cloning PTY reader...
🪟 Using Windows ConPTY (non-blocking handled by portable-pty)
🔧 Taking PTY writer...
✅ PTY initialized successfully
```

**Look for error messages showing:**
- Error kind (WouldBlock, BrokenPipe, Other, etc.)
- OS error code (Windows system error number)
- Which initialization step fails or hangs

### 2. Check Windows Requirements

**ConPTY requires Windows 10 version 1809 or later.**

Check your Windows version:
```powershell
winver
# Or
systeminfo | findstr /B /C:"OS Version"
```

If < 1809, ConPTY is not available and we'll need a fallback strategy.

### 3. Test portable-pty Directly

Create a minimal test to isolate the issue:

```rust
// test-pty.rs
use portable_pty::{native_pty_system, CommandBuilder, PtySize};

fn main() -> anyhow::Result<()> {
    println!("Testing portable-pty on Windows...");

    let pty_system = native_pty_system();
    let pair = pty_system.openpty(PtySize {
        rows: 24,
        cols: 80,
        pixel_width: 0,
        pixel_height: 0,
    })?;

    println!("PTY created successfully");

    let mut cmd = CommandBuilder::new("cmd.exe");
    let child = pair.slave.spawn_command(cmd)?;

    println!("Shell spawned successfully");

    let mut reader = pair.master.try_clone_reader()?;
    let mut buf = [0u8; 1024];

    use std::io::Read;
    match reader.read(&mut buf) {
        Ok(n) => println!("Read {} bytes", n),
        Err(e) => println!("Read error: {} (kind: {:?}, os: {:?})",
            e, e.kind(), e.raw_os_error()),
    }

    Ok(())
}
```

Build and run:
```powershell
cargo new --bin test-pty
cd test-pty
# Add portable-pty = "0.8" to Cargo.toml
cargo build --release
.\target\release\test-pty.exe
```

## Possible Root Causes

### 1. ConPTY Availability
- **Fix**: Check Windows version, consider fallback to WinPTY
- **Code**: Add runtime ConPTY detection

### 2. Non-Blocking I/O on Windows
- Windows handles non-blocking differently than Unix
- `portable-pty` should handle this, but may have bugs
- **Fix**: Test with blocking reads first, add async I/O later

### 3. Shell Path Issues
- COMSPEC environment variable not set
- cmd.exe permissions
- **Fix**: Add better shell detection and error messages (already done in latest code)

### 4. Read Buffer Size
- 4KB buffer may be too large for Windows
- **Fix**: Try smaller buffer (1KB or 512 bytes)

### 5. Every-Frame Polling
- Bevy Update runs at 60 FPS
- 60 read attempts/sec may overwhelm Windows PTY
- **Fix**: Add frame throttling or use event-based I/O

## Alternative Approaches

If ConPTY continues to be problematic:

### Option A: Use WinPTY Backend
```toml
[dependencies]
portable-pty = { version = "0.8", features = ["winpty"] }
```

### Option B: Windows-Only: Use Named Pipes
- Spawn cmd.exe with redirected stdin/stdout
- Use Windows named pipes for I/O
- More reliable but Windows-specific code

### Option C: Feature Flag for PTY
```rust
#[cfg(not(windows))]
use pty_approach;

#[cfg(windows)]
use pipe_approach;
```

## Build Commands Reference

**Linux:**
```bash
cargo build --release
./target/release/claude_crt
```

**Windows MSVC (from Linux via xwin):**
```bash
cargo build-windows -p claude_crt
# Or
cargo xwin build --release --target x86_64-pc-windows-msvc -p claude_crt
```

**Windows MSVC (native):**
```powershell
cargo build --release
.\target\release\claude_crt.exe
```

## File Locations

**Binaries:**
- Linux: `target/release/claude_crt`
- Windows: `target/x86_64-pc-windows-msvc/release/claude_crt.exe`

**Key Source Files:**
- PTY logic: `bevy-terminal/src/pty.rs` (lines 63-144 for spawn, 146-198 for poll)
- Input routing: `bevy-terminal/src/input.rs` (lines 15-40)
- Zoom system: `examples/claude_crt/src/main.rs` (lines 59-195)

**Diagnostics Added:**
- Detailed PTY init logging (lines 64-137 in pty.rs)
- OS error codes in poll_pty (lines 186-195 in pty.rs)
- Shell detection logging (lines 93-94 in pty.rs)

## Expected Behavior (Working on Linux)

1. **Startup:** Small dark rectangle (54×28 pixels) at 0.05 scale
2. **Press E:** Smooth zoom to fullscreen, bash prompt visible
3. **Type commands:** Live terminal updates, text appears in texture
4. **Press ESC:** Smooth zoom back to tiny
5. **Press D:** Toggle debug atlas view (font glyphs)

## HalfRemembered Launcher

Already configured in `.hrlauncher.toml`:
```bash
# Sync to Windows/WSL laptop
halfremembered-launcher config-sync --server your-server
```

Syncs both binaries:
- `~/bin/claude_crt` (Linux, for WSL)
- `~/bin/claude_crt.exe` (Windows MSVC)

## Questions to Answer

1. What exact error messages appear in the Windows console?
2. Does the application hang during PTY initialization or during polling?
3. What Windows version are you running? (check `winver`)
4. Does the minimal test-pty example work?
5. Can you run PowerShell or cmd.exe directly without issues?

## Jujutsu Status

Current change:
```bash
jj show @
# knrltqkl: feat: Add Claude CRT character with zoom interaction (Phase 4)
```

Changes since last push:
- Phase 4 implementation (zoom, input routing)
- Windows build setup (cargo-xwin, .cargo/config.toml)
- PTY diagnostics and error handling improvements

**To commit diagnostics improvements:**
```bash
jj new -m "fix: Add Windows PTY diagnostics and improved error handling"
jj describe  # Add detailed notes about the Windows issues
```

## Contact Points for Help

- **portable-pty**: https://github.com/wez/wezterm/tree/main/pty
  - Maintained by wez (WezTerm author) - highly reliable
  - Check issues for Windows-specific problems

- **ConPTY docs**: https://devblogs.microsoft.com/commandline/windows-command-line-introducing-the-windows-pseudo-console-conpty/

## Success Criteria

Windows binary should:
1. Spawn cmd.exe/PowerShell without errors
2. Display tiny terminal at 0.05 scale
3. Zoom smoothly on E key press
4. Accept input and show terminal updates
5. No console spam or hangs

---

**Next Session Start:**
```bash
cd ~/src/halfremembered-bevy-alacritty
jj log -r 'mine()' -n 3  # Check recent work
cargo build --release     # Ensure Linux still works
# Then debug Windows with captured diagnostics
```
