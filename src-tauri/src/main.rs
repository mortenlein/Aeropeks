// Aeropeks v0.1.0 - Terminal Fix Build
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
use tauri_plugin_shell::ShellExt;

#[derive(Serialize, Clone)]
struct PtyPayload {
    data: String,
}

fn default_accent_color() -> String { "#22c55e".to_string() }
fn default_plex_url() -> String { "http://localhost:32400".to_string() }
fn default_plex_token() -> String { "".to_string() }
fn default_true() -> bool { true }

#[derive(Serialize, Deserialize, Clone, Debug)]
struct TerminalShortcut {
    id: String,
    label: String,
    cmd: String,
    shortcut: String,
}

fn default_shortcuts() -> Vec<TerminalShortcut> {
    vec![
        TerminalShortcut { id: "ssh-home".to_string(), label: "SSH: Home Lab (pi@homeserver)".to_string(), cmd: "ssh pi@homeserver.local".to_string(), shortcut: "".to_string() },
        TerminalShortcut { id: "ssh-prod".to_string(), label: "SSH: Production (root@vps)".to_string(), cmd: "ssh root@production-vps".to_string(), shortcut: "".to_string() },
        TerminalShortcut { id: "git-status".to_string(), label: "Git Status".to_string(), cmd: "git status".to_string(), shortcut: "".to_string() },
        TerminalShortcut { id: "git-fetch".to_string(), label: "Git Fetch All".to_string(), cmd: "git fetch --all".to_string(), shortcut: "".to_string() },
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
        }
    }
}

// Shared settings type
type SharedSettings = Arc<Mutex<AppSettings>>;

struct TerminalState {
    writer: Arc<Mutex<Option<Box<dyn Write + Send>>>>,
    master: Arc<Mutex<Option<Box<dyn MasterPty + Send>>>>,
}

fn get_settings_path(handle: tauri::AppHandle) -> PathBuf {
    let mut path = handle.path().app_data_dir().unwrap();
    eprintln!("DEBUG: App Data Dir: {:?}", path);
    fs::create_dir_all(&path).ok();
    path.push("settings.json");
    path
}

fn fetch_settings_helper(handle: tauri::AppHandle) -> AppSettings {
    let path = get_settings_path(handle);
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
    let path = get_settings_path(handle.clone());
    let content = serde_json::to_string(&settings).map_err(|e| e.to_string())?;
    fs::write(path, content).map_err(|e| e.to_string())?;
    
    let mut current_settings = state.lock().map_err(|e| e.to_string())?;
    *current_settings = settings;
    handle.emit("settings-changed", current_settings.clone()).ok();
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
            icon: "Search".to_string(),
            action_type: "web".to_string(),
            action_value: format!("https://www.google.com/search?q={}", urlencoding::encode(q)),
        });
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
            let shell = handle.shell();
            if result.action_value.contains(" ") && result.action_type == "system" {
                let parts: Vec<&str> = result.action_value.split_whitespace().collect();
                let cmd = parts[0];
                let args = &parts[1..];
                shell.command(cmd).args(args).spawn().map_err(|e| e.to_string())?;
            } else {
                shell.command("cmd").args(["/C", "start", "", &result.action_value]).spawn().map_err(|e| e.to_string())?;
            }
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
        "shutdown" => { let _ = std::process::Command::new("shutdown").args(["/s", "/t", "0"]).spawn(); },
        "restart" => { let _ = std::process::Command::new("shutdown").args(["/r", "/t", "0"]).spawn(); },
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
        let enumerator: IMMDeviceEnumerator = CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL).map_err(|e| e.to_string())?;
        let device = enumerator.GetDefaultAudioEndpoint(eCapture, eConsole).map_err(|e| e.to_string())?;
        let volume = device.Activate::<IAudioEndpointVolume>(CLSCTX_ALL, None).map_err(|e| e.to_string())?;
        Ok(volume.GetMute().map_err(|e| e.to_string())?.as_bool())
    }
}

#[tauri::command]
fn toggle_mic_mute() -> Result<bool, String> {
    unsafe {
        let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
        let enumerator: IMMDeviceEnumerator = CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL).map_err(|e| e.to_string())?;
        let device = enumerator.GetDefaultAudioEndpoint(eCapture, eConsole).map_err(|e| e.to_string())?;
        let volume = device.Activate::<IAudioEndpointVolume>(CLSCTX_ALL, None).map_err(|e| e.to_string())?;
        let current_mute = volume.GetMute().map_err(|e| e.to_string())?;
        volume.SetMute(!current_mute, std::ptr::null()).map_err(|e| e.to_string())?;
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
        .args([
            "-NoProfile",
            "-Command",
            &format!("Get-PnpDevice -Class Camera -ErrorAction SilentlyContinue | {} -Confirm:$false", cmd)
        ])
        .status();
    
    Ok(())
}


// ── Bluetooth & Weather ─────────────────────────────────────────────

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
    
    if let Some(cmd_args) = args {
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
                Ok(0) => { break; }
                Ok(n) => {
                    let data = &buffer[..n];
                    let b64 = general_purpose::STANDARD.encode(data);
                    let _ = app_handle.emit_to("terminal-panel", "pty-data", PtyPayload { data: b64 });
                }
                Err(e) => { println!("ERROR: PTY read error: {}", e); break; }
            }
        }
    });

    Ok(())
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
    }
}

// ── Main ────────────────────────────────────────────────────────────

fn main() {
    let shared: SharedSettings = Arc::new(Mutex::new(AppSettings::default()));

    let terminal_state = TerminalState {
        writer: Arc::new(Mutex::new(None)),
        master: Arc::new(Mutex::new(None)),
    };

    tauri::Builder::default()
        .manage(shared.clone())
        .manage(terminal_state)
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
                    *lock = initial_settings;
                }
            }


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
                                                let app_c = app.clone();
                                                let cmd = s.cmd.clone();
                                                std::thread::spawn(move || {
                                                    let _ = app_c.shell().command("cmd").args(["/C", &cmd]).spawn();
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
            set_window_height
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
