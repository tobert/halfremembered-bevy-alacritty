# BOTS.md: LLM Agent Development Protocol

This document outlines the development protocol for LLM agents working on the HalfRemembered project. Adherence to these guidelines is critical for effective collaboration and high-quality output.

## 1. Core Principle: Jujutsu as a Persistent Context Store

The most important concept is that **Jujutsu (jj) changes are persistent context stores**.

Unlike git commits, a `jj` change has a stable ID that survives rebases and amendments. The description associated with this change is our shared, persistent memory. When you read a change description, you are reading the reasoning and intent of the previous agent. When you write one, you are passing critical context to the next.

**Your primary goal is to maintain the integrity and clarity of this context.**

## 2. Agent Workflow Checklist

Follow this sequence for every task.

### Step 1: Understand Context
Always begin by reviewing the history of changes you've authored. This is your memory.

```bash
# 1. See your last 5 changes
jj log -r 'mine()' -n 5

# 2. Review the full context of the most recent change
jj show @
```
- Read the change descriptions carefully. They contain the "why" behind the code.
- Use `jj obslog` to understand the evolution of a change if needed.

### Step 2: Start New Work
Create a new change with a clear, descriptive message.

```bash
jj new -m "<type>: <what you are building>"
```
- **Types**: `feat`, `fix`, `refactor`, `test`, `docs`, `chore`

### Step 3: Implement and Iterate
As you work, continuously update the change's description. This is how you "think" and record your process in the persistent context store.

```bash
# Review your work-in-progress
jj diff

# Update the description with your approach, findings, and status
jj describe -m "Why: <problem being solved>
Approach: <key decisions, algorithms, patterns>
Status: <complete | next: remaining work>"
```
- **Update descriptions frequently.** Don't wait until the end.
- Use `jj squash` to fold fixups and minor corrections into the main change, keeping the history clean and atomic.

### Step 4: Finalize and Sync
Once the work is complete, tested, and builds pass, push the change.

```bash
# Push only the current change to GitHub
jj git push -c @

# Create a pull request using the rich description
gh pr create --fill
```

## 3. Technical Guidelines

### Code Style & Quality
- **Correctness & Clarity First**: Prioritize readable, correct code over premature optimization.
- **Strong Types**: Use modern, idiomatic types.
- **No Shortcuts**: Avoid hacks or workarounds. Refactor messy code as you encounter it.
- **No Organizational Comments**: Do not write comments that summarize what code does. Code should be self-documenting.
- **"Why" Comments Only**: Comments should only explain the "why" behind a non-obvious implementation choice.
- **File Structure**: Add new functionality to existing files unless it represents a new, distinct logical component. Avoid `mod.rs`.
- **Naming**: Use full, descriptive words for variables. No abbreviations.

### Error Handling
- **`anyhow::Result`**: Use for all fallible operations.
- **No `unwrap()`**: Always propagate errors with `?`.
- **Add Context**: Use `.context()` to provide useful debugging information on errors.
- **No Silent Errors**: Never discard errors with `let _ =`.
- **Graceful Failures**: Handle potential network failures (e.g., reconnection logic) gracefully.

### Jujutsu (jj) Command Reference
| Command | Agent Usage |
| :--- | :--- |
| `jj new -m "..."` | Start a new atomic change. |
| `jj describe -m "..."` | Update the description (the persistent context). **Use frequently.** |
| `jj log -r 'mine()' -n 5` | Review your recent work history. |
| `jj show <id>` | Read the full diff and description of a change. |
| `jj diff` | Review current working copy changes. |
| `jj squash` | Fold the current change into its parent. |
| `jj split` | Separate a change into smaller, more logical units. |
| `jj abandon` | Discard the current change entirely. |
| `jj git push -c @` | Push the current change to the remote. |

### Commit Description Format
The `Why` section should succinctly explain the problem being solved. **It is highly recommended to include the original user prompt here for full context.**
```
<type>: <summary>

Why: <user prompt or problem being solved>
Approach: <key decisions, algorithms, patterns>
Status: <complete | next: remaining work>

Co-authored-by: <Your Name> <your@email>
```

**Agent Attribution:**
- **Claude**: `Co-authored-by: Claude <claude@anthropic.com>`
- **Gemini**: `Co-authored-by: Gemini <gemini@google.com>`
- **Kimi**: `Co-authored-by: Kimi <kimi@moonshot.ai>`

