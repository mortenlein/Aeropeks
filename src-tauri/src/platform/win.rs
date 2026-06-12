//! Windows implementations: Win32 appbar, WASAPI audio, GSMTC media,
//! Credential Manager secrets, Start Menu search, PowerShell terminal.

use std::ffi::c_void;
use std::os::windows::process::CommandExt;
use std::path::{Path, PathBuf};
use std::ptr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use portable_pty::CommandBuilder;
use tauri::{AppHandle, Emitter, Manager};
use walkdir::WalkDir;
use windows::core::{PCWSTR, PWSTR};
use windows::Win32::Foundation::{HWND, LPARAM, RECT};
use windows::Win32::Graphics::Dwm::{DwmGetWindowAttribute, DWMWA_EXTENDED_FRAME_BOUNDS};
use windows::Win32::Media::Audio::Endpoints::IAudioEndpointVolume;
use windows::Win32::Media::Audio::{
    eCapture, eConsole, eRender, IMMDeviceEnumerator, MMDeviceEnumerator,
};
use windows::Win32::Security::Credentials::{
    CredDeleteW, CredFree, CredReadW, CredWriteW, CREDENTIALW, CRED_PERSIST_LOCAL_MACHINE,
    CRED_TYPE_GENERIC,
};
use windows::Win32::System::Com::{
    CoCreateInstance, CoInitializeEx, CLSCTX_ALL, COINIT_MULTITHREADED,
};
use windows::Win32::System::Power::{GetSystemPowerStatus, SYSTEM_POWER_STATUS};
use windows::Win32::UI::Shell::{
    SHAppBarMessage, ABE_TOP, ABM_NEW, ABM_QUERYPOS, ABM_SETPOS, APPBARDATA,
};
use windows::Win32::UI::WindowsAndMessaging::{
    EnumWindows, GetClassNameW, GetForegroundWindow, GetSystemMetrics, GetWindowLongW,
    GetWindowRect, GetWindowThreadProcessId, IsWindowVisible, SetWindowLongW, SetWindowPos,
    ShowWindow, SystemParametersInfoW, GWL_EXSTYLE, GWL_STYLE, HWND_TOPMOST, SM_CXSCREEN,
    SM_CYSCREEN, SPI_SETWORKAREA, SWP_FRAMECHANGED, SWP_NOACTIVATE, SWP_NOZORDER, SWP_SHOWWINDOW,
    SW_HIDE, SW_SHOW, SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS, WS_CAPTION, WS_EX_NOACTIVATE,
    WS_EX_TOOLWINDOW, WS_MAXIMIZE, WS_POPUP,
};

use crate::launcher::SearchResult;
use crate::media::MediaInfo;
use crate::shell::BAR_HEIGHT;
use crate::system_status::{BatteryStatus, BluetoothStatus};

const CREATE_NO_WINDOW: u32 = 0x08000000;

// ── Bar placement ───────────────────────────────────────────────────

pub fn install_bar(
    window: tauri::WebviewWindow,
    width: u32,
    reserve_screen_space: bool,
    shutdown: Arc<AtomicBool>,
) -> Result<(), String> {
    let hwnd = HWND(window.hwnd().map_err(|e| e.to_string())?.0);

    if !reserve_screen_space {
        unsafe {
            SetWindowPos(
                hwnd,
                HWND_TOPMOST,
                0,
                0,
                width as i32,
                BAR_HEIGHT,
                SWP_NOACTIVATE | SWP_SHOWWINDOW | SWP_FRAMECHANGED,
            )
            .map_err(|e| e.to_string())?;
        }
        window.show().map_err(|e| e.to_string())?;
        return Ok(());
    }

    register_app_bar(hwnd, width);
    window.show().map_err(|e| e.to_string())?;
    thread::spawn(move || {
        let mut prev_was_fullscreen = false;
        let mut tick = 0u32;
        while !shutdown.load(Ordering::Relaxed) {
            let bar_hwnd = window
                .hwnd()
                .ok()
                .map(|h| HWND(h.0))
                .unwrap_or(HWND(std::ptr::null_mut()));

            if foreground_covers_screen(bar_hwnd) {
                prev_was_fullscreen = true;
                tick = 0;
                thread::sleep(Duration::from_secs(5));
                continue;
            }

            if prev_was_fullscreen {
                // The game just exited. Display mode restoration (e.g. 1440→1920)
                // takes ~1 s; skip one tick so we measure the right width.
                prev_was_fullscreen = false;
                thread::sleep(Duration::from_millis(1500));
                continue;
            }

            // Nudge any windows whose title bar ended up behind the bar.
            nudge_stuck_windows();
            // Re-assert app bar position every 5 seconds.
            if tick.is_multiple_of(5) {
                let _ = maintain_app_bar(&window);
            }
            tick = tick.wrapping_add(1);
            thread::sleep(Duration::from_secs(1));
        }
    });
    Ok(())
}

