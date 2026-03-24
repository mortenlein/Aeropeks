use std::sync::{Arc, Mutex};
use std::path::PathBuf;
use std::fs;
use windows::Win32::UI::WindowsAndMessaging::{
    GetForegroundWindow, GetWindowTextW, SetWindowPos, 
    HWND_TOPMOST, SWP_NOACTIVATE, SWP_SHOWWINDOW, SWP_FRAMECHANGED,
    GetWindowLongW, SetWindowLongW, GWL_STYLE, WS_POPUP,
    SystemParametersInfoW, SPI_SETWORKAREA, SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS
};
use windows::Win32::UI::Shell::{
    SHAppBarMessage, APPBARDATA, ABM_NEW, ABM_SETPOS, ABM_QUERYPOS, ABE_TOP
};
use windows::Win32::Foundation::{HWND, RECT};
use tauri::{Window, Manager, Emitter};
use tauri::menu::{Menu, MenuItem};
use tauri::tray::TrayIconBuilder;
use std::thread;
use std::time::Duration;
use serde::{Serialize, Deserialize};

// ── Settings ────────────────────────────────────────────────────────

#[derive(Serialize, Deserialize, Clone, Debug)]
struct AppSettings {
    plex_url: String,
    plex_token: String,
}

type SharedSettings = Arc<Mutex<AppSettings>>;

fn get_settings_path(handle: tauri::AppHandle) -> PathBuf {
    let mut path = handle.path().app_data_dir().unwrap();
    fs::create_dir_all(&path).ok();
    path.push("settings.json");
    path
}

fn fetch_settings_helper(handle: tauri::AppHandle) -> AppSettings {
    let path = get_settings_path(handle);
    if let Ok(content) = fs::read_to_string(path) {
        if let Ok(settings) = serde_json::from_str(&content) {
            return settings;
        }
    }
    AppSettings {
        plex_url: "http://localhost:32400".to_string(),
        plex_token: "".to_string(),
    }
}

#[tauri::command]
fn save_settings(settings: AppSettings, handle: tauri::AppHandle, state: tauri::State<'_, SharedSettings>) -> Result<(), String> {
    let path = get_settings_path(handle);
    let content = serde_json::to_string(&settings).map_err(|e| e.to_string())?;
    fs::write(path, content).map_err(|e| e.to_string())?;
    
    let mut current_settings = state.lock().map_err(|e| e.to_string())?;
    *current_settings = settings;
    Ok(())
}

#[tauri::command]
fn get_settings(state: tauri::State<'_, SharedSettings>) -> Result<AppSettings, String> {
    let settings = state.lock().map_err(|e| e.to_string())?;
    Ok(settings.clone())
}

// ── Media Info ──────────────────────────────────────────────────────

#[derive(Serialize, Clone, Debug)]
struct MediaInfo {
    title: String,
    artist: String,
    is_playing: bool,
    session_id: String,
    machine_id: String,
    address: String, 
}

fn fetch_plex_media(settings: &AppSettings) -> Option<MediaInfo> {
    let url = format!("{}/status/sessions?X-Plex-Token={}", settings.plex_url.trim_end_matches('/'), settings.plex_token);
    let client = reqwest::blocking::Client::builder().timeout(Duration::from_secs(3)).build().ok()?;
    let resp = client.get(&url).header("Accept", "application/json").send().ok()?;
    let json: serde_json::Value = resp.json().ok()?;
    
    let container = json.get("MediaContainer")?;
    let metadata = container.get("Metadata")?.as_array()?;

    for item in metadata {
        let title = item.get("title").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let artist = item.get("grandparentTitle").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let session_id = item.get("sessionKey").and_then(|v| v.as_str()).unwrap_or("").to_string();
        
        let player = item.get("Player").and_then(|v| v.as_object());
        let machine_id = player.and_then(|p| p.get("machineIdentifier")).and_then(|v| v.as_str()).unwrap_or("").to_string();
        let address = player.and_then(|p| p.get("address")).and_then(|v| v.as_str()).unwrap_or("").to_string();
        let state = player.and_then(|p| p.get("state")).and_then(|v| v.as_str()).unwrap_or("");

        if !title.is_empty() {
            return Some(MediaInfo {
                title,
                artist,
                is_playing: state == "playing",
                session_id,
                machine_id,
                address,
            });
        }
    }
    None
}

