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
    _reserve_screen_space: bool,
    _shutdown: Arc<AtomicBool>,
) -> Result<(), String> {
    // Phase 1 will add a layer-shell exclusive zone here; for now the bar is
    // just an always-on-top strip (alwaysOnTop comes from tauri.conf.json).
    window.show().map_err(|e| e.to_string())
}

pub fn set_native_taskbar_visible(_visible: bool) {
    // Desktop panels are owned by the DE on Linux; nothing to hide.
}

pub fn restore_bar(_handle: &AppHandle) -> Result<(), String> {
    // No appbar registration or work-area changes to undo.
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
    wpctl(&["set-volume", "@DEFAULT_AUDIO_SINK@", &format!("{volume:.2}")]).map(|_| ())
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
    entry(target)?.set_password(secret).map_err(|e| e.to_string())
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
