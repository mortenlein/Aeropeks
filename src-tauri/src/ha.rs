//! Home Assistant integration: one background poller fetches the bulk
//! `/api/states` endpoint and derives every HA module's status from that
//! single response, instead of one REST request per entity.

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, State, Window};
use tokio::sync::Notify;

use crate::http::HttpClient;
use crate::security;
use crate::settings::{AppSettings, MowerModule, SharedSettings};

#[derive(Serialize, Clone)]
pub struct VacuumStatus {
    state: String,
    battery: u8,
    charging: bool,
    cleaning: bool,
    cleaning_progress: u8,
    status: String,
    selected_map: String,
}

#[derive(Serialize, Clone)]
pub struct HaMowerStatus {
    state: String,
    state_label: String,
    firmware: String,
    cleaning_count: u32,
    total_area_m2: u32,
    total_time_min: u32,
    dnd: bool,
    zone_id: String,
    zone_state: String,
    has_update: bool,
}

#[derive(Serialize, Clone)]
pub struct PhoneStatus {
    battery: u8,
    charging: bool,
    battery_state: String,
    charge_time_min: i32,
    at_home: bool,
    wifi_ssid: String,
    activity: String,
}

#[derive(Serialize, Clone, Default)]
pub struct HaSnapshot {
    vacuum: Option<VacuumStatus>,
    mower: Option<HaMowerStatus>,
    phone: Option<PhoneStatus>,
}

/// Last snapshot produced by the poller; served to the frontend on startup.
#[derive(Clone, Default)]
pub struct HaState {
    snapshot: Arc<Mutex<HaSnapshot>>,
}

/// Nudged by save_settings so config changes apply without waiting a cycle.
#[derive(Default)]
pub struct HaRefresh(pub Notify);

#[tauri::command]
pub fn get_ha_snapshot(window: Window, state: State<'_, HaState>) -> Result<HaSnapshot, String> {
    security::require_window(&window, &["main"])?;
    Ok(state.snapshot.lock().map_err(|e| e.to_string())?.clone())
}

pub fn start_poller(app: AppHandle, shutdown: Arc<AtomicBool>) {
    tauri::async_runtime::spawn(async move {
        while !shutdown.load(Ordering::Relaxed) {
            let interval = poll_once(&app).await;
            let refresh = app.state::<HaRefresh>();
            tokio::select! {
                _ = tokio::time::sleep(interval) => {}
                _ = refresh.0.notified() => {}
            }
        }
    });
}

async fn poll_once(app: &AppHandle) -> Duration {
    let settings = app
        .state::<SharedSettings>()
        .lock()
        .map(|guard| guard.clone())
        .ok();
    let Some(settings) = settings else {
        return Duration::from_secs(30);
    };
    let interval =
        Duration::from_secs(u64::from(settings.homeassistant_poll_seconds.clamp(5, 600)));

    // None means HA was unreachable — keep the last snapshot rather than
    // flickering the bar items away during an outage.
    if let Some(snapshot) = build_snapshot(app, &settings).await {
        if let Ok(mut current) = app.state::<HaState>().snapshot.lock() {
            *current = snapshot.clone();
        }
        let _ = app.emit("ha-snapshot", snapshot);
    }
    interval
}

async fn build_snapshot(app: &AppHandle, settings: &AppSettings) -> Option<HaSnapshot> {
    let m = &settings.modules;
    let vacuum_on =
        m.vacuum.enabled && security::validate_ha_entity_id(&m.vacuum.entity_id).is_ok();
    let mower_on = m.mower.enabled && security::validate_ha_entity_id(&m.mower.entity_id).is_ok();
    let phone_on = m.phone.enabled && security::validate_ha_slug(&m.phone.device_slug).is_ok();

    if settings.homeassistant_url.is_empty()
        || settings.homeassistant_token.is_empty()
        || (!vacuum_on && !mower_on && !phone_on)
    {
        return Some(HaSnapshot::default());
    }

    let states = fetch_all_states(app, settings).await?;
    Some(HaSnapshot {
        vacuum: vacuum_on.then(|| vacuum_from(&states, &m.vacuum.entity_id)),
        mower: mower_on.then(|| mower_from(&states, &m.mower)),
        phone: phone_on.then(|| phone_from(&states, &m.phone.device_slug)),
    })
}

