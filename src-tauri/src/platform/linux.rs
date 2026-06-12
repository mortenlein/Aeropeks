//! Linux implementations: wpctl (PipeWire) audio, sysfs battery, bluetoothctl,
//! Secret Service secrets via keyring, logind power actions, $SHELL terminal.
//!
//! Phase 0 of docs/linux-port.md: the bar is a plain always-on-top window
//! (no layer-shell exclusive zone yet), local media and app search are
//! stubbed, and privacy mode is mic-only.

use std::path::Path;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use portable_pty::CommandBuilder;
use tauri::AppHandle;

use crate::launcher::SearchResult;
use crate::media::MediaInfo;
use crate::system_status::{BatteryStatus, BluetoothStatus};

// ── Bar placement ───────────────────────────────────────────────────

pub fn install_bar(
    window: tauri::WebviewWindow,
    _width: u32,
    reserve_screen_space: bool,
    _shutdown: Arc<AtomicBool>,
) -> Result<(), String> {
    // Anchor the bar to the top edge on the `Top` layer via wlr-layer-shell.
    // The compositor owns placement and (optionally) reserves the screen
    // space — no maintenance loop, no work-area bookkeeping, and fullscreen
    // surfaces still stack above the bar. Must run before the GTK window is
    // realized, which holds here because the window starts `visible: false`.
    // Without layer-shell support (X11 or an exotic compositor) the bar is a
    // plain always-on-top strip, as in Phase 0.
    if gtk_layer_shell::is_supported() {
        use gtk_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};
        let gtk_win = window.gtk_window().map_err(|e| e.to_string())?;
        gtk_win.init_layer_shell();
        gtk_win.set_namespace("aeropeks-bar");
        gtk_win.set_layer(Layer::Top);
        gtk_win.set_anchor(Edge::Top, true);
        gtk_win.set_anchor(Edge::Left, true);
        gtk_win.set_anchor(Edge::Right, true);
        // Like WS_EX_NOACTIVATE on Windows: take the keyboard only when the
        // user interacts with the bar instead of stealing startup focus.
        gtk_win.set_keyboard_mode(KeyboardMode::OnDemand);
        if reserve_screen_space {
            gtk_win.set_exclusive_zone(crate::shell::BAR_HEIGHT);
        }
    }
    window.show().map_err(|e| e.to_string())
}

pub fn set_native_taskbar_visible(visible: bool) {
    // On COSMIC, "hide the native taskbar" means auto-hiding the DE's
    // top-anchored panel(s) so the bar owns the top edge. cosmic-panel
    // hot-reloads its plain-file config, so this takes effect immediately.
    // Other desktops: panels are owned by the DE; nothing to hide (yet).
    if !is_cosmic() {
        return;
    }
    let result = if visible {
        cosmic_panel::restore_all()
    } else {
        cosmic_panel::hide_top_panels()
    };
    if let Err(error) = result {
        log::warn!("cosmic panel visibility change failed: {error}");
    }
}

fn is_cosmic() -> bool {
    std::env::var("XDG_CURRENT_DESKTOP")
        .map(|desktop| desktop.to_ascii_uppercase().contains("COSMIC"))
        .unwrap_or(false)
}

/// Hide/restore COSMIC's own panels by flipping their `autohide` and
/// `exclusive_zone` config entries. Originals are backed up under
/// `~/.aeropeks/cosmic-panel-backup/` before the first change and restored
/// on demand — also across crashes, since `restore_all` replays whatever
/// backup is on disk the next time the app starts with the setting off.
mod cosmic_panel {
    use std::fs;
    use std::path::{Path, PathBuf};

    /// Matches what COSMIC Settings writes when "Automatically hide panel"
    /// is enabled.
    const AUTOHIDE: &str = "Some((wait_time: 1000, transition_time: 200, handle_size: 4))";
    const FIELDS: [&str; 2] = ["autohide", "exclusive_zone"];
    const HIDDEN_VALUES: [&str; 2] = [AUTOHIDE, "false"];

