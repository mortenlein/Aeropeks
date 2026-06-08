// Aeropeks v0.1.0 - Terminal Fix Build
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod integrations;
mod launcher;
mod media;
mod projects;
mod security;
mod settings;
mod shell;
mod system_status;
mod terminal;

use serde::Serialize;
use settings::{AppSettings, SharedSettings};
use std::collections::{HashMap, HashSet};
use std::fs;
#[cfg(windows)]
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::{AppHandle, Emitter, Manager, State, Window};
use tokio::time::interval;
use windows::Win32::Foundation::{CloseHandle, HWND, LPARAM, RECT, SIZE, WPARAM};
use windows::Win32::Graphics::Dwm::{DwmGetWindowAttribute, DWMWA_EXTENDED_FRAME_BOUNDS};
use windows::Win32::Graphics::Gdi::{
    CreateCompatibleBitmap, CreateCompatibleDC, DeleteDC, DeleteObject, GetDC, GetDIBits,
    GetObjectW, ReleaseDC, SelectObject, SetStretchBltMode, StretchBlt, BITMAP, BITMAPINFO,
    BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS, HALFTONE, HBITMAP, HGDIOBJ, SRCCOPY,
};
use windows::Win32::Storage::EnhancedStorage::{
    PKEY_AppUserModel_ID, PKEY_AppUserModel_RelaunchCommand, PKEY_AppUserModel_RelaunchIconResource,
};
use windows::Win32::Storage::Xps::{PrintWindow, PRINT_WINDOW_FLAGS};
use windows::Win32::System::Com::StructuredStorage::{PropVariantClear, PropVariantToStringAlloc};
use windows::Win32::System::Com::{CoTaskMemFree, IBindCtx};
use windows::Win32::System::Threading::{
    OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION,
};
use windows::Win32::UI::Shell::PropertiesSystem::{
    IPropertyStore, SHGetPropertyStoreForWindow, PROPERTYKEY,
};
use windows::Win32::UI::Shell::{
    ExtractIconExW, IShellItemImageFactory, SHCreateItemFromParsingName, SHGetFileInfoW,
    SHFILEINFOW, SHGFI_ICON, SHGFI_LARGEICON, SIIGBF_BIGGERSIZEOK,
};
use windows::Win32::UI::WindowsAndMessaging::{
    DestroyIcon, EnumWindows, GetClassLongPtrW, GetClassNameW, GetForegroundWindow, GetIconInfo,
    GetWindowPlacement, GetWindowRect, GetWindowTextW, GetWindowThreadProcessId, IsWindowVisible,
    SendMessageTimeoutW, SetForegroundWindow, ShowWindow, GCLP_HICON, GCLP_HICONSM, HICON,
    ICON_BIG, ICON_SMALL, ICON_SMALL2, SMTO_ABORTIFHUNG, SW_MINIMIZE, SW_RESTORE, SW_SHOWMINIMIZED,
    WINDOWPLACEMENT, WM_GETICON,
};
const ICON_CACHE_VERSION: &str = "v2-bgra-bmp";

#[derive(Serialize, Clone)]
struct PtyPayload {
    data: String,
}

struct ShutdownState(Arc<AtomicBool>);

