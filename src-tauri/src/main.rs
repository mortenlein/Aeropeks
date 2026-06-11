// Aeropeks v0.1.0 - Terminal Fix Build
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod ha;
mod http;
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
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::{AppHandle, Emitter, Manager, State, Window};
use tokio::time::interval;

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

    // Wake the HA poller so module changes apply without waiting a cycle.
    handle.state::<ha::HaRefresh>().0.notify_one();

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
    let popover_y = shell::BAR_HEIGHT + 4;
    let demo_layout: &[(&str, i32, i32)] = &[
        ("demo-weather",  w - 480, popover_y),
        ("demo-usage",    w - 880, popover_y),
        ("demo-projects", w - 1330, popover_y),
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
                let y = shell::BAR_HEIGHT + 4;
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
                let y = shell::BAR_HEIGHT + 4;
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

// ── Main ────────────────────────────────────────────────────────────

fn main() {
    let _ = env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or("aeropeks=info"),
    )
    .try_init();
    let shared: SharedSettings = Arc::new(Mutex::new(AppSettings::default()));

    let terminal_state = terminal::TerminalState::new();

    let shutdown: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));

    tauri::Builder::default()
        .manage(shared.clone())
        .manage(http::HttpClient::new())
        .manage(ha::HaState::default())
        .manage(ha::HaRefresh::default())
        .manage(media::MediaState::default())
        .manage(system_status::PrivacyState::default())
        .manage(terminal_state)
        .manage(ShutdownState(shutdown.clone()))
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
            let app_handle_media = app.handle().clone();

            // Load real settings
            let initial_settings = settings::load(app.handle());
            {
                if let Ok(mut lock) = shared.lock() {
                    *lock = initial_settings.clone();
                }
            }

            shell::set_native_taskbar_visible(!initial_settings.hide_native_taskbar);
            ha::start_poller(app.handle().clone(), shutdown.clone());

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

                        // Slow fallback refresh; GSMTC change events are the primary signal.
                        let h_poll = app_handle_media.clone();
                        let poll_shutdown = media_shutdown.clone();
                        tauri::async_runtime::spawn(async move {
                            let mut interval = interval(Duration::from_secs(30));
                            while !poll_shutdown.load(Ordering::Relaxed) {
                                interval.tick().await;
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
            shell::restore_shell_state,
            open_demo_mode,
            exit_demo_mode,
            ha::get_ha_camera_snapshot,
            ha::get_ha_snapshot,
            ha::get_calendar_events
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}