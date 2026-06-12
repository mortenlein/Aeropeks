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
        if shortcut.name.len() > 60 {
            return Err("shortcut name is too long".to_string());
        }
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

/// Infer a mime type from the URL path when the server sends a generic
/// content-type (common for .ico files on small self-hosted services).
fn mime_from_path(url: &str) -> Option<&'static str> {
    let path = url.split(['?', '#']).next()?.to_ascii_lowercase();
    if path.ends_with(".ico") {
        Some("image/x-icon")
    } else if path.ends_with(".png") {
        Some("image/png")
    } else if path.ends_with(".svg") {
        Some("image/svg+xml")
    } else if path.ends_with(".jpg") || path.ends_with(".jpeg") {
        Some("image/jpeg")
    } else if path.ends_with(".gif") {
        Some("image/gif")
    } else if path.ends_with(".webp") {
        Some("image/webp")
    } else {
        None
    }
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
    let header_type = response
        .headers()
        .get("content-type")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.split(';').next())
        .map(|value| value.trim().to_ascii_lowercase())
        .unwrap_or_default();
    let content_type = if header_type.starts_with("image/") {
        header_type
    } else if header_type.is_empty() || header_type == "application/octet-stream" {
        mime_from_path(url)?.to_string()
    } else {
        return None;
    };
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

/// Pull an attribute value out of a raw HTML tag without a full parser.
fn extract_attr(tag: &str, name: &str) -> Option<String> {
    let lower = tag.to_ascii_lowercase();
    let needle = format!("{name}=");
    let mut search = 0;
    while let Some(found) = lower[search..].find(&needle) {
        let pos = search + found;
        // Avoid matching inside other attribute names (e.g. data-href=).
        let preceded_ok = lower[..pos]
            .chars()
            .last()
            .is_some_and(|c| c.is_whitespace());
        if !preceded_ok {
            search = pos + needle.len();
            continue;
        }
        let rest = &tag[pos + needle.len()..];
        let value = match rest.chars().next() {
            Some(quote @ ('"' | '\'')) => {
                let inner = &rest[1..];
                inner[..inner.find(quote).unwrap_or(inner.len())].to_string()
            }
            Some(_) => rest[..rest
                .find(|c: char| c.is_whitespace() || c == '>')
                .unwrap_or(rest.len())]
                .to_string(),
            None => String::new(),
        };
        return Some(value);
    }
    None
}

/// Find the first `<link rel="…icon…" href="…">` in a page. This is how
/// self-hosted dashboards (Frigate, Home Assistant, …) declare their icons —
/// they often have no /favicon.ico and are invisible to Google's service.
fn find_icon_href(html: &str) -> Option<String> {
    let lower = html.to_ascii_lowercase();
    let mut from = 0;
    while let Some(found) = lower[from..].find("<link") {
        let start = from + found;
        let end = lower[start..].find('>').map(|e| start + e)?;
        let tag = &html[start..end];
        let rel = extract_attr(tag, "rel")
            .unwrap_or_default()
            .to_ascii_lowercase();
        let is_icon = rel
            .split_whitespace()
            .any(|part| part == "icon" || part == "apple-touch-icon");
        if is_icon {
            if let Some(href) = extract_attr(tag, "href") {
                if !href.is_empty() {
                    return Some(href);
                }
            }
        }
        from = end + 1;
    }
    None
}

/// Fetch the site root and follow its declared icon link.
async fn fetch_declared_icon(client: &reqwest::Client, base: &reqwest::Url) -> Option<String> {
    let root = base.join("/").ok()?;
    let response = client
        .get(root.clone())
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .ok()?;
    if !response.status().is_success() {
        return None;
    }
    let bytes = response.bytes().await.ok()?;
    if bytes.len() > 1_048_576 {
        return None;
    }
    let html = String::from_utf8_lossy(&bytes);
    let href = find_icon_href(&html)?;
    let icon_url = root.join(href.trim()).ok()?;
    if !matches!(icon_url.scheme(), "http" | "https") {
        return None;
    }
    fetch_icon(client, icon_url.as_str()).await
}

/// Returns the site's favicon as a data URI. Strategy: the icon declared in
/// the page's HTML first (works for self-hosted services), then /favicon.ico
/// on the same origin, then Google's favicon service for public sites.
/// Successes are cached to disk so the dropdown never shows resolving slots
/// on later launches.
#[tauri::command]
pub async fn get_favicon(
    url: String,
    window: Window,
    http: State<'_, HttpClient>,
) -> Result<String, String> {
    security::require_window(&window, &["main"])?;
    validate_shortcut_url(&url)?;
    let parsed = reqwest::Url::parse(&url).map_err(|e| e.to_string())?;
    let host = parsed.host_str().unwrap_or_default().to_ascii_lowercase();
    if !is_safe_host(&host) {
        return Err("invalid shortcut host".to_string());
    }
    let port = parsed.port_or_known_default().unwrap_or(443);

    let cache_path = favicon_cache_dir().join(format!("{host}-{port}.uri"));
    if let Ok(cached) = std::fs::read_to_string(&cache_path) {
        if cached.starts_with("data:image/") {
            return Ok(cached);
        }
    }

    let mut icon = fetch_declared_icon(&http.0, &parsed).await;
    if icon.is_none() {
        if let Ok(direct) = parsed.join("/favicon.ico") {
            icon = fetch_icon(&http.0, direct.as_str()).await;
        }
    }
    if icon.is_none() {
        let fallback = format!("https://www.google.com/s2/favicons?domain={host}&sz=64");
        icon = fetch_icon(&http.0, &fallback).await;
    }

    match icon {
        Some(uri) => {
            let _ = std::fs::create_dir_all(favicon_cache_dir());
            let _ = std::fs::write(&cache_path, &uri);
            Ok(uri)
        }
        None => Err("no favicon found".to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        find_icon_href, is_safe_host, mime_from_path, validate_shortcut_id, validate_shortcut_url,
    };

    #[test]
    fn icon_links_are_found_in_page_html() {
        assert_eq!(
            find_icon_href(r#"<html><head><link rel="icon" href="/favicon.svg" /></head>"#),
            Some("/favicon.svg".to_string())
        );
        assert_eq!(
            find_icon_href(r#"<link rel='shortcut icon' href='/static/icons/favicon.ico'>"#),
            Some("/static/icons/favicon.ico".to_string())
        );
        assert_eq!(
            find_icon_href(r#"<link rel="apple-touch-icon" href="/apple.png">"#),
            Some("/apple.png".to_string())
        );
        // stylesheet links and mask icons are not favicons
        assert_eq!(
            find_icon_href(r#"<link rel="stylesheet" href="/x.css">"#),
            None
        );
        assert_eq!(
            find_icon_href(r#"<link rel="mask-icon" href="/m.svg">"#),
            None
        );
        assert_eq!(find_icon_href("<p>no links</p>"), None);
    }

    #[test]
    fn generic_content_types_fall_back_to_path_extension() {
        assert_eq!(
            mime_from_path("https://x.com/favicon.ico"),
            Some("image/x-icon")
        );
        assert_eq!(mime_from_path("https://x.com/i.png?v=2"), Some("image/png"));
        assert_eq!(mime_from_path("https://x.com/page.html"), None);
    }

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
