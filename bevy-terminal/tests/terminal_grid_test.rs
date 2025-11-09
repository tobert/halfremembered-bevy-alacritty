//! Cross-platform test for PTY → VTE → Terminal Grid data flow.
//!
//! This test verifies that:
//! 1. PTY receives output from shell
//! 2. VTE parser processes the bytes correctly
//! 3. Terminal grid contains the parsed characters
//!
//! If this test passes, we know the issue is in rendering, not data flow.

use bevy_terminal::pty::PtyResource;
use bevy_terminal::TerminalState;
use std::io::Write;
use std::thread;
use std::time::{Duration, Instant};

#[test]
fn test_terminal_grid_receives_shell_output() {
    println!("\n🧪 Testing PTY → Terminal Grid data flow...\n");

    // 1. Create PTY and Terminal
    let pty = PtyResource::new().expect("Failed to create PTY");
    let mut term_state = TerminalState::new();

    println!("✅ PTY and TerminalState created");

    // 2. Wait for initial shell output (banner + prompt)
    // PowerShell/bash should write something immediately
    let start = Instant::now();
    let timeout = Duration::from_secs(3);
    let mut total_bytes = 0;

    println!("⏳ Waiting for shell output...");

    while start.elapsed() < timeout {
        let rx = pty.rx.lock().unwrap();
        if let Ok(bytes) = rx.try_recv() {
            total_bytes += bytes.len();
            let display = String::from_utf8_lossy(&bytes);
            println!("📥 Received {} bytes (total: {})", bytes.len(), total_bytes);
            println!("   Display: {:?}", display);

            // Feed to terminal state
            term_state.process_bytes(&bytes);
        }
        drop(rx);

        // Give shell time to write prompt (don't stop at first data - get the actual prompt)
        thread::sleep(Duration::from_millis(100));

        // Stop once we've received multiple chunks (title + prompt)
        if total_bytes > 100 {
            break;
        }
    }

    assert!(
        total_bytes > 0,
        "Should have received some output from shell within {}s",
        timeout.as_secs()
    );

    println!("✅ Received {} total bytes from shell", total_bytes);

    // 3. Give VTE parser a moment to fully process
    thread::sleep(Duration::from_millis(200));

    // 4. Extract terminal grid content
    println!("\n📋 Terminal Grid Content:\n");
    let content_summary = term_state.get_content_summary();

    if content_summary.is_empty() {
        println!("❌ Terminal grid is EMPTY!");
        println!("\nFull grid dump:");
        println!("{}", term_state.get_visible_text());
        panic!(
            "Terminal grid should contain content after receiving {} bytes",
            total_bytes
        );
    }

    println!("✅ Terminal grid contains {} non-empty lines:", content_summary.len());
    for (line_num, content) in &content_summary {
        println!("  Line {}: {}", line_num, content);
    }

        // 5. Verify we can find expected content
        // let full_text = term_state.get_visible_text(); // Unused
        
        // On Windows, PowerShell typically shows "PS C:\..." prompt    // On Linux, bash/zsh shows some prompt
    // We just verify SOMETHING is there
    let has_content = content_summary.len() > 0;

    assert!(
        has_content,
        "Terminal grid should contain visible characters after shell initialization"
    );

    println!("\n✅ TEST PASSED: PTY → Terminal Grid data flow works!");
    println!("   If rendering is broken, the issue is in the renderer, not VTE/grid.");
}

#[test]
fn test_terminal_grid_processes_simple_echo() {
    println!("\n🧪 Testing echo command through terminal grid...\n");

    let pty = PtyResource::new().expect("Failed to create PTY");
    let mut term_state = TerminalState::new();

    // Clear initial output
    thread::sleep(Duration::from_millis(500));
    {
        let rx = pty.rx.lock().unwrap();
        while rx.try_recv().is_ok() {
            // Drain
        }
    }

    // Send a simple echo command
    let test_string = "TERMINAL_GRID_TEST_123";
    let command = format!("echo {}\n", test_string);

    {
        let mut writer = pty.writer.lock().unwrap();
        writer.write_all(command.as_bytes()).expect("Write failed");
        writer.flush().expect("Flush failed");
    }

    println!("📤 Sent command: {}", command.trim());

    // Collect output for a few seconds
    let start = Instant::now();
    let timeout = Duration::from_secs(3);
    let mut found_test_string = false;

    while start.elapsed() < timeout {
        let rx = pty.rx.lock().unwrap();
        if let Ok(bytes) = rx.try_recv() {
            println!("📥 Received {} bytes", bytes.len());
            term_state.process_bytes(&bytes);

            // Check if our test string appears in the grid
            let content = term_state.get_visible_text();
            if content.contains(test_string) {
                found_test_string = true;
                break;
            }
        }
        drop(rx);

        thread::sleep(Duration::from_millis(50));
    }

    println!("\n📋 Final terminal content:");
    let summary = term_state.get_content_summary();
    for (line_num, content) in &summary {
        println!("  Line {}: {}", line_num, content);
    }

    if !found_test_string {
        println!("\n❌ Test string '{}' NOT FOUND in grid!", test_string);
        println!("\nFull grid dump:");
        println!("{}", term_state.get_visible_text());
    }

    assert!(
        found_test_string,
        "Terminal grid should contain the echoed test string '{}'",
        test_string
    );

    println!("\n✅ TEST PASSED: Echo command processed correctly!");
}