#[tauri::command]
fn get_media_info(state: tauri::State<'_, SharedSettings>) -> Result<MediaInfo, String> {
    let settings = state.lock().map_err(|e| e.to_string())?;
    fetch_plex_media(&settings).ok_or("Nothing Playing".to_string())
}

#[tauri::command]
fn plex_control(command: String, session_id: String, machine_id: String, address: String, state: tauri::State<'_, SharedSettings>) -> Result<(), String> {
    let settings = state.lock().map_err(|e| e.to_string())?;
    
    let plex_command = match command.as_str() {
        "play" => "play",
        "pause" => "pause",
        "next" => "skipNext",
        "prev" => "skipPrevious",
        _ => return Err("Invalid command".to_string()),
    };

    let client = reqwest::blocking::Client::builder().timeout(Duration::from_secs(2)).build().unwrap_or_default();
    
    let mut urls = vec![
        // Attempt 1: Standard Player Proxy
        format!("{}/player/proxy/playback/{}?X-Plex-Target-Client-Identifier={}&X-Plex-Token={}&sessionKey={}", 
            settings.plex_url.trim_end_matches('/'), plex_command, machine_id, settings.plex_token, session_id),
        // Attempt 2: Alternative System Players Proxy
        format!("{}/system/players/{}/playback/{}?X-Plex-Token={}", 
            settings.plex_url.trim_end_matches('/'), machine_id, plex_command, settings.plex_token)
    ];

    if !address.is_empty() {
        // Attempt 3: Direct Player IP (Plexamp listens on 32500)
        urls.push(format!("http://{}:32500/player/playback/{}?X-Plex-Token={}&commandID=1", address, plex_command, settings.plex_token));
        // Attempt 4: Direct Player IP (32400)
        urls.push(format!("http://{}:32400/player/playback/{}?X-Plex-Token={}&commandID=1", address, plex_command, settings.plex_token));
    }

    for url in urls {
        if let Ok(resp) = client.get(&url)
            .header("X-Plex-Target-Client-Identifier", &machine_id)
            .header("X-Plex-Token", &settings.plex_token)
            .send() {
                if resp.status().is_success() { return Ok(()); }
            }
    }
    
    Err("All Plex control attempts failed".to_string())
}

// ── Volume ──────────────────────────────────────────────────────────

use windows::Win32::Media::Audio::{IMMDeviceEnumerator, MMDeviceEnumerator, eRender, eConsole};
use windows::Win32::Media::Audio::Endpoints::IAudioEndpointVolume;
use windows::Win32::System::Com::{CoCreateInstance, CLSCTX_ALL, CoInitializeEx, COINIT_MULTITHREADED};

#[tauri::command]
fn get_volume() -> f32 {
    unsafe {
        let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
        let enumerator: IMMDeviceEnumerator = CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL).unwrap();
        if let Ok(device) = enumerator.GetDefaultAudioEndpoint(eRender, eConsole) {
            if let Ok(volume) = device.Activate::<IAudioEndpointVolume>(CLSCTX_ALL, None) {
                return volume.GetMasterVolumeLevelScalar().unwrap_or(0.5);
            }
        }
        0.5
    }
}

#[tauri::command]
fn set_volume(volume: f32) {
    unsafe {
        let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
        let enumerator: IMMDeviceEnumerator = CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL).unwrap();
        if let Ok(device) = enumerator.GetDefaultAudioEndpoint(eRender, eConsole) {
            if let Ok(volume_ctrl) = device.Activate::<IAudioEndpointVolume>(CLSCTX_ALL, None) {
                let _ = volume_ctrl.SetMasterVolumeLevelScalar(volume, std::ptr::null());
            }
        }
    }
}

// ── Settings Window ─────────────────────────────────────────────────

#[tauri::command]
fn open_settings(handle: tauri::AppHandle) {
    if let Some(window) = handle.get_webview_window("settings") {
        let _ = window.show().ok();
        let _ = window.set_focus().ok();
    }
}

