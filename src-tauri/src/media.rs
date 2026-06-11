use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use std::time::Duration;
use tauri::{Manager, Window};

use crate::security;
use crate::settings::SharedSettings;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MediaInfo {
    pub title: String,
    pub artist: String,
    pub album: String,
    pub is_playing: bool,
    pub thumbnail: Option<String>,
    pub duration_ms: u64,
    pub view_offset_ms: u64,
    pub source: String,
    pub session_id: Option<String>,
    pub machine_id: Option<String>,
    pub address: Option<String>,
}

#[derive(Default)]
pub struct MediaState {
    current: Mutex<Option<MediaInfo>>,
}

pub fn select_active(local: Option<MediaInfo>, remote: Option<MediaInfo>) -> Option<MediaInfo> {
    match (local, remote) {
        (Some(local), _) if local.is_playing => Some(local),
        (Some(_), Some(remote)) if remote.is_playing => Some(remote),
        (Some(local), _) => Some(local),
        (None, remote) => remote,
    }
}

async fn fetch_plex_media(
    client: &reqwest::Client,
    plex_url: &str,
    plex_token: &str,
) -> Option<MediaInfo> {
    if plex_url.is_empty() {
        return None;
    }
    let response = client
        .get(format!(
            "{}/status/sessions?X-Plex-Token={}",
            plex_url.trim_end_matches('/'),
            plex_token
        ))
        .header("Accept", "application/json")
        .timeout(Duration::from_secs(3))
        .send()
        .await
        .ok()?;
    if !response.status().is_success() {
        return None;
    }
    let json: serde_json::Value = response.json().await.ok()?;
    for item in json.pointer("/MediaContainer/Metadata")?.as_array()? {
        let title = item
            .get("title")
            .and_then(|value| value.as_str())?
            .to_string();
        if title.is_empty() {
            continue;
        }
        let player = item.get("Player").and_then(|value| value.as_object());
        return Some(MediaInfo {
            title,
            artist: item
                .get("grandparentTitle")
                .and_then(|value| value.as_str())
                .unwrap_or_default()
                .to_string(),
            album: item
                .get("parentTitle")
                .and_then(|value| value.as_str())
                .unwrap_or_default()
                .to_string(),
            thumbnail: item
                .get("thumb")
                .and_then(|value| value.as_str())
                .map(str::to_string),
            duration_ms: item
                .get("duration")
                .and_then(|value| value.as_u64())
                .unwrap_or_default(),
            view_offset_ms: item
                .get("viewOffset")
                .and_then(|value| value.as_u64())
                .unwrap_or_default(),
            is_playing: player
                .and_then(|value| value.get("state"))
                .and_then(|value| value.as_str())
                == Some("playing"),
            source: "plex".to_string(),
            session_id: item
                .get("sessionKey")
                .and_then(|value| value.as_str())
                .map(str::to_string),
            machine_id: player
                .and_then(|value| value.get("machineIdentifier"))
                .and_then(|value| value.as_str())
                .map(str::to_string),
            address: player
                .and_then(|value| value.get("address"))
                .and_then(|value| value.as_str())
                .map(str::to_string),
        });
    }
    None
}

