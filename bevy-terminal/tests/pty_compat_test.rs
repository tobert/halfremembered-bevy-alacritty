use bevy_terminal::pty::PtyResource;
use std::io::Write;
use std::thread;
use std::time::{Duration, Instant};

#[test]
fn test_pty_integration_echo() {
    // 1. Create PTY
    let pty = PtyResource::new().expect("Failed to create PTY");

    // 2. Write command that works on both Windows (cmd.exe) and Linux (bash/sh)
    // 'echo' is built-in for both.
    let command = "echo hello_world\n";
    {
        let mut writer = pty.writer.lock().unwrap();
        writer.write_all(command.as_bytes()).expect("Write failed");
        writer.flush().expect("Flush failed");
    }

    // 3. Read output via channel (integration test for the background thread)
    let start = Instant::now();
    let timeout = Duration::from_secs(5);
    let mut output_acc = String::new();

    loop {
        if start.elapsed() > timeout {
            panic!("Timeout waiting for 'hello_world'. Got:\n{}", output_acc);
        }

        let rx = pty.rx.lock().unwrap();
        match rx.try_recv() {
            Ok(bytes) => {
                let chunk = String::from_utf8_lossy(&bytes);
                output_acc.push_str(&chunk);
                
                if output_acc.contains("hello_world") {
                    break; // Success
                }
            }
            Err(_) => {
                // Empty, wait a bit
                drop(rx);
                thread::sleep(Duration::from_millis(50));
            }
        }
    }
}

#[test]
fn test_pty_integration_exit() {
    let mut pty = PtyResource::new().expect("Failed to create PTY");

    {
        let mut writer = pty.writer.lock().unwrap();
        writer.write_all(b"exit\n").expect("Write failed");
        writer.flush().expect("Flush failed");
    }

    let start = Instant::now();
    let timeout = Duration::from_secs(5);

    loop {
        if start.elapsed() > timeout {
            panic!("Timeout waiting for child process to exit");
        }

        match pty.child.try_wait() {
            Ok(Some(status)) => {
                assert!(status.success(), "Child should exit with success");
                break;
            }
            Ok(None) => {
                thread::sleep(Duration::from_millis(50));
            }
            Err(e) => panic!("Wait error: {}", e),
        }
    }
}
