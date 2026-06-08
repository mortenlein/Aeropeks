use std::os::windows::process::CommandExt;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use tauri::{Manager, Window};
use walkdir::WalkDir;

use crate::security;

const CREATE_NO_WINDOW: u32 = 0x08000000;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SearchResult {
    id: String,
    title: String,
    description: String,
    icon: String,
    action_type: String,
    action_value: String,
}

#[tauri::command]
pub fn search_query(query: String, window: Window) -> Result<Vec<SearchResult>, String> {
    security::require_window(&window, &["launcher-panel"])?;
    let query = query.trim();
    if query.is_empty() || query.len() > 200 {
        return Ok(Vec::new());
    }
    let normalized = query.to_lowercase();
    if normalized.starts_with("g ") || normalized.starts_with("google ") {
        let offset = if normalized.starts_with("g ") { 2 } else { 7 };
        let term = query[offset..].trim();
        return Ok(vec![SearchResult {
            id: "web-google".to_string(),
            title: format!("Search Google for '{term}'"),
            description: "Open in default browser".to_string(),
            icon: "Globe".to_string(),
            action_type: "web".to_string(),
            action_value: format!(
                "https://www.google.com/search?q={}",
                urlencoding::encode(term)
            ),
        }]);
    }

    let mut results = system_results(&normalized);
    for root in start_menu_roots() {
        for entry in WalkDir::new(root).into_iter().filter_map(Result::ok) {
            let path = entry.path();
            let Some(file_name) = path.file_name().and_then(|name| name.to_str()) else {
                continue;
            };
            if path.extension().and_then(|ext| ext.to_str()) == Some("lnk")
                && file_name.to_lowercase().contains(&normalized)
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
    Ok(results)
}

fn system_results(query: &str) -> Vec<SearchResult> {
    [
        ("lock", "Lock Workstation"),
        ("shutdown", "Shut Down"),
        ("restart", "Restart"),
        ("sleep", "Sleep"),
    ]
    .into_iter()
    .filter(|(action, _)| action.contains(query))
    .map(|(action, label)| SearchResult {
        id: format!("sys-{action}"),
        title: label.to_string(),
        description: format!("Execute: {action}"),
        icon: "Settings".to_string(),
        action_type: "system".to_string(),
        action_value: action.to_string(),
    })
    .collect()
}

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

fn allowed_shortcut(path: &Path) -> bool {
    path.extension().and_then(|ext| ext.to_str()) == Some("lnk")
        && start_menu_roots().iter().any(|root| path.starts_with(root))
}

#[tauri::command]
pub fn launch_result(
    handle: tauri::AppHandle,
    window: Window,
    result: SearchResult,
) -> Result<(), String> {
    security::require_window(&window, &["launcher-panel"])?;
    match result.action_type.as_str() {
        "web"
            if result
                .action_value
                .starts_with("https://www.google.com/search?q=") =>
        {
            open::that(result.action_value).map_err(|e| e.to_string())?;
        }
        "app" => {
            let path = PathBuf::from(result.action_value);
            if !allowed_shortcut(&path) {
                return Err("application is outside the Start Menu".to_string());
            }
            open::that(path).map_err(|e| e.to_string())?;
        }
        "system" => run_power_action(
            result
                .id
                .strip_prefix("sys-")
                .ok_or("invalid system action")?,
        )?,
        _ => return Err("unsupported launcher action".to_string()),
    }
    if let Some(window) = handle.get_webview_window("launcher-panel") {
        let _ = window.hide();
    }
    Ok(())
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

#[cfg(test)]
mod tests {
    use super::system_results;

    #[test]
    fn system_search_only_returns_allowlisted_actions() {
        let results = system_results("shut");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "sys-shutdown");
    }
}