#[tauri::command]
fn toggle_expanded_player(handle: tauri::AppHandle) {
    if let Some(window) = handle.get_webview_window("expanded-player") {
        let is_visible = window.is_visible().unwrap_or(false);
        if is_visible {
            let _ = window.hide().ok();
        } else {
            // Position it below the bar, centered
            if let Ok(Some(monitor)) = handle.primary_monitor() {
                let m_size = monitor.size();
                let width = 640;
                let height = 280; // Increased to 280 to be safe
                let x = (m_size.width as i32 - width) / 2;
                let y = 36; 
                let _ = window.set_size(tauri::Size::Physical(tauri::PhysicalSize { width: width as u32, height: height as u32 })).ok();
                let _ = window.set_position(tauri::Position::Physical(tauri::PhysicalPosition { x, y })).ok();
            }
            let _ = window.show().ok();
            let _ = window.set_focus().ok();
        }
    }
}

// ── AppBar ──────────────────────────────────────────────────────────

fn register_app_bar(hwnd_v: HWND, width: u32) {
    unsafe {
        let current_style = GetWindowLongW(hwnd_v, GWL_STYLE);
        let _ = SetWindowLongW(hwnd_v, GWL_STYLE, (current_style as u32 | WS_POPUP.0) as i32);

        let mut abd = APPBARDATA {
            cbSize: std::mem::size_of::<APPBARDATA>() as u32,
            hWnd: hwnd_v,
            uCallbackMessage: 0x0401,
            uEdge: ABE_TOP as u32,
            rc: RECT { left: 0, top: 0, right: width as i32, bottom: 32 },
            lParam: windows::Win32::Foundation::LPARAM(0),
        };

        let _ = SHAppBarMessage(windows::Win32::UI::Shell::ABM_REMOVE, &mut abd);
        SHAppBarMessage(ABM_NEW, &mut abd);
        SHAppBarMessage(ABM_QUERYPOS, &mut abd);
        abd.rc.top = 0;
        abd.rc.bottom = 32;
        SHAppBarMessage(ABM_SETPOS, &mut abd);

        let _ = SetWindowPos(hwnd_v, HWND_TOPMOST, 0, 0, width as i32, 32, SWP_NOACTIVATE | SWP_SHOWWINDOW | SWP_FRAMECHANGED);
    }
}

fn cleanup_app_bar(hwnd_v: HWND) {
    unsafe {
        let mut abd = APPBARDATA {
            cbSize: std::mem::size_of::<APPBARDATA>() as u32,
            hWnd: hwnd_v,
            uCallbackMessage: 0,
            uEdge: ABE_TOP as u32,
            rc: RECT::default(),
            lParam: windows::Win32::Foundation::LPARAM(0),
        };
        SHAppBarMessage(windows::Win32::UI::Shell::ABM_REMOVE, &mut abd);

        // RESET WORK AREA via SPI_SETWORKAREA (Nuclear Cleanup)
        // Hardcoded safe reset or better: get screen size.
        // For now, setting top to 0 is the primary goal.
        let mut reset_rect = RECT { left: 0, top: 0, right: 3840, bottom: 2160 }; 
        SystemParametersInfoW(
            SPI_SETWORKAREA,
            0,
            Some(&mut reset_rect as *mut _ as *mut _),
            SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS(0),
        ).ok();

        println!("DEBUG appbar: Cleanup complete (Reserved space released)");
    }
}

