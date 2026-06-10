use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use reqwest::Client;
use serde::{Deserialize, Serialize};
use tauri::State;
use tauri::Window;

use crate::security;
use crate::settings::SharedSettings;

const BASE: &str = "https://eu.iot.dreame.tech:13267";
const SIGN_SALT: &str = "RAylYC%fmSKp7%Tq";
const AUTHORIZATION: &str = "Basic ZHJlYW1lX2FwcHYxOkFQXmR2QHpAU1FZVnhOODg=";
const DREAME_RLC: &str = "1c80b3787b2266776bcdc481f37d8fa42ba10a30af81a6df-1";
const DREAME_USER_AGENT: &str = "Dreame_Smarthome/1.5.59 (iPhone; iOS 16.0; Scale/3.00)";
const TENANT_ID: &str = "000000";

// Battery and mow stats (area, time) are only available via AliyunIoT MQTT,
// not through any cloud REST endpoint. The device/info endpoint provides
// online status and latestStatus reliably.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MowerStatus {
    pub online: bool,
    pub state: u8,
    pub state_label: String,
    pub firmware: String,
}

struct DreameToken {
    access: String,
    refresh: String,
    expires_at: u64,
}

pub struct DreameTokenCache(Mutex<Option<DreameToken>>);

impl Default for DreameTokenCache {
    fn default() -> Self {
        Self(Mutex::new(None))
    }
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn hash_password(password: &str) -> String {
    format!("{:x}", md5::compute(format!("{password}{SIGN_SALT}")))
}

fn state_label(state: u8) -> &'static str {
    match state {
        1 => "Mowing",
        2 => "Standby",
        3 => "Paused",
        4 => "Paused (error)",
        5 => "Returning",
        6 => "Charging",
        11 => "Mapping",
        13 => "Charge complete",
        14 => "Updating",
        _ => "Unknown",
    }
}

#[derive(Deserialize)]
struct TokenResponse {
    access_token: Option<String>,
    refresh_token: Option<String>,
    expires_in: Option<u64>,
    error: Option<String>,
    error_description: Option<String>,
}

#[derive(Deserialize)]
struct DeviceInfoResponse {
    code: i32,
    data: Option<serde_json::Value>,
}

fn dreame_client() -> Result<Client, String> {
    Client::builder()
        .timeout(std::time::Duration::from_secs(8))
        .build()
        .map_err(|e| e.to_string())
}

async fn do_login(client: &Client, username: &str, password: &str) -> Result<DreameToken, String> {
    let pass_hash = hash_password(password);
    let body = format!(
        "platform=IOS&scope=all&grant_type=password&username={username}&password={pass_hash}&type=account"
    );
    let parsed: TokenResponse = client
        .post(format!("{BASE}/dreame-auth/oauth/token"))
        .header("Authorization", AUTHORIZATION)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .header("User-Agent", DREAME_USER_AGENT)
        .header("Tenant-Id", TENANT_ID)
        .body(body)
        .send()
        .await
        .map_err(|e| format!("login request failed: {e}"))?
        .json()
        .await
        .map_err(|e| e.to_string())?;

    if let Some(err) = parsed.error {
        return Err(format!(
            "login failed: {err} — {}",
            parsed.error_description.unwrap_or_default()
        ));
    }
    Ok(DreameToken {
        access: parsed.access_token.ok_or("no access_token in login response")?,
        refresh: parsed.refresh_token.ok_or("no refresh_token in login response")?,
        expires_at: now_secs() + parsed.expires_in.unwrap_or(7200).saturating_sub(300),
    })
}

