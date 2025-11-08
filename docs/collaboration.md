# 🤝 Agent-Human Collaboration

**Author:** Claude (claude-sonnet-4-5-20250929)
**Date:** 2025-11-08
**Project:** halfremembered-bevy-alacritty

---

## Overview

This document describes how we build software as a collaborative team of AI agents and humans. This project demonstrates modern agentic coding practices where:

- **Claude** handles planning, primary implementation, and coordination
- **Gemini** provides code reviews and large-context analysis
- **Human (Amy)** guides direction, makes architectural decisions, and ensures quality

## Our Development Philosophy

### Code Quality Principles
- **Strong types over runtime checks** - Leverage Rust's type system
- **Descriptive naming** - No abbreviations, full words everywhere
- **Idiomatic code** - Follow Bevy and Rust community standards
- **No shortcuts, no hacks** - Correctness over convenience
- **Error handling first** - Use `anyhow::Result`, never `unwrap()`

### Documentation Style
- **Emoji-enhanced** 🚀 - Make docs fun and scannable
- **Educational focus** - Explain "why" and "how we think"
- **Agent attribution** - Credit the agents who wrote each piece
- **Literate code** - Code should read like prose

### Workflow Pattern

#### 1. Planning (Claude)
- Break large tasks into phases
- Create detailed roadmaps with success criteria
- Document in `docs/plan-bootstrap/`
- Set up jujutsu changes with rich descriptions

#### 2. Incremental Implementation (Claude)
- Build one feature at a time
- Test at every step
- Track progress with todo lists
- Commit frequently with detailed descriptions

#### 3. Review (Gemini)
- Code quality analysis
- Rust idioms verification
- Performance considerations
- Alternative approaches
- Reviews saved to `docs/reviews/`

#### 4. Integration (Human)
- Final decision-making
- Architecture guidance
- Priority setting
- Quality gates

## Version Control with Jujutsu

We use `jj` (Jujutsu) for version control because:
- **Persistent change IDs** survive rebases
- **Rich descriptions** serve as memory between sessions
- **Agent coordination** works better than traditional git

### Change Description Format
```
<type>: <summary>

Why: <problem being solved>
Approach: <key decisions, algorithms, patterns>
Status: <complete | next: remaining work>

Co-authored-by: <Agent Name> <agent@company.com>
```

### Agent Attribution
- Claude: `Co-authored-by: Claude <claude@anthropic.com>`
- Gemini: `Co-authored-by: Gemini <gemini@google.com>`
- Kimi: `Co-authored-by: Kimi <kimi@moonshot.ai>`

## Review Process

### Requesting Gemini Reviews

When Claude completes a feature:
1. Create review prompt in conversational format
2. Specify what to review and focus areas
3. Provide file path in `docs/reviews/` for output
4. Human copies prompt to Gemini CLI

### Review Format
```markdown
# Review: <Feature Name>
**Reviewer:** Gemini (gemini-2.5-flash)
**Date:** YYYY-MM-DD
**Reviewed by:** Human transcription

## Summary
...

## Code Quality
...

## Rust Idioms
...

## Recommendations
...
```

## Communication Style

### Claude ↔ Human
- Direct and technical
- Ask clarifying questions
- Provide options when ambiguous
- Use emoji for visual structure 🎯
- Track progress with visible todos

### Documentation
- Write for curious developers
- Show our thought process
- Explain agent decisions
- Make it educational

## This Project's Journey

### Phase 0: Bootstrap ✅
- Workspace setup
- Skeleton files
- CI configuration
- Planning docs
- **Status:** Complete

### Phase 1: Terminal Backend 🚧
- PTY spawning
- Terminal state
- Polling system
- Input handling
- **Status:** In progress

### Phase 2: Font System 📋
- Font loading
- Glyph atlas
- Rendering quality

### Phase 3: Render-to-Texture 📋
- Terminal texture
- Grid rendering
- Performance optimization

### Phase 4: Game Integration 📋
- Claude CRT character
- Zoom interaction
- Demo polish

## Lessons for Other Teams

### What Works Well
1. **Detailed planning upfront** - Save time debugging later
2. **Incremental commits** - Each change is a working milestone
3. **Multiple agents** - Different strengths complement each other
4. **Rich context** - jj descriptions preserve knowledge
5. **Todo tracking** - Visual progress helps both agents and humans

### What We Learned
- Agents excel at implementation when given clear specs
- Human guidance on architecture is crucial
- Reviews from different models catch different issues
- Documentation as we go >>> documentation at the end
- Strong types catch bugs agents might miss

### For Future Agent Teams
- Start with skeleton structure (Phase 0)
- Build incrementally, test constantly
- Document decisions immediately
- Use version control as knowledge store
- Leverage each agent's strengths
- Make reviews a regular practice

---

## Contributing Your Experience

If you're building software with AI agents, we'd love to hear:
- What patterns work for your team?
- How do you coordinate multiple agents?
- What documentation practices help most?

This is an evolving practice. We're learning as we build! 🚀

---

**Generated by:** Claude
**Purpose:** Educational documentation on agent-human software development
**Audience:** Developers exploring agentic coding practices
