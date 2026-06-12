use std::sync::Mutex;

use serde::Serialize;
use tauri::Window;

use crate::{platform, security};

#[derive(Serialize)]
pub struct BatteryStatus {
    pub(crate) percentage: u8,
    pub(crate) is_charging: bool,
    pub(crate) has_battery: bool,
}

#[derive(Debug, Serialize, Clone, Default)]
pub struct BluetoothStatus {
    pub(crate) connected: bool,
    pub(crate) devices: Vec<String>,
}

#[derive(Default)]
pub struct PrivacyState {
    inner: Mutex<PrivacyStateInner>,
}

#[derive(Default)]
struct PrivacyStateInner {
    enabled: bool,
    prior_mic_muted: Option<bool>,
}

#[tauri::command]
pub fn get_volume(window: Window) -> Result<f32, String> {
    security::require_window(&window, &["main"])?;
    platform::get_volume()
}

#[tauri::command]
pub fn set_volume(volume: f32, window: Window) -> Result<(), String> {
    security::require_window(&window, &["main"])?;
    security::validate_volume(volume)?;
    platform::set_volume(volume)
}

#[tauri::command]
pub fn get_battery_status(window: Window) -> Result<BatteryStatus, String> {
    security::require_window(&window, &["main"])?;
    platform::battery_status()
}

#[tauri::command]
pub fn get_mic_status(window: Window) -> Result<bool, String> {
    security::require_window(&window, &["main"])?;
    platform::mic_muted()
}

#[tauri::command]
pub fn toggle_mic_mute(
    window: Window,
    privacy: tauri::State<'_, PrivacyState>,
) -> Result<bool, String> {
    security::require_window(&window, &["main"])?;
    if privacy.inner.lock().map_err(|e| e.to_string())?.enabled {
        return Err("microphone is controlled by privacy mode".to_string());
    }
    let next = !platform::mic_muted()?;
    platform::set_mic_muted(next)?;
    Ok(next)
}

#[tauri::command]
pub fn get_privacy_status(
    window: Window,
    privacy: tauri::State<'_, PrivacyState>,
) -> Result<bool, String> {
    security::require_window(&window, &["main"])?;
    Ok(privacy.inner.lock().map_err(|e| e.to_string())?.enabled)
}

#[tauri::command]
pub async fn set_privacy_mode(
    enabled: bool,
    window: Window,
    privacy: tauri::State<'_, PrivacyState>,
) -> Result<(), String> {
    security::require_window(&window, &["main"])?;
    let prior_mic_muted = {
        let state = privacy.inner.lock().map_err(|e| e.to_string())?;
        if state.enabled == enabled {
            return Ok(());
        }
        if enabled {
            Some(platform::mic_muted()?)
        } else {
            state.prior_mic_muted
        }
    };

    if enabled {
        platform::set_mic_muted(true)?;
    }

    let camera_result =
        tauri::async_runtime::spawn_blocking(move || platform::set_cameras_disabled(enabled))
            .await
            .map_err(|e| e.to_string())?;

    if let Err(error) = camera_result {
        if enabled {
            let _ = platform::set_mic_muted(prior_mic_muted.unwrap_or(false));
        }
        return Err(error);
    }

    if !enabled {
        platform::set_mic_muted(prior_mic_muted.unwrap_or(false))?;
    }
    let mut state = privacy.inner.lock().map_err(|e| e.to_string())?;
    state.enabled = enabled;
    state.prior_mic_muted = enabled.then_some(prior_mic_muted.unwrap_or(false));
    Ok(())
}

#[tauri::command]
pub async fn get_bluetooth_status(window: Window) -> Result<BluetoothStatus, String> {
    security::require_window(&window, &["main"])?;
    tauri::async_runtime::spawn_blocking(platform::bluetooth_status)
        .await
        .map_err(|e| e.to_string())?
}

#[cfg(test)]
mod tests {
    use super::PrivacyStateInner;

    #[test]
    fn privacy_state_starts_disabled() {
        let state = PrivacyStateInner::default();
        assert!(!state.enabled);
        assert_eq!(state.prior_mic_muted, None);
    }
}
