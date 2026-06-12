use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use tauri::{Manager, Window};

use crate::{platform, security};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SearchResult {
    pub(crate) id: String,
    pub(crate) title: String,
    pub(crate) description: String,
    pub(crate) icon: String,
    pub(crate) action_type: String,
    pub(crate) action_value: String,
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
    results.extend(platform::installed_app_results(&normalized));
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
            platform::validate_app_target(&path)?;
            open::that(path).map_err(|e| e.to_string())?;
        }
        "system" => platform::run_power_action(
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