// Returns true when the foreground window covers the entire primary monitor,
// indicating an exclusive-fullscreen app (e.g. a game). We must not broadcast
// WM_SETTINGCHANGE via ABM_SETPOS while that is the case, as it kicks games
// out of fullscreen.
fn foreground_covers_screen(bar_hwnd: HWND) -> bool {
    unsafe {
        let fg = GetForegroundWindow();
        if fg.is_invalid() || fg == bar_hwnd {
            return false;
        }
        let mut rect = RECT::default();
        if GetWindowRect(fg, &mut rect).is_err() {
            return false;
        }
        let screen_w = GetSystemMetrics(SM_CXSCREEN);
        let screen_h = GetSystemMetrics(SM_CYSCREEN);
        rect.left <= 0 && rect.top <= 0 && rect.right >= screen_w && rect.bottom >= screen_h
    }
}

fn maintain_app_bar(window: &tauri::WebviewWindow) -> Result<(), String> {
    let monitor = window
        .primary_monitor()
        .map_err(|e| e.to_string())?
        .ok_or("no primary monitor available")?;
    let width = monitor.size().width;
    let hwnd = HWND(window.hwnd().map_err(|e| e.to_string())?.0);
    window
        .set_position(tauri::Position::Physical(tauri::PhysicalPosition {
            x: 0,
            y: 0,
        }))
        .map_err(|e| e.to_string())?;
    unsafe {
        let mut app_bar = app_bar_data(hwnd, ABE_TOP, width);
        SHAppBarMessage(ABM_QUERYPOS, &mut app_bar);
        app_bar.rc.top = 0;
        app_bar.rc.bottom = BAR_HEIGHT;
        SHAppBarMessage(ABM_SETPOS, &mut app_bar);
        let height = window.inner_size().map_err(|e| e.to_string())?.height;
        SetWindowPos(
            hwnd,
            HWND_TOPMOST,
            0,
            0,
            width as i32,
            height as i32,
            SWP_NOACTIVATE | SWP_FRAMECHANGED,
        )
        .map_err(|e| e.to_string())?;
    }
    Ok(())
}

fn app_bar_data(hwnd: HWND, edge: u32, width: u32) -> APPBARDATA {
    APPBARDATA {
        cbSize: std::mem::size_of::<APPBARDATA>() as u32,
        hWnd: hwnd,
        uCallbackMessage: 0x0401,
        uEdge: edge,
        rc: RECT {
            left: 0,
            top: 0,
            right: width as i32,
            bottom: BAR_HEIGHT,
        },
        lParam: LPARAM(0),
    }
}

