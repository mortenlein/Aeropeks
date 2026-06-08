use std::os::windows::process::CommandExt;
use std::sync::Mutex;

use serde::Serialize;
use tauri::Window;
use windows::Win32::Media::Audio::Endpoints::IAudioEndpointVolume;
use windows::Win32::Media::Audio::{
    eCapture, eConsole, eRender, IMMDeviceEnumerator, MMDeviceEnumerator,
};
use windows::Win32::System::Com::{
    CoCreateInstance, CoInitializeEx, CLSCTX_ALL, COINIT_MULTITHREADED,
};
use windows::Win32::System::Power::{GetSystemPowerStatus, SYSTEM_POWER_STATUS};

use crate::security;

const CREATE_NO_WINDOW: u32 = 0x08000000;

#[derive(Serialize)]
pub struct BatteryStatus {
    percentage: u8,
    is_charging: bool,
    has_battery: bool,
}

#[derive(Debug, Serialize, Clone, Default)]
pub struct BluetoothStatus {
    connected: bool,
    devices: Vec<String>,
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

fn endpoint_volume(capture: bool) -> Result<IAudioEndpointVolume, String> {
    unsafe {
        let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
        let enumerator: IMMDeviceEnumerator =
            CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)
                .map_err(|e: windows::core::Error| e.to_string())?;
        let device = enumerator
            .GetDefaultAudioEndpoint(if capture { eCapture } else { eRender }, eConsole)
            .map_err(|e: windows::core::Error| e.to_string())?;
        device
            .Activate::<IAudioEndpointVolume>(CLSCTX_ALL, None)
            .map_err(|e: windows::core::Error| e.to_string())
    }
}

fn mic_muted() -> Result<bool, String> {
    unsafe {
        endpoint_volume(true)?
            .GetMute()
            .map(|value| value.as_bool())
            .map_err(|e| e.to_string())
    }
}

fn set_mic_muted(muted: bool) -> Result<(), String> {
    unsafe {
        endpoint_volume(true)?
            .SetMute(muted, std::ptr::null())
            .map_err(|e| e.to_string())
    }
}

#[tauri::command]
pub fn get_volume(window: Window) -> Result<f32, String> {
    security::require_window(&window, &["main"])?;
    unsafe {
        endpoint_volume(false)?
            .GetMasterVolumeLevelScalar()
            .map_err(|e| e.to_string())
    }
}

#[tauri::command]
pub fn set_volume(volume: f32, window: Window) -> Result<(), String> {
    security::require_window(&window, &["main"])?;
    security::validate_volume(volume)?;
    unsafe {
        endpoint_volume(false)?
            .SetMasterVolumeLevelScalar(volume, std::ptr::null())
            .map_err(|e| e.to_string())
    }
}

#[tauri::command]
pub fn get_battery_status(window: Window) -> Result<BatteryStatus, String> {
    security::require_window(&window, &["main"])?;
    unsafe {
        let mut status = SYSTEM_POWER_STATUS::default();
        GetSystemPowerStatus(&mut status).map_err(|e| e.to_string())?;
        Ok(BatteryStatus {
            percentage: status.BatteryLifePercent,
            is_charging: status.ACLineStatus == 1,
            has_battery: status.BatteryFlag != 128,
        })
    }
}

#[tauri::command]
pub fn get_mic_status(window: Window) -> Result<bool, String> {
    security::require_window(&window, &["main"])?;
    mic_muted()
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
    let next = !mic_muted()?;
    set_mic_muted(next)?;
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
            Some(mic_muted()?)
        } else {
            state.prior_mic_muted
        }
    };

    if enabled {
        set_mic_muted(true)?;
    }

    let camera_result = tauri::async_runtime::spawn_blocking(move || {
        let command = if enabled {
            "Disable-PnpDevice"
        } else {
            "Enable-PnpDevice"
        };
        let status = std::process::Command::new("powershell")
            .creation_flags(CREATE_NO_WINDOW)
            .args([
                "-NoProfile",
                "-NonInteractive",
                "-Command",
                &format!(
                    "Get-PnpDevice -Class Camera -ErrorAction Stop | {command} -Confirm:$false -ErrorAction Stop"
                ),
            ])
            .status()
            .map_err(|e| e.to_string())?;
        if status.success() {
            Ok(())
        } else {
            Err(format!("camera command exited with {status}"))
        }
    })
    .await
    .map_err(|e| e.to_string())?;

    if let Err(error) = camera_result {
        if enabled {
            let _ = set_mic_muted(prior_mic_muted.unwrap_or(false));
        }
        return Err(error);
    }

    if !enabled {
        set_mic_muted(prior_mic_muted.unwrap_or(false))?;
    }
    let mut state = privacy.inner.lock().map_err(|e| e.to_string())?;
    state.enabled = enabled;
    state.prior_mic_muted = enabled.then_some(prior_mic_muted.unwrap_or(false));
    Ok(())
}

#[tauri::command]
pub async fn get_bluetooth_status(window: Window) -> Result<BluetoothStatus, String> {
    security::require_window(&window, &["main"])?;
    tauri::async_runtime::spawn_blocking(|| {
        let output = std::process::Command::new("powershell")
            .creation_flags(CREATE_NO_WINDOW)
            .args([
                "-NoProfile",
                "-NonInteractive",
                "-Command",
                "Get-PnpDevice -Class Bluetooth | Where-Object { $_.Status -eq 'OK' -and $_.Present -eq $true -and $_.InstanceId -like 'BTHENUM*' -and $_.FriendlyName -notmatch 'Service|Transport|Enumerator|Gateway|Radio|Adapter|Controller|Generic' } | Select-Object -ExpandProperty FriendlyName",
            ])
            .output()
            .map_err(|e| e.to_string())?;
        if !output.status.success() {
            return Err(format!("Bluetooth query exited with {}", output.status));
        }
        let devices = String::from_utf8_lossy(&output.stdout)
            .lines()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string)
            .collect::<Vec<_>>();
        Ok(BluetoothStatus {
            connected: !devices.is_empty(),
            devices,
        })
    })
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