#[tauri::command]
fn show_terminal_context_menu(
    window: tauri::Window,
    state: tauri::State<'_, SharedSettings>,
) -> Result<(), String> {
    security::require_window(&window, &["main"])?;
    let handle = window.app_handle();
    let settings = state.lock().map_err(|e| e.to_string())?;

    let mut items: Vec<Box<dyn tauri::menu::IsMenuItem<tauri::Wry>>> = Vec::new();

    for (i, shortcut) in settings.terminal_shortcuts.iter().enumerate() {
        if i > 0
            && shortcut.id.contains("git")
            && !settings.terminal_shortcuts[i - 1].id.contains("git")
        {
            if let Ok(sep) = PredefinedMenuItem::separator(handle) {
                items.push(Box::new(sep));
            }
        }

        let item = MenuItem::with_id(handle, &shortcut.id, &shortcut.label, true, None::<&str>)
            .map_err(|e| e.to_string())?;
        items.push(Box::new(item));
    }

    // Convert Vec<Box<dyn IsMenuItem>> to Vec<&dyn IsMenuItem> for with_items
    let item_refs: Vec<&dyn tauri::menu::IsMenuItem<tauri::Wry>> =
        items.iter().map(|b| b.as_ref()).collect();
    let menu = Menu::with_items(handle, &item_refs).map_err(|e| e.to_string())?;

    window.popup_menu(&menu).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn save_settings(
    settings: AppSettings,
    handle: tauri::AppHandle,
    window: Window,
    state: tauri::State<'_, SharedSettings>,
) -> Result<(), String> {
    security::require_window(&window, &["settings"])?;
    settings::save(&handle, &settings)?;

    let mut current_settings = state.lock().map_err(|e| e.to_string())?;
    let previous_settings = current_settings.clone();
    *current_settings = settings.clone();
    handle
        .emit_to(
            "main",
            "settings-changed",
            current_settings.without_secrets(),
        )
        .ok();
    drop(current_settings);

    if previous_settings.hide_native_taskbar != settings.hide_native_taskbar {
        shell::set_native_taskbar_visible(!settings.hide_native_taskbar);
    }

    if previous_settings.reserve_screen_space && !settings.reserve_screen_space {
        let _ = shell::restore(&handle);
    }

    Ok(())
}

#[tauri::command]
fn set_window_height(window: tauri::Window, height: u32) -> Result<(), String> {
    security::require_window(&window, &["main"])?;
    if !(32..=900).contains(&height) {
        return Err("invalid window height".to_string());
    }
    let size = tauri::Size::Physical(tauri::PhysicalSize {
        width: window
            .inner_size()
            .unwrap_or(tauri::PhysicalSize {
                width: 1920,
                height: 32,
            })
            .width,
        height,
    });
    window.set_size(size).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn get_settings(
    state: tauri::State<'_, SharedSettings>,
    window: Window,
) -> Result<AppSettings, String> {
    security::require_window(&window, &["main", "settings", "demo-weather"])?;
    let settings = state.lock().map_err(|e| e.to_string())?;
    if window.label() == "settings" {
        Ok(settings.clone())
    } else {
        Ok(settings.without_secrets())
    }
}

// ── Launcher ────────────────────────────────────────────────────────

#[tauri::command]
fn toggle_launcher(handle: tauri::AppHandle, window: Window) -> Result<(), String> {
    security::require_window(&window, &["main", "launcher-panel"])?;
    toggle_launcher_internal(&handle);
    Ok(())
}

fn toggle_launcher_internal(handle: &tauri::AppHandle) {
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

#[tauri::command]
fn system_power_action(action: String, window: Window) -> Result<(), String> {
    security::require_window(&window, &["main", "launcher-panel"])?;
    launcher::run_power_action(&action)
}

// ── Bluetooth & Weather & Desktops ───────────────────────────────────────────

#[tauri::command]
fn get_virtual_desktop_status(window: Window) -> Result<(usize, usize), String> {
    security::require_window(&window, &["main"])?;
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
fn switch_virtual_desktop(index: usize, window: Window) -> Result<(), String> {
    security::require_window(&window, &["main"])?;
    if let Ok(desktops) = winvd::get_desktops() {
        if let Some(d) = desktops.get(index) {
            let _ = winvd::switch_desktop(*d);
        }
    }
    Ok(())
}

#[tauri::command]
fn register_hotkeys(
    app: AppHandle,
    settings: State<'_, SharedSettings>,
    window: Window,
) -> Result<(), String> {
    security::require_window(&window, &["settings"])?;
    use tauri_plugin_global_shortcut::GlobalShortcutExt;
    let _ = app.global_shortcut().unregister_all();

    // Re-register launcher
    use tauri_plugin_global_shortcut::{Code, Modifiers, Shortcut};
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
fn open_settings(handle: tauri::AppHandle, window: Window) -> Result<(), String> {
    security::require_window(&window, &["main"])?;
    open_settings_internal(&handle);
    Ok(())
}

fn open_settings_internal(handle: &tauri::AppHandle) {
    if let Some(window) = handle.get_webview_window("settings") {
        let _ = window.show().ok();
        let _ = window.set_focus().ok();
    }
}

#[tauri::command]
fn open_demo_mode(app: AppHandle, window: Window) -> Result<(), String> {
    security::require_window(&window, &["settings"])?;
    let monitor = window
        .primary_monitor()
        .map_err(|e| e.to_string())?
        .ok_or("no primary monitor")?;
    let w = monitor.size().width as i32;

    // Panels: player centered, terminal left, launcher right — below bar+popovers.
    let panel_layout: &[(&str, i32, i32)] = &[
        ("expanded-player", (w - 640) / 2, 440),
        ("terminal-panel",  20,            440),
        ("launcher-panel",  w - 720,       440),
    ];
    for (label, x, y) in panel_layout {
        if let Some(panel) = app.get_webview_window(label) {
            let _ = panel.set_position(tauri::Position::Physical(tauri::PhysicalPosition {
                x: *x,
                y: *y,
            }));
            let _ = panel.show();
        }
    }

    // Standalone demo popup windows for draggable popover screenshots.
    // Positioned near the matching bar items along the right side.
    let demo_layout: &[(&str, i32, i32)] = &[
        ("demo-weather",  w - 480, 36),
        ("demo-usage",    w - 880, 36),
        ("demo-projects", w - 1330, 36),
    ];
    for (label, x, y) in demo_layout {
        if let Some(panel) = app.get_webview_window(label) {
            let _ = panel.set_position(tauri::Position::Physical(tauri::PhysicalPosition {
                x: *x,
                y: *y,
            }));
            let _ = panel.show();
        }
    }

    // Tell the main bar to open its inline popovers (volume, power, bluetooth),
    // then make it click-through so its 760px surface doesn't block panels below.
    if let Some(main) = app.get_webview_window("main") {
        let _ = main.emit("demo-mode", ());
        let _ = main.set_ignore_cursor_events(true);
    }
    // Close settings — not part of the screenshot.
    if let Some(settings) = app.get_webview_window("settings") {
        let _ = settings.hide();
    }
    Ok(())
}

fn exit_demo_mode_internal(app: &AppHandle) {
    for label in &["demo-weather", "demo-usage", "demo-projects",
                   "expanded-player", "terminal-panel", "launcher-panel"] {
        if let Some(w) = app.get_webview_window(label) {
            let _ = w.hide();
        }
    }
    if let Some(main) = app.get_webview_window("main") {
        let _ = main.set_ignore_cursor_events(false);
        let _ = main.emit("demo-mode-exit", ());
    }
}

#[tauri::command]
fn exit_demo_mode(app: AppHandle, window: Window) -> Result<(), String> {
    security::require_window(&window, &["demo-weather", "demo-usage", "demo-projects"])?;
    exit_demo_mode_internal(&app);
    Ok(())
}

#[tauri::command]
fn toggle_expanded_player(handle: tauri::AppHandle, window: Window) -> Result<(), String> {
    security::require_window(&window, &["main"])?;
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
                let _ = window
                    .set_size(tauri::Size::Physical(tauri::PhysicalSize {
                        width: width as u32,
                        height: height as u32,
                    }))
                    .ok();
                let _ = window
                    .set_position(tauri::Position::Physical(tauri::PhysicalPosition { x, y }))
                    .ok();
            }
            let _ = window.show().ok();
            let _ = window.set_shadow(false).ok();
            let _ = window.set_focus().ok();
        }
    }
    Ok(())
}

#[tauri::command]
fn toggle_terminal_panel(handle: tauri::AppHandle, window: Window) -> Result<(), String> {
    security::require_window(&window, &["main", "terminal-panel"])?;
    toggle_terminal_panel_internal(&handle);
    Ok(())
}

fn toggle_terminal_panel_internal(handle: &tauri::AppHandle) {
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
                let _ = window
                    .set_size(tauri::Size::Physical(tauri::PhysicalSize {
                        width: width as u32,
                        height: height as u32,
                    }))
                    .ok();
                let _ = window
                    .set_position(tauri::Position::Physical(tauri::PhysicalPosition { x, y }))
                    .ok();
            }
            let _ = window.show().ok();
            let _ = window.set_shadow(false).ok();
            let _ = window.set_focus().ok();
        }
    }
}

