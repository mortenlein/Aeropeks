// Aeropeks v0.1.0 - Terminal Fix Build
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use std::path::PathBuf;
use std::fs;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use windows::Win32::UI::WindowsAndMessaging::{
    GetForegroundWindow, GetWindowTextW, SetWindowPos, 
    HWND_TOPMOST, SWP_NOACTIVATE, SWP_SHOWWINDOW, SWP_FRAMECHANGED,
    GetWindowLongW, SetWindowLongW, GWL_STYLE, GWL_EXSTYLE, WS_POPUP,
    WS_EX_TOOLWINDOW, WS_EX_NOACTIVATE,
    SystemParametersInfoW, SPI_SETWORKAREA, SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS,
    EnumWindows, IsWindowVisible, SetForegroundWindow, ShowWindow, SW_RESTORE,
    GetWindowThreadProcessId, GetClassNameW, GetIconInfo, DestroyIcon,
    SendMessageTimeoutW, GetClassLongPtrW, WM_GETICON, ICON_BIG, ICON_SMALL,
    ICON_SMALL2, SMTO_ABORTIFHUNG, GCLP_HICON, GCLP_HICONSM, HICON,
    SW_HIDE, SW_SHOW, GetWindowRect, WINDOWPLACEMENT, GetWindowPlacement,
    SW_SHOWMINIMIZED, SW_MINIMIZE
};
use windows::Win32::UI::Shell::{
    SHAppBarMessage, APPBARDATA, ABM_NEW, ABM_SETPOS, ABM_QUERYPOS, ABE_TOP, ABE_BOTTOM,
    ExtractIconExW, IShellItemImageFactory, SHCreateItemFromParsingName, SHGetFileInfoW,
    SHGFI_ICON, SHGFI_LARGEICON, SHFILEINFOW, SIIGBF_BIGGERSIZEOK
};
use windows::Win32::UI::Shell::PropertiesSystem::{IPropertyStore, PROPERTYKEY, SHGetPropertyStoreForWindow};
use windows::Win32::Graphics::Gdi::{
    GetDIBits, GetObjectW, CreateCompatibleDC, DeleteDC, DeleteObject,
    BITMAP, BITMAPINFO, BITMAPINFOHEADER, DIB_RGB_COLORS, BI_RGB, HBITMAP,
    CreateCompatibleBitmap, SelectObject, SetStretchBltMode, StretchBlt,
    HALFTONE, SRCCOPY, HGDIOBJ, GetDC, ReleaseDC
};
use windows::Win32::Storage::Xps::{PrintWindow, PRINT_WINDOW_FLAGS};
use windows::Win32::Graphics::Dwm::{DwmGetWindowAttribute, DWMWA_EXTENDED_FRAME_BOUNDS};
use std::collections::{HashMap, HashSet};
use windows::Win32::System::Threading::{
    OpenProcess, QueryFullProcessImageNameW, PROCESS_QUERY_LIMITED_INFORMATION, PROCESS_NAME_WIN32
};
use windows::Win32::Storage::EnhancedStorage::{
    PKEY_AppUserModel_ID, PKEY_AppUserModel_RelaunchCommand, PKEY_AppUserModel_RelaunchIconResource
};
use windows::Win32::System::Com::{CoTaskMemFree, IBindCtx};
use windows::Win32::System::Com::StructuredStorage::{PropVariantClear, PropVariantToStringAlloc};
use windows::Win32::Foundation::{CloseHandle, HWND, RECT, SIZE, LPARAM, WPARAM};
use windows::Win32::System::Power::{GetSystemPowerStatus, SYSTEM_POWER_STATUS};
use tauri::{Window, Manager, Emitter, AppHandle, State};
use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::TrayIconBuilder;
use std::thread;
use std::time::Duration;
use tokio::time::interval;
use serde::{Serialize, Deserialize};
use portable_pty::{CommandBuilder, NativePtySystem, PtySize, PtySystem, MasterPty};
use std::io::{Write, Read};
use base64::{Engine as _, engine::general_purpose};
use serde_json;
use walkdir::WalkDir;
#[cfg(windows)]

use std::os::windows::process::CommandExt;
const CREATE_NO_WINDOW: u32 = 0x08000000;
const ICON_CACHE_VERSION: &str = "v2-bgra-bmp";


#[derive(Serialize, Clone)]
struct PtyPayload {
    data: String,
}

fn default_accent_color() -> String { "#22c55e".to_string() }
fn default_plex_url() -> String { "http://localhost:32400".to_string() }
fn default_true() -> bool { true }
fn default_reserve_screen_space() -> bool { true }
fn default_hide_native_taskbar() -> bool { false }
fn default_debug_inspector() -> bool { false }

#[derive(Serialize, Deserialize, Clone, Debug)]
struct TerminalShortcut {
    id: String,
    label: String,
    cmd: String,
    shortcut: String,
}

fn default_shortcuts() -> Vec<TerminalShortcut> {
    vec![
        TerminalShortcut { id: "local-bash".to_string(), label: "Local Terminal".to_string(), cmd: "".to_string(), shortcut: "Alt+T".to_string() },
        TerminalShortcut { id: "ssh-home".to_string(), label: "SSH: Home Lab".to_string(), cmd: "ssh pi@homeserver.local".to_string(), shortcut: "".to_string() },
        TerminalShortcut { id: "git-status".to_string(), label: "Git Status".to_string(), cmd: "git status".to_string(), shortcut: "".to_string() },
    ]
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct AppSettings {
    #[serde(default = "default_plex_url")]
    plex_url: String,
    #[serde(default)]
    plex_token: String,
    #[serde(default = "default_accent_color")]
    accent_color: String,
    #[serde(default = "default_shortcuts")]
    terminal_shortcuts: Vec<TerminalShortcut>,
    #[serde(default)]
    weather_location: String,
    #[serde(default)]
    weather_lat: Option<f64>,
    #[serde(default)]
    weather_lon: Option<f64>,
    #[serde(default)]
    obs_websocket_url: String,
    #[serde(default)]
    obs_websocket_password: String,
    #[serde(default = "default_true")]
    use_24h: bool,
    #[serde(default = "default_reserve_screen_space")]
    reserve_screen_space: bool,
    #[serde(default = "default_hide_native_taskbar")]
    hide_native_taskbar: bool,
    #[serde(default = "default_debug_inspector")]
    debug_inspector: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        AppSettings {
            plex_url: "http://localhost:32400".to_string(),
            plex_token: "".to_string(),
            accent_color: "#22c55e".to_string(),
            terminal_shortcuts: default_shortcuts(),
            weather_location: "Oslo, Norge".to_string(),
            weather_lat: Some(59.9127),
            weather_lon: Some(10.7461),
            obs_websocket_url: "".to_string(),
            obs_websocket_password: "".to_string(),
            use_24h: true,
            reserve_screen_space: true,
            hide_native_taskbar: false,
            debug_inspector: false,
        }
    }
}

// Shared settings type
type SharedSettings = Arc<Mutex<AppSettings>>;

struct TerminalState {
    writer: Arc<Mutex<Option<Box<dyn Write + Send>>>>,
    master: Arc<Mutex<Option<Box<dyn MasterPty + Send>>>>,
}

fn get_settings_path(handle: &tauri::AppHandle) -> PathBuf {
    let mut path = handle.path().home_dir().unwrap_or_else(|_| PathBuf::from("."));
    path.push(".aeropeks");
    
    if !path.exists() {
        let _ = fs::create_dir_all(&path);
    }
    
    path.push("settings.json");
    path
}

fn migrate_settings(handle: &tauri::AppHandle, new_path: &std::path::Path) {
    if new_path.exists() {
        return;
    }

    if let Ok(old_dir) = handle.path().app_data_dir() {
        let old_path = old_dir.join("settings.json");
        if old_path.exists() {
            println!("DEBUG: Migrating settings from {:?} to {:?}", old_path, new_path);
            if let Err(e) = fs::rename(&old_path, new_path) {
                eprintln!("DEBUG ERROR: Migration failed: {}", e);
            }
        }
    }
}

fn fetch_settings_helper(handle: tauri::AppHandle) -> AppSettings {
    let path = get_settings_path(&handle);
    migrate_settings(&handle, &path);
    
    println!("DEBUG: Checking settings file at {:?}. Exists: {}", path, path.exists());
    if !path.exists() {
        return AppSettings::default();
    }

    match fs::read_to_string(&path) {
        Ok(content) => {
            match serde_json::from_str::<AppSettings>(&content) {
                Ok(settings) => return settings,
                Err(e) => {
                    eprintln!("DEBUG ERROR: Critical parse failure for settings at {:?}: {}.", path, e);
                    // Create a backup of the corrupted file before falling back
                    let mut backup_path = path.clone();
                    backup_path.set_extension("json.error");
                    if let Err(be) = fs::rename(&path, &backup_path) {
                        eprintln!("DEBUG ERROR: Failed to backup corrupted settings: {}", be);
                    } else {
                        eprintln!("DEBUG: Corrupted settings backed up to {:?}", backup_path);
                    }
                    return AppSettings::default();
                }
            }
        },
        Err(e) => {
            eprintln!("DEBUG ERROR: Failed to read settings file: {}", e);
            AppSettings::default()
        },
    }
}

