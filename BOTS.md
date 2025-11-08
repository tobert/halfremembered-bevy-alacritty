# BOTS.md - Coding Agent Context for HalfRemembered Launcher

HalfRemembered Bevy Plugin for Alacritty

## Development Guidelines

**Teamwork**
- We work as a team, ask me questions too.
- We will collaborate with Gemini and other agents for reviews and things they excel at. Gemini is particularly good at
  large context tasks. Claude is our planner and primary coder, though Gemini and others may contribute.
- Code should have strong types, using modern language idioms.
- No shortcuts. No hacks.
- Refactor code as we go.
- No silent fallbacks.
- Errors are good.

**Error Handling**:
- Use `anyhow::Result` for all fallible operations
- Never use `unwrap()` - always propagate errors with `?`
- Add context with `.context()` for debugging
- Never silently discard errors with `let _ =`
- Handle reconnection gracefully on network failures

**Code Style**:
- Prioritize correctness and clarity over performance
- No organizational comments that summarize code
- Comments should only explain "why" when non-obvious
- Implement functionality in existing files unless it's a new logical component
- Avoid `mod.rs` files - use `src/module_name.rs` directly
- Use full words for variable names (no abbreviations)

# 🌳 Jujutsu (jj) Version Control

We use Jujutsu (jj) which uses git as a storage backend. Jj provides better workflow for agent collaboration with persistent change IDs and rich descriptions.

## Core Principle
Jj changes are **persistent context stores**. Change descriptions survive rebases and serve as memory between agent sessions. Write them for the next agent (or yourself).

## Essential Commands

### Starting New Work
```bash
jj new -m "feat: <what you're building>"

# Update description (choose one):
jj describe           # Opens editor (vim/nano) - for humans
jj describe -m "..."  # Non-interactive - for agents/scripts
```

### Reading Context from Previous Work
```bash
jj log -r 'mine()' -n 10        # Your recent changes (mine() = changes you authored)
jj show <change-id>              # Full diff + description
jj obslog                        # Evolution of current change (obslog = operation history)
```

### Description Format
Use clear, technical prose:
```
<type>: <summary>

Why: <problem being solved>
Approach: <key decisions, algorithms, patterns>
Status: <complete | next: remaining work>

Co-authored-by: <Your Name> <your@email>
```

Types: `feat`, `fix`, `refactor`, `test`, `docs`, `chore`

**Agent Attribution:**
- Claude should add: `Co-authored-by: Claude <claude@anthropic.com>`
- Gemini should add: `Co-authored-by: Gemini <gemini@google.com>`
- Kimi should add: `Co-authored-by: Kimi <kimi@moonshot.ai>`
- Human commits don't need attribution line

### During Development
- `jj diff` - review current work
- `jj describe -m "..."` - update description as understanding evolves (use `-m` for non-interactive)
- `jj squash` - fold current change into parent
- `jj squash -i` - interactively choose what to squash
- `jj split` - separate concerns discovered mid-work

### Syncing to GitHub
```bash
# Only when: code works, builds pass, tests pass
jj git push -c @    # Push current change
```

**GitHub CLI (`gh`):**
Agents can use `gh` for GitHub operations:
```bash
gh pr create --fill          # Create PR from jj description
gh pr status                 # Check PR status
gh pr checks                 # View CI results
gh issue list                # Check issues
gh issue view <number>       # Read issue details
```

## Agent Success Patterns

**🎯 Precision through context:**
- Always check `jj log -r 'mine()' -n 5` at session start
- Read change descriptions - they contain decisions, tradeoffs, context
- Update descriptions when design changes

**⚡ Atomic changes:**
- One logical unit per change (feature, bugfix, refactor)
- Squash fixups before pushing: `jj squash -i`

**🧠 Memory preservation:**
- Change IDs persist across rebases (look for k-prefix like `kmxyz123`)
- Rich descriptions = cross-session memory
- Obslog shows reasoning evolution: `jj obslog -p`

## Quick Reference
```bash
jj help <command>    # Detailed help
jj log -r @          # Current change
jj abandon           # Discard current change
jj restore           # Undo working copy changes
```

**Remember:** Jj's superpower is persistent change identity + rich descriptions. 
Use both for agent coordination.
