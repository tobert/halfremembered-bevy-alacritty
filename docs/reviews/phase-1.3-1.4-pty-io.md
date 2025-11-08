Thank you for the detailed context. I've reviewed the code for Phase 1.3 (PTY Polling) and 1.4 (Keyboard Input). Here is my feedback based on your eight points.

### 1. System Scheduling & Polling Approach

The approach of using Bevy `Update` systems to poll the PTY and keyboard every frame is sound. It's idiomatic for Bevy and, combined with non-blocking I/O (`try_lock`, non-blocking reads), it integrates cleanly into the game loop without risking frame drops. For a UI-driven application like a terminal, this is a simple and effective architecture.

### 2. Borrow Checker Solutions

The use of `try_lock` is correct and essential for this polling-based design.

In `poll_pty`, the `Arc::clone` pattern is a valid way to satisfy the borrow checker when you need to access two fields on a `ResMut` where one access depends on the other (here, `processor.advance` needs a lock on `term`).

For improved encapsulation and ergonomics, you could move this logic into a method on `TerminalState`:

```rust
// In a new `impl TerminalState` block
pub fn process_bytes(&mut self, bytes: &[u8]) {
    let mut term_guard = self.term.lock();
    self.processor.advance(&mut *term_guard, bytes);
}

// The poll_pty system then becomes cleaner:
pub fn poll_pty(pty: Res<PtyResource>, mut term_state: ResMut<TerminalState>) {
    // ... (buffer setup and reader lock) ...
    match reader.read(&mut buf) {
        Ok(n) if n > 0 => {
            term_state.process_bytes(&buf[..n]); // Cleaner call
            trace!("📥 Read {} bytes from PTY", n);
        }
        // ... (rest of match) ...
    }
}
```

This refactoring hides the internal locking mechanism from the system, making the system's intent clearer.

### 3. Buffer Sizing

A 4KB buffer is a standard and sensible choice. It aligns with common memory page sizes and is a typical chunk size for terminal I/O. This is not a performance concern.

### 4. Error Handling

The error handling is robust.
- **`try_lock` failures** are handled gracefully by skipping the frame.
- **`WouldBlock` errors** on read are correctly ignored.
- **Other I/O errors** are logged, which is appropriate.

One minor suggestion: you could explicitly handle the `Ok(0)` case (EOF) from `reader.read()` with an `info!` log to make it clear when the PTY child process has closed.

### 5. Keyboard Mapping Completeness for MVP

This is the area needing the most significant improvement. While the current mapping is a good start, the lack of modifier key support (especially **Shift** and **Ctrl**) severely limits the terminal's usability, even for an MVP.

- **Shift:** Without it, you cannot produce uppercase characters or common symbols (`!`, `@`, `#`, `*`, `(`, `)`), which are required for many basic shell commands.
- **Ctrl:** The absence of `Ctrl+C` (interrupt), `Ctrl+D` (end of input), and `Ctrl+L` (clear screen) prevents fundamental shell interactions.

I recommend refactoring `handle_keyboard_input` and `keycode_to_bytes` to handle modifiers.

**Suggested Refactor:**

```rust
// in bevy-terminal/src/input.rs

pub fn handle_keyboard_input(
    mut key_evr: EventReader<KeyboardInput>,
    keyboard: Res<ButtonInput<KeyCode>>,
    pty: Res<PtyResource>,
) {
    let shift = keyboard.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]);
    let ctrl = keyboard.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight]);

    for ev in key_evr.read() {
        if ev.state.is_pressed() {
            if let Some(bytes) = keycode_to_bytes(ev.key_code, shift, ctrl) {
                // ... (write bytes to PTY as before) ...
            }
        }
    }
}

fn keycode_to_bytes(key: KeyCode, shift: bool, ctrl: bool) -> Option<Vec<u8>> {
    // Handle Ctrl sequences first
    if ctrl {
        return match key {
            // Ctrl+C -> ETX (End of Text)
            KeyCode::KeyC => Some(vec![0x03]),
            // Ctrl+D -> EOT (End of Transmission)
            KeyCode::KeyD => Some(vec![0x04]),
            // ... other ctrl mappings
            _ => None,
        };
    }

    // Handle letters
    if let Some(c) = key.to_letter() {
        let letter = if shift {
            c.to_ascii_uppercase()
        } else {
            c
        };
        return Some(letter.to_string().into_bytes());
    }
    
    // Handle other keys (numbers, symbols, arrows, etc.)
    // This part will need to be expanded to check `shift` for symbols like '!' vs '1'
    match key {
        KeyCode::Digit1 => Some(if shift { b"!" } else { b"1" }.to_vec()),
        // ...
        KeyCode::Enter => Some(b"\r".to_vec()),
        // ... same as before
        _ => None,
    }
}
```
*Note: `KeyCode::to_letter()` is a conceptual helper. You would implement this logic based on the `KeyCode` enum values.*

### 6. Test Coverage

The existing unit tests for `keycode_to_bytes` are good. However, once you add modifier support, the test suite should be expanded significantly to cover cases like:
- `Shift + A` -> `"A"`
- `Shift + 1` -> `"!"`
- `Ctrl + C` -> `\x03`
- `Shift + Arrow Key` (if you support modified arrow sequences)

### 7. Performance Implications

The performance impact of the current per-frame polling approach is negligible. The operations within each system (`try_lock`, a single non-blocking `read`, and key checks) are extremely fast. The VTE parser from Alacritty is highly optimized. This design will not be a performance bottleneck.

### 8. Alternative Approaches

The main alternative is a fully async, event-driven model, likely using `bevy_tasks` to run an async runtime that `await`s PTY I/O and communicates with the main Bevy app via channels.

- **Pros:** More CPU-efficient (thread sleeps instead of polling).
- **Cons:** Significantly more complex to implement, integrate, and debug.

For this application, the current polling approach is a pragmatic and excellent choice. It's simple, easy to reason about, and performant enough not to warrant the complexity of a fully async model.

### Summary

The overall architecture is solid, clean, and well-suited for Bevy. The PTY polling implementation is robust. The primary area for improvement is in the keyboard input handling, which needs modifier support to be functionally useful as an MVP.