async fn fetch_all_states(
    app: &AppHandle,
    settings: &AppSettings,
) -> Option<HashMap<String, String>> {
    #[derive(serde::Deserialize)]
    struct EntityState {
        entity_id: String,
        state: String,
    }

    let response = crate::http::client(app)
        .get(format!(
            "{}/api/states",
            settings.homeassistant_url.trim_end_matches('/')
        ))
        .header(
            "Authorization",
            format!("Bearer {}", settings.homeassistant_token),
        )
        .timeout(Duration::from_secs(15))
        .send()
        .await
        .ok()?;
    if !response.status().is_success() {
        return None;
    }
    let entities: Vec<EntityState> = response.json().await.ok()?;
    Some(
        entities
            .into_iter()
            .map(|entity| (entity.entity_id, entity.state))
            .collect(),
    )
}

fn state_of(states: &HashMap<String, String>, entity: &str) -> String {
    states.get(entity).cloned().unwrap_or_default()
}

fn object_id(entity_id: &str) -> &str {
    entity_id
        .split_once('.')
        .map(|(_, o)| o)
        .unwrap_or_default()
}

fn vacuum_from(states: &HashMap<String, String>, entity_id: &str) -> VacuumStatus {
    let obj = object_id(entity_id);
    VacuumStatus {
        state: state_of(states, entity_id),
        battery: state_of(states, &format!("sensor.{obj}_battery"))
            .parse()
            .unwrap_or(0),
        charging: state_of(states, &format!("binary_sensor.{obj}_charging")) == "on",
        cleaning: state_of(states, &format!("binary_sensor.{obj}_cleaning")) == "on",
        cleaning_progress: state_of(states, &format!("sensor.{obj}_cleaning_progress"))
            .parse()
            .unwrap_or(0),
        status: state_of(states, &format!("sensor.{obj}_status")),
        selected_map: state_of(states, &format!("select.{obj}_selected_map")),
    }
}

fn mower_from(states: &HashMap<String, String>, module: &MowerModule) -> HaMowerStatus {
    let obj = object_id(&module.entity_id);
    let state = state_of(states, &module.entity_id);
    let state_label = match state.as_str() {
        "mowing" => "Mowing",
        "docked" => "Docked",
        "paused" => "Paused",
        "error" => "Error",
        _ => "Unknown",
    }
    .to_string();
    let has_update = security::validate_ha_entity_id(&module.update_entity_id).is_ok()
        && state_of(states, &module.update_entity_id) == "on";

    HaMowerStatus {
        state,
        state_label,
        firmware: state_of(states, &format!("sensor.{obj}_firmware_version")),
        cleaning_count: state_of(states, &format!("sensor.{obj}_cleaning_count"))
            .parse()
            .unwrap_or(0),
        total_area_m2: state_of(states, &format!("sensor.{obj}_total_cleaned_area"))
            .parse()
            .unwrap_or(0),
        total_time_min: state_of(states, &format!("sensor.{obj}_total_cleaning_time"))
            .parse()
            .unwrap_or(0),
        dnd: state_of(states, &format!("switch.{obj}_dnd")) == "on",
        zone_id: state_of(states, &format!("sensor.{obj}_current_zone_id")),
        zone_state: state_of(states, &format!("sensor.{obj}_current_zone_state")),
        has_update,
    }
}

fn phone_from(states: &HashMap<String, String>, slug: &str) -> PhoneStatus {
    let battery_state = state_of(states, &format!("sensor.{slug}_battery_state"));
    let charging = state_of(states, &format!("binary_sensor.{slug}_is_charging")) == "on"
        || matches!(battery_state.as_str(), "charging" | "full");

    PhoneStatus {
        battery: state_of(states, &format!("sensor.{slug}_battery_level"))
            .parse()
            .unwrap_or(0),
        charging,
        battery_state,
        charge_time_min: state_of(states, &format!("sensor.{slug}_remaining_charge_time"))
            .parse()
            .unwrap_or(-1),
        at_home: state_of(states, &format!("device_tracker.{slug}")) == "home",
        wifi_ssid: state_of(states, &format!("sensor.{slug}_wifi_connection")),
        activity: state_of(states, &format!("sensor.{slug}_activity")),
    }
}

