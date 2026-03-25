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
use tauri::{Window, Manager, Emitter, AppHandle, State};
use tauri::menu::{Menu, MenuItem, MenuEvent, PredefinedMenuItem};
use tauri::tray::TrayIconBuilder;
use std::thread;
use std::time::Duration;
use serde::{Serialize, Deserialize};
use portable_pty::{CommandBuilder, NativePtySystem, PtySize, PtySystem, MasterPty};
use std::io::{Write, Read};
use base64::{Engine as _, engine::general_purpose};
use serde_json;

#[derive(Serialize, Clone)]
struct PtyPayload {
    data: String,
}

fn default_accent_color() -> String {
    "#22c55e".to_string()
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct TerminalShortcut {
    id: String,
    label: String,
    cmd: String,
}

fn default_shortcuts() -> Vec<TerminalShortcut> {
    vec![
        TerminalShortcut { id: "ssh-home".to_string(), label: "SSH: Home Lab (pi@homeserver)".to_string(), cmd: "ssh pi@homeserver.local".to_string() },
        TerminalShortcut { id: "ssh-prod".to_string(), label: "SSH: Production (root@vps)".to_string(), cmd: "ssh root@production-vps".to_string() },
        TerminalShortcut { id: "git-status".to_string(), label: "Git Status".to_string(), cmd: "git status".to_string() },
        TerminalShortcut { id: "git-fetch".to_string(), label: "Git Fetch All".to_string(), cmd: "git fetch --all".to_string() },
    ]
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct AppSettings {
    plex_url: String,
    plex_token: String,
    #[serde(default = "default_accent_color")]
    accent_color: String,
    #[serde(default = "default_shortcuts")]
    terminal_shortcuts: Vec<TerminalShortcut>,
}

type SharedSettings = Arc<Mutex<AppSettings>>;

struct TerminalState {
    writer: Arc<Mutex<Option<Box<dyn Write + Send>>>>,
    master: Arc<Mutex<Option<Box<dyn MasterPty + Send>>>>,
}

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
        accent_color: "#22c55e".to_string(),
        terminal_shortcuts: default_shortcuts(),
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
    album: String,
    thumb: String,
    duration_ms: u64,
    view_offset_ms: u64,
    is_playing: bool,
    session_id: String,
    machine_id: String,
    address: String,
}

fn fetch_plex_media(settings: &AppSettings) -> Option<MediaInfo> {
    let url = format!("{}/status/sessions?X-Plex-Token={}", settings.plex_url.trim_end_matches('/'), settings.plex_token);
    // println!("Plex DEBUG: Fetching from {}", settings.plex_url);
    
    let client = reqwest::blocking::Client::builder().timeout(Duration::from_secs(5)).build().ok()?;
    let resp = match client.get(&url).header("Accept", "application/json").send() {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Plex ERROR: Request failed: {}", e);
            return None;
        }
    };
    
    if !resp.status().is_success() {
        eprintln!("Plex ERROR: HTTP status {}", resp.status());
        return None;
    }

    let json: serde_json::Value = match resp.json() {
        Ok(j) => j,
        Err(e) => {
            eprintln!("Plex ERROR: JSON parse failed: {}", e);
            return None;
        }
    };
    
    let container = json.get("MediaContainer")?;
    let metadata = container.get("Metadata")?.as_array()?;

    for item in metadata {
        let title = item.get("title").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let artist = item.get("grandparentTitle").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let album = item.get("parentTitle").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let thumb = item.get("thumb").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let duration_ms = item.get("duration").and_then(|v| v.as_u64()).unwrap_or(0);
        let view_offset_ms = item.get("viewOffset").and_then(|v| v.as_u64()).unwrap_or(0);
        let session_id = item.get("sessionKey").and_then(|v| v.as_str()).unwrap_or("").to_string();

        let player = item.get("Player").and_then(|v| v.as_object());
        let machine_id = player.and_then(|p| p.get("machineIdentifier")).and_then(|v| v.as_str()).unwrap_or("").to_string();
        let address = player.and_then(|p| p.get("address")).and_then(|v| v.as_str()).unwrap_or("").to_string();
        let state = player.and_then(|p| p.get("state")).and_then(|v| v.as_str()).unwrap_or("");

        if !title.is_empty() {
            return Some(MediaInfo {
                title,
                artist,
                album,
                thumb,
                duration_ms,
                view_offset_ms,
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

    let client = reqwest::blocking::Client::builder().timeout(Duration::from_secs(3)).build().unwrap_or_default();

    // Build a unique command ID from current time
    let command_id = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(1);

    let mut attempts: Vec<(String, &str)> = vec![
        // Attempt 1: Plex server proxy to player
        (format!("{}/player/proxy/playback/{}?X-Plex-Target-Client-Identifier={}&X-Plex-Token={}&commandID={}",
            settings.plex_url.trim_end_matches('/'), plex_command, machine_id, settings.plex_token, command_id), "GET"),
    ];

    if !address.is_empty() {
        // Attempt 2: Direct to Plexamp (port 32500) — POST
        attempts.push((format!("http://{}:32500/player/playback/{}?commandID={}&X-Plex-Token={}",
            address, plex_command, command_id, settings.plex_token), "POST"));
        // Attempt 3: Direct to Plexamp (port 32500) — GET
        attempts.push((format!("http://{}:32500/player/playback/{}?commandID={}&X-Plex-Token={}",
            address, plex_command, command_id, settings.plex_token), "GET"));
        // Attempt 4: Via Plex server system/players
        attempts.push((format!("{}/system/players/{}/playback/{}?X-Plex-Token={}",
            settings.plex_url.trim_end_matches('/'), address, plex_command, settings.plex_token), "GET"));
    }

    for (url, method) in &attempts {
        let req = if *method == "POST" { client.post(url) } else { client.get(url) };
        match req
            .header("X-Plex-Client-Identifier", "aeropeks")
            .header("X-Plex-Target-Client-Identifier", &machine_id)
            .header("X-Plex-Token", &settings.plex_token)
            .send()
        {
            Ok(resp) => {
                println!("Plex control {} {} → {}", method, url, resp.status());
                if resp.status().is_success() || resp.status().as_u16() == 200 {
                    return Ok(());
                }
            }
            Err(e) => println!("Plex control {} {} failed: {}", method, url, e),
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
    println!("INFO: start_pty called with {}x{}, args: {:?}", rows, cols, args);
    
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
    println!("INFO: PTY_READY emitted to terminal-panel");

    let app_handle_hb = window.app_handle().clone();
    thread::spawn(move || {
        for _ in 0..15 {
            thread::sleep(Duration::from_secs(1));
            let _ = app_handle_hb.emit_to("terminal-panel", "pty-heartbeat", "HB");
            println!("DEBUG: Heartbeat emitted to terminal-panel");
        }
    });

    let app_handle = window.app_handle().clone();
    thread::spawn(move || {
        let mut reader = reader;
        let mut buffer = [0u8; 1024];
        loop {
            match reader.read(&mut buffer) {
                Ok(0) => { println!("INFO: PTY reader reached EOF"); break; }
                Ok(n) => {
                    let data = &buffer[..n];
                    let b64 = general_purpose::STANDARD.encode(data);
                    println!("INFO: PTY read {} bytes", n);
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

        println!("DEBUG appbar: Cleanup complete (Reserved space released)");
    }
}

// ── Main ────────────────────────────────────────────────────────────

fn main() {
    tauri::Builder::default()
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
        .setup(|app| {
            let app_handle = app.handle().clone();
            let app_handle_media = app.handle().clone();

            let initial_settings = fetch_settings_helper(app.handle().clone());
            let shared: SharedSettings = Arc::new(Mutex::new(initial_settings));
            app.manage(shared.clone());

            app.manage(TerminalState {
                writer: Arc::new(Mutex::new(None)),
                master: Arc::new(Mutex::new(None)),
            });

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
                    let settings = match media_settings.lock() { 
                        Ok(s) => s.clone(), 
                        Err(_) => { 
                            eprintln!("Plex ERROR: Failed to lock shared settings");
                            thread::sleep(Duration::from_secs(5)); 
                            continue; 
                        } 
                    };
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
        .invoke_handler(tauri::generate_handler![
            get_volume, 
            set_volume, 
            get_media_info, 
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
            show_terminal_context_menu
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
