//! PTY lifecycle management and polling.
//!
//! PTYs are spawned in Startup system and run persistently.
//! Polling system runs in Update to read PTY output and feed to terminal.
//! Uses portable-pty for cross-platform PTY spawning.
//!
//! ## Current Architecture: Arc<Mutex<>> Shared State
//!
//! Reader/writer are wrapped in Arc<Mutex<>> for thread-safe access from Bevy systems.
//! This is a straightforward approach suitable for the MVP.
//!
//! ## Future Alternative: Channel-Based I/O Thread
//!
//! For improved decoupling, consider migrating to:
//! - Dedicated I/O thread owning PTY reader (blocking reads)
//! - mpsc::channel to send data to main Bevy loop
//! - Bevy system receives data via mpsc::Receiver
//!
//! Benefits: No mutex contention, clearer separation of concerns.
//! See: docs/reviews/phase-1.1-pty-spawning.md (Gemini's recommendation)

use anyhow::{Context, Result};
use bevy::prelude::*;
use portable_pty::{native_pty_system, Child, CommandBuilder, PtySize};
use std::io::{ErrorKind, Read, Write};
use std::sync::{Arc, Mutex};

use crate::terminal::TerminalState;

/// Resource holding PTY handles for the terminal.
///
/// The PTY runs persistently from Startup until app shutdown.
/// Reader/writer are thread-safe to allow background polling.
#[derive(Resource)]
pub struct PtyResource {
    pub reader: Arc<Mutex<Box<dyn Read + Send>>>,
    pub writer: Arc<Mutex<Box<dyn Write + Send>>>,
    pub child: Box<dyn Child + Send + Sync>,
}

/// Spawns a persistent PTY running the default shell.
///
/// System: Startup
/// Runs: Once at application start
///
/// Configuration (MVP hardcoded):
/// - Size: 120 cols × 30 rows
/// - Shell: bash (or platform default)
/// - Non-blocking IO enabled
pub fn spawn_pty(mut commands: Commands) {
    match spawn_pty_internal() {
        Ok(pty_resource) => {
            info!("✅ PTY spawned successfully");
            commands.insert_resource(pty_resource);
        }
        Err(error) => {
            error!("❌ Failed to spawn PTY: {:#}", error);
            panic!("Cannot continue without PTY");
        }
    }
}

fn spawn_pty_internal() -> Result<PtyResource> {
    let pty_system = native_pty_system();

    let pty_size = PtySize {
        rows: 30,
        cols: 120,
        pixel_width: 0,
        pixel_height: 0,
    };

    let pair = pty_system
        .openpty(pty_size)
        .context("Failed to create PTY pair")?;

    // Spawn default shell with robust fallback chain
    #[cfg(unix)]
    let shell_cmd = std::env::var("SHELL").unwrap_or_else(|_| {
        // Fallback chain: bash → zsh → sh
        if std::path::Path::new("/bin/bash").exists() {
            "/bin/bash".to_string()
        } else if std::path::Path::new("/bin/zsh").exists() {
            "/bin/zsh".to_string()
        } else {
            "/bin/sh".to_string()
        }
    });

    #[cfg(windows)]
    let shell_cmd = std::env::var("COMSPEC").unwrap_or_else(|_| "cmd.exe".to_string());

    let mut cmd = CommandBuilder::new(&shell_cmd);
    cmd.env("TERM", "xterm-256color");

    let child = pair
        .slave
        .spawn_command(cmd)
        .context("Failed to spawn shell process")?;

    // Clone reader and take writer
    let reader = pair
        .master
        .try_clone_reader()
        .context("Failed to clone PTY reader")?;

    let writer = pair
        .master
        .take_writer()
        .context("Failed to take PTY writer")?;

    info!("🐚 Spawned shell: {}", shell_cmd);

    Ok(PtyResource {
        reader: Arc::new(Mutex::new(Box::new(reader))),
        writer: Arc::new(Mutex::new(Box::new(writer))),
        child,
    })
}

/// Polls the PTY for output and feeds bytes to the terminal.
///
/// System: Update
/// Runs: Every frame
///
/// Reads up to 4KB of data per frame from the PTY (non-blocking).
/// Bytes are fed through the VTE processor and update the terminal grid.
pub fn poll_pty(pty: Res<PtyResource>, mut term_state: ResMut<TerminalState>) {
    const BUF_SIZE: usize = 4096;
    let mut buf = [0u8; BUF_SIZE];

    // Try to lock reader (non-blocking)
    let mut reader = match pty.reader.try_lock() {
        Ok(guard) => guard,
        Err(_) => {
            // Reader locked elsewhere, skip this frame
            return;
        }
    };

    // Read from PTY (non-blocking)
    match reader.read(&mut buf) {
        Ok(0) => {
            // EOF: PTY child process has exited
            info!("📭 PTY EOF - child process exited");
        }
        Ok(n) => {
            // Feed bytes to terminal
            term_state.process_bytes(&buf[..n]);
            trace!("📥 Read {} bytes from PTY", n);
        }
        Err(error) if error.kind() == ErrorKind::WouldBlock => {
            // Expected for non-blocking reads
        }
        Err(error) => {
            error!("❌ PTY read error: {}", error);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pty_spawns() {
        let result = spawn_pty_internal();
        assert!(result.is_ok(), "PTY should spawn successfully");

        let pty = result.unwrap();

        // Verify we can lock reader/writer
        assert!(pty.reader.lock().is_ok(), "Should be able to lock reader");
        assert!(pty.writer.lock().is_ok(), "Should be able to lock writer");

        // Child process should be running
        // Note: We don't check child status here as it might complete quickly
    }

    #[test]
    fn test_pty_io() {
        let pty = spawn_pty_internal().expect("PTY spawn failed");

        // Write a command
        {
            let mut writer = pty.writer.lock().unwrap();
            writer.write_all(b"echo test\n").expect("Write failed");
            writer.flush().expect("Flush failed");
        }

        // Poll for output with timeout (more robust than sleep)
        let timeout = std::time::Duration::from_secs(2);
        let start = std::time::Instant::now();

        loop {
            if start.elapsed() > timeout {
                panic!("Timeout waiting for PTY output after {}ms", timeout.as_millis());
            }

            let mut reader = pty.reader.lock().unwrap();
            let mut buf = [0u8; 1024];

            match reader.read(&mut buf) {
                Ok(n) if n > 0 => {
                    let output = String::from_utf8_lossy(&buf[..n]);

                    // Look for our echoed command or output
                    if output.contains("test") {
                        // Success! We got output from the shell
                        return;
                    }
                }
                Ok(_) => {
                    // No data yet, continue polling
                }
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    // Expected for non-blocking reads, continue polling
                }
                Err(e) => {
                    panic!("PTY read error: {}", e);
                }
            }

            // Brief sleep to avoid busy-waiting
            drop(reader);
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
    }
}