    fn config_root() -> Result<PathBuf, String> {
        std::env::var_os("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")))
            .map(|base| base.join("cosmic"))
            .ok_or_else(|| "no home directory".to_string())
    }

    fn backup_root() -> Result<PathBuf, String> {
        std::env::var_os("HOME")
            .map(|home| {
                PathBuf::from(home)
                    .join(".aeropeks")
                    .join("cosmic-panel-backup")
            })
            .ok_or_else(|| "no home directory".to_string())
    }

    /// Panel entry names from `com.system76.CosmicPanel/v1/entries`, e.g.
    /// `["Panel", "Dock"]`. The file is a RON string list; pulling the
    /// quoted strings out is enough.
    fn panel_entries(config: &Path) -> Vec<String> {
        let Ok(content) = fs::read_to_string(
            config
                .join("com.system76.CosmicPanel")
                .join("v1")
                .join("entries"),
        ) else {
            return Vec::new();
        };
        content
            .split('"')
            .skip(1)
            .step_by(2)
            .map(str::to_string)
            .collect()
    }

    fn panel_config_dir(config: &Path, name: &str) -> PathBuf {
        config
            .join(format!("com.system76.CosmicPanel.{name}"))
            .join("v1")
    }

    pub fn hide_top_panels() -> Result<(), String> {
        let config = config_root()?;
        let backups = backup_root()?;
        for name in panel_entries(&config) {
            let panel_dir = panel_config_dir(&config, &name);
            let anchor = fs::read_to_string(panel_dir.join("anchor")).unwrap_or_default();
            if anchor.trim() != "Top" {
                continue;
            }
            let backup_dir = backups.join(&name);
            fs::create_dir_all(&backup_dir).map_err(|e| e.to_string())?;
            for (field, hidden_value) in FIELDS.iter().zip(HIDDEN_VALUES) {
                let config_file = panel_dir.join(field);
                let backup_file = backup_dir.join(field);
                // Keep the oldest backup: repeated hides must not capture
                // our own values as "originals".
                if !backup_file.exists() {
                    let original = fs::read_to_string(&config_file)
                        .map_err(|e| format!("read {field} for {name}: {e}"))?;
                    fs::write(&backup_file, original).map_err(|e| e.to_string())?;
                }
                fs::write(&config_file, hidden_value).map_err(|e| e.to_string())?;
            }
        }
        Ok(())
    }

    pub fn restore_all() -> Result<(), String> {
        let config = config_root()?;
        let backups = backup_root()?;
        let Ok(entries) = fs::read_dir(&backups) else {
            return Ok(()); // nothing was hidden
        };
        for entry in entries.flatten() {
            let name = entry.file_name();
            let Some(name) = name.to_str() else { continue };
            let panel_dir = panel_config_dir(&config, name);
            for field in FIELDS {
                let backup_file = entry.path().join(field);
                if let Ok(original) = fs::read_to_string(&backup_file) {
                    fs::write(panel_dir.join(field), original).map_err(|e| e.to_string())?;
                }
            }
            let _ = fs::remove_dir_all(entry.path());
        }
        let _ = fs::remove_dir(&backups);
        Ok(())
    }
}

pub fn restore_bar(handle: &AppHandle) -> Result<(), String> {
    // Bring the DE's own panels back (mirrors the Windows quit path) and
    // give back the reserved screen space. There is no other registration
    // to undo: the layer surface and its zone die with the window.
    use tauri::Manager;
    set_native_taskbar_visible(true);
    if let Some(window) = handle.get_webview_window("main") {
        if gtk_layer_shell::is_supported() {
            use gtk_layer_shell::LayerShell;
            let gtk_win = window.gtk_window().map_err(|e| e.to_string())?;
            if gtk_win.is_layer_window() {
                gtk_win.set_exclusive_zone(0);
            }
        }
    }
    Ok(())
}

// ── Audio (wpctl / WirePlumber) ─────────────────────────────────────