async fn get_gsmtc_media() -> Result<Option<MediaInfo>, String> {
    use windows::Media::Control::{
        GlobalSystemMediaTransportControlsSession, GlobalSystemMediaTransportControlsSessionManager,
    };
    let manager = GlobalSystemMediaTransportControlsSessionManager::RequestAsync()
        .map_err(|e| e.to_string())?
        .get()
        .map_err(|e| e.to_string())?;
    let sessions = manager.GetSessions().map_err(|e| e.to_string())?;
    let mut best: Option<(i32, GlobalSystemMediaTransportControlsSession)> = None;
    for session in sessions {
        let status = session
            .GetPlaybackInfo()
            .ok()
            .and_then(|playback| playback.PlaybackStatus().ok())
            .map(|status| status.0)
            .unwrap_or_default();
        let score = match status {
            4 => 10,
            5 => 5,
            _ => 0,
        };
        if best
            .as_ref()
            .is_none_or(|(best_score, _)| score > *best_score)
        {
            best = Some((score, session));
        }
    }
    let Some((_, session)) = best else {
        return Ok(None);
    };
    let playback = session.GetPlaybackInfo().map_err(|e| e.to_string())?;
    let properties = session
        .TryGetMediaPropertiesAsync()
        .map_err(|e| e.to_string())?
        .get()
        .map_err(|e| e.to_string())?;
    Ok(Some(MediaInfo {
        title: properties.Title().unwrap_or_default().to_string(),
        artist: properties.Artist().unwrap_or_default().to_string(),
        album: properties.AlbumTitle().unwrap_or_default().to_string(),
        is_playing: playback.PlaybackStatus().unwrap_or_default().0 == 4,
        thumbnail: None,
        duration_ms: 0,
        view_offset_ms: 0,
        source: "gsmtc".to_string(),
        session_id: None,
        machine_id: None,
        address: None,
    }))
}

pub async fn active_media(handle: tauri::AppHandle) -> Result<Option<MediaInfo>, String> {
    let local = get_gsmtc_media().await.unwrap_or(None);
    let selected = if local.as_ref().is_some_and(|media| media.is_playing) {
        local
    } else {
        let (url, token) = {
            let state = handle.state::<SharedSettings>();
            let settings = state.lock().map_err(|e| e.to_string())?;
            (settings.plex_url.clone(), settings.plex_token.clone())
        };
        let client = crate::http::client(&handle);
        select_active(local, fetch_plex_media(&client, &url, &token).await)
    };
    *handle
        .state::<MediaState>()
        .current
        .lock()
        .map_err(|e| e.to_string())? = selected.clone();
    Ok(selected)
}

#[tauri::command]
pub async fn get_media_info_unified(
    handle: tauri::AppHandle,
    window: Window,
) -> Result<Option<MediaInfo>, String> {
    security::require_window(&window, &["main", "expanded-player"])?;
    active_media(handle).await
}

#[tauri::command]
pub async fn get_album_art(
    thumb: String,
    state: tauri::State<'_, SharedSettings>,
    http: tauri::State<'_, crate::http::HttpClient>,
    window: Window,
) -> Result<String, String> {
    security::require_window(&window, &["expanded-player"])?;
    if thumb.is_empty() {
        return Ok(String::new());
    }
    let (plex_url, plex_token) = {
        let settings = state.lock().map_err(|e| e.to_string())?;
        (settings.plex_url.clone(), settings.plex_token.clone())
    };
    let url = format!(
        "{}/photo/:/transcode?url={}&width=200&height=200&X-Plex-Token={}",
        plex_url.trim_end_matches('/'),
        urlencoding::encode(&thumb),
        plex_token
    );
    let bytes = http
        .0
        .get(url)
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .and_then(|response| response.error_for_status())
        .map_err(|e| e.to_string())?
        .bytes()
        .await
        .map_err(|e| e.to_string())?;
    use base64::Engine;
    Ok(base64::engine::general_purpose::STANDARD.encode(bytes))
}

async fn plex_control(
    client: reqwest::Client,
    action: &str,
    is_playing: bool,
    machine_id: &str,
    address: &str,
    state: tauri::State<'_, SharedSettings>,
) -> Result<(), String> {
    let (url, token) = {
        let settings = state.lock().map_err(|e| e.to_string())?;
        (settings.plex_url.clone(), settings.plex_token.clone())
    };
    let command = match action {
        "play_pause" if is_playing => "pause",
        "play_pause" => "play",
        "next" => "skipNext",
        "previous" => "skipPrevious",
        _ => return Err("invalid media action".to_string()),
    };
    let command_id = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(1);
    let mut attempts = vec![(format!(
        "{}/player/proxy/playback/{command}?X-Plex-Target-Client-Identifier={machine_id}&X-Plex-Token={token}&commandID={command_id}",
        url.trim_end_matches('/')
    ), false)];
    if !address.is_empty() {
        attempts.push((format!("http://{address}:32500/player/playback/{command}?commandID={command_id}&X-Plex-Token={token}"), true));
    }
    for (url, post) in attempts {
        let request = if post {
            client.post(url)
        } else {
            client.get(url)
        };
        if request
            .timeout(Duration::from_secs(3))
            .header("X-Plex-Client-Identifier", "aeropeks")
            .header("X-Plex-Target-Client-Identifier", machine_id)
            .send()
            .await
            .is_ok_and(|response| response.status().is_success())
        {
            return Ok(());
        }
    }
    Err("all Plex control attempts failed".to_string())
}

