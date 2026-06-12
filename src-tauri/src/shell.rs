use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use tauri::{AppHandle, Window};

use crate::{platform, security};

/// Height of the top bar in physical pixels. Keep in sync with the
/// `--bar-height` / menu-bar height in src/index.css and tauri.conf.json.
pub const BAR_HEIGHT: i32 = 40;

pub fn configure_main_window(
    window: tauri::WebviewWindow,
    reserve_screen_space: bool,
    shutdown: Arc<AtomicBool>,
) -> Result<(), String> {
    // Wayland has no primary-monitor concept, so fall back to the monitor the
    // window landed on, then to the first known one, before giving up.
    let monitor = window
        .primary_monitor()
        .ok()
        .flatten()
        .or_else(|| window.current_monitor().ok().flatten())
        .or_else(|| {
            window
                .available_monitors()
                .ok()
                .and_then(|monitors| monitors.into_iter().next())
        });
    let width = monitor.map(|m| m.size().width).unwrap_or(1280);
    window
        .set_size(tauri::Size::Physical(tauri::PhysicalSize {
            width,
            height: BAR_HEIGHT as u32,
        }))
        .map_err(|e| e.to_string())?;
    let _ = window.set_shadow(false);
    let _ = window.set_skip_taskbar(true);
    platform::install_bar(window, width, reserve_screen_space, shutdown)
}

pub fn set_native_taskbar_visible(visible: bool) {
    platform::set_native_taskbar_visible(visible);
}

pub fn restore(handle: &AppHandle) -> Result<(), String> {
    platform::restore_bar(handle)
}

#[tauri::command]
pub fn restore_shell_state(app: AppHandle, window: Window) -> Result<(), String> {
    security::require_window(&window, &["settings"])?;
    restore(&app)
}
