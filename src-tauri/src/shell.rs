use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use tauri::{AppHandle, Manager, Window};
use windows::Win32::Foundation::{HWND, LPARAM, RECT};
use windows::Win32::Graphics::Dwm::{DwmGetWindowAttribute, DWMWA_EXTENDED_FRAME_BOUNDS};
use windows::Win32::UI::Shell::{
    SHAppBarMessage, ABE_TOP, ABM_NEW, ABM_QUERYPOS, ABM_SETPOS, APPBARDATA,
};
use windows::Win32::UI::WindowsAndMessaging::{
    EnumWindows, GetClassNameW, GetForegroundWindow, GetSystemMetrics, GetWindowLongW,
    GetWindowRect, GetWindowThreadProcessId, IsWindowVisible, SetWindowLongW, SetWindowPos, ShowWindow,
    SystemParametersInfoW, GWL_EXSTYLE, GWL_STYLE, HWND_TOPMOST, SM_CXSCREEN, SM_CYSCREEN,
    SPI_SETWORKAREA, SWP_FRAMECHANGED, SWP_NOACTIVATE, SWP_NOZORDER, SWP_SHOWWINDOW, SW_HIDE,
    SW_SHOW, SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS, WS_CAPTION, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW,
    WS_MAXIMIZE, WS_POPUP,
};

use crate::security;

/// Height of the top bar in physical pixels. Keep in sync with the
/// `--bar-height` / menu-bar height in src/index.css and tauri.conf.json.
pub const BAR_HEIGHT: i32 = 40;

pub fn configure_main_window(
    window: tauri::WebviewWindow,
    reserve_screen_space: bool,
    shutdown: Arc<AtomicBool>,
) -> Result<(), String> {
    let monitor = window
        .primary_monitor()
        .map_err(|e| e.to_string())?
        .ok_or("no primary monitor available")?;
    let width = monitor.size().width;
    window
        .set_size(tauri::Size::Physical(tauri::PhysicalSize {
            width,
            height: BAR_HEIGHT as u32,
        }))
        .map_err(|e| e.to_string())?;
    let _ = window.set_shadow(false);
    let _ = window.set_skip_taskbar(true);
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

pub fn restore(handle: &AppHandle) -> Result<(), String> {
    if let Some(window) = handle.get_webview_window("main") {
        if let Ok(hwnd) = window.hwnd() {
            cleanup_app_bar(HWND(hwnd.0), ABE_TOP);
        }
    }
    set_native_taskbar_visible(true);
    reset_primary_work_area(handle)
}

#[tauri::command]
pub fn restore_shell_state(app: AppHandle, window: Window) -> Result<(), String> {
    security::require_window(&window, &["settings"])?;
    restore(&app)
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