async fn do_refresh(client: &Client, refresh_token: &str) -> Result<DreameToken, String> {
    let body = format!(
        "platform=IOS&scope=all&grant_type=refresh_token&refresh_token={refresh_token}"
    );
    let parsed: TokenResponse = client
        .post(format!("{BASE}/dreame-auth/oauth/token"))
        .header("Authorization", AUTHORIZATION)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .header("User-Agent", DREAME_USER_AGENT)
        .header("Tenant-Id", TENANT_ID)
        .body(body)
        .send()
        .await
        .map_err(|e| format!("token refresh failed: {e}"))?
        .json()
        .await
        .map_err(|e| e.to_string())?;

    if let Some(err) = parsed.error {
        return Err(format!(
            "refresh failed: {err} — {}",
            parsed.error_description.unwrap_or_default()
        ));
    }
    Ok(DreameToken {
        access: parsed.access_token.ok_or("no access_token in refresh response")?,
        refresh: parsed.refresh_token.ok_or("no refresh_token in refresh response")?,
        expires_at: now_secs() + parsed.expires_in.unwrap_or(7200).saturating_sub(300),
    })
}

async fn get_access_token(
    client: &Client,
    username: &str,
    password: &str,
    cache: &DreameTokenCache,
) -> Result<String, String> {
    let (is_valid, maybe_access, maybe_refresh) = {
        let guard = cache.0.lock().map_err(|e| e.to_string())?;
        match &*guard {
            Some(t) if now_secs() < t.expires_at => (true, Some(t.access.clone()), None),
            Some(t) => (false, None, Some(t.refresh.clone())),
            None => (false, None, None),
        }
    };

    if is_valid {
        return Ok(maybe_access.unwrap());
    }

    if let Some(refresh) = maybe_refresh {
        if let Ok(new_token) = do_refresh(client, &refresh).await {
            let access = new_token.access.clone();
            *cache.0.lock().map_err(|e| e.to_string())? = Some(new_token);
            return Ok(access);
        }
    }

    let new_token = do_login(client, username, password).await?;
    let access = new_token.access.clone();
    *cache.0.lock().map_err(|e| e.to_string())? = Some(new_token);
    Ok(access)
}

async fn fetch_device_status(
    client: &Client,
    access_token: &str,
    device_id: &str,
) -> Result<MowerStatus, String> {
    let parsed: DeviceInfoResponse = client
        .post(format!("{BASE}/dreame-user-iot/iotuserbind/device/info"))
        .header("Authorization", AUTHORIZATION)
        .header("Dreame-Auth", access_token)
        .header("Tenant-Id", TENANT_ID)
        .header("Dreame-Rlc", DREAME_RLC)
        .header("User-Agent", DREAME_USER_AGENT)
        .header("Content-Type", "application/json")
        .json(&serde_json::json!({ "did": device_id }))
        .send()
        .await
        .map_err(|e| format!("device info request failed: {e}"))?
        .json()
        .await
        .map_err(|e| e.to_string())?;

    if parsed.code != 0 {
        return Err(format!("device info returned code {}", parsed.code));
    }
    let data = parsed.data.ok_or("no device data in response")?;
    let state = data.get("latestStatus").and_then(|v| v.as_u64()).unwrap_or(0) as u8;
    Ok(MowerStatus {
        online: data.get("online").and_then(|v| v.as_bool()).unwrap_or(false),
        state,
        state_label: state_label(state).to_string(),
        firmware: data.get("ver").and_then(|v| v.as_str()).unwrap_or_default().to_string(),
    })
}

#[tauri::command]
pub async fn dreame_get_status(
    window: Window,
    settings: State<'_, SharedSettings>,
    cache: State<'_, DreameTokenCache>,
) -> Result<Option<MowerStatus>, String> {
    security::require_window(&window, &["main"])?;

    let (username, password, device_id) = {
        let s = settings.lock().map_err(|e| e.to_string())?;
        (
            s.dreame_username.clone(),
            s.dreame_password.clone(),
            s.dreame_device_id.clone(),
        )
    };

    if username.is_empty() || password.is_empty() || device_id.is_empty() {
        return Ok(None);
    }

    let client = dreame_client()?;
    let token = get_access_token(&client, &username, &password, &cache).await?;
    Ok(Some(fetch_device_status(&client, &token, &device_id).await?))
}