fn register_app_bar(hwnd: HWND, width: u32) {
    unsafe {
        let style = GetWindowLongW(hwnd, GWL_STYLE);
        let _ = SetWindowLongW(hwnd, GWL_STYLE, (style as u32 | WS_POPUP.0) as i32);
        let ex_style = GetWindowLongW(hwnd, GWL_EXSTYLE);
        let _ = SetWindowLongW(
            hwnd,
            GWL_EXSTYLE,
            (ex_style as u32 | WS_EX_TOOLWINDOW.0 | WS_EX_NOACTIVATE.0) as i32,
        );
        let mut app_bar = app_bar_data(hwnd, ABE_TOP, width);
        let _ = SHAppBarMessage(windows::Win32::UI::Shell::ABM_REMOVE, &mut app_bar);
        SHAppBarMessage(ABM_NEW, &mut app_bar);
        SHAppBarMessage(ABM_QUERYPOS, &mut app_bar);
        app_bar.rc.top = 0;
        app_bar.rc.bottom = BAR_HEIGHT;
        SHAppBarMessage(ABM_SETPOS, &mut app_bar);
    }
}

fn cleanup_app_bar(hwnd: HWND, edge: u32) {
    unsafe {
        let mut app_bar = app_bar_data(hwnd, edge, 0);
        SHAppBarMessage(windows::Win32::UI::Shell::ABM_REMOVE, &mut app_bar);
    }
}

unsafe extern "system" fn enum_native_taskbar(
    hwnd: HWND,
    lparam: LPARAM,
) -> windows::Win32::Foundation::BOOL {
    let visible = *(lparam.0 as *const bool);
    let mut class_name = [0u16; 256];
    let length = GetClassNameW(hwnd, &mut class_name);
    let class_name = String::from_utf16_lossy(&class_name[..length as usize]);
    if class_name == "Shell_TrayWnd" || class_name == "Shell_SecondaryTrayWnd" {
        let _ = ShowWindow(hwnd, if visible { SW_SHOW } else { SW_HIDE });
    }
    true.into()
}

pub fn set_native_taskbar_visible(visible: bool) {
    unsafe {
        let mut visible = visible;
        let _ = EnumWindows(
            Some(enum_native_taskbar),
            LPARAM(&mut visible as *mut _ as isize),
        );
    }
}

fn reset_primary_work_area(handle: &AppHandle) -> Result<(), String> {
    let monitor = handle
        .primary_monitor()
        .map_err(|e| e.to_string())?
        .ok_or("no primary monitor available")?;
    let position = monitor.position();
    let size = monitor.size();
    let mut area = RECT {
        left: position.x,
        top: position.y,
        right: position.x + size.width as i32,
        bottom: position.y + size.height as i32,
    };
    unsafe {
        SystemParametersInfoW(
            SPI_SETWORKAREA,
            0,
            Some(&mut area as *mut _ as *mut _),
            SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS(0),
        )
        .map_err(|e| e.to_string())
    }
}

pub fn restore_bar(handle: &AppHandle) -> Result<(), String> {
    if let Some(window) = handle.get_webview_window("main") {
        if let Ok(hwnd) = window.hwnd() {
            cleanup_app_bar(HWND(hwnd.0), ABE_TOP);
        }
    }
    set_native_taskbar_visible(true);
    reset_primary_work_area(handle)
}

unsafe extern "system" fn nudge_window_if_stuck(
    hwnd: HWND,
    _lparam: LPARAM,
) -> windows::Win32::Foundation::BOOL {
    if !IsWindowVisible(hwnd).as_bool() {
        return true.into();
    }
    // Never nudge our own windows (bar, settings, panels).
    let mut pid = 0u32;
    GetWindowThreadProcessId(hwnd, Some(&mut pid));
    if pid == std::process::id() {
        return true.into();
    }
    let style = GetWindowLongW(hwnd, GWL_STYLE) as u32;
    if style & WS_CAPTION.0 == 0 || style & WS_MAXIMIZE.0 != 0 {
        return true.into();
    }
    let mut rect = RECT::default();
    // Quick check: skip any window whose raw top is already well below the bar.
    if GetWindowRect(hwnd, &mut rect).is_err() || rect.top >= BAR_HEIGHT {
        return true.into();
    }
    // Use DWM visible frame bounds to distinguish "correctly at work-area boundary"
    // (GetWindowRect.top ≈ 24 due to ~8px invisible DWM shadow) from truly stuck.
    let mut frame = RECT::default();
    let (visible_top, check_left, check_right) = match DwmGetWindowAttribute(
        hwnd,
        DWMWA_EXTENDED_FRAME_BOUNDS,
        std::ptr::addr_of_mut!(frame).cast(),
        std::mem::size_of::<RECT>() as u32,
    ) {
        Ok(_) => (frame.top, frame.left, frame.right),
        Err(_) => (rect.top + 8, rect.left, rect.right),
    };
    if visible_top >= BAR_HEIGHT {
        return true.into();
    }
    let screen_w = GetSystemMetrics(SM_CXSCREEN);
    if check_left >= screen_w || check_right <= 0 {
        return true.into();
    }
    let dwm_offset = visible_top - rect.top;
    let _ = SetWindowPos(
        hwnd,
        HWND(std::ptr::null_mut()),
        rect.left,
        BAR_HEIGHT - dwm_offset,
        rect.right - rect.left,
        rect.bottom - rect.top,
        SWP_NOACTIVATE | SWP_NOZORDER,
    );
    true.into()
}