// ── Calendar ─────────────────────────────────────────────────────────

#[derive(Serialize, Clone, Debug)]
pub struct CalendarEvent {
    summary: String,
    start: String,
    end: String,
    all_day: bool,
    description: String,
    location: String,
}

#[tauri::command]
pub async fn get_calendar_events(
    start: String,
    end: String,
    window: Window,
    state: State<'_, SharedSettings>,
    http: State<'_, HttpClient>,
) -> Result<Option<Vec<CalendarEvent>>, String> {
    security::require_window(&window, &["main"])?;

    let (url, token, module) = {
        let s = state.lock().map_err(|e| e.to_string())?;
        (
            s.homeassistant_url.clone(),
            s.homeassistant_token.clone(),
            s.modules.calendar.clone(),
        )
    };

    if !module.enabled || module.entity_id.is_empty() || url.is_empty() || token.is_empty() {
        return Ok(None);
    }
    security::validate_ha_entity_id(&module.entity_id)?;

    let api_url = format!(
        "{}/api/calendars/{}?start={}&end={}",
        url.trim_end_matches('/'),
        module.entity_id,
        urlencoding::encode(&start),
        urlencoding::encode(&end),
    );

    let resp = http
        .0
        .get(&api_url)
        .header("Authorization", format!("Bearer {token}"))
        .header("Content-Type", "application/json")
        .timeout(Duration::from_secs(8))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !resp.status().is_success() {
        return Ok(Some(Vec::new()));
    }

    let raw: Vec<serde_json::Value> = resp.json().await.map_err(|e| e.to_string())?;

    let events = raw
        .iter()
        .filter_map(|e| {
            let start_obj = e.get("start")?;
            let end_obj = e.get("end")?;

            let (start_str, all_day) =
                if let Some(dt) = start_obj.get("dateTime").and_then(|v| v.as_str()) {
                    (dt.to_string(), false)
                } else if let Some(d) = start_obj.get("date").and_then(|v| v.as_str()) {
                    (d.to_string(), true)
                } else {
                    return None;
                };

            let end_str = if let Some(dt) = end_obj.get("dateTime").and_then(|v| v.as_str()) {
                dt.to_string()
            } else {
                end_obj
                    .get("date")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string()
            };

            Some(CalendarEvent {
                summary: e
                    .get("summary")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                start: start_str,
                end: end_str,
                all_day,
                description: e
                    .get("description")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                location: e
                    .get("location")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
            })
        })
        .collect();

    Ok(Some(events))
}

// ── Camera ───────────────────────────────────────────────────────────

#[tauri::command]
pub async fn get_ha_camera_snapshot(
    window: Window,
    state: State<'_, SharedSettings>,
    http: State<'_, HttpClient>,
) -> Result<String, String> {
    security::require_window(&window, &["main"])?;

    let (url, token, module) = {
        let s = state.lock().map_err(|e| e.to_string())?;
        (
            s.homeassistant_url.clone(),
            s.homeassistant_token.clone(),
            s.modules.camera.clone(),
        )
    };

    if !module.enabled || module.entity_id.is_empty() || url.is_empty() || token.is_empty() {
        return Err("Home Assistant camera not configured".to_string());
    }
    security::validate_ha_entity_id(&module.entity_id)?;

    let response = http
        .0
        .get(format!(
            "{}/api/camera_proxy/{}",
            url.trim_end_matches('/'),
            module.entity_id
        ))
        .header("Authorization", format!("Bearer {token}"))
        .timeout(Duration::from_secs(8))
        .send()
        .await
        .map_err(|e| format!("Camera request failed: {e}"))?;

    if !response.status().is_success() {
        return Err(format!("Camera returned {}", response.status()));
    }

    let bytes = response.bytes().await.map_err(|e| e.to_string())?;
    use base64::{engine::general_purpose, Engine as _};
    Ok(general_purpose::STANDARD.encode(&bytes))
}