#[tauri::command]
fn show_terminal_context_menu(window: tauri::Window, state: tauri::State<'_, SharedSettings>) -> Result<(), String> {
    let handle = window.app_handle();
    let settings = state.lock().map_err(|e| e.to_string())?;
    
    let mut items: Vec<Box<dyn tauri::menu::IsMenuItem<tauri::Wry>>> = Vec::new();
    
    for (i, shortcut) in settings.terminal_shortcuts.iter().enumerate() {
        if i > 0 && shortcut.id.contains("git") && !settings.terminal_shortcuts[i-1].id.contains("git") {
            if let Ok(sep) = PredefinedMenuItem::separator(handle) {
                items.push(Box::new(sep));
            }
        }
        
        let item = MenuItem::with_id(handle, &shortcut.id, &shortcut.label, true, None::<&str>).map_err(|e| e.to_string())?;
        items.push(Box::new(item));
    }
    
    // Convert Vec<Box<dyn IsMenuItem>> to Vec<&dyn IsMenuItem> for with_items
    let item_refs: Vec<&dyn tauri::menu::IsMenuItem<tauri::Wry>> = items.iter().map(|b| b.as_ref()).collect();
    let menu = Menu::with_items(handle, &item_refs).map_err(|e| e.to_string())?;

    window.popup_menu(&menu).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn save_settings(settings: AppSettings, handle: tauri::AppHandle, state: tauri::State<'_, SharedSettings>) -> Result<(), String> {
    println!("DEBUG: Saving settings. Plex URL: {}, Token Length: {} chars", settings.plex_url, settings.plex_token.len());
    let path = get_settings_path(&handle);
    let content = serde_json::to_string(&settings).map_err(|e| e.to_string())?;
    fs::write(path, content).map_err(|e| e.to_string())?;
    
    let mut current_settings = state.lock().map_err(|e| e.to_string())?;
    let previous_settings = current_settings.clone();
    *current_settings = settings.clone();
    handle.emit("settings-changed", current_settings.clone()).ok();
    drop(current_settings);

    if previous_settings.hide_native_taskbar != settings.hide_native_taskbar {
        set_native_taskbar_visible(!settings.hide_native_taskbar);
    }

    if previous_settings.reserve_screen_space && !settings.reserve_screen_space {
        let _ = restore_shell_state_internal(&handle);
    }

    Ok(())
}

#[tauri::command]
fn set_window_height(window: tauri::Window, height: u32) -> Result<(), String> {
    let size = tauri::Size::Physical(tauri::PhysicalSize {
        width: window.inner_size().unwrap_or(tauri::PhysicalSize { width: 1920, height: 32 }).width,
        height,
    });
    window.set_size(size).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn get_settings(state: tauri::State<'_, SharedSettings>) -> Result<AppSettings, String> {
    let settings = state.lock().map_err(|e| e.to_string())?;
    Ok(settings.clone())
}

// ── Media Info ──────────────────────────────────────────────────────

#[derive(Serialize, Deserialize, Clone, Debug)]
struct MediaInfo {
    title: String,
    artist: String,
    album: String,
    is_playing: bool,
    thumbnail: Option<String>,
    duration_ms: u64,
    view_offset_ms: u64,
    source: String,
    session_id: Option<String>,
    machine_id: Option<String>,
    address: Option<String>,
}

async fn fetch_plex_media(plex_url: &str, plex_token: &str) -> Option<MediaInfo> {
    if plex_url.is_empty() { return None; }
    println!("DEBUG: fetch_plex_media checking: {}", plex_url);
    
    let url = format!("{}/status/sessions?X-Plex-Token={}", plex_url.trim_end_matches('/'), plex_token);
    // println!("Plex DEBUG: Fetching from server...");
    
    let client = reqwest::Client::builder().timeout(Duration::from_secs(3)).build().ok()?;
    let resp = match client.get(&url).header("Accept", "application/json").send().await {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Plex DEBUG: Request failed: {}", e);
            return None;
        }
    };
    
    if !resp.status().is_success() {
        eprintln!("Plex DEBUG: API returned status {}", resp.status());
        return None;
    }

    let json: serde_json::Value = match resp.json().await {
        Ok(j) => j,
        Err(e) => {
            eprintln!("Plex DEBUG: JSON parse error: {}", e);
            return None;
        }
    };
    
    let container = json.get("MediaContainer")?;
    let metadata = container.get("Metadata")?.as_array()?;

    // println!("Plex DEBUG: Found {} active sessions", metadata.len());

    for item in metadata {
        let title = item.get("title").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let artist = item.get("grandparentTitle").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let album = item.get("parentTitle").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let thumb = item.get("thumb").and_then(|v| v.as_str()).map(|v| v.to_string());
        let duration_ms = item.get("duration").and_then(|v| v.as_u64()).unwrap_or(0);
        let view_offset_ms = item.get("viewOffset").and_then(|v| v.as_u64()).unwrap_or(0);
        let session_id = item.get("sessionKey").and_then(|v| v.as_str()).map(|v| v.to_string());

        let player = item.get("Player").and_then(|v| v.as_object());
        let machine_id = player.and_then(|p| p.get("machineIdentifier")).and_then(|v| v.as_str()).map(|v| v.to_string());
        let address = player.and_then(|p| p.get("address")).and_then(|v| v.as_str()).map(|v| v.to_string());
        let state = player.and_then(|p| p.get("state")).and_then(|v| v.as_str()).unwrap_or("");

        if !title.is_empty() {
            // println!("Plex DEBUG: Session '{}' - {}", title, state);
            return Some(MediaInfo {
                title,
                artist,
                album,
                thumbnail: thumb,
                duration_ms,
                view_offset_ms,
                is_playing: state == "playing",
                source: "plex".to_string(),
                session_id,
                machine_id,
                address,
            });
        }
    }
    None
}

#[tauri::command]
fn get_album_art(thumb: String, state: tauri::State<'_, SharedSettings>) -> Result<String, String> {
    if thumb.is_empty() { return Ok(String::new()); }
    let settings = state.lock().map_err(|e| e.to_string())?;
    let url = format!("{}/photo/:/transcode?url={}&width=200&height=200&X-Plex-Token={}",
        settings.plex_url.trim_end_matches('/'),
        urlencoding::encode(&thumb),
        settings.plex_token
    );
    let client = reqwest::blocking::Client::builder().timeout(Duration::from_secs(5)).build().map_err(|e| e.to_string())?;
    let resp = client.get(&url).send().map_err(|e| e.to_string())?;
    let bytes = resp.bytes().map_err(|e| e.to_string())?;
    use base64::Engine;
    Ok(base64::engine::general_purpose::STANDARD.encode(&bytes))
}

#[tauri::command]
async fn plex_control(command: String, _session_id: String, machine_id: String, address: String, state: tauri::State<'_, SharedSettings>) -> Result<(), String> {
    let (plex_url, plex_token) = {
        let settings = state.lock().map_err(|e| e.to_string())?;
        (settings.plex_url.clone(), settings.plex_token.clone())
    };
    
    let plex_command = match command.as_str() {
        "play" | "play_pause" => "play",
        "pause" => "pause",
        "next" => "skipNext",
        "previous" | "prev" => "skipPrevious",
        _ => return Err("Invalid command".to_string()),
    };

    let client = reqwest::Client::builder().timeout(Duration::from_secs(3)).build().unwrap_or_default();

    let command_id = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(1);

    let mut attempts: Vec<(String, &str)> = vec![
        (format!("{}/player/proxy/playback/{}?X-Plex-Target-Client-Identifier={}&X-Plex-Token={}&commandID={}",
            plex_url.trim_end_matches('/'), plex_command, machine_id, plex_token, command_id), "GET"),
    ];

    if !address.is_empty() {
        attempts.push((format!("http://{}:32500/player/playback/{}?commandID={}&X-Plex-Token={}",
            address, plex_command, command_id, plex_token), "POST"));
        attempts.push((format!("http://{}:32500/player/playback/{}?commandID={}&X-Plex-Token={}",
            address, plex_command, command_id, plex_token), "GET"));
        attempts.push((format!("{}/system/players/{}/playback/{}?X-Plex-Token={}",
            plex_url.trim_end_matches('/'), address, plex_command, plex_token), "GET"));
    }

    for (url, method) in &attempts {
        let req = if *method == "POST" { client.post(url) } else { client.get(url) };
        if let Ok(resp) = req
            .header("X-Plex-Client-Identifier", "aeropeks")
            .header("X-Plex-Target-Client-Identifier", &machine_id)
            .header("X-Plex-Token", &plex_token)
            .send()
            .await 
        {
            if resp.status().is_success() {
                return Ok(());
            }
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

// ── Launcher ────────────────────────────────────────────────────────

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SearchResult {
    id: String,
    title: String,
    description: String,
    icon: String,      // Base64 or icon name
    action_type: String, // 'app', 'web', 'system', 'cmd'
    action_value: String,
}

#[tauri::command]
fn search_query(query: String) -> Result<Vec<SearchResult>, String> {
    let mut results = Vec::new();
    let query_lower = query.to_lowercase().trim().to_string();

    if query_lower.is_empty() {
        return Ok(results);
    }

    // 1. Web Search (Prefix 'g ' or 'google ')
    if query_lower.starts_with("g ") || query_lower.starts_with("google ") {
        let q = if query_lower.starts_with("g ") { &query[2..] } else { &query[7..] };
        results.push(SearchResult {
            id: "web-google".to_string(),
            title: format!("Search Google for '{}'", q),
            description: "Open in default browser".to_string(),
            icon: "Globe".to_string(),
            action_type: "web".to_string(),
            action_value: format!("https://www.google.com/search?q={}", urlencoding::encode(q)),
        });
        return Ok(results);
    }

    // Command runner
    if query_lower.starts_with(">") {
        let cmd = query[1..].trim();
        results.push(SearchResult {
            id: format!("cmd-{}", cmd),
            title: format!("Run: {}", cmd),
            description: "Execute command securely".to_string(),
            icon: "Command".to_string(),
            action_type: "cmd".to_string(),
            action_value: cmd.to_string(),
        });
        return Ok(results);
    }

    // 2. System Commands
    let sys_cmds = vec![
        ("lock", "Lock Workstation", "system", "rundll32.exe user32.dll,LockWorkStation"),
        ("shutdown", "Shut Down", "system", "shutdown /s /t 0"),
        ("restart", "Restart", "system", "shutdown /r /t 0"),
        ("sleep", "Sleep", "system", "rundll32.exe powrprof.dll,SetSuspendState 0,1,0"),
    ];

    for (cmd, label, atype, aval) in sys_cmds {
        if cmd.contains(&query_lower) {
            results.push(SearchResult {
                id: format!("sys-{}", cmd),
                title: label.to_string(),
                description: format!("Execute: {}", cmd),
                icon: "Settings".to_string(),
                action_type: atype.to_string(),
                action_value: aval.to_string(),
            });
        }
    }

    // 3. Application Search (Simple Start Menu Scan)
    let mut start_menu_paths = vec![
        "C:\\ProgramData\\Microsoft\\Windows\\Start Menu\\Programs".to_string(),
    ];
    if let Ok(profile) = std::env::var("USERPROFILE") {
        start_menu_paths.push(format!("{}\\AppData\\Roaming\\Microsoft\\Windows\\Start Menu\\Programs", profile));
    }

    for path in start_menu_paths {
        for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
            let file_name = entry.file_name().to_string_lossy();
                if file_name.to_lowercase().contains(&query_lower) && file_name.ends_with(".lnk") {
                    let name = file_name.trim_end_matches(".lnk").to_string();
                    results.push(SearchResult {
                        id: format!("app-{}", name),
                        title: name,
                        description: entry.path().to_string_lossy().to_string(),
                        icon: "AppWindow".to_string(),
                        action_type: "app".to_string(),
                        action_value: entry.path().to_string_lossy().to_string(),
                    });
                }
            }
        }

    Ok(results)
}

#[tauri::command]
fn launch_result(handle: AppHandle, result: SearchResult) -> Result<(), String> {
    match result.action_type.as_str() {
        "web" => {
            let _ = open::that(result.action_value);
        }
        "app" | "system" => {
            if result.action_value.contains(" ") && result.action_type == "system" {
                let parts: Vec<&str> = result.action_value.split_whitespace().collect();
                let cmd = parts[0];
                let args = &parts[1..];
                std::process::Command::new(cmd)
                    .args(args)
                    .creation_flags(CREATE_NO_WINDOW)
                    .spawn()
                    .map_err(|e| e.to_string())?;
            } else {
                std::process::Command::new("cmd")
                    .args(["/C", "start", "", &result.action_value])
                    .creation_flags(CREATE_NO_WINDOW)
                    .spawn()
                    .map_err(|e| e.to_string())?;
            }
        }

        "cmd" => {
            std::thread::spawn(move || {
                let _ = std::process::Command::new("cmd")
                    .args(["/C", &result.action_value])
                    .creation_flags(CREATE_NO_WINDOW)
                    .spawn();
            });
        }
        _ => return Err("Unknown action type".to_string()),
    }

    if let Some(window) = handle.get_webview_window("launcher-panel") {
        let _ = window.hide().ok();
    }
    Ok(())
}

#[tauri::command]
fn toggle_launcher(handle: tauri::AppHandle) {
    if let Some(window) = handle.get_webview_window("launcher-panel") {
        let is_visible = window.is_visible().unwrap_or(false);
        if is_visible {
            let _ = window.hide().ok();
        } else {
            let _ = window.show().ok();
            let _ = window.set_focus().ok();
        }
    }
}

// ── Power & Battery ────────────────────────────────────────────────

#[derive(Serialize)]
struct BatteryStatus {
    percentage: u8,
    is_charging: bool,
    has_battery: bool,
}

#[tauri::command]
fn get_battery_status() -> BatteryStatus {
    unsafe {
        let mut status = SYSTEM_POWER_STATUS::default();
        if GetSystemPowerStatus(&mut status).is_ok() {
            return BatteryStatus {
                percentage: status.BatteryLifePercent,
                is_charging: (status.ACLineStatus == 1),
                has_battery: status.BatteryFlag != 128,
            };
        }
    }
    BatteryStatus { percentage: 0, is_charging: false, has_battery: false }
}

#[tauri::command]
fn system_power_action(action: String) {
    match action.as_str() {
        "shutdown" => { let _ = std::process::Command::new("shutdown").args(["/s", "/t", "0"]).creation_flags(CREATE_NO_WINDOW).spawn(); },
        "restart" => { let _ = std::process::Command::new("shutdown").args(["/r", "/t", "0"]).creation_flags(CREATE_NO_WINDOW).spawn(); },
        "sleep" => { unsafe { let _ = windows::Win32::System::Power::SetSuspendState(false, false, false); } },
        "lock" => { unsafe { let _ = windows::Win32::System::Shutdown::LockWorkStation(); } },

        _ => {},
    }
}

// ── Microphone ──────────────────────────────────────────────────────

use windows::Win32::Media::Audio::{eCapture};

#[tauri::command]
fn get_mic_status() -> Result<bool, String> {
    unsafe {
        let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
        let enumerator: IMMDeviceEnumerator = CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL).map_err(|e: windows::core::Error| e.to_string())?;
        let device = enumerator.GetDefaultAudioEndpoint(eCapture, eConsole).map_err(|e: windows::core::Error| e.to_string())?;
        let volume = device.Activate::<IAudioEndpointVolume>(CLSCTX_ALL, None).map_err(|e: windows::core::Error| e.to_string())?;
        Ok(volume.GetMute().map_err(|e: windows::core::Error| e.to_string())?.as_bool())
    }
}

#[tauri::command]
fn toggle_mic_mute() -> Result<bool, String> {
    unsafe {
        let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
        let enumerator: IMMDeviceEnumerator = CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL).map_err(|e: windows::core::Error| e.to_string())?;
        let device = enumerator.GetDefaultAudioEndpoint(eCapture, eConsole).map_err(|e: windows::core::Error| e.to_string())?;
        let volume = device.Activate::<IAudioEndpointVolume>(CLSCTX_ALL, None).map_err(|e: windows::core::Error| e.to_string())?;
        let current_mute = volume.GetMute().map_err(|e: windows::core::Error| e.to_string())?;
        volume.SetMute(!current_mute, std::ptr::null()).map_err(|e: windows::core::Error| e.to_string())?;
        Ok(!current_mute.as_bool())
    }
}

#[tauri::command]
fn get_privacy_status() -> Result<bool, String> {
    let mic_muted = get_mic_status()?;
    Ok(mic_muted) // For now, we use mic as primary indicator for Privacy Mode
}

#[tauri::command]
async fn set_privacy_mode(enabled: bool) -> Result<(), String> {
    // 1. Mic
    let current_mute = get_mic_status()?;
    if (enabled && !current_mute) || (!enabled && current_mute) {
        let _ = toggle_mic_mute();
    }
    
    // 2. Camera (PowerShell)
    let cmd = if enabled { "Disable-PnpDevice" } else { "Enable-PnpDevice" };
    let _ = std::process::Command::new("powershell")
        .creation_flags(CREATE_NO_WINDOW)
        .args([
            "-NoProfile",
            "-Command",
            &format!("Get-PnpDevice -Class Camera -ErrorAction SilentlyContinue | {} -Confirm:$false", cmd)
        ])
        .status();

    
    Ok(())
}


// ── Bluetooth & Weather & Desktops ───────────────────────────────────────────

#[tauri::command]
fn get_virtual_desktop_status() -> Result<(usize, usize), String> {
    let count = winvd::get_desktop_count().unwrap_or(1) as usize;
    let current_index = match winvd::get_current_desktop() {
        Ok(desktop) => winvd::get_desktops()
            .unwrap_or_default()
            .iter()
            .position(|d| d == &desktop)
            .unwrap_or(0),
        Err(_) => 0,
    };
    Ok((count, current_index))
}

#[tauri::command]
fn switch_virtual_desktop(index: usize) -> Result<(), String> {
    if let Ok(desktops) = winvd::get_desktops() {
        if let Some(d) = desktops.get(index) {
            let _ = winvd::switch_desktop(*d);
        }
    }
    Ok(())
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct WeatherDetailed {
    temp: f32,
    symbol: String,
    precip: f32,
    place_name: String,
    hourly: Vec<HourlyForecast>,
    daily: Vec<DailyForecast>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct HourlyForecast {
    time: String,
    temp: f32,
    symbol: String,
    precip: f32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct DailyForecast {
    date: String,
    temp_min: f32,
    temp_max: f32,
    symbol: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct LocationSearchResult {
    name: String,
    lat: f64,
    lon: f64,
    country: String,
    url_path: String,
}

#[tauri::command]
async fn get_weather(lat: f64, lon: f64, place_name: String) -> Result<WeatherDetailed, String> {
    let client = reqwest::Client::builder()
        .user_agent("Aeropeks/0.1.0 (https://github.com/mortenlein/Aeropeks)")
        .build()
        .map_err(|e| e.to_string())?;

    let url = format!("https://api.met.no/weatherapi/locationforecast/2.0/compact?lat={:.4}&lon={:.4}", lat, lon);
    let resp = client.get(url).send().await.map_err(|e| e.to_string())?;
    let json: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;

    let properties = json.get("properties").ok_or("Invalid forecast data")?;
    let timeseries = properties.get("timeseries").and_then(|t| t.as_array()).ok_or("No timeseries data")?;

    // Current
    let latest = timeseries.get(0).ok_or("No current data")?;
    let instant_data = latest.get("data").and_then(|d| d.get("instant")).and_then(|i| i.get("details")).ok_or("No instant details")?;
    let temp = instant_data.get("air_temperature").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;
    
    let next_1h = latest.get("data").and_then(|d| d.get("next_1_hours"));
    let symbol = next_1h.and_then(|n| n.get("summary")).and_then(|s| s.get("symbol_code")).and_then(|v| v.as_str()).unwrap_or("clearsky_day").to_string();
    let precip = next_1h.and_then(|n| n.get("details")).and_then(|d| d.get("precipitation_amount")).and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;

    // Hourly (next 24 hours)
    let mut hourly = Vec::new();
    for i in 0..24.min(timeseries.len()) {
        let entry = &timeseries[i];
        let time = entry.get("time").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let details = entry.get("data").and_then(|d| d.get("instant")).and_then(|i| i.get("details")).ok_or("No details")?;
        let h_temp = details.get("air_temperature").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;
        
        let h_next_1h = entry.get("data").and_then(|d| d.get("next_1_hours"));
        let h_symbol = h_next_1h.and_then(|n| n.get("summary")).and_then(|s| s.get("symbol_code")).and_then(|v| v.as_str()).unwrap_or("").to_string();
        let h_precip = h_next_1h.and_then(|n| n.get("details")).and_then(|d| d.get("precipitation_amount")).and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;

        hourly.push(HourlyForecast { time, temp: h_temp, symbol: h_symbol, precip: h_precip });
    }

    // Daily (simple group by day)
    let mut daily = Vec::new();
    let mut current_day = String::new();
    let mut d_min = 100.0;
    let mut d_max = -100.0;
    let mut d_symbol = String::new();

    for entry in timeseries {
        let time = entry.get("time").and_then(|v| v.as_str()).unwrap_or("");
        let day = time.split('T').next().unwrap_or("");
        
        if day != current_day {
            if !current_day.is_empty() {
                daily.push(DailyForecast { 
                    date: current_day.clone(), 
                    temp_min: d_min as f32, 
                    temp_max: d_max as f32, 
                    symbol: d_symbol.clone() 
                });
            }
            current_day = day.to_string();
            d_min = 100.0;
            d_max = -100.0;
            d_symbol = String::new();
        }

        let details = entry.get("data").and_then(|d| d.get("instant")).and_then(|i| i.get("details")).ok_or("No details")?;
        let t = details.get("air_temperature").and_then(|v| v.as_f64()).unwrap_or(0.0);
        if t < d_min { d_min = t; }
        if t > d_max { d_max = t; }

        if d_symbol.is_empty() {
            let next_6h = entry.get("data").and_then(|d| d.get("next_6_hours"));
            d_symbol = next_6h.and_then(|n| n.get("summary")).and_then(|s| s.get("symbol_code")).and_then(|v| v.as_str()).unwrap_or("").to_string();
        }
        
        if daily.len() >= 7 { break; }
    }

    Ok(WeatherDetailed {
        temp,
        symbol,
        precip,
        place_name,
        hourly,
        daily,
    })
}

#[tauri::command]
async fn search_locations(query: String) -> Result<Vec<LocationSearchResult>, String> {
    let client = reqwest::Client::builder()
        .user_agent("Aeropeks/0.1.0 (https://github.com/mortenlein/Aeropeks)")
        .build()
        .map_err(|e| e.to_string())?;

    let url = format!("https://www.yr.no/api/v0/locations/suggest?q={}", urlencoding::encode(&query));
    let resp = client.get(url).send().await.map_err(|e| e.to_string())?;
    let json: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;

    let mut results = Vec::new();
    if let Some(locations) = json.get("_embedded").and_then(|e| e.get("location")).and_then(|l| l.as_array()) {
        for loc in locations {
            let name = loc.get("name").and_then(|v| v.as_str()).unwrap_or("Unknown").to_string();
            let pos = loc.get("position").ok_or("No position")?;
            let lat = pos.get("lat").and_then(|v| v.as_f64()).unwrap_or(0.0);
            let lon = pos.get("lon").and_then(|v| v.as_f64()).unwrap_or(0.0);
            let country = loc.get("country").and_then(|c| c.get("name")).and_then(|v| v.as_str()).unwrap_or("").to_string();
            let url_path = loc.get("urlPath").and_then(|v| v.as_str()).unwrap_or("").to_string();

            results.push(LocationSearchResult { name, lat, lon, country, url_path });
        }
    }

    Ok(results)
}

#[derive(Debug, serde::Serialize, Clone, Default)]
struct BluetoothStatus {
    connected: bool,
    devices: Vec<String>,
}

#[tauri::command]
fn get_bluetooth_status() -> BluetoothStatus {
    let mut status = BluetoothStatus::default();
    
    // Improved PowerShell command:
    // - Targets BTHENUM which are typically paired/connected devices.
    // - Excludes system-facing names like 'Transport', 'Service', 'Enumerator', 'Gateway', etc.
    let output = std::process::Command::new("powershell")
        .creation_flags(CREATE_NO_WINDOW)
        .args([
            "-NoProfile",
            "-Command",
            "Get-PnpDevice -Class Bluetooth | Where-Object { $_.Status -eq 'OK' -and $_.Present -eq $true -and $_.InstanceId -like 'BTHENUM*' -and $_.FriendlyName -notmatch 'Service|Transport|Enumerator|Gateway|Radio|Adapter|Controller|Generic' } | Select-Object -ExpandProperty FriendlyName"
        ])
        .output();


    if let Ok(out) = output {
        let stdout = String::from_utf8_lossy(&out.stdout);
        let devices: Vec<String> = stdout
            .lines()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        
        status.connected = !devices.is_empty();
        status.devices = devices;
    }
    
    status
}

#[derive(Serialize)]
struct ObsStatus {
    is_streaming: bool,
    is_recording: bool,
}

#[tauri::command]
async fn get_obs_status(handle: tauri::AppHandle) -> Result<ObsStatus, String> {
    let settings = {
        let state = handle.state::<SharedSettings>();
        let lock = state.lock().map_err(|e| e.to_string())?;
        lock.clone()
    };

    if settings.obs_websocket_url.is_empty() {
        return Ok(ObsStatus { is_streaming: false, is_recording: false });
    }

    let (host, port) = if let Some(stripped) = settings.obs_websocket_url.strip_prefix("ws://") {
        let parts: Vec<&str> = stripped.split(':').collect();
        (parts[0].to_string(), parts.get(1).and_then(|p| p.parse().ok()).unwrap_or(4455))
    } else {
        (settings.obs_websocket_url.clone(), 4455)
    };

    // Connect with a timeout
    let client = tokio::time::timeout(
        std::time::Duration::from_millis(500),
        obws::Client::connect(host, port, Some(&settings.obs_websocket_password))
    ).await.map_err(|_| "OBS connection timeout".to_string())?.map_err(|e| e.to_string())?;
    
    let stream = client.streaming().status().await.map_err(|e| e.to_string())?;
    let record = client.recording().status().await.map_err(|e| e.to_string())?;

    Ok(ObsStatus {
        is_streaming: stream.active,
        is_recording: record.active,
    })
}


// ── Media (GSMTC) ───────────────────────────────────────────────────

// ── Media (GSMTC) ───────────────────────────────────────────────────

// MediaUpdate removed, using MediaInfo unified

async fn get_gsmtc_media_internal() -> Result<Option<MediaInfo>, String> {
    use windows::Media::Control::{
        GlobalSystemMediaTransportControlsSessionManager,
        GlobalSystemMediaTransportControlsSession,
        GlobalSystemMediaTransportControlsSessionPlaybackInfo,
    };
    
    let manager = GlobalSystemMediaTransportControlsSessionManager::RequestAsync()
        .map_err(|e: windows::core::Error| e.to_string())?
        .get()
        .map_err(|e: windows::core::Error| e.to_string())?;
    
    let sessions = manager.GetSessions().map_err(|e: windows::core::Error| e.to_string())?;
    
    let mut best_session: Option<GlobalSystemMediaTransportControlsSession> = None;
    let mut best_status = -1;
    
    for session in sessions {
        let session: GlobalSystemMediaTransportControlsSession = session;
        if let Ok(playback) = session.GetPlaybackInfo() {
            let playback: GlobalSystemMediaTransportControlsSessionPlaybackInfo = playback;
            let status = playback.PlaybackStatus().unwrap_or_default().0;
            let score = match status {
                4 => 10,
                5 => 5,
                _ => 0,
            };
            
            if score > best_status {
                best_status = score;
                best_session = Some(session);
            }
        }
    }
    
    if let Some(session) = best_session {
        let playback: GlobalSystemMediaTransportControlsSessionPlaybackInfo = 
            session.GetPlaybackInfo().map_err(|e: windows::core::Error| e.to_string())?;
        let media = session.TryGetMediaPropertiesAsync()
            .map_err(|e: windows::core::Error| e.to_string())?
            .get()
            .map_err(|e: windows::core::Error| e.to_string())?;
        
        Ok(Some(MediaInfo {
            title: media.Title().unwrap_or_default().to_string(),
            artist: media.Artist().unwrap_or_default().to_string(),
            album: media.AlbumTitle().unwrap_or_default().to_string(),
            is_playing: playback.PlaybackStatus().unwrap_or_default().0 == 4,
            thumbnail: None,
            duration_ms: 0,
            view_offset_ms: 0,
            source: "gsmtc".to_string(),
            session_id: None,
            machine_id: None,
            address: None,
        }))
    } else {
        Ok(None)
    }
}

async fn get_active_media_internal(handle: tauri::AppHandle) -> Result<Option<MediaInfo>, String> {
    // println!("DEBUG: get_active_media_internal checking sources...");
    // 1. Try GSMTC first (Local system media)
    if let Ok(Some(gsmtc)) = get_gsmtc_media_internal().await {
        if gsmtc.is_playing {
            return Ok(Some(gsmtc));
        }
        
        // If GSMTC is paused, check if Plex is playing
        let (plex_url, plex_token) = {
            let state = handle.state::<SharedSettings>();
            let lock = state.lock().map_err(|e| e.to_string())?;
            if !lock.plex_token.is_empty() {
                // println!("DEBUG: Token is present ({} chars)", lock.plex_token.len());
            } else {
                eprintln!("DEBUG WARNING: Plex token is EMPTY in backend state!");
            }
            (lock.plex_url.clone(), lock.plex_token.clone())
        };
        
        if let Some(plex) = fetch_plex_media(&plex_url, &plex_token).await {
            if plex.is_playing {
                return Ok(Some(plex));
            }
        }
        
        return Ok(Some(gsmtc));
    }
    
    // 2. Fallback to Plex API
    let (plex_url, plex_token) = {
        let state = handle.state::<SharedSettings>();
        let lock = state.lock().map_err(|e| e.to_string())?;
        (lock.plex_url.clone(), lock.plex_token.clone())
    };
    Ok(fetch_plex_media(&plex_url, &plex_token).await)
}

#[tauri::command]
async fn get_media_info_unified(handle: tauri::AppHandle) -> Result<Option<MediaInfo>, String> {
    get_active_media_internal(handle).await
}

#[tauri::command]
async fn media_control_unified(
    handle: tauri::AppHandle, 
    action: String, 
    media: Option<MediaInfo>
) -> Result<(), String> {
    if let Some(m) = media {
        if m.source == "plex" {
            let session_id = m.session_id.clone().unwrap_or_default();
            let machine_id = m.machine_id.clone().unwrap_or_default();
            let address = m.address.clone().unwrap_or_default();
            return plex_control(action, session_id, machine_id, address, handle.state()).await;
        } else {
            return gsmtc_action(action).await;
        }
    }
    Ok(())
}
#[tauri::command]
async fn gsmtc_action(action: String) -> Result<(), String> {
    use windows::Media::Control::GlobalSystemMediaTransportControlsSessionManager;
    let manager = GlobalSystemMediaTransportControlsSessionManager::RequestAsync().map_err(|e: windows::core::Error| e.to_string())?.get().map_err(|e: windows::core::Error| e.to_string())?;
    let session = manager.GetCurrentSession();
    if let Ok(session) = session {
        match action.as_str() {
            "play_pause" => { let _ = session.TryTogglePlayPauseAsync().map_err(|e: windows::core::Error| e.to_string())?.get(); },
            "next" => { let _ = session.TrySkipNextAsync().map_err(|e: windows::core::Error| e.to_string())?.get(); },
            "previous" => { let _ = session.TrySkipPreviousAsync().map_err(|e: windows::core::Error| e.to_string())?.get(); },
            _ => {}
        }
    }
    Ok(())
}

#[tauri::command]
fn register_hotkeys(app: AppHandle, settings: State<'_, SharedSettings>) -> Result<(), String> {
    use tauri_plugin_global_shortcut::GlobalShortcutExt;
    let _ = app.global_shortcut().unregister_all();
    
    // Re-register launcher
    use tauri_plugin_global_shortcut::{Shortcut, Modifiers, Code};
    let launcher_shortcut = Shortcut::new(Some(Modifiers::ALT), Code::Space);
    let _ = app.global_shortcut().register(launcher_shortcut);

    let shortcuts = {
        let lock = settings.lock().map_err(|e| e.to_string())?;
        lock.terminal_shortcuts.clone()
    };

    for s in shortcuts {
        if let Ok(shortcut) = s.shortcut.parse::<Shortcut>() {
             let _ = app.global_shortcut().register(shortcut);
        }
    }
    Ok(())
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
            let _ = window.set_shadow(false).ok();
            let _ = window.set_focus().ok();
        }
    }
}

#[tauri::command]
fn toggle_terminal_panel(handle: tauri::AppHandle) {
    if let Some(window) = handle.get_webview_window("terminal-panel") {
        let is_visible = window.is_visible().unwrap_or(false);
        if is_visible {
            let _ = window.hide().ok();
        } else {
            if let Ok(Some(monitor)) = handle.primary_monitor() {
                let m_size = monitor.size();
                let width = 860;
                let height = 460;
                let x = m_size.width as i32 - width - 12;
                let y = 36;
                let _ = window.set_size(tauri::Size::Physical(tauri::PhysicalSize { width: width as u32, height: height as u32 })).ok();
                let _ = window.set_position(tauri::Position::Physical(tauri::PhysicalPosition { x, y })).ok();
            }
            let _ = window.show().ok();
            let _ = window.set_shadow(false).ok();
            let _ = window.set_focus().ok();
        }
    }
}

#[tauri::command]
fn start_pty(rows: u16, cols: u16, args: Option<Vec<String>>, state: tauri::State<'_, TerminalState>, window: Window) -> Result<(), String> {
    // Kill existing master if any to allow restart
    {
        let mut m = state.master.lock().unwrap();
        *m = None;
        let mut w = state.writer.lock().unwrap();
        *w = None;
    }

    let pty_system = NativePtySystem::default();
    let pair = pty_system.openpty(PtySize {
        rows,
        cols,
        pixel_width: 0,
        pixel_height: 0,
    }).map_err(|e| e.to_string())?;

    let shell_path = "C:\\Program Files\\Git\\bin\\bash.exe";
    let mut cmd = CommandBuilder::new(shell_path);
    
    if let Some(cmd_args) = args.filter(|a| !a.is_empty()) {
        // Use --login to ensure profile is loaded, and -c to run the command
        cmd.arg("--login");
        cmd.arg("-c");
        // Join args with spaces to form the full command string for bash -c
        cmd.arg(cmd_args.join(" "));
    } else {
        // Default to interactive login shell
        cmd.arg("--login");
        cmd.arg("-i");
    }

    let child = pair.slave.spawn_command(cmd).map_err(|e| e.to_string())?;
    
    thread::spawn(move || {
        let mut child = child;
        let _ = child.wait();
    });

    let reader = pair.master.try_clone_reader().map_err(|e| e.to_string())?;
    let writer = pair.master.take_writer().map_err(|e| e.to_string())?;

    {
        let mut w = state.writer.lock().unwrap();
        *w = Some(writer);
        let mut m = state.master.lock().unwrap();
        *m = Some(pair.master);
    }

    let _ = window.emit("pty-ready", "OK");
    let _ = window.app_handle().emit_to("terminal-panel", "pty-ready-global", "OK");

    let app_handle_hb = window.app_handle().clone();
    thread::spawn(move || {
        for _ in 0..15 {
            thread::sleep(Duration::from_secs(1));
            let _ = app_handle_hb.emit_to("terminal-panel", "pty-heartbeat", "HB");
        }
    });

    let app_handle = window.app_handle().clone();
    thread::spawn(move || {
        let mut reader = reader;
        let mut buffer = [0u8; 1024];
        loop {
            match reader.read(&mut buffer) {
                Ok(0) => { 
                    let _ = app_handle.emit_to("terminal-panel", "pty-exit", "EOF");
                    break; 
                }
                Ok(n) => {
                    let data = &buffer[..n];
                    let b64 = general_purpose::STANDARD.encode(data);
                    let _ = app_handle.emit_to("terminal-panel", "pty-data", PtyPayload { data: b64 });
                }
                Err(e) => { 
                    println!("ERROR: PTY read error: {}", e); 
                    let _ = app_handle.emit_to("terminal-panel", "pty-exit", format!("Error: {}", e));
                    break; 
                }
            }
        }
    });

    Ok(())
}

#[tauri::command]
fn kill_pty(state: tauri::State<'_, TerminalState>) -> Result<(), String> {
    let mut m = state.master.lock().unwrap();
    *m = None;
    let mut w = state.writer.lock().unwrap();
    *w = None;
    Ok(())
}

fn bitmap_handle_as_base64(h_bitmap: HBITMAP) -> Option<String> {
    unsafe {
        if h_bitmap.is_invalid() {
            return None;
        }

        let mut bitmap = BITMAP::default();
        if GetObjectW(
            h_bitmap,
            std::mem::size_of::<BITMAP>() as i32,
            Some(&mut bitmap as *mut _ as *mut _),
        ) == 0 {
            return None;
        }

        let width = bitmap.bmWidth;
        let height = bitmap.bmHeight.abs();
        if width <= 0 || height <= 0 {
            return None;
        }

        let hdc_screen = CreateCompatibleDC(None);
        let mut bitmap_info = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: width,
                biHeight: -height,
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB.0 as u32,
                ..Default::default()
            },
            ..Default::default()
        };

        let mut buffer = vec![0u8; (width * height * 4) as usize];

        if GetDIBits(hdc_screen, h_bitmap, 0, height as u32, Some(buffer.as_mut_ptr() as *mut _), &mut bitmap_info, DIB_RGB_COLORS) == 0 {
            let _ = DeleteDC(hdc_screen);
            return None;
        }

        // We keep it as BGRA because BMP format expects BGRA/BGR pixel ordering.
        let _ = DeleteDC(hdc_screen);

        // Simple BMP header + data encoding or just raw for now?
        // Let's use a simple BMP header to make it a valid Image data URI
        let file_size = 54 + buffer.len() as u32;
        let mut bmp_file = Vec::with_capacity(file_size as usize);
        
        // File Header
        bmp_file.extend_from_slice(b"BM");
        bmp_file.extend_from_slice(&file_size.to_le_bytes());
        bmp_file.extend_from_slice(&[0, 0, 0, 0]); // Reserved
        bmp_file.extend_from_slice(&54u32.to_le_bytes()); // Offset
        
        // Info Header
        bmp_file.extend_from_slice(&40u32.to_le_bytes()); // Size
        bmp_file.extend_from_slice(&width.to_le_bytes());
        bmp_file.extend_from_slice(&(-height).to_le_bytes()); // Top-down
        bmp_file.extend_from_slice(&1u16.to_le_bytes()); // Planes
        bmp_file.extend_from_slice(&32u16.to_le_bytes()); // BitCount
        bmp_file.extend_from_slice(&0u32.to_le_bytes()); // Compression (BI_RGB)
        bmp_file.extend_from_slice(&(buffer.len() as u32).to_le_bytes());
        bmp_file.extend_from_slice(&0u32.to_le_bytes()); // XPels
        bmp_file.extend_from_slice(&0u32.to_le_bytes()); // YPels
        bmp_file.extend_from_slice(&0u32.to_le_bytes()); // Colors
        bmp_file.extend_from_slice(&0u32.to_le_bytes()); // Important

        bmp_file.extend_from_slice(&buffer);

        use base64::Engine;
        Some(format!("data:image/bmp;base64,{}", base64::engine::general_purpose::STANDARD.encode(&bmp_file)))
    }
}

fn icon_handle_as_base64(h_icon: HICON) -> Option<String> {
    unsafe {
        if h_icon.is_invalid() {
            return None;
        }

        let mut icon_info = windows::Win32::UI::WindowsAndMessaging::ICONINFO::default();
        if GetIconInfo(h_icon, &mut icon_info).is_err() {
            return None;
        }

        let icon = bitmap_handle_as_base64(icon_info.hbmColor);
        let _ = DeleteObject(icon_info.hbmColor);
        let _ = DeleteObject(icon_info.hbmMask);
        icon
    }
}

fn extract_icon_as_base64(path: &str) -> Option<String> {
    unsafe {
        let mut path_u16: Vec<u16> = path.encode_utf16().collect();
        path_u16.push(0);
        
        let mut sfi = SHFILEINFOW::default();
        let h_success = SHGetFileInfoW(
            windows::core::PCWSTR(path_u16.as_ptr()),
            windows::Win32::Storage::FileSystem::FILE_FLAGS_AND_ATTRIBUTES(0),
            Some(&mut sfi as *mut _ as *mut _),
            std::mem::size_of::<SHFILEINFOW>() as u32,
            SHGFI_ICON | SHGFI_LARGEICON
        );

        if h_success == 0 || sfi.hIcon.is_invalid() {
            return None;
        }

        let icon = icon_handle_as_base64(sfi.hIcon);
        let _ = DestroyIcon(sfi.hIcon);
        icon
    }
}

fn pwstr_to_string(ptr: *mut u16) -> Option<String> {
    if ptr.is_null() {
        return None;
    }

    unsafe {
        let mut len = 0;
        while *ptr.add(len) != 0 {
            len += 1;
        }

        Some(String::from_utf16_lossy(std::slice::from_raw_parts(ptr, len)))
    }
}

fn window_property_string(hwnd: HWND, key: &PROPERTYKEY) -> Option<String> {
    unsafe {
        let store: IPropertyStore = SHGetPropertyStoreForWindow(hwnd).ok()?;
        let mut value = store.GetValue(key as *const _).ok()?;
        let raw = PropVariantToStringAlloc(&value).ok()?;
        let text = pwstr_to_string(raw.0).map(|s| s.trim().to_string());

        CoTaskMemFree(Some(raw.0 as *const _));
        let _ = PropVariantClear(&mut value);

        text.filter(|s| !s.is_empty())
    }
}

fn extract_apps_folder_icon_as_base64(app_id: &str) -> Option<String> {
    unsafe {
        let parsing_name = format!("shell:AppsFolder\\{}", app_id);
        let mut parsing_name_u16: Vec<u16> = parsing_name.encode_utf16().collect();
        parsing_name_u16.push(0);

        let item: IShellItemImageFactory = SHCreateItemFromParsingName(
            windows::core::PCWSTR(parsing_name_u16.as_ptr()),
            None::<&IBindCtx>,
        ).ok()?;

        let bitmap = item.GetImage(SIZE { cx: 32, cy: 32 }, SIIGBF_BIGGERSIZEOK).ok()?;
        let icon = bitmap_handle_as_base64(bitmap);
        let _ = DeleteObject(bitmap);
        icon
    }
}

fn parse_icon_resource(resource: &str) -> Option<(String, i32)> {
    let trimmed = resource.trim().trim_matches('"').trim_start_matches('@').trim();
    let (path, index) = match trimmed.rsplit_once(',') {
        Some((path, index)) => {
            let parsed_index = index.trim().parse::<i32>().unwrap_or(0);
            (path.trim().trim_matches('"'), parsed_index)
        }
        None => (trimmed, 0),
    };

    if path.is_empty() {
        None
    } else {
        Some((path.to_string(), index))
    }
}

fn extract_icon_resource_as_base64(resource: &str) -> Option<String> {
    unsafe {
        let (path, index) = parse_icon_resource(resource)?;
        let mut path_u16: Vec<u16> = path.encode_utf16().collect();
        path_u16.push(0);

        let mut large_icon = HICON::default();
        if ExtractIconExW(
            windows::core::PCWSTR(path_u16.as_ptr()),
            index,
            Some(&mut large_icon),
            None,
            1,
        ) == 0 || large_icon.is_invalid() {
            return None;
        }

        let icon = icon_handle_as_base64(large_icon);
        let _ = DestroyIcon(large_icon);
        icon
    }
}

fn taskbar_identity_for_window(hwnd: HWND, process_path: Option<&str>) -> String {
    if let Some(app_id) = window_property_string(hwnd, &PKEY_AppUserModel_ID) {
        return format!("app-id:{}", app_id);
    }

    if let Some(relaunch_command) = window_property_string(hwnd, &PKEY_AppUserModel_RelaunchCommand) {
        return format!("relaunch-command:{}", relaunch_command);
    }

    process_path
        .map(|path| format!("path:{}", path))
        .unwrap_or_else(|| format!("hwnd:{}", hwnd.0 as isize))
}

#[derive(Serialize, Clone, Debug)]
struct ResolvedIcon {
    data_uri: String,
    source: String,
}

fn extract_taskbar_icon(hwnd: HWND, process_path: Option<&str>) -> Option<ResolvedIcon> {
    let relaunch_icon = window_property_string(hwnd, &PKEY_AppUserModel_RelaunchIconResource);
    let app_id = window_property_string(hwnd, &PKEY_AppUserModel_ID);

    if let Some(icon) = relaunch_icon
        .as_deref()
        .and_then(extract_icon_resource_as_base64) {
        return Some(ResolvedIcon { data_uri: icon, source: "relaunch-icon".to_string() });
    }

    if let Some(icon) = app_id
        .as_deref()
        .and_then(extract_apps_folder_icon_as_base64) {
        return Some(ResolvedIcon { data_uri: icon, source: "aumid-apps-folder".to_string() });
    }

    if let Some(icon) = process_path.and_then(extract_icon_as_base64) {
        return Some(ResolvedIcon { data_uri: icon, source: "process-exe".to_string() });
    }

    if let Some(icon) = extract_window_icon_as_base64(hwnd) {
        return Some(ResolvedIcon { data_uri: icon, source: "hwnd-window-icon".to_string() });
    }

    None
}

fn extract_window_icon_as_base64(hwnd: HWND) -> Option<String> {
    unsafe {
        let icon_candidates = [
            query_window_icon(hwnd, ICON_BIG as usize),
            query_window_icon(hwnd, ICON_SMALL2 as usize),
            query_window_icon(hwnd, ICON_SMALL as usize),
            HICON(GetClassLongPtrW(hwnd, GCLP_HICON) as *mut _),
            HICON(GetClassLongPtrW(hwnd, GCLP_HICONSM) as *mut _),
        ];

        icon_candidates
            .into_iter()
            .filter(|icon| !icon.is_invalid())
            .find_map(icon_handle_as_base64)
    }
}

unsafe fn query_window_icon(hwnd: HWND, icon_type: usize) -> HICON {
    let mut result = 0usize;
    let _ = SendMessageTimeoutW(
        hwnd,
        WM_GETICON,
        WPARAM(icon_type),
        LPARAM(0),
        SMTO_ABORTIFHUNG,
        100,
        Some(&mut result),
    );
    HICON(result as *mut _)
}

#[tauri::command]
fn get_window_thumbnail(hwnd: isize) -> Result<Option<String>, String> {
    unsafe {
        let hwnd = HWND(hwnd as *mut _);
        let mut info = WINDOWPLACEMENT {
            length: std::mem::size_of::<WINDOWPLACEMENT>() as u32,
            ..Default::default()
        };
        let is_minimized = if GetWindowPlacement(hwnd, &mut info).is_ok() {
            info.showCmd == SW_SHOWMINIMIZED.0 as u32 || info.showCmd == SW_MINIMIZE.0 as u32
        } else {
            false
        };

        if is_minimized {
            return Ok(None);
        }

        // Get the full window rect (includes shadow) and the DWM frame (content only)
        let mut full_rect = RECT::default();
        let _ = GetWindowRect(hwnd, &mut full_rect);

        let mut frame_rect = RECT::default();
        let has_frame = DwmGetWindowAttribute(
            hwnd,
            DWMWA_EXTENDED_FRAME_BOUNDS,
            &mut frame_rect as *mut _ as *mut _,
            std::mem::size_of::<RECT>() as u32
        ).is_ok();

        // Calculate the shadow/border offsets so we can crop them from PrintWindow output
        let (crop_left, crop_top, crop_right, crop_bottom) = if has_frame {
            (
                (frame_rect.left - full_rect.left).max(0),
                (frame_rect.top - full_rect.top).max(0),
                (full_rect.right - frame_rect.right).max(0),
                (full_rect.bottom - frame_rect.bottom).max(0),
            )
        } else {
            (0, 0, 0, 0)
        };

        let full_width = full_rect.right - full_rect.left;
        let full_height = full_rect.bottom - full_rect.top;
        if full_width <= 0 || full_height <= 0 {
            return Ok(None);
        }

        // The content region after cropping shadows
        let content_width = full_width - crop_left - crop_right;
        let content_height = full_height - crop_top - crop_bottom;
        if content_width <= 0 || content_height <= 0 {
            return Ok(None);
        }

        let max_width = 280;
        let max_height = 158;
        let scale = (max_width as f32 / content_width as f32).min(max_height as f32 / content_height as f32).min(1.0);
        let thumb_width = ((content_width as f32 * scale).round() as i32).max(1);
        let thumb_height = ((content_height as f32 * scale).round() as i32).max(1);

        let screen_dc = GetDC(None);
        if screen_dc.is_invalid() {
            return Ok(None);
        }

        let full_dc = CreateCompatibleDC(screen_dc);
        let thumb_dc = CreateCompatibleDC(screen_dc);
        let full_bitmap = CreateCompatibleBitmap(screen_dc, full_width, full_height);
        let thumb_bitmap = CreateCompatibleBitmap(screen_dc, thumb_width, thumb_height);

        if full_dc.is_invalid() || thumb_dc.is_invalid() || full_bitmap.is_invalid() || thumb_bitmap.is_invalid() {
            if !full_bitmap.is_invalid() {
                let _ = DeleteObject(full_bitmap);
            }
            if !thumb_bitmap.is_invalid() {
                let _ = DeleteObject(thumb_bitmap);
            }
            let _ = DeleteDC(full_dc);
            let _ = DeleteDC(thumb_dc);
            let _ = ReleaseDC(None, screen_dc);
            return Ok(None);
        }

        let old_full = SelectObject(full_dc, HGDIOBJ(full_bitmap.0));
        let old_thumb = SelectObject(thumb_dc, HGDIOBJ(thumb_bitmap.0));

        let printed = PrintWindow(hwnd, full_dc, PRINT_WINDOW_FLAGS(2)).as_bool();
        let thumbnail = if printed {
            let _ = SetStretchBltMode(thumb_dc, HALFTONE);
            // Copy from the content region (skipping shadow borders) into the thumbnail
            let stretched = StretchBlt(
                thumb_dc,
                0,
                0,
                thumb_width,
                thumb_height,
                full_dc,
                crop_left,
                crop_top,
                content_width,
                content_height,
                SRCCOPY,
            )
            .as_bool();

            if stretched {
                bitmap_handle_as_base64(thumb_bitmap)
            } else {
                None
            }
        } else {
            None
        };

        let _ = SelectObject(full_dc, old_full);
        let _ = SelectObject(thumb_dc, old_thumb);
        let _ = DeleteObject(full_bitmap);
        let _ = DeleteObject(thumb_bitmap);
        let _ = DeleteDC(full_dc);
        let _ = DeleteDC(thumb_dc);
        let _ = ReleaseDC(None, screen_dc);

        Ok(thumbnail)
    }
}


#[tauri::command]
fn write_pty(data: String, state: tauri::State<'_, TerminalState>) -> Result<(), String> {
    let mut writer = state.writer.lock().unwrap();
    if let Some(w) = writer.as_mut() {
        w.write_all(data.as_bytes()).map_err(|e| e.to_string())?;
        w.flush().map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
fn resize_pty(rows: u16, cols: u16, state: tauri::State<'_, TerminalState>) -> Result<(), String> {
    let master = state.master.lock().unwrap();
    if let Some(m) = master.as_ref() {
        m.resize(PtySize {
            rows,
            cols,
            pixel_width: 0,
            pixel_height: 0,
        }).map_err(|e| e.to_string())?;
    }
    Ok(())
}

// ── Taskbar/Window Management ──────────────────────────────────────
#[derive(Serialize, Clone, Debug)]
struct WindowInfo {
    hwnd: isize,
    title: String,
    app_name: String,
    is_active: bool,
    icon: Option<String>,
    icon_source: String,
    identity_key: String,
    class_name: String,
    process_path: Option<String>,
    app_id: Option<String>,
    relaunch_command: Option<String>,
    relaunch_icon: Option<String>,
    inclusion_reason: String,
}

#[derive(Serialize, Clone, Debug)]
struct IconRecord {
    data_uri: String,
    source: String,
}

#[derive(Clone)]
struct IconService {
    cache: Arc<Mutex<HashMap<String, IconRecord>>>,
    cache_dir: Arc<PathBuf>,
}

impl IconService {
    fn new() -> Self {
        let base_dir = std::env::var_os("LOCALAPPDATA")
            .map(PathBuf::from)
            .unwrap_or_else(std::env::temp_dir);
        let cache_dir = base_dir.join("Aeropeks").join("icon-cache");
        let _ = fs::create_dir_all(&cache_dir);

        Self {
            cache: Arc::new(Mutex::new(HashMap::new())),
            cache_dir: Arc::new(cache_dir),
        }
    }

    fn cache_path(&self, key: &str) -> PathBuf {
        let mut hasher = DefaultHasher::new();
        ICON_CACHE_VERSION.hash(&mut hasher);
        key.hash(&mut hasher);
        self.cache_dir.join(format!("{:x}.txt", hasher.finish()))
    }

    fn resolve<F>(&self, key: &str, resolver: F) -> (Option<String>, String)
    where
        F: FnOnce() -> Option<ResolvedIcon>,
    {
        if let Ok(cache) = self.cache.lock() {
            if let Some(record) = cache.get(key) {
                return (Some(record.data_uri.clone()), format!("memory/{}", record.source));
            }
        }

        let path = self.cache_path(key);
        if let Ok(raw) = fs::read_to_string(&path) {
            let mut parts = raw.splitn(2, '\n');
            let source = parts.next().unwrap_or("disk-cache").to_string();
            let data_uri = parts.next().unwrap_or("").trim().to_string();

            if data_uri.starts_with("data:image/") {
                let record = IconRecord { data_uri: data_uri.clone(), source: source.clone() };
                if let Ok(mut cache) = self.cache.lock() {
                    cache.insert(key.to_string(), record);
                }
                return (Some(data_uri), format!("disk/{}", source));
            }
        }

        if let Some(resolved) = resolver() {
            let record = IconRecord {
                data_uri: resolved.data_uri.clone(),
                source: resolved.source.clone(),
            };
            let _ = fs::write(&path, format!("{}\n{}", record.source, record.data_uri));
            if let Ok(mut cache) = self.cache.lock() {
                cache.insert(key.to_string(), record.clone());
            }
            return (Some(record.data_uri), record.source);
        }

        (None, "missing".to_string())
    }

    fn clear(&self) -> Result<(), String> {
        if let Ok(mut cache) = self.cache.lock() {
            cache.clear();
        }

        if self.cache_dir.exists() {
            fs::remove_dir_all(self.cache_dir.as_ref()).map_err(|e| e.to_string())?;
        }
        fs::create_dir_all(self.cache_dir.as_ref()).map_err(|e| e.to_string())?;
        Ok(())
    }
}

#[derive(Clone)]
struct WindowRegistry {
    windows: Arc<Mutex<Vec<WindowInfo>>>,
    signature: Arc<Mutex<String>>,
    order_keys: Arc<Mutex<Vec<String>>>,
}

impl WindowRegistry {
    fn new() -> Self {
        Self {
            windows: Arc::new(Mutex::new(Vec::new())),
            signature: Arc::new(Mutex::new(String::new())),
            order_keys: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn apply_stable_order(&self, windows: &mut Vec<WindowInfo>) {
        if let Ok(mut order_keys) = self.order_keys.lock() {
            let present_keys = windows
                .iter()
                .map(stable_window_order_key)
                .collect::<HashSet<_>>();

            order_keys.retain(|key| present_keys.contains(key));

            for window in windows.iter() {
                let key = stable_window_order_key(window);
                if !order_keys.contains(&key) {
                    order_keys.push(key);
                }
            }

            let positions = order_keys
                .iter()
                .enumerate()
                .map(|(index, key)| (key.clone(), index))
                .collect::<HashMap<_, _>>();

            windows.sort_by_key(|window| {
                positions
                    .get(&stable_window_order_key(window))
                    .copied()
                    .unwrap_or(usize::MAX)
            });
        }
    }
}

fn stable_window_order_key(window: &WindowInfo) -> String {
    format!("hwnd:{}", window.hwnd)
}

#[tauri::command]
fn get_open_windows(
    app: AppHandle,
    registry: tauri::State<'_, WindowRegistry>,
    icons: tauri::State<'_, IconService>,
) -> Result<Vec<WindowInfo>, String> {
    refresh_window_registry(Some(&app), &registry, &icons)
}

#[tauri::command]
fn get_window_debug_snapshot(
    app: AppHandle,
    registry: tauri::State<'_, WindowRegistry>,
    icons: tauri::State<'_, IconService>,
) -> Result<Vec<WindowInfo>, String> {
    refresh_window_registry(Some(&app), &registry, &icons)
}

#[tauri::command]
fn clear_icon_cache(icons: tauri::State<'_, IconService>) -> Result<(), String> {
    icons.clear()
}

fn refresh_window_registry(
    app: Option<&AppHandle>,
    registry: &WindowRegistry,
    icons: &IconService,
) -> Result<Vec<WindowInfo>, String> {
    let mut windows = enumerate_open_windows(icons)?;
    registry.apply_stable_order(&mut windows);
    let signature = windows
        .iter()
        .map(|win| format!("{}:{}:{}:{}", win.hwnd, win.identity_key, win.is_active, win.icon_source))
        .collect::<Vec<_>>()
        .join("|");

    if let Ok(mut current) = registry.windows.lock() {
        *current = windows.clone();
    }

    let mut changed = false;
    if let Ok(mut last) = registry.signature.lock() {
        changed = *last != signature;
        if changed {
            *last = signature;
        }
    }

    if changed {
        if let Some(handle) = app {
            let _ = handle.emit("open-windows-changed", windows.clone());
        }
    }

    Ok(windows)
}

fn start_window_registry_thread(app: AppHandle, registry: WindowRegistry, icons: IconService) {
    thread::spawn(move || loop {
        let _ = refresh_window_registry(Some(&app), &registry, &icons);
        thread::sleep(Duration::from_millis(1200));
    });
}

fn enumerate_open_windows(icons: &IconService) -> Result<Vec<WindowInfo>, String> {
    let mut windows: Vec<WindowInfo> = Vec::new();
    let mut context = (&mut windows, icons.clone());

    unsafe {
        let _ = windows::Win32::System::Com::CoInitializeEx(
            None,
            windows::Win32::System::Com::COINIT_APARTMENTTHREADED,
        );
        let _ = EnumWindows(Some(enum_windows_proc), windows::Win32::Foundation::LPARAM(&mut context as *mut _ as isize));
    }

    Ok(windows)
}

unsafe extern "system" fn enum_windows_proc(hwnd: HWND, lparam: windows::Win32::Foundation::LPARAM) -> windows::Win32::Foundation::BOOL {
    let context = &mut *(lparam.0 as *mut (&mut Vec<WindowInfo>, IconService));
    let windows = &mut context.0;
    let icons = &context.1;
    
    if IsWindowVisible(hwnd).as_bool() {
        let mut text: [u16; 512] = [0; 512];
        let len = GetWindowTextW(hwnd, &mut text);
        let title = String::from_utf16_lossy(&text[..len as usize]);

        if !title.is_empty() && title != "Program Manager" && title != "Aeropeks" {
            // Filter out Aeropeks own windows or specific background windows
            let mut class_text: [u16; 512] = [0; 512];
            let class_len = GetClassNameW(hwnd, &mut class_text);
            let class_name = String::from_utf16_lossy(&class_text[..class_len as usize]);

            let mut process_id = 0;
            GetWindowThreadProcessId(hwnd, Some(&mut process_id));
            
            let mut app_name = class_name.clone();
            let mut process_path = None;

            if let Ok(process_handle) = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, process_id) {
                let mut path_chars: [u16; 1024] = [0; 1024];
                let mut size = path_chars.len() as u32;
                if QueryFullProcessImageNameW(process_handle, PROCESS_NAME_WIN32, windows::core::PWSTR(path_chars.as_mut_ptr()), &mut size).is_ok() {
                    let full_path = String::from_utf16_lossy(&path_chars[..size as usize]);
                    if let Some(name) = std::path::Path::new(&full_path).file_name() {
                        app_name = name.to_string_lossy().to_string();
                    }
                    process_path = Some(full_path);
                }
                let _ = CloseHandle(process_handle);
            }

            let app_id = window_property_string(hwnd, &PKEY_AppUserModel_ID);
            let relaunch_command = window_property_string(hwnd, &PKEY_AppUserModel_RelaunchCommand);
            let relaunch_icon = window_property_string(hwnd, &PKEY_AppUserModel_RelaunchIconResource);
            let identity_key = taskbar_identity_for_window(hwnd, process_path.as_deref());
            let (icon, icon_source) = icons.resolve(&identity_key, || {
                extract_taskbar_icon(hwnd, process_path.as_deref())
            });

            let is_active = GetForegroundWindow() == hwnd;

            windows.push(WindowInfo {
                hwnd: hwnd.0 as isize,
                title,
                app_name,
                is_active,
                icon,
                icon_source,
                identity_key,
                class_name,
                process_path,
                app_id,
                relaunch_command,
                relaunch_icon,
                inclusion_reason: "visible titled top-level window".to_string(),
            });
        }
    }
    true.into()
}

#[tauri::command]
fn focus_window(hwnd: isize) -> Result<(), String> {
    let h = HWND(hwnd as *mut _);
    unsafe {
        let _ = ShowWindow(h, SW_RESTORE);
        let _ = SetForegroundWindow(h);
    }
    Ok(())
}

// ── AppBar ──────────────────────────────────────────────────────────

#[tauri::command]
fn close_window(hwnd: isize) -> Result<(), String> {
    unsafe {
        let hwnd = HWND(hwnd as *mut _);
        let _ = SendMessageTimeoutW(
            hwnd,
            windows::Win32::UI::WindowsAndMessaging::WM_CLOSE,
            WPARAM(0),
            LPARAM(0),
            SMTO_ABORTIFHUNG,
            1000,
            None
        );
    }
    Ok(())
}

#[tauri::command]
fn set_preview_mode(active: bool, state: tauri::State<'_, Arc<AtomicBool>>) {
    state.store(active, Ordering::Relaxed);
}

fn register_app_bar(hwnd_v: HWND, width: u32) {
    unsafe {
        let current_style = GetWindowLongW(hwnd_v, GWL_STYLE);
        let _ = SetWindowLongW(hwnd_v, GWL_STYLE, (current_style as u32 | WS_POPUP.0) as i32);

        let current_ex_style = GetWindowLongW(hwnd_v, GWL_EXSTYLE);
        let _ = SetWindowLongW(hwnd_v, GWL_EXSTYLE, (current_ex_style as u32 | WS_EX_TOOLWINDOW.0 | WS_EX_NOACTIVATE.0) as i32);

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

fn cleanup_app_bar(hwnd_v: HWND, edge: u32) {
    unsafe {
        let mut abd = APPBARDATA {
            cbSize: std::mem::size_of::<APPBARDATA>() as u32,
            hWnd: hwnd_v,
            uCallbackMessage: 0,
            uEdge: edge,
            rc: RECT::default(),
            lParam: windows::Win32::Foundation::LPARAM(0),
        };
        SHAppBarMessage(windows::Win32::UI::Shell::ABM_REMOVE, &mut abd);
    }
}

unsafe extern "system" fn enum_native_taskbar_proc(hwnd: HWND, lparam: windows::Win32::Foundation::LPARAM) -> windows::Win32::Foundation::BOOL {
    let should_show = *(lparam.0 as *const bool);
    let mut class_text: [u16; 256] = [0; 256];
    let class_len = GetClassNameW(hwnd, &mut class_text);
    let class_name = String::from_utf16_lossy(&class_text[..class_len as usize]);

    if class_name == "Shell_TrayWnd" || class_name == "Shell_SecondaryTrayWnd" {
        let _ = ShowWindow(hwnd, if should_show { SW_SHOW } else { SW_HIDE });
    }

    true.into()
}

fn set_native_taskbar_visible(visible: bool) {
    unsafe {
        let mut should_show = visible;
        let _ = EnumWindows(
            Some(enum_native_taskbar_proc),
            windows::Win32::Foundation::LPARAM(&mut should_show as *mut _ as isize),
        );
    }
}

fn reset_primary_work_area(handle: &AppHandle) -> Result<(), String> {
    let monitor = handle
        .primary_monitor()
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "No primary monitor available".to_string())?;
    let position = monitor.position();
    let size = monitor.size();

    let mut reset_rect = RECT {
        left: position.x,
        top: position.y,
        right: position.x + size.width as i32,
        bottom: position.y + size.height as i32,
    };

    unsafe {
        SystemParametersInfoW(
            SPI_SETWORKAREA,
            0,
            Some(&mut reset_rect as *mut _ as *mut _),
            SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS(0),
        )
        .map_err(|e| e.to_string())?;
    }

    Ok(())
}

fn restore_shell_state_internal(handle: &AppHandle) -> Result<(), String> {
    if let Some(window) = handle.get_webview_window("main") {
        if let Ok(hwnd) = window.hwnd() {
            cleanup_app_bar(HWND(hwnd.0), ABE_TOP as u32);
        }
    }

    if let Some(window) = handle.get_webview_window("taskbar-bottom") {
        if let Ok(hwnd) = window.hwnd() {
            cleanup_app_bar(HWND(hwnd.0), ABE_BOTTOM as u32);
        }
    }

    set_native_taskbar_visible(true);
    reset_primary_work_area(handle)
}

#[tauri::command]
fn restore_shell_state(app: AppHandle) -> Result<(), String> {
    restore_shell_state_internal(&app)
}

fn register_bottom_app_bar(hwnd_v: HWND, width: u32, screen_height: u32) {
    unsafe {
        let current_style = GetWindowLongW(hwnd_v, GWL_STYLE);
        let _ = SetWindowLongW(hwnd_v, GWL_STYLE, (current_style as u32 | WS_POPUP.0) as i32);

        let current_ex_style = GetWindowLongW(hwnd_v, GWL_EXSTYLE);
        let _ = SetWindowLongW(hwnd_v, GWL_EXSTYLE, (current_ex_style as u32 | WS_EX_TOOLWINDOW.0 | WS_EX_NOACTIVATE.0) as i32);

        let mut abd = APPBARDATA {
            cbSize: std::mem::size_of::<APPBARDATA>() as u32,
            hWnd: hwnd_v,
            uCallbackMessage: 0x0401,
            uEdge: ABE_BOTTOM as u32,
            rc: RECT { left: 0, top: (screen_height - 60) as i32, right: width as i32, bottom: screen_height as i32 },
            lParam: windows::Win32::Foundation::LPARAM(0),
        };

        let _ = SHAppBarMessage(windows::Win32::UI::Shell::ABM_REMOVE, &mut abd);
        SHAppBarMessage(ABM_NEW, &mut abd);
        SHAppBarMessage(ABM_QUERYPOS, &mut abd);
        abd.rc.top = (screen_height - 60) as i32;
        abd.rc.bottom = screen_height as i32;
        SHAppBarMessage(ABM_SETPOS, &mut abd);

        // AppBar reserves 60px at screen bottom; window matches
        let _ = SetWindowPos(hwnd_v, HWND_TOPMOST, 0, (screen_height - 60) as i32, width as i32, 60, SWP_NOACTIVATE | SWP_SHOWWINDOW | SWP_FRAMECHANGED);
    }
}

// ── Main ────────────────────────────────────────────────────────────

fn main() {
    let shared: SharedSettings = Arc::new(Mutex::new(AppSettings::default()));

    let terminal_state = TerminalState {
        writer: Arc::new(Mutex::new(None)),
        master: Arc::new(Mutex::new(None)),
    };

    let icon_service = IconService::new();
    let window_registry = WindowRegistry::new();
    let preview_active: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));

    tauri::Builder::default()
        .manage(shared.clone())
        .manage(terminal_state)
        .manage(icon_service.clone())
        .manage(preview_active.clone())
        .manage(window_registry.clone())
        .on_menu_event(|app, event| {
            let id = event.id().as_ref();
            let handle = app.app_handle();
            let state = handle.state::<SharedSettings>();
            let settings = match state.lock() {
                Ok(s) => s.clone(),
                Err(_) => return,
            };

            let shortcut = settings.terminal_shortcuts.iter().find(|s| s.id == id);
            
            if let Some(s) = shortcut {
                // Parse command into args (simple split for now)
                let final_args: Vec<String> = s.cmd.split_whitespace().map(|part| part.to_string()).collect();
                
                // 1. Toggle terminal panel ON if hidden
                if let Some(window) = handle.get_webview_window("terminal-panel") {
                    if !window.is_visible().unwrap_or(false) {
                        toggle_terminal_panel(handle.clone());
                    }
                }
                // 2. Emit start-session to the terminal window
                let _ = handle.emit_to("terminal-panel", "start-session", PtyPayload { data: serde_json::to_string(&final_args).unwrap_or_default() });
            }
        })
        .setup(move |app| {
            let app_handle = app.handle().clone();
            let app_handle_media = app.handle().clone();

            // Load real settings
            let initial_settings = fetch_settings_helper(app.handle().clone());
            {
                if let Ok(mut lock) = shared.lock() {
                    *lock = initial_settings.clone();
                }
            }

            set_native_taskbar_visible(!initial_settings.hide_native_taskbar);
            start_window_registry_thread(app.handle().clone(), window_registry.clone(), icon_service.clone());

            if let Ok(Some(monitor)) = app.handle().primary_monitor() {
                let size = monitor.size();
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.set_size(tauri::Size::Physical(tauri::PhysicalSize { width: size.width, height: 32 }));
                    let _ = window.set_shadow(false);
                    let _ = window.set_skip_taskbar(true); // Ensure no taskbar presence
                    if let Ok(hwnd_raw) = window.hwnd() {
                        let hwnd = HWND(hwnd_raw.0);
                        if initial_settings.reserve_screen_space {
                            register_app_bar(hwnd, size.width);

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
                                        let current_height = w_clone.inner_size().unwrap_or(tauri::PhysicalSize { width: 0, height: 32 }).height;
                                        let _ = SetWindowPos(hwnd_v, HWND_TOPMOST, 0, 0, width as i32, current_height as i32, SWP_NOACTIVATE | SWP_FRAMECHANGED);
                                        
                                        if work_area.position.y != 32 {
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
                    } else {
                        unsafe {
                            let _ = SetWindowPos(hwnd, HWND_TOPMOST, 0, 0, size.width as i32, 32, SWP_NOACTIVATE | SWP_SHOWWINDOW | SWP_FRAMECHANGED);
                        }
                    }
                    }
                }
                if let Some(window) = app.get_webview_window("taskbar-bottom") {
                    // Height must be large enough for preview popups to render above the dock
                    let _ = window.set_size(tauri::Size::Physical(tauri::PhysicalSize { width: size.width, height: 60 }));
                    let _ = window.set_shadow(false);
                    let _ = window.set_skip_taskbar(true);
                    
                    if let Ok(hwnd_raw) = window.hwnd() {
                        let hwnd = HWND(hwnd_raw.0);
                        if initial_settings.reserve_screen_space {
                            register_bottom_app_bar(hwnd, size.width, size.height);

                            let w_clone = window.clone();
                            let preview_flag = preview_active.clone();
                            thread::spawn(move || {
                                loop {
                                if let Ok(Some(monitor)) = w_clone.primary_monitor() {
                                    let width = monitor.size().width;
                                    let height = monitor.size().height;
                                    let hwnd = w_clone.hwnd().unwrap();
                                    let hwnd_v = HWND(hwnd.0);
                                    unsafe {
                                        let mut abd = APPBARDATA {
                                            cbSize: std::mem::size_of::<APPBARDATA>() as u32,
                                            hWnd: hwnd_v,
                                            uCallbackMessage: 0x0401,
                                            uEdge: ABE_BOTTOM as u32,
                                            rc: RECT { left: 0, top: (height - 60) as i32, right: width as i32, bottom: height as i32 },
                                            lParam: windows::Win32::Foundation::LPARAM(0),
                                        };
                                        SHAppBarMessage(ABM_QUERYPOS, &mut abd);
                                        abd.rc.top = (height - 60) as i32; abd.rc.bottom = height as i32;
                                        SHAppBarMessage(ABM_SETPOS, &mut abd);
                                        // Only reposition window if no preview popup is active
                                        if !preview_flag.load(Ordering::Relaxed) {
                                            let _ = SetWindowPos(hwnd_v, HWND_TOPMOST, 0, (height - 60) as i32, width as i32, 60, SWP_NOACTIVATE | SWP_FRAMECHANGED);
                                        }

                                        // Keep the reserved area honest if Explorer or display changes nudge it.
                                        let mut new_work_area = RECT {
                                            left: 0,
                                            top: 32,
                                            right: width as i32,
                                            bottom: (height - 60) as i32,
                                        };
                                        SystemParametersInfoW(
                                            SPI_SETWORKAREA,
                                            0,
                                            Some(&mut new_work_area as *mut _ as *mut _),
                                            SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS(0),
                                        ).ok();
                                    }
                                }
                                thread::sleep(Duration::from_secs(5));
                            }
                        });
                    } else {
                        unsafe {
                            let _ = SetWindowPos(hwnd, HWND_TOPMOST, 0, (size.height - 60) as i32, size.width as i32, 60, SWP_NOACTIVATE | SWP_SHOWWINDOW | SWP_FRAMECHANGED);
                        }
                    }
                    }
                }
            }

            let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>).expect("");
            let settings_i = MenuItem::with_id(app, "settings", "Settings", true, None::<&str>).expect("");
            let menu = Menu::with_items(app, &[&settings_i, &quit_i]).expect("");
            
            let tray_builder = TrayIconBuilder::new()
                .menu(&menu)
                .show_menu_on_left_click(true)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "quit" => {
                        let _ = restore_shell_state_internal(app);
                        app.exit(0);
                    }
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

            // Background GSMTC listener
            tauri::async_runtime::spawn(async move {
                use windows::Media::Control::GlobalSystemMediaTransportControlsSessionManager;
                use windows::Foundation::TypedEventHandler;
                
                if let Ok(manager_op) = GlobalSystemMediaTransportControlsSessionManager::RequestAsync() {
                    if let Ok(manager) = manager_op.get() {
                        let h1 = app_handle_media.clone();
                        let _ = manager.CurrentSessionChanged(&TypedEventHandler::new(move |_, _| {
                            let h2 = h1.clone();
                            tauri::async_runtime::spawn(async move {
                                if let Ok(update) = get_active_media_internal(h2.clone()).await {
                                    let _ = h2.emit("media-change", update);
                                }
                            });
                            Ok(())
                        }));

                        let h3 = app_handle_media.clone();
                        let _ = manager.SessionsChanged(&TypedEventHandler::new(move |_, _| {
                            let h4 = h3.clone();
                            tauri::async_runtime::spawn(async move {
                                if let Ok(update) = get_active_media_internal(h4.clone()).await {
                                    let _ = h4.emit("media-change", update);
                                }
                            });
                            Ok(())
                        }));
                        
                        // Periodic fallback/refresh
                        let h_poll = app_handle_media.clone();
                        tauri::async_runtime::spawn(async move {
                            let mut interval = interval(Duration::from_secs(5));
                            loop {
                                interval.tick().await;
                                // println!("DEBUG: Polling loop tick");
                                if let Ok(update) = get_active_media_internal(h_poll.clone()).await {
                                    h_poll.emit("media-change", update).ok();
                                }
                            }
                        });
                    }
                }
            });
            
            // Global Shortcut: Alt+Space
            use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, Modifiers, Code};
            let launcher_shortcut = Shortcut::new(Some(Modifiers::ALT), Code::Space);
            app.handle().plugin(
                tauri_plugin_global_shortcut::Builder::new()
                    .with_handler(move |app, shortcut, event| {
                        if event.state() == tauri_plugin_global_shortcut::ShortcutState::Pressed {
                            if shortcut == &launcher_shortcut {
                                toggle_launcher(app.clone());
                            } else {
                                if let Ok(settings) = app.state::<SharedSettings>().lock() {
                                    for s in &settings.terminal_shortcuts {
                                        if let Ok(registered) = s.shortcut.parse::<tauri_plugin_global_shortcut::Shortcut>() {
                                            if &registered == shortcut {
                                                // Execute command
                                                let cmd = s.cmd.clone();
                                                std::thread::spawn(move || {
                                                    let _ = std::process::Command::new("cmd")
                                                        .args(["/C", &cmd])
                                                        .creation_flags(CREATE_NO_WINDOW)
                                                        .spawn();
                                                });


                                            }
                                        }
                                    }
                                }
                            }
                        }
                    })
                    .build(),
            ).ok();
            app.global_shortcut().register(launcher_shortcut).ok();

            Ok(())
        })
        .on_window_event(|window, event| {
            match event {
                tauri::WindowEvent::CloseRequested { api, .. } => {
                    if window.label() == "settings" {
                        api.prevent_close();
                        let _ = window.hide();
                    } else if window.label() == "main" {
                        let _ = restore_shell_state_internal(window.app_handle());
                    }
                }
                tauri::WindowEvent::Destroyed => {
                    if window.label() == "main" {
                        let _ = restore_shell_state_internal(window.app_handle());
                    }
                }
                _ => {}
            }
        })
        .invoke_handler(tauri::generate_handler![
            get_volume, 
            set_volume, 
            get_media_info_unified, 
            media_control_unified,
            get_settings, 
            save_settings, 
            open_settings, 
            toggle_expanded_player, 
            plex_control, 
            get_album_art,
            toggle_terminal_panel,
            start_pty,
            write_pty,
            resize_pty,
            show_terminal_context_menu,
            search_query,
            launch_result,
            toggle_launcher,
            get_battery_status,
            system_power_action,
            get_mic_status,
            toggle_mic_mute,
            get_privacy_status,
            set_privacy_mode,
            get_weather,
            search_locations,
            get_bluetooth_status,
            get_obs_status,
            gsmtc_action,
            register_hotkeys,
            set_window_height,
            kill_pty,
            get_open_windows,
            get_window_debug_snapshot,
            clear_icon_cache,
            restore_shell_state,
            get_window_thumbnail,
            close_window,
            focus_window,
            get_virtual_desktop_status,
            switch_virtual_desktop,
            set_preview_mode
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
