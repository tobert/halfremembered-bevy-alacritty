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
use portable_pty::{native_pty_system, Child, CommandBuilder, MasterPty, PtySize};
use std::io::{Read, Write};
use std::sync::{mpsc::{channel, Receiver}, Arc, Mutex};
use std::thread;
use log::{info, error};

use crate::terminal::TerminalState;

/// Resource holding PTY handles for the terminal.
///
/// The PTY runs persistently from Startup until app shutdown.
/// Reader is handled in a background thread to avoid blocking the main loop.
#[derive(Resource)]
pub struct PtyResource {
    /// Channel receiver for PTY output (filled by background thread)
    pub rx: Arc<Mutex<Receiver<Vec<u8>>>>,
    /// Writer for sending input to the PTY
    pub writer: Arc<Mutex<Box<dyn Write + Send>>>,
    /// The child process (shell)
    pub child: Box<dyn Child + Send + Sync>,
    /// Master PTY handle - kept alive for Windows ConPTY compatibility
    /// On Windows, ConPTY requires the master handle to persist for the session.
    /// Wrapped in Arc<Mutex<>> for thread safety (Bevy requires Sync).
    #[allow(dead_code)]
    _master: Arc<Mutex<Box<dyn MasterPty + Send>>>,
}

/// Spawns a persistent PTY running the default shell.
///
/// System: Startup
/// Runs: Once at application start
///
/// Configuration (MVP hardcoded):
/// - Size: 120 cols × 30 rows
/// - Shell: bash (Linux) / powershell (Windows)
/// - Background thread handles reading
pub fn spawn_pty(mut commands: Commands) {
    match PtyResource::new() {
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

impl PtyResource {
    pub fn new() -> Result<Self> {
        info!("🔧 Initializing PTY system...");
        let pty_system = native_pty_system();

        let pty_size = PtySize {
            rows: 30,
            cols: 120,
            pixel_width: 0,
            pixel_height: 0,
        };

        info!("🔧 Opening PTY pair ({}×{} cells)...", pty_size.cols, pty_size.rows);
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
        let shell_cmd = {
            // Prefer PowerShell over cmd.exe for better ConPTY compatibility
            if let Ok(comspec) = std::env::var("COMSPEC") {
                if comspec.to_lowercase().contains("powershell") {
                    comspec
                } else {
                    // COMSPEC is cmd.exe, but we prefer PowerShell
                    "powershell.exe".to_string()
                }
            } else {
                "powershell.exe".to_string()
            }
        };

        info!("🐚 Spawning shell: {}", shell_cmd);
        let mut cmd = CommandBuilder::new(&shell_cmd);

        // Windows shells need explicit flags to stay alive in interactive mode
        #[cfg(windows)]
        {
            if shell_cmd.to_lowercase().contains("powershell") {
                // PowerShell: -NoExit keeps the shell alive, -NoLogo reduces startup noise
                cmd.arg("-NoExit");
                cmd.arg("-NoLogo");
            } else {
                // cmd.exe: /K keeps the shell alive after executing startup commands
                cmd.arg("/K");
            }
        }

        cmd.env("TERM", "xterm-256color");

        // Explicitly set CWD to avoid issues with weird startup paths
        if let Ok(cwd) = std::env::current_dir() {
            info!("📂 Using CWD: {}", cwd.display());
            cmd.cwd(cwd);
        } else {
            // Fallback env vars
            #[cfg(windows)]
            if let Ok(user_profile) = std::env::var("USERPROFILE") {
                 cmd.cwd(user_profile);
            }
            #[cfg(unix)]
            if let Ok(home) = std::env::var("HOME") {
                cmd.cwd(home);
            }
        }
        
        info!("🔧 Spawning child process...");
        let child = pair
            .slave
            .spawn_command(cmd)
            .context("Failed to spawn shell process")?;

        // Setup I/O
        info!("🔧 Setting up I/O threads...");
        
        // Clone reader for background thread
        let mut reader = pair
            .master
            .try_clone_reader()
            .context("Failed to clone PTY reader")?;

        // Take writer for main thread
        let writer = pair
            .master
            .take_writer()
            .context("Failed to take PTY writer")?;

        // Channel for sending data from thread to main loop
        let (tx, rx) = channel();

        // Spawn background reader thread
        // This avoids blocking the main game loop, critical for Windows ConPTY
        thread::spawn(move || {
            let mut buf = [0u8; 4096];
            loop {
                match reader.read(&mut buf) {
                    Ok(0) => {
                        // EOF
                        eprintln!("🔚 PTY reader: EOF received");
                        break;
                    }
                    Ok(n) => {
                        // Debug: log what we read
                        eprintln!("📖 PTY reader: Read {} bytes", n);
                        // Send data to main thread
                        if tx.send(buf[..n].to_vec()).is_err() {
                            // Receiver dropped, app probably closing
                            eprintln!("❌ PTY reader: Channel send failed");
                            break;
                        }
                    }
                    Err(e) => {
                        // Read error
                        eprintln!("❌ PTY reader: Read error: {}", e);
                        break;
                    }
                }
            }
            eprintln!("🛑 PTY reader thread exiting");
        });

        info!("✅ PTY initialized successfully: {}", shell_cmd);

        // Keep master alive for Windows ConPTY compatibility
        // On Windows, ConPTY requires the master handle to persist for the session,
        // even after we've cloned the reader and taken the writer.
        Ok(PtyResource {
            rx: Arc::new(Mutex::new(rx)),
            writer: Arc::new(Mutex::new(Box::new(writer))),
            child,
            _master: Arc::new(Mutex::new(pair.master)),
        })
    }
}

/// Polls the PTY channel for output and feeds bytes to the terminal.
///
/// System: Update
/// Runs: Every frame
///
/// Drains the channel of any data read by the background thread.
/// This is non-blocking and safe for the main loop.
pub fn poll_pty(pty: Res<PtyResource>, mut term_state: ResMut<TerminalState>) {
    if let Ok(rx) = pty.rx.try_lock() {
        // Read all available chunks
        while let Ok(bytes) = rx.try_recv() {
            term_state.process_bytes(&bytes);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pty_spawns() {
        let result = PtyResource::new();
        assert!(result.is_ok(), "PTY should spawn successfully");

        let pty = result.unwrap();

        // Verify we can lock reader/writer
        assert!(pty.rx.lock().is_ok(), "Should be able to lock rx");
        assert!(pty.writer.lock().is_ok(), "Should be able to lock writer");

        // Child process should be running
        // Note: We don't check child status here as it might complete quickly
    }

    #[test]
    fn test_pty_io() {
        let pty = PtyResource::new().expect("PTY spawn failed");

        // Write a command
        {
            let mut writer = pty.writer.lock().unwrap();
            writer.write_all(b"echo test\n").expect("Write failed");
            writer.flush().expect("Flush failed");
        }

        // Poll for output with timeout
        let timeout = std::time::Duration::from_secs(2);
        let start = std::time::Instant::now();

        loop {
            if start.elapsed() > timeout {
                panic!("Timeout waiting for PTY output after {}ms", timeout.as_millis());
            }

            let rx = pty.rx.lock().unwrap();
            if let Ok(bytes) = rx.try_recv() {
                let output = String::from_utf8_lossy(&bytes);

                // Look for our echoed command or output
                if output.contains("test") {
                    // Success! We got output from the shell
                    return;
                }
            }
            drop(rx); // Unlock

            // Brief sleep to avoid busy-waiting
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
    }

    #[test]
    fn test_pty_child_exit() {
        let mut pty = PtyResource::new().expect("PTY spawn failed");

        // Write exit command
        {
            let mut writer = pty.writer.lock().unwrap();
            writer.write_all(b"exit\n").expect("Write failed");
            writer.flush().expect("Flush failed");
        }

        // Poll for child exit with timeout
        let timeout = std::time::Duration::from_secs(2);
        let start = std::time::Instant::now();

        let final_status = loop {
            if start.elapsed() > timeout {
                panic!("Timeout waiting for child process to exit");
            }

            match pty.child.try_wait() {
                Ok(Some(status)) => {
                    break status;
                }
                Ok(None) => {
                    // Process not exited yet, continue polling
                }
                Err(e) => {
                    panic!("Error waiting for child process: {}", e);
                }
            }

            std::thread::sleep(std::time::Duration::from_millis(10));
        };

        assert!(final_status.success(), "Child process should have exited successfully");
    }
}