async fn gsmtc_action(action: &str) -> Result<(), String> {
    use windows::Media::Control::GlobalSystemMediaTransportControlsSessionManager;
    let manager = GlobalSystemMediaTransportControlsSessionManager::RequestAsync()
        .map_err(|e| e.to_string())?
        .get()
        .map_err(|e| e.to_string())?;
    if let Ok(session) = manager.GetCurrentSession() {
        match action {
            "play_pause" => {
                session
                    .TryTogglePlayPauseAsync()
                    .map_err(|e| e.to_string())?
                    .get()
                    .map_err(|e| e.to_string())?;
            }
            "next" => {
                session
                    .TrySkipNextAsync()
                    .map_err(|e| e.to_string())?
                    .get()
                    .map_err(|e| e.to_string())?;
            }
            "previous" => {
                session
                    .TrySkipPreviousAsync()
                    .map_err(|e| e.to_string())?
                    .get()
                    .map_err(|e| e.to_string())?;
            }
            _ => return Err("invalid media action".to_string()),
        }
    }
    Ok(())
}

#[tauri::command]
pub async fn media_control_unified(
    handle: tauri::AppHandle,
    window: Window,
    action: String,
) -> Result<(), String> {
    security::require_window(&window, &["main", "expanded-player"])?;
    if !matches!(action.as_str(), "play_pause" | "next" | "previous") {
        return Err("invalid media action".to_string());
    }
    let media = handle
        .state::<MediaState>()
        .current
        .lock()
        .map_err(|e| e.to_string())?
        .clone();
    let Some(media) = media else {
        return Ok(());
    };
    let result = if media.source == "plex" {
        plex_control(
            crate::http::client(&handle),
            &action,
            media.is_playing,
            media.machine_id.as_deref().unwrap_or_default(),
            media.address.as_deref().unwrap_or_default(),
            handle.state(),
        )
        .await
    } else {
        gsmtc_action(&action).await
    };
    if result.is_ok() && action == "play_pause" {
        if let Ok(mut current) = handle.state::<MediaState>().current.lock() {
            if let Some(current) = current.as_mut() {
                current.is_playing = !current.is_playing;
            }
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::{select_active, MediaInfo};
    use std::fs;
    use std::path::Path;

    fn media(source: &str, is_playing: bool) -> MediaInfo {
        MediaInfo {
            title: source.to_string(),
            artist: String::new(),
            album: String::new(),
            is_playing,
            thumbnail: None,
            duration_ms: 0,
            view_offset_ms: 0,
            source: source.to_string(),
            session_id: None,
            machine_id: None,
            address: None,
        }
    }

    #[test]
    fn playing_local_media_has_priority() {
        let selected =
            select_active(Some(media("gsmtc", true)), Some(media("plex", true))).unwrap();
        assert_eq!(selected.source, "gsmtc");
    }

    #[test]
    fn playing_remote_media_beats_paused_local_media() {
        let selected =
            select_active(Some(media("gsmtc", false)), Some(media("plex", true))).unwrap();
        assert_eq!(selected.source, "plex");
    }

    #[test]
    fn frontend_media_contract_contains_serialized_fields() {
        let contract_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../src/contracts.ts");
        let contract = fs::read_to_string(contract_path).unwrap();
        for field in [
            "title",
            "artist",
            "album",
            "is_playing",
            "thumbnail",
            "duration_ms",
            "view_offset_ms",
            "source",
            "session_id",
            "machine_id",
            "address",
        ] {
            assert!(contract.contains(&format!("{field}:")));
        }
    }
}
