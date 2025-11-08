# Continuation Prompt for HalfRemembered Bevy Alacritty Terminal Plugin

## Project Context

We're building a Bevy 0.17 plugin for embedding terminal emulation in games. The architecture uses render-to-texture, where the terminal renders to an `Image` texture that can be used as a sprite, UI element, or material.

**Tech Stack:**
- Bevy 0.17 (game engine with ECS)
- alacritty_terminal 0.25 (ANSI/VT parsing and grid management)
- portable-pty 0.8 (cross-platform PTY management)
- ab_glyph 0.2 (font rendering)

**Repository:** `/home/atobey/src/halfremembered-bevy-alacritty`

**Version Control:** We use Jujutsu (jj), not git. Changes persist across rebases via change IDs.

## Current Status: Phase 1 Complete ✅

**Phase 1: Terminal Backend** - Fully functional bidirectional I/O

Implemented in jj changes:
- `mkmuxxwp`: Phase 1.1-1.2 (PTY spawning + Terminal state)
- `vxqlmzmz`: Phase 1.3-1.4 + Gemini review improvements

**What Works:**
- ✅ Persistent PTY with robust shell spawning (bash → zsh → sh fallback)
- ✅ Terminal grid state (120×30, alacritty_terminal integration)
- ✅ PTY polling system (4KB/frame, non-blocking)
- ✅ Keyboard input with **full Shift and Ctrl modifier support**
- ✅ Essential terminal operations (Ctrl+C interrupt, Ctrl+D EOF, uppercase, symbols)
- ✅ 9 passing unit tests
- ✅ Clean encapsulation (TerminalState::process_bytes method)

**Key Files:**
- `bevy-terminal/src/pty.rs` - PTY spawning and polling
- `bevy-terminal/src/terminal.rs` - Plugin and TerminalState
- `bevy-terminal/src/input.rs` - Keyboard mapping with modifiers
- `bevy-terminal/src/colors.rs` - Tokyo Night colors (hardcoded)

**Test Results:**
```bash
cargo test --lib -p bevy-terminal
# 9 tests pass: PTY spawn/IO, keyboard mappings (lowercase, uppercase, symbols, Ctrl)
```

**Gemini Reviews Completed:**
- Phase 1.1: `docs/reviews/phase-1.1-pty-spawning.md` (addressed)
- Phase 1.3-1.4: `docs/reviews/phase-1.3-1.4-pty-io.md` (addressed)

## Next Steps: Phase 2 - Font System & Glyph Atlas

**Goal:** Load fonts, generate high-quality glyph atlas, prepare for render-to-texture

**Tasks:**
1. **Font Loading** (`bevy-terminal/src/font.rs`)
   - Load Cascadia Mono from `bevy-terminal/assets/fonts/CascadiaMono-Regular.ttf` (already downloaded ✅)
   - Parse with ab_glyph
   - Calculate font metrics (cell width, cell height, baseline)

2. **Glyph Atlas Generation** (`bevy-terminal/src/atlas.rs`)
   - Create 4096×4096 (or larger) RGBA texture
   - Pre-render ASCII (32-126)
   - Pre-render box-drawing characters (U+2500-257F)
   - Pre-render block elements (U+2580-259F)
   - Build UV coordinate map (char → Rect)

3. **Quality Verification**
   - Display atlas as sprite in debug mode
   - Verify glyphs are crisp and aligned
   - Ensure box-drawing characters connect perfectly

**Reference:** See `docs/plan-bootstrap/03-implementation-roadmap.md` Phase 2 section (lines 231-400)

**Success Criteria:**
- Font loads successfully
- Atlas texture generated (4096×4096 RGBA)
- All target characters rendered clearly
- Test sprite shows atlas contents
- Glyphs are crisp with proper alignment

## Important Context

**Development Guidelines (from BOTS.md):**
- Use `jj new -m "..."` to start new work
- Update `jj describe -m "..."` frequently (non-interactive mode for agents)
- Strong types, descriptive names, no abbreviations
- `anyhow::Result` for all fallible operations, never `unwrap()`
- Add context with `.context()` for debugging
- Comments explain "why" not "what"

**Jujutsu Commands:**
```bash
jj log -r 'mine()' -n 5          # Check recent work
jj show @                         # Current change details
jj new -m "feat: <description>"  # Start new work
jj describe -m "..."             # Update description (use -m for non-interactive)
jj diff                          # Review changes
```

**Agent Attribution:**
Add to jj descriptions:
```
Co-authored-by: Claude <claude@anthropic.com>
```

**Build and Test:**
```bash
cargo build                       # Should succeed with minor warnings
cargo test --lib -p bevy-terminal # 9 tests should pass
cargo run --example claude_crt    # Opens window (requires display)
```

## Key Documentation

- `docs/plan-bootstrap/` - Complete bootstrap planning (7 files)
- `docs/collaboration.md` - Agent-human collaboration approach
- `docs/reviews/` - Gemini code reviews
- `docs/technical-design.md` - Architecture details
- `README.md` - Project overview

## Current Todo (If Needed)

Phase 2 tasks can be broken down:
1. Implement FontMetrics struct and loading
2. Implement GlyphAtlas generation
3. Wire into plugin as resources
4. Add debug sprite system for verification
5. Request Gemini review

## How to Continue

1. Review this prompt and key documentation
2. Check jj status: `jj log -r 'mine()' -n 5`
3. Start Phase 2: `jj new -m "feat: Implement font loading and metrics (Phase 2.1)"`
4. Implement incrementally, testing as you go
5. Request Gemini reviews at major milestones

## Important Notes

- **MVP scope:** Hardcoded 120×30, Tokyo Night colors, Cascadia Mono 14pt
- **No shortcuts:** Correctness and clarity over performance
- **Test frequently:** Add unit tests for new functionality
- **Use TodoWrite:** Track progress for multi-step tasks
- **Fonts already downloaded:** `bevy-terminal/assets/fonts/*.ttf` exists

---

**Context for next agent:** Pick up from Phase 2 (Font System). Phase 1 terminal backend is complete and tested. All core I/O works. Now we need visual rendering capabilities.