fn fnv1a(value: &str) -> u64 {
    value.bytes().fold(14695981039346656037u64, |hash, byte| {
        hash.wrapping_mul(1099511628211) ^ byte as u64
    })
}

mod native_windows {
    use super::*;

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
            ) == 0
            {
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
                    biCompression: BI_RGB.0,
                    ..Default::default()
                },
                ..Default::default()
            };

            let mut buffer = vec![0u8; (width * height * 4) as usize];

            if GetDIBits(
                hdc_screen,
                h_bitmap,
                0,
                height as u32,
                Some(buffer.as_mut_ptr() as *mut _),
                &mut bitmap_info,
                DIB_RGB_COLORS,
            ) == 0
            {
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
            Some(format!(
                "data:image/bmp;base64,{}",
                base64::engine::general_purpose::STANDARD.encode(&bmp_file)
            ))
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
                SHGFI_ICON | SHGFI_LARGEICON,
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

            Some(String::from_utf16_lossy(std::slice::from_raw_parts(
                ptr, len,
            )))
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
            )
            .ok()?;

            let bitmap = item
                .GetImage(SIZE { cx: 32, cy: 32 }, SIIGBF_BIGGERSIZEOK)
                .ok()?;
            let icon = bitmap_handle_as_base64(bitmap);
            let _ = DeleteObject(bitmap);
            icon
        }
    }

    fn parse_icon_resource(resource: &str) -> Option<(String, i32)> {
        let trimmed = resource
            .trim()
            .trim_matches('"')
            .trim_start_matches('@')
            .trim();
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
            ) == 0
                || large_icon.is_invalid()
            {
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

        if let Some(relaunch_command) =
            window_property_string(hwnd, &PKEY_AppUserModel_RelaunchCommand)
        {
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
            .and_then(extract_icon_resource_as_base64)
        {
            return Some(ResolvedIcon {
                data_uri: icon,
                source: "relaunch-icon".to_string(),
            });
        }

        if let Some(icon) = app_id
            .as_deref()
            .and_then(extract_apps_folder_icon_as_base64)
        {
            return Some(ResolvedIcon {
                data_uri: icon,
                source: "aumid-apps-folder".to_string(),
            });
        }

        if let Some(icon) = process_path.and_then(extract_icon_as_base64) {
            return Some(ResolvedIcon {
                data_uri: icon,
                source: "process-exe".to_string(),
            });
        }

        if let Some(icon) = extract_window_icon_as_base64(hwnd) {
            return Some(ResolvedIcon {
                data_uri: icon,
                source: "hwnd-window-icon".to_string(),
            });
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
    pub fn get_window_thumbnail(
        hwnd: isize,
        window: Window,
        registry: tauri::State<'_, WindowRegistry>,
    ) -> Result<Option<String>, String> {
        security::require_window(&window, &["main"])?;
        registry.require_known(hwnd)?;
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
                std::mem::size_of::<RECT>() as u32,
            )
            .is_ok();

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
            let scale = (max_width as f32 / content_width as f32)
                .min(max_height as f32 / content_height as f32)
                .min(1.0);
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

            if full_dc.is_invalid()
                || thumb_dc.is_invalid()
                || full_bitmap.is_invalid()
                || thumb_bitmap.is_invalid()
            {
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

    // ── Taskbar/Window Management ──────────────────────────────────────
    #[derive(Serialize, Clone, Debug)]
    pub struct WindowInfo {
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
    pub struct IconService {
        cache: Arc<Mutex<HashMap<String, IconRecord>>>,
        cache_dir: Arc<PathBuf>,
    }

    impl IconService {
        pub fn new() -> Self {
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
            let combined = format!("{ICON_CACHE_VERSION}:{key}");
            self.cache_dir.join(format!("{:016x}.txt", fnv1a(&combined)))
        }

        fn resolve<F>(&self, key: &str, resolver: F) -> (Option<String>, String)
        where
            F: FnOnce() -> Option<ResolvedIcon>,
        {
            if let Ok(cache) = self.cache.lock() {
                if let Some(record) = cache.get(key) {
                    return (
                        Some(record.data_uri.clone()),
                        format!("memory/{}", record.source),
                    );
                }
            }

            let path = self.cache_path(key);
            if let Ok(raw) = fs::read_to_string(&path) {
                let mut parts = raw.splitn(2, '\n');
                let source = parts.next().unwrap_or("disk-cache").to_string();
                let data_uri = parts.next().unwrap_or("").trim().to_string();

                if data_uri.starts_with("data:image/") {
                    let record = IconRecord {
                        data_uri: data_uri.clone(),
                        source: source.clone(),
                    };
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
    pub struct WindowRegistry {
        windows: Arc<Mutex<Vec<WindowInfo>>>,
        signature: Arc<Mutex<String>>,
        order_keys: Arc<Mutex<Vec<String>>>,
    }

    impl WindowRegistry {
        pub fn new() -> Self {
            Self {
                windows: Arc::new(Mutex::new(Vec::new())),
                signature: Arc::new(Mutex::new(String::new())),
                order_keys: Arc::new(Mutex::new(Vec::new())),
            }
        }

        fn apply_stable_order(&self, windows: &mut [WindowInfo]) {
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

        fn require_known(&self, hwnd: isize) -> Result<(), String> {
            let windows = self.windows.lock().map_err(|e| e.to_string())?;
            if windows.iter().any(|window| window.hwnd == hwnd) {
                Ok(())
            } else {
                Err("unknown or stale window handle".to_string())
            }
        }
    }

    fn stable_window_order_key(window: &WindowInfo) -> String {
        format!("hwnd:{}", window.hwnd)
    }

    #[tauri::command]
    pub fn get_open_windows(
        app: AppHandle,
        window: Window,
        registry: tauri::State<'_, WindowRegistry>,
        icons: tauri::State<'_, IconService>,
    ) -> Result<Vec<WindowInfo>, String> {
        security::require_window(&window, &["main"])?;
        refresh_window_registry(Some(&app), &registry, &icons)
    }

    #[tauri::command]
    pub fn get_window_debug_snapshot(
        app: AppHandle,
        window: Window,
        registry: tauri::State<'_, WindowRegistry>,
        icons: tauri::State<'_, IconService>,
    ) -> Result<Vec<WindowInfo>, String> {
        security::require_window(&window, &["settings"])?;
        refresh_window_registry(Some(&app), &registry, &icons)
    }

    #[tauri::command]
    pub fn clear_icon_cache(
        icons: tauri::State<'_, IconService>,
        window: Window,
    ) -> Result<(), String> {
        security::require_window(&window, &["settings"])?;
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
            .map(|win| {
                format!(
                    "{}:{}:{}:{}",
                    win.hwnd, win.identity_key, win.is_active, win.icon_source
                )
            })
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

    pub fn start_window_registry_thread(
        app: AppHandle,
        registry: WindowRegistry,
        icons: IconService,
        shutdown: Arc<AtomicBool>,
    ) {
        thread::spawn(move || {
            while !shutdown.load(Ordering::Relaxed) {
                let _ = refresh_window_registry(Some(&app), &registry, &icons);
                thread::sleep(Duration::from_millis(1200));
            }
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
            let _ = EnumWindows(
                Some(enum_windows_proc),
                windows::Win32::Foundation::LPARAM(&mut context as *mut _ as isize),
            );
        }

        Ok(windows)
    }

    unsafe extern "system" fn enum_windows_proc(
        hwnd: HWND,
        lparam: windows::Win32::Foundation::LPARAM,
    ) -> windows::Win32::Foundation::BOOL {
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

                if let Ok(process_handle) =
                    OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, process_id)
                {
                    let mut path_chars: [u16; 1024] = [0; 1024];
                    let mut size = path_chars.len() as u32;
                    if QueryFullProcessImageNameW(
                        process_handle,
                        PROCESS_NAME_WIN32,
                        windows::core::PWSTR(path_chars.as_mut_ptr()),
                        &mut size,
                    )
                    .is_ok()
                    {
                        let full_path = String::from_utf16_lossy(&path_chars[..size as usize]);
                        if let Some(name) = std::path::Path::new(&full_path).file_name() {
                            app_name = name.to_string_lossy().to_string();
                        }
                        process_path = Some(full_path);
                    }
                    let _ = CloseHandle(process_handle);
                }

                let app_id = window_property_string(hwnd, &PKEY_AppUserModel_ID);
                let relaunch_command =
                    window_property_string(hwnd, &PKEY_AppUserModel_RelaunchCommand);
                let relaunch_icon =
                    window_property_string(hwnd, &PKEY_AppUserModel_RelaunchIconResource);
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
    pub fn focus_window(
        hwnd: isize,
        window: Window,
        registry: tauri::State<'_, WindowRegistry>,
    ) -> Result<(), String> {
        security::require_window(&window, &["main"])?;
        registry.require_known(hwnd)?;
        let h = HWND(hwnd as *mut _);
        unsafe {
            let _ = ShowWindow(h, SW_RESTORE);
            let _ = SetForegroundWindow(h);
        }
        Ok(())
    }

    // ── AppBar ──────────────────────────────────────────────────────────

    #[tauri::command]
    pub fn close_window(
        hwnd: isize,
        window: Window,
        registry: tauri::State<'_, WindowRegistry>,
    ) -> Result<(), String> {
        security::require_window(&window, &["main"])?;
        registry.require_known(hwnd)?;
        unsafe {
            let hwnd = HWND(hwnd as *mut _);
            let _ = SendMessageTimeoutW(
                hwnd,
                windows::Win32::UI::WindowsAndMessaging::WM_CLOSE,
                WPARAM(0),
                LPARAM(0),
                SMTO_ABORTIFHUNG,
                1000,
                None,
            );
        }
        Ok(())
    }

    #[tauri::command]
    pub fn set_preview_mode(
        active: bool,
        state: tauri::State<'_, Arc<AtomicBool>>,
        window: Window,
    ) -> Result<(), String> {
        security::require_window(&window, &["main"])?;
        state.store(active, Ordering::Relaxed);
        Ok(())
    }
}

// ── Main ────────────────────────────────────────────────────────────

fn main() {
    let _ = env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or("aeropeks=info"),
    )
    .try_init();
    let shared: SharedSettings = Arc::new(Mutex::new(AppSettings::default()));

    let terminal_state = terminal::TerminalState::new();

    let icon_service = native_windows::IconService::new();
    let window_registry = native_windows::WindowRegistry::new();
    let preview_active: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
    let shutdown: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));

    tauri::Builder::default()
        .manage(shared.clone())
        .manage(media::MediaState::default())
        .manage(system_status::PrivacyState::default())
        .manage(terminal_state)
        .manage(icon_service.clone())
        .manage(preview_active.clone())
        .manage(ShutdownState(shutdown.clone()))
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
                // 1. Toggle terminal panel ON if hidden
                if let Some(window) = handle.get_webview_window("terminal-panel") {
                    if !window.is_visible().unwrap_or(false) {
                        toggle_terminal_panel_internal(handle);
                    }
                }
                // 2. Emit start-session to the terminal window
                let _ = handle.emit_to("terminal-panel", "start-session", PtyPayload { data: serde_json::to_string(&s.cmd).unwrap_or_default() });
            }
        })
        .setup(move |app| {
            let app_handle = app.handle().clone();
            let app_handle_media = app.handle().clone();

            // Load real settings
            let initial_settings = settings::load(app.handle());
            {
                if let Ok(mut lock) = shared.lock() {
                    *lock = initial_settings.clone();
                }
            }

            shell::set_native_taskbar_visible(!initial_settings.hide_native_taskbar);
            native_windows::start_window_registry_thread(
                app.handle().clone(),
                window_registry.clone(),
                icon_service.clone(),
                shutdown.clone(),
            );

            if let Some(window) = app.get_webview_window("main") {
                shell::configure_main_window(
                    window,
                    initial_settings.reserve_screen_space,
                    shutdown.clone(),
                )?;
            }

            let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let settings_i =
                MenuItem::with_id(app, "settings", "Settings", true, None::<&str>)?;
            let sep_i = PredefinedMenuItem::separator(app)?;
            let exit_demo_i =
                MenuItem::with_id(app, "exit-demo", "Exit Screenshot Mode", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&settings_i, &sep_i, &exit_demo_i, &quit_i])?;

            let tray_builder = TrayIconBuilder::new()
                .menu(&menu)
                .show_menu_on_left_click(true)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "quit" => {
                        let _ = shell::restore(app);
                        app.exit(0);
                    }
                    "settings" => {
                        open_settings_internal(app);
                    }
                    "exit-demo" => {
                        exit_demo_mode_internal(app);
                    }
                    _ => {}
                });

            let mut tray = tray_builder;
            if let Some(icon) = app.default_window_icon() { tray = tray.icon(icon.clone()); }
            let _ = tray.build(app);

            let foreground_shutdown = shutdown.clone();
            thread::spawn(move || {
                let mut last_title = String::new();
                while !foreground_shutdown.load(Ordering::Relaxed) {
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
            let media_shutdown = shutdown.clone();
            tauri::async_runtime::spawn(async move {
                use windows::Media::Control::GlobalSystemMediaTransportControlsSessionManager;
                use windows::Foundation::TypedEventHandler;

                if let Ok(manager_op) = GlobalSystemMediaTransportControlsSessionManager::RequestAsync() {
                    if let Ok(manager) = manager_op.get() {
                        let h1 = app_handle_media.clone();
                        let _ = manager.CurrentSessionChanged(&TypedEventHandler::new(move |_, _| {
                            let h2 = h1.clone();
                            tauri::async_runtime::spawn(async move {
                                if let Ok(update) = media::active_media(h2.clone()).await {
                                    let _ = h2.emit("media-change", update);
                                }
                            });
                            Ok(())
                        }));

                        let h3 = app_handle_media.clone();
                        let _ = manager.SessionsChanged(&TypedEventHandler::new(move |_, _| {
                            let h4 = h3.clone();
                            tauri::async_runtime::spawn(async move {
                                if let Ok(update) = media::active_media(h4.clone()).await {
                                    let _ = h4.emit("media-change", update);
                                }
                            });
                            Ok(())
                        }));

                        // Periodic fallback/refresh
                        let h_poll = app_handle_media.clone();
                        let poll_shutdown = media_shutdown.clone();
                        tauri::async_runtime::spawn(async move {
                            let mut interval = interval(Duration::from_secs(5));
                            while !poll_shutdown.load(Ordering::Relaxed) {
                                interval.tick().await;
                                // println!("DEBUG: Polling loop tick");
                                if let Ok(update) = media::active_media(h_poll.clone()).await {
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
                                toggle_launcher_internal(app);
                            } else {
                                if let Ok(settings) = app.state::<SharedSettings>().lock() {
                                    for s in &settings.terminal_shortcuts {
                                        if let Ok(registered) = s.shortcut.parse::<tauri_plugin_global_shortcut::Shortcut>() {
                                            if &registered == shortcut {
                                                let cmd = s.cmd.clone();
                                                toggle_terminal_panel_internal(app);
                                                let _ = app.emit_to(
                                                    "terminal-panel",
                                                    "start-session",
                                                    PtyPayload {
                                                        data: serde_json::to_string(&cmd)
                                                            .unwrap_or_default(),
                                                    },
                                                );
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
                        window
                            .state::<ShutdownState>()
                            .0
                            .store(true, Ordering::Relaxed);
                        let _ = shell::restore(window.app_handle());
                    }
                }
                tauri::WindowEvent::Destroyed => {
                    if window.label() == "main" {
                        window
                            .state::<ShutdownState>()
                            .0
                            .store(true, Ordering::Relaxed);
                        let _ = shell::restore(window.app_handle());
                    }
                }
                _ => {}
            }
        })
        .invoke_handler(tauri::generate_handler![
            system_status::get_volume,
            system_status::set_volume,
            media::get_media_info_unified,
            media::media_control_unified,
            get_settings,
            save_settings,
            open_settings,
            toggle_expanded_player,
            media::get_album_art,
            toggle_terminal_panel,
            terminal::start_pty,
            terminal::write_pty,
            terminal::resize_pty,
            show_terminal_context_menu,
            launcher::search_query,
            launcher::launch_result,
            toggle_launcher,
            system_status::get_battery_status,
            system_power_action,
            system_status::get_mic_status,
            system_status::toggle_mic_mute,
            system_status::get_privacy_status,
            system_status::set_privacy_mode,
            integrations::get_weather,
            integrations::search_locations,
            system_status::get_bluetooth_status,
            integrations::get_obs_status,
            integrations::get_usage_limits,
            projects::get_projects,
            projects::open_project_url,
            register_hotkeys,
            set_window_height,
            terminal::kill_pty,
            native_windows::get_open_windows,
            native_windows::get_window_debug_snapshot,
            native_windows::clear_icon_cache,
            shell::restore_shell_state,
            native_windows::get_window_thumbnail,
            native_windows::close_window,
            native_windows::focus_window,
            get_virtual_desktop_status,
            switch_virtual_desktop,
            native_windows::set_preview_mode,
            open_demo_mode,
            exit_demo_mode
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