// ── Main ────────────────────────────────────────────────────────────

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            let app_handle = app.handle().clone();
            let app_handle_media = app.handle().clone();

            let initial_settings = fetch_settings_helper(app.handle().clone());
            let shared: SharedSettings = Arc::new(Mutex::new(initial_settings));
            app.manage(shared.clone());

            let media_settings = shared.clone();

            if let Ok(Some(monitor)) = app.handle().primary_monitor() {
                let size = monitor.size();
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.set_size(tauri::Size::Physical(tauri::PhysicalSize { width: size.width, height: 32 }));
                    let _ = window.set_shadow(false);
                    let hwnd = window.hwnd().unwrap();
                    register_app_bar(HWND(hwnd.0), size.width);

                    let w_clone = window.clone();
                    thread::spawn(move || {
                        loop {
                            let _ = w_clone.set_position(tauri::Position::Physical(tauri::PhysicalPosition { x: 0, y: 0 }));
                            if let Ok(Some(monitor)) = w_clone.primary_monitor() {
                                let width = monitor.size().width;
                                let work_area = monitor.work_area();
                                let hwnd = w_clone.hwnd().unwrap();
                                let hwnd_v = HWND(hwnd.0);
                                unsafe {
                                    let mut abd = APPBARDATA {
                                        cbSize: std::mem::size_of::<APPBARDATA>() as u32,
                                        hWnd: hwnd_v,
                                        uCallbackMessage: 0x0401,
                                        uEdge: ABE_TOP as u32,
                                        rc: RECT { left: 0, top: 0, right: width as i32, bottom: 32 },
                                        lParam: windows::Win32::Foundation::LPARAM(0),
                                    };
                                    SHAppBarMessage(ABM_QUERYPOS, &mut abd);
                                    abd.rc.top = 0; abd.rc.bottom = 32;
                                    SHAppBarMessage(ABM_SETPOS, &mut abd);
                                    let _ = SetWindowPos(hwnd_v, HWND_TOPMOST, 0, 0, width as i32, 32, SWP_NOACTIVATE | SWP_FRAMECHANGED);
                                    
                                    if work_area.position.y != 32 {
                                        println!("DEBUG workarea: Failure! y={}. Attempting SPI_SETWORKAREA...", work_area.position.y);
                                        let mut new_work_area = RECT {
                                            left: 0,
                                            top: 32,
                                            right: width as i32,
                                            bottom: monitor.size().height as i32,
                                        };
                                        SystemParametersInfoW(
                                            SPI_SETWORKAREA,
                                            0,
                                            Some(&mut new_work_area as *mut _ as *mut _),
                                            SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS(0),
                                        ).ok();
                                    }
                                }
                            }
                            thread::sleep(Duration::from_secs(5));
                        }
                    });
                }
            }

            let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>).expect("");
            let settings_i = MenuItem::with_id(app, "settings", "Settings", true, None::<&str>).expect("");
            let menu = Menu::with_items(app, &[&settings_i, &quit_i]).expect("");
            
            let tray_builder = TrayIconBuilder::new()
                .menu(&menu)
                .show_menu_on_left_click(true)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "quit" => { app.exit(0); }
                    "settings" => { open_settings(app.clone()); }
                    _ => {}
                });
            
            let mut tray = tray_builder;
            if let Some(icon) = app.default_window_icon() { tray = tray.icon(icon.clone()); }
            let _ = tray.build(app);

            thread::spawn(move || {
                let mut last_title = String::new();
                loop {
                    unsafe {
                        let hwnd = windows::Win32::Foundation::HWND(GetForegroundWindow().0);
                        if !hwnd.0.is_null() {
                            let mut buffer = [0u16; 512];
                            let length = GetWindowTextW(hwnd, &mut buffer);
                            if length > 0 {
                                let title = String::from_utf16_lossy(&buffer[..length as usize]);
                                if title != last_title {
                                    let _ = app_handle.emit("window-change", title.clone());
                                    last_title = title;
                                }
                            }
                        }
                    }
                    thread::sleep(Duration::from_millis(1000));
                }
            });

            thread::spawn(move || {
                loop {
                    let settings = match media_settings.lock() { Ok(s) => s.clone(), Err(_) => { thread::sleep(Duration::from_secs(5)); continue; } };
                    let current = fetch_plex_media(&settings);
                    let _ = app_handle_media.emit("media-change", current);
                    thread::sleep(Duration::from_secs(3));
                }
            });
            Ok(())
        })
        .on_window_event(|window, event| {
            match event {
                tauri::WindowEvent::CloseRequested { api, .. } => {
                    if window.label() == "settings" {
                        api.prevent_close();
                        let _ = window.hide();
                    } else if window.label() == "main" {
                        if let Ok(hwnd) = window.hwnd() {
                            cleanup_app_bar(HWND(hwnd.0));
                        }
                    }
                }
                tauri::WindowEvent::Destroyed => {
                    if window.label() == "main" {
                        if let Ok(hwnd) = window.hwnd() {
                            cleanup_app_bar(HWND(hwnd.0));
                        }
                    }
                }
                _ => {}
            }
        })
        .invoke_handler(tauri::generate_handler![get_volume, set_volume, get_media_info, get_settings, save_settings, open_settings, toggle_expanded_player, plex_control])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
