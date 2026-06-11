//! Shortcuts: user-pinned websites surfaced in a bar dropdown. URLs are
//! persisted with the other settings; favicons are fetched through the
//! backend (the webview CSP blocks external images) and cached to disk.

use std::path::PathBuf;
use std::time::Duration;

use tauri::{AppHandle, Emitter, State, Window};

use crate::http::HttpClient;
use crate::security;
use crate::settings::{self, PinnedShortcut, SharedSettings};

const MAX_SHORTCUTS: usize = 8;
const MAX_ICON_BYTES: usize = 262_144;

fn validate_shortcut_url(url: &str) -> Result<(), String> {
    if url.len() > 2048 {
        return Err("shortcut URL is too long".to_string());
    }
    let parsed = reqwest::Url::parse(url).map_err(|e| format!("invalid shortcut URL: {e}"))?;
    if !matches!(parsed.scheme(), "http" | "https") {
        return Err("only http(s) shortcuts are supported".to_string());
    }
    let host = parsed.host_str().unwrap_or_default();
    if !host.contains('.') {
        return Err("shortcut URL needs a full hostname".to_string());
    }
    Ok(())
}

fn validate_shortcut_id(id: &str) -> Result<(), String> {
    if !id.is_empty()
        && id.len() <= 40
        && id.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-')
    {
        Ok(())
    } else {
        Err("invalid shortcut id".to_string())
    }
}

#[tauri::command]
pub fn set_pinned_shortcuts(
    shortcuts: Vec<PinnedShortcut>,
    handle: AppHandle,
    window: Window,
    state: State<'_, SharedSettings>,
) -> Result<(), String> {
    security::require_window(&window, &["main"])?;
    if shortcuts.len() > MAX_SHORTCUTS {
        return Err(format!("at most {MAX_SHORTCUTS} shortcuts are allowed"));
    }
    for shortcut in &shortcuts {
        validate_shortcut_id(&shortcut.id)?;
        validate_shortcut_url(&shortcut.url)?;
    }

    let updated = {
        let mut current = state.lock().map_err(|e| e.to_string())?;
        current.pinned_shortcuts = shortcuts;
        current.clone()
    };
    settings::save(&handle, &updated)?;
    handle
        .emit_to("main", "settings-changed", updated.without_secrets())
        .ok();
    Ok(())
}

/// Opens a pinned shortcut in the default browser. Takes the id rather than a
/// URL so only saved, validated shortcuts can ever be opened from the bar.
#[tauri::command]
pub fn open_shortcut(
    id: String,
    window: Window,
    state: State<'_, SharedSettings>,
) -> Result<(), String> {
    security::require_window(&window, &["main"])?;
    let url = state
        .lock()
        .map_err(|e| e.to_string())?
        .pinned_shortcuts
        .iter()
        .find(|shortcut| shortcut.id == id)
        .map(|shortcut| shortcut.url.clone())
        .ok_or("unknown shortcut")?;
    open::that(url).map_err(|e| e.to_string())
}

fn is_safe_host(host: &str) -> bool {
    !host.is_empty()
        && host.len() <= 253
        && host
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'.' || b == b'-')
}

fn favicon_cache_dir() -> PathBuf {
    std::env::var_os("LOCALAPPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir)
        .join("Aeropeks")
        .join("favicon-cache")
}

async fn fetch_icon(client: &reqwest::Client, url: &str) -> Option<String> {
    let response = client
        .get(url)
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .ok()?;
    if !response.status().is_success() {
        return None;
    }
    let content_type = response
        .headers()
        .get("content-type")?
        .to_str()
        .ok()?
        .split(';')
        .next()?
        .trim()
        .to_string();
    if !content_type.starts_with("image/") {
        return None;
    }
    let bytes = response.bytes().await.ok()?;
    if bytes.is_empty() || bytes.len() > MAX_ICON_BYTES {
        return None;
    }
    use base64::Engine;
    Some(format!(
        "data:{content_type};base64,{}",
        base64::engine::general_purpose::STANDARD.encode(&bytes)
    ))
}

/// Returns the site's favicon as a data URI: the site's own /favicon.ico
/// first, Google's favicon service as fallback. Successes are cached to disk
/// so the dropdown never shows resolving slots on later launches.
#[tauri::command]
pub async fn get_favicon(
    url: String,
    window: Window,
    http: State<'_, HttpClient>,
) -> Result<String, String> {
    security::require_window(&window, &["main"])?;
    validate_shortcut_url(&url)?;
    let host = reqwest::Url::parse(&url)
        .map_err(|e| e.to_string())?
        .host_str()
        .unwrap_or_default()
        .to_ascii_lowercase();
    if !is_safe_host(&host) {
        return Err("invalid shortcut host".to_string());
    }

    let cache_path = favicon_cache_dir().join(format!("{host}.uri"));
    if let Ok(cached) = std::fs::read_to_string(&cache_path) {
        if cached.starts_with("data:image/") {
            return Ok(cached);
        }
    }

    let candidates = [
        format!("https://{host}/favicon.ico"),
        format!("https://www.google.com/s2/favicons?domain={host}&sz=64"),
    ];
    for candidate in candidates {
        if let Some(uri) = fetch_icon(&http.0, &candidate).await {
            let _ = std::fs::create_dir_all(favicon_cache_dir());
            let _ = std::fs::write(&cache_path, &uri);
            return Ok(uri);
        }
    }
    Err("no favicon found".to_string())
}

#[cfg(test)]
mod tests {
    use super::{is_safe_host, validate_shortcut_id, validate_shortcut_url};

    #[test]
    fn shortcut_urls_are_http_only_with_real_hosts() {
        assert!(validate_shortcut_url("https://github.com").is_ok());
        assert!(validate_shortcut_url("http://news.ycombinator.com/news").is_ok());
        assert!(validate_shortcut_url("file:///C:/Windows/system32").is_err());
        assert!(validate_shortcut_url("javascript:alert(1)").is_err());
        assert!(validate_shortcut_url("https://localhost").is_err());
        assert!(validate_shortcut_url("not a url").is_err());
    }

    #[test]
    fn shortcut_ids_are_constrained() {
        assert!(validate_shortcut_id("sc-1718100000000").is_ok());
        assert!(validate_shortcut_id("").is_err());
        assert!(validate_shortcut_id("../escape").is_err());
    }

    #[test]
    fn favicon_hosts_are_filename_safe() {
        assert!(is_safe_host("github.com"));
        assert!(is_safe_host("news.ycombinator.com"));
        assert!(!is_safe_host("bad/host"));
        assert!(!is_safe_host("UPPER.com"));
        assert!(!is_safe_host(""));
    }
}
