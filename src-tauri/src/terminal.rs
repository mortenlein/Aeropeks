use std::io::{Read, Write};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;

use base64::{engine::general_purpose, Engine as _};
use portable_pty::{ChildKiller, CommandBuilder, MasterPty, NativePtySystem, PtySize, PtySystem};
use serde::Serialize;
use tauri::{Emitter, Manager, Window};

use crate::security;

fn powershell_command() -> CommandBuilder {
    let shell = if std::process::Command::new("pwsh.exe")
        .arg("-NoLogo")
        .arg("-NoProfile")
        .arg("-Command")
        .arg("exit")
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
    {
        "pwsh.exe"
    } else {
        "powershell.exe"
    };
    CommandBuilder::new(shell)
}

#[derive(Serialize, Clone)]
struct PtyPayload {
    data: String,
}

pub struct TerminalState {
    writer: Arc<Mutex<Option<Box<dyn Write + Send>>>>,
    master: Arc<Mutex<Option<Box<dyn MasterPty + Send>>>>,
    killer: Arc<Mutex<Option<Box<dyn ChildKiller + Send + Sync>>>>,
    generation: Arc<AtomicU64>,
}

impl TerminalState {
    pub fn new() -> Self {
        Self {
            writer: Arc::new(Mutex::new(None)),
            master: Arc::new(Mutex::new(None)),
            killer: Arc::new(Mutex::new(None)),
            generation: Arc::new(AtomicU64::new(0)),
        }
    }

    fn stop(&self) -> Result<(), String> {
        self.generation.fetch_add(1, Ordering::SeqCst);
        if let Some(mut killer) = self.killer.lock().map_err(|e| e.to_string())?.take() {
            let _ = killer.kill();
        }
        *self.master.lock().map_err(|e| e.to_string())? = None;
        *self.writer.lock().map_err(|e| e.to_string())? = None;
        Ok(())
    }

    #[cfg(test)]
    fn generation(&self) -> u64 {
        self.generation.load(Ordering::SeqCst)
    }
}

#[tauri::command]
pub fn start_pty(
    rows: u16,
    cols: u16,
    command: Option<String>,
    state: tauri::State<'_, TerminalState>,
    window: Window,
) -> Result<(), String> {
    security::require_window(&window, &["main", "terminal-panel"])?;
    security::validate_pty_size(rows, cols)?;
    state.stop()?;
    let generation = state.generation.load(Ordering::SeqCst);

    let pair = NativePtySystem::default()
        .openpty(PtySize {
            rows,
            cols,
            pixel_width: 0,
            pixel_height: 0,
        })
        .map_err(|e| e.to_string())?;

    let mut cmd = powershell_command();
    cmd.arg("-NoLogo");
    if let Some(command) = command.filter(|value| !value.trim().is_empty()) {
        cmd.arg("-Command");
        cmd.arg(command);
    } else {
        cmd.arg("-NoExit");
        cmd.arg("-Command");
        cmd.arg(
            "if (Get-Command oh-my-posh -ErrorAction SilentlyContinue) { \
             oh-my-posh init pwsh | Invoke-Expression \
             }",
        );
    }

    let child = pair.slave.spawn_command(cmd).map_err(|e| e.to_string())?;
    *state.killer.lock().map_err(|e| e.to_string())? = Some(child.clone_killer());

    thread::spawn(move || {
        let mut child = child;
        let _ = child.wait();
    });

    let mut reader = pair.master.try_clone_reader().map_err(|e| e.to_string())?;
    let writer = pair.master.take_writer().map_err(|e| e.to_string())?;
    *state.writer.lock().map_err(|e| e.to_string())? = Some(writer);
    *state.master.lock().map_err(|e| e.to_string())? = Some(pair.master);

    let _ = window
        .app_handle()
        .emit_to("terminal-panel", "pty-ready-global", "OK");

    let app_handle = window.app_handle().clone();
    let active_generation = state.generation.clone();
    thread::spawn(move || {
        let mut buffer = [0u8; 4096];
        loop {
            if active_generation.load(Ordering::SeqCst) != generation {
                break;
            }
            match reader.read(&mut buffer) {
                Ok(0) => {
                    let _ = app_handle.emit_to("terminal-panel", "pty-exit", "EOF");
                    break;
                }
                Ok(n) => {
                    let payload = PtyPayload {
                        data: general_purpose::STANDARD.encode(&buffer[..n]),
                    };
                    let _ = app_handle.emit_to("terminal-panel", "pty-data", payload);
                }
                Err(error) => {
                    let _ =
                        app_handle.emit_to("terminal-panel", "pty-exit", format!("Error: {error}"));
                    break;
                }
            }
        }
    });

    Ok(())
}

#[tauri::command]
pub fn kill_pty(state: tauri::State<'_, TerminalState>, window: Window) -> Result<(), String> {
    security::require_window(&window, &["terminal-panel"])?;
    state.stop()
}

#[tauri::command]
pub fn write_pty(
    data: String,
    state: tauri::State<'_, TerminalState>,
    window: Window,
) -> Result<(), String> {
    security::require_window(&window, &["terminal-panel"])?;
    let mut writer = state.writer.lock().map_err(|e| e.to_string())?;
    if let Some(writer) = writer.as_mut() {
        writer
            .write_all(data.as_bytes())
            .and_then(|_| writer.flush())
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub fn resize_pty(
    rows: u16,
    cols: u16,
    state: tauri::State<'_, TerminalState>,
    window: Window,
) -> Result<(), String> {
    security::require_window(&window, &["terminal-panel"])?;
    security::validate_pty_size(rows, cols)?;
    if let Some(master) = state.master.lock().map_err(|e| e.to_string())?.as_ref() {
        master
            .resize(PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::TerminalState;

    #[test]
    fn stop_invalidates_existing_session_generation() {
        let state = TerminalState::new();
        let initial = state.generation();
        state.stop().unwrap();
        assert!(state.generation() > initial);
    }
}