fn wpctl(args: &[&str]) -> Result<String, String> {
    let output = std::process::Command::new("wpctl")
        .args(args)
        .output()
        .map_err(|e| format!("wpctl failed (is WirePlumber installed?): {e}"))?;
    if !output.status.success() {
        return Err(format!("wpctl exited with {}", output.status));
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

// wpctl get-volume prints e.g. "Volume: 0.45" or "Volume: 0.45 [MUTED]".
fn parse_volume(output: &str) -> Result<f32, String> {
    output
        .split_whitespace()
        .nth(1)
        .and_then(|value| value.parse::<f32>().ok())
        .ok_or_else(|| format!("unexpected wpctl output: {output}"))
}

pub fn get_volume() -> Result<f32, String> {
    parse_volume(&wpctl(&["get-volume", "@DEFAULT_AUDIO_SINK@"])?)
}

pub fn set_volume(volume: f32) -> Result<(), String> {
    wpctl(&[
        "set-volume",
        "@DEFAULT_AUDIO_SINK@",
        &format!("{volume:.2}"),
    ])
    .map(|_| ())
}

pub fn mic_muted() -> Result<bool, String> {
    Ok(wpctl(&["get-volume", "@DEFAULT_AUDIO_SOURCE@"])?.contains("[MUTED]"))
}

pub fn set_mic_muted(muted: bool) -> Result<(), String> {
    wpctl(&[
        "set-mute",
        "@DEFAULT_AUDIO_SOURCE@",
        if muted { "1" } else { "0" },
    ])
    .map(|_| ())
}

// ── Power / devices ─────────────────────────────────────────────────

pub fn battery_status() -> Result<BatteryStatus, String> {
    let supplies = std::fs::read_dir("/sys/class/power_supply")
        .map_err(|e| e.to_string())?
        .flatten();
    for entry in supplies {
        let path = entry.path();
        let Ok(capacity) = std::fs::read_to_string(path.join("capacity")) else {
            continue; // AC adapters have no capacity file.
        };
        let Ok(percentage) = capacity.trim().parse::<u8>() else {
            continue;
        };
        let status = std::fs::read_to_string(path.join("status")).unwrap_or_default();
        return Ok(BatteryStatus {
            percentage,
            is_charging: status.trim() == "Charging",
            has_battery: true,
        });
    }
    Ok(BatteryStatus {
        percentage: 100,
        is_charging: false,
        has_battery: false,
    })
}

pub fn set_cameras_disabled(_disabled: bool) -> Result<(), String> {
    // No unprivileged camera kill switch on Linux; privacy mode is mic-only.
    Ok(())
}

pub fn bluetooth_status() -> Result<BluetoothStatus, String> {
    // Lines look like "Device AA:BB:CC:DD:EE:FF Friendly Name".
    let Ok(output) = std::process::Command::new("bluetoothctl")
        .args(["devices", "Connected"])
        .output()
    else {
        return Ok(BluetoothStatus::default());
    };
    if !output.status.success() {
        return Ok(BluetoothStatus::default());
    }
    let devices = String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter_map(|line| {
            line.strip_prefix("Device ")?
                .split_once(' ')
                .map(|(_, name)| name.trim().to_string())
        })
        .filter(|name| !name.is_empty())
        .collect::<Vec<_>>();
    Ok(BluetoothStatus {
        connected: !devices.is_empty(),
        devices,
    })
}

pub fn run_power_action(action: &str) -> Result<(), String> {
    let (program, args): (&str, &[&str]) = match action {
        "shutdown" => ("systemctl", &["poweroff"]),
        "restart" => ("systemctl", &["reboot"]),
        "sleep" => ("systemctl", &["suspend"]),
        "lock" => ("loginctl", &["lock-session"]),
        _ => return Err("invalid power action".to_string()),
    };
    std::process::Command::new(program)
        .args(args)
        .spawn()
        .map_err(|e| e.to_string())?;
    Ok(())
}

// ── Local media ─────────────────────────────────────────────────────

pub async fn local_media() -> Result<Option<MediaInfo>, String> {
    // Phase 2: MPRIS over D-Bus. Until then only Plex media is shown.
    Ok(None)
}

pub async fn local_media_action(_action: &str) -> Result<(), String> {
    Ok(())
}

pub fn watch_local_media(_handle: AppHandle) {
    // Phase 2: subscribe to MPRIS PropertiesChanged/NameOwnerChanged signals.
    // The 30 s fallback poll in main.rs covers Plex in the meantime.
}

// ── Secrets (Secret Service via keyring) ────────────────────────────

fn entry(target: &str) -> Result<keyring::Entry, String> {
    keyring::Entry::new("Aeropeks", target).map_err(|e| e.to_string())
}

pub fn read_secret(target: &str) -> Result<Option<String>, String> {
    match entry(target)?.get_password() {
        Ok(secret) => Ok(Some(secret)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(error) => Err(error.to_string()),
    }
}

pub fn write_secret(target: &str, secret: &str) -> Result<(), String> {
    entry(target)?
        .set_password(secret)
        .map_err(|e| e.to_string())
}

pub fn restore_secret(target: &str, secret: Option<&str>) -> Result<(), String> {
    match secret {
        Some(secret) => write_secret(target, secret),
        None => match entry(target)?.delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(error) => Err(error.to_string()),
        },
    }
}

// ── App launcher ────────────────────────────────────────────────────

pub fn installed_app_results(_normalized_query: &str) -> Vec<SearchResult> {
    // Phase 3: scan XDG .desktop entries. Web and system results still work.
    Vec::new()
}

pub fn validate_app_target(_path: &Path) -> Result<(), String> {
    Err("app launching is not implemented on Linux yet".to_string())
}

// ── Terminal shell ──────────────────────────────────────────────────

pub fn shell_command(command: Option<String>) -> CommandBuilder {
    let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/bash".to_string());
    let mut cmd = CommandBuilder::new(shell);
    if let Some(command) = command.filter(|value| !value.trim().is_empty()) {
        cmd.arg("-c");
        cmd.arg(command);
    }
    cmd
}