fn nudge_stuck_windows() {
    unsafe {
        let _ = EnumWindows(Some(nudge_window_if_stuck), LPARAM(0));
    }
}

// ── Audio ───────────────────────────────────────────────────────────

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

pub fn get_volume() -> Result<f32, String> {
    unsafe {
        endpoint_volume(false)?
            .GetMasterVolumeLevelScalar()
            .map_err(|e| e.to_string())
    }
}

pub fn set_volume(volume: f32) -> Result<(), String> {
    unsafe {
        endpoint_volume(false)?
            .SetMasterVolumeLevelScalar(volume, std::ptr::null())
            .map_err(|e| e.to_string())
    }
}

pub fn mic_muted() -> Result<bool, String> {
    unsafe {
        endpoint_volume(true)?
            .GetMute()
            .map(|value| value.as_bool())
            .map_err(|e| e.to_string())
    }
}

pub fn set_mic_muted(muted: bool) -> Result<(), String> {
    unsafe {
        endpoint_volume(true)?
            .SetMute(muted, std::ptr::null())
            .map_err(|e| e.to_string())
    }
}

// ── Power / devices ─────────────────────────────────────────────────

pub fn battery_status() -> Result<BatteryStatus, String> {
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

pub fn set_cameras_disabled(disabled: bool) -> Result<(), String> {
    let command = if disabled {
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
}

pub fn bluetooth_status() -> Result<BluetoothStatus, String> {
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
}

pub fn run_power_action(action: &str) -> Result<(), String> {
    match action {
        "shutdown" => {
            std::process::Command::new("shutdown")
                .args(["/s", "/t", "0"])
                .creation_flags(CREATE_NO_WINDOW)
                .spawn()
                .map_err(|e| e.to_string())?;
        }
        "restart" => {
            std::process::Command::new("shutdown")
                .args(["/r", "/t", "0"])
                .creation_flags(CREATE_NO_WINDOW)
                .spawn()
                .map_err(|e| e.to_string())?;
        }
        "sleep" => unsafe {
            if !windows::Win32::System::Power::SetSuspendState(false, false, false).as_bool() {
                return Err("Windows rejected the sleep request".to_string());
            }
        },
        "lock" => unsafe {
            windows::Win32::System::Shutdown::LockWorkStation().map_err(|e| e.to_string())?;
        },
        _ => return Err("invalid power action".to_string()),
    }
    Ok(())
}

// ── Local media (GSMTC) ─────────────────────────────────────────────

pub async fn local_media() -> Result<Option<MediaInfo>, String> {
    use windows::Media::Control::{
        GlobalSystemMediaTransportControlsSession, GlobalSystemMediaTransportControlsSessionManager,
    };
    let manager = GlobalSystemMediaTransportControlsSessionManager::RequestAsync()
        .map_err(|e| e.to_string())?
        .get()
        .map_err(|e| e.to_string())?;
    let sessions = manager.GetSessions().map_err(|e| e.to_string())?;
    let mut best: Option<(i32, GlobalSystemMediaTransportControlsSession)> = None;
    for session in sessions {
        let status = session
            .GetPlaybackInfo()
            .ok()
            .and_then(|playback| playback.PlaybackStatus().ok())
            .map(|status| status.0)
            .unwrap_or_default();
        let score = match status {
            4 => 10,
            5 => 5,
            _ => 0,
        };
        if best
            .as_ref()
            .is_none_or(|(best_score, _)| score > *best_score)
        {
            best = Some((score, session));
        }
    }
    let Some((_, session)) = best else {
        return Ok(None);
    };
    let playback = session.GetPlaybackInfo().map_err(|e| e.to_string())?;
    let properties = session
        .TryGetMediaPropertiesAsync()
        .map_err(|e| e.to_string())?
        .get()
        .map_err(|e| e.to_string())?;
    Ok(Some(MediaInfo {
        title: properties.Title().unwrap_or_default().to_string(),
        artist: properties.Artist().unwrap_or_default().to_string(),
        album: properties.AlbumTitle().unwrap_or_default().to_string(),
        is_playing: playback.PlaybackStatus().unwrap_or_default().0 == 4,
        thumbnail: None,
        duration_ms: 0,
        view_offset_ms: 0,
        source: "gsmtc".to_string(),
        session_id: None,
        machine_id: None,
        address: None,
    }))
}

pub async fn local_media_action(action: &str) -> Result<(), String> {
    use windows::Media::Control::GlobalSystemMediaTransportControlsSessionManager;
    let manager = GlobalSystemMediaTransportControlsSessionManager::RequestAsync()
        .map_err(|e| e.to_string())?
        .get()
        .map_err(|e| e.to_string())?;
    if let Ok(session) = manager.GetCurrentSession() {
        match action {
            "play_pause" => {
                session
                    .TryTogglePlayPauseAsync()
                    .map_err(|e| e.to_string())?
                    .get()
                    .map_err(|e| e.to_string())?;
            }
            "next" => {
                session
                    .TrySkipNextAsync()
                    .map_err(|e| e.to_string())?
                    .get()
                    .map_err(|e| e.to_string())?;
            }
            "previous" => {
                session
                    .TrySkipPreviousAsync()
                    .map_err(|e| e.to_string())?
                    .get()
                    .map_err(|e| e.to_string())?;
            }
            _ => return Err("invalid media action".to_string()),
        }
    }
    Ok(())
}

/// Subscribe to GSMTC session changes so the bar refreshes without polling.
pub fn watch_local_media(handle: AppHandle) {
    tauri::async_runtime::spawn(async move {
        use windows::Foundation::TypedEventHandler;
        use windows::Media::Control::GlobalSystemMediaTransportControlsSessionManager;

        if let Ok(manager_op) = GlobalSystemMediaTransportControlsSessionManager::RequestAsync() {
            if let Ok(manager) = manager_op.get() {
                let h1 = handle.clone();
                let _ = manager.CurrentSessionChanged(&TypedEventHandler::new(move |_, _| {
                    let h2 = h1.clone();
                    tauri::async_runtime::spawn(async move {
                        if let Ok(update) = crate::media::active_media(h2.clone()).await {
                            let _ = h2.emit("media-change", update);
                        }
                    });
                    Ok(())
                }));

                let h3 = handle.clone();
                let _ = manager.SessionsChanged(&TypedEventHandler::new(move |_, _| {
                    let h4 = h3.clone();
                    tauri::async_runtime::spawn(async move {
                        if let Ok(update) = crate::media::active_media(h4.clone()).await {
                            let _ = h4.emit("media-change", update);
                        }
                    });
                    Ok(())
                }));
            }
        }
    });
}

// ── Secrets (Credential Manager) ────────────────────────────────────

fn wide(value: &str) -> Vec<u16> {
    value.encode_utf16().chain(Some(0)).collect()
}

pub fn write_secret(target: &str, secret: &str) -> Result<(), String> {
    let mut target = wide(target);
    let mut username = wide("Aeropeks");
    let mut blob = secret.as_bytes().to_vec();
    let credential = CREDENTIALW {
        Type: CRED_TYPE_GENERIC,
        TargetName: PWSTR(target.as_mut_ptr()),
        CredentialBlobSize: blob.len() as u32,
        CredentialBlob: blob.as_mut_ptr(),
        Persist: CRED_PERSIST_LOCAL_MACHINE,
        UserName: PWSTR(username.as_mut_ptr()),
        ..Default::default()
    };
    unsafe { CredWriteW(&credential, 0).map_err(|e| e.to_string()) }
}

pub fn restore_secret(target: &str, secret: Option<&str>) -> Result<(), String> {
    match secret {
        Some(secret) => write_secret(target, secret),
        None => {
            let target = wide(target);
            unsafe {
                CredDeleteW(PCWSTR(target.as_ptr()), CRED_TYPE_GENERIC, 0)
                    .or_else(|error| {
                        if error.code().0 as u32 == 1168 {
                            Ok(())
                        } else {
                            Err(error)
                        }
                    })
                    .map_err(|e| e.to_string())
            }
        }
    }
}

pub fn read_secret(target: &str) -> Result<Option<String>, String> {
    let target = wide(target);
    let mut credential = ptr::null_mut::<CREDENTIALW>();
    unsafe {
        if let Err(error) = CredReadW(
            PCWSTR(target.as_ptr()),
            CRED_TYPE_GENERIC,
            0,
            &mut credential,
        ) {
            if error.code().0 as u32 == 1168 {
                return Ok(None);
            }
            return Err(error.to_string());
        }
        if credential.is_null() {
            return Ok(None);
        }
        let value = {
            let credential_ref = &*credential;
            if credential_ref.CredentialBlobSize == 0 {
                Some(String::new())
            } else if credential_ref.CredentialBlob.is_null() {
                None
            } else {
                let bytes = std::slice::from_raw_parts(
                    credential_ref.CredentialBlob,
                    credential_ref.CredentialBlobSize as usize,
                );
                String::from_utf8(bytes.to_vec()).ok()
            }
        };
        CredFree(credential.cast::<c_void>());
        Ok(value)
    }
}

// ── App launcher (Start Menu) ───────────────────────────────────────

fn start_menu_roots() -> Vec<PathBuf> {
    let mut roots = vec![PathBuf::from(
        r"C:\ProgramData\Microsoft\Windows\Start Menu\Programs",
    )];
    if let Some(profile) = std::env::var_os("USERPROFILE") {
        roots.push(
            PathBuf::from(profile).join(r"AppData\Roaming\Microsoft\Windows\Start Menu\Programs"),
        );
    }
    roots
}

pub fn installed_app_results(normalized_query: &str) -> Vec<SearchResult> {
    let mut results = Vec::new();
    for root in start_menu_roots() {
        for entry in WalkDir::new(root).into_iter().filter_map(Result::ok) {
            let path = entry.path();
            let Some(file_name) = path.file_name().and_then(|name| name.to_str()) else {
                continue;
            };
            if path.extension().and_then(|ext| ext.to_str()) == Some("lnk")
                && file_name.to_lowercase().contains(normalized_query)
            {
                results.push(SearchResult {
                    id: format!("app-{}", path.display()),
                    title: file_name.trim_end_matches(".lnk").to_string(),
                    description: path.display().to_string(),
                    icon: "AppWindow".to_string(),
                    action_type: "app".to_string(),
                    action_value: path.display().to_string(),
                });
            }
        }
    }
    results
}

pub fn validate_app_target(path: &Path) -> Result<(), String> {
    let allowed = path.extension().and_then(|ext| ext.to_str()) == Some("lnk")
        && start_menu_roots().iter().any(|root| path.starts_with(root));
    if allowed {
        Ok(())
    } else {
        Err("application is outside the Start Menu".to_string())
    }
}

// ── Terminal shell ──────────────────────────────────────────────────

pub fn shell_command(command: Option<String>) -> CommandBuilder {
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
    let mut cmd = CommandBuilder::new(shell);
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
    cmd
}
