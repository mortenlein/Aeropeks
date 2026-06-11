use serde::{Deserialize, Serialize};
use tauri::{Manager, Window};

use crate::security;
use crate::settings::SharedSettings;

const USER_AGENT: &str = "Aeropeks/0.1.0 (https://github.com/mortenlein/Aeropeks)";

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct RateLimitWindow {
    label: String,
    used_percent: Option<f32>,
    remaining_percent: Option<f32>,
    resets_at: Option<i64>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct LimitProvider {
    enabled: bool,
    ok: bool,
    plan_type: Option<String>,
    short_window: RateLimitWindow,
    long_window: RateLimitWindow,
    rate_limit_reached_type: Option<String>,
    error: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LimitsSnapshot {
    providers: std::collections::HashMap<String, LimitProvider>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct WeatherDetailed {
    temp: f32,
    symbol: String,
    precip: f32,
    place_name: String,
    hourly: Vec<HourlyForecast>,
    daily: Vec<DailyForecast>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct HourlyForecast {
    time: String,
    temp: f32,
    symbol: String,
    precip: f32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct DailyForecast {
    date: String,
    temp_min: f32,
    temp_max: f32,
    symbol: String,
    humidity: f32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LocationSearchResult {
    name: String,
    lat: f64,
    lon: f64,
    country: String,
    url_path: String,
}

#[derive(Serialize)]
pub struct ObsStatus {
    is_streaming: bool,
    is_recording: bool,
}

#[tauri::command]
pub async fn get_usage_limits(
    window: Window,
    settings: tauri::State<'_, SharedSettings>,
) -> Result<LimitsSnapshot, String> {
    security::require_window(&window, &["main", "demo-usage", "settings"])?;
    let url = settings
        .lock()
        .map_err(|e| e.to_string())?
        .usage_limits_url
        .trim()
        .to_string();
    if url.is_empty() {
        return Err("usage limits URL is not configured".to_string());
    }
    let response = reqwest::Client::builder()
        .user_agent(USER_AGENT)
        .timeout(std::time::Duration::from_secs(4))
        .build()
        .map_err(|e| e.to_string())?
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("usage limits service unavailable: {e}"))?;
    if !response.status().is_success() {
        return Err(format!(
            "usage limits service returned {}",
            response.status()
        ));
    }
    response.json().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_weather(
    lat: f64,
    lon: f64,
    place_name: String,
    window: Window,
) -> Result<WeatherDetailed, String> {
    security::require_window(&window, &["main", "demo-weather"])?;
    if !lat.is_finite()
        || !lon.is_finite()
        || !(-90.0..=90.0).contains(&lat)
        || !(-180.0..=180.0).contains(&lon)
    {
        return Err("invalid coordinates".to_string());
    }
    let client = reqwest::Client::builder()
        .user_agent(USER_AGENT)
        .timeout(std::time::Duration::from_secs(8))
        .build()
        .map_err(|e| e.to_string())?;
    let url = format!(
        "https://api.met.no/weatherapi/locationforecast/2.0/compact?lat={lat:.4}&lon={lon:.4}"
    );
    let response = client.get(url).send().await.map_err(|e| e.to_string())?;
    if !response.status().is_success() {
        return Err(format!("weather service returned {}", response.status()));
    }
    parse_weather(
        response.json().await.map_err(|e| e.to_string())?,
        place_name,
    )
}

fn parse_weather(json: serde_json::Value, place_name: String) -> Result<WeatherDetailed, String> {
    let timeseries = json
        .pointer("/properties/timeseries")
        .and_then(|value| value.as_array())
        .ok_or("invalid forecast data")?;
    let latest = timeseries.first().ok_or("no current forecast data")?;
    let instant = latest
        .pointer("/data/instant/details")
        .ok_or("no instant forecast details")?;
    let temp = instant
        .get("air_temperature")
        .and_then(|value| value.as_f64())
        .unwrap_or_default() as f32;
    let symbol = latest
        .pointer("/data/next_1_hours/summary/symbol_code")
        .and_then(|value| value.as_str())
        .unwrap_or("clearsky_day")
        .to_string();
    let precip = latest
        .pointer("/data/next_1_hours/details/precipitation_amount")
        .and_then(|value| value.as_f64())
        .unwrap_or_default() as f32;

    let hourly = timeseries
        .iter()
        .take(24)
        .map(|entry| HourlyForecast {
            time: entry
                .get("time")
                .and_then(|value| value.as_str())
                .unwrap_or_default()
                .to_string(),
            temp: entry
                .pointer("/data/instant/details/air_temperature")
                .and_then(|value| value.as_f64())
                .unwrap_or_default() as f32,
            symbol: entry
                .pointer("/data/next_1_hours/summary/symbol_code")
                .and_then(|value| value.as_str())
                .unwrap_or_default()
                .to_string(),
            precip: entry
                .pointer("/data/next_1_hours/details/precipitation_amount")
                .and_then(|value| value.as_f64())
                .unwrap_or_default() as f32,
        })
        .collect();

    let mut daily = Vec::new();
    for entry in timeseries {
        let date = entry
            .get("time")
            .and_then(|value| value.as_str())
            .unwrap_or_default()
            .split('T')
            .next()
            .unwrap_or_default();
        let temperature = entry
            .pointer("/data/instant/details/air_temperature")
            .and_then(|value| value.as_f64())
            .unwrap_or_default() as f32;
        if let Some(day) = daily
            .iter_mut()
            .find(|day: &&mut DailyForecast| day.date == date)
        {
            day.temp_min = day.temp_min.min(temperature);
            day.temp_max = day.temp_max.max(temperature);
        } else if daily.len() < 7 {
            daily.push(DailyForecast {
                date: date.to_string(),
                temp_min: temperature,
                temp_max: temperature,
                symbol: entry
                    .pointer("/data/next_6_hours/summary/symbol_code")
                    .and_then(|value| value.as_str())
                    .unwrap_or_default()
                    .to_string(),
                humidity: entry
                    .pointer("/data/instant/details/relative_humidity")
                    .and_then(|value| value.as_f64())
                    .unwrap_or_default() as f32,
            });
        }
    }

    Ok(WeatherDetailed {
        temp,
        symbol,
        precip,
        place_name,
        hourly,
        daily,
    })
}

#[tauri::command]
pub async fn search_locations(
    query: String,
    window: Window,
) -> Result<Vec<LocationSearchResult>, String> {
    security::require_window(&window, &["settings"])?;
    let query = query.trim();
    if query.len() < 3 || query.len() > 100 {
        return Ok(Vec::new());
    }
    let client = reqwest::Client::builder()
        .user_agent(USER_AGENT)
        .timeout(std::time::Duration::from_secs(8))
        .build()
        .map_err(|e| e.to_string())?;
    let url = format!(
        "https://www.yr.no/api/v0/locations/suggest?q={}",
        urlencoding::encode(query)
    );
    let response = client.get(url).send().await.map_err(|e| e.to_string())?;
    if !response.status().is_success() {
        return Err(format!("location service returned {}", response.status()));
    }
    let json: serde_json::Value = response.json().await.map_err(|e| e.to_string())?;
    Ok(json
        .pointer("/_embedded/location")
        .and_then(|value| value.as_array())
        .into_iter()
        .flatten()
        .filter_map(|location| {
            Some(LocationSearchResult {
                name: location.get("name")?.as_str()?.to_string(),
                lat: location.pointer("/position/lat")?.as_f64()?,
                lon: location.pointer("/position/lon")?.as_f64()?,
                country: location
                    .pointer("/country/name")
                    .and_then(|value| value.as_str())
                    .unwrap_or_default()
                    .to_string(),
                url_path: location
                    .get("urlPath")
                    .and_then(|value| value.as_str())
                    .unwrap_or_default()
                    .to_string(),
            })
        })
        .collect())
}

#[tauri::command]
pub async fn get_obs_status(handle: tauri::AppHandle, window: Window) -> Result<ObsStatus, String> {
    security::require_window(&window, &["main"])?;
    let settings = handle
        .state::<SharedSettings>()
        .lock()
        .map_err(|e| e.to_string())?
        .clone();
    if settings.obs_websocket_url.is_empty() {
        return Ok(ObsStatus {
            is_streaming: false,
            is_recording: false,
        });
    }
    let obs_url = &settings.obs_websocket_url;
    let endpoint = obs_url
        .strip_prefix("wss://")
        .or_else(|| obs_url.strip_prefix("ws://"))
        .unwrap_or(obs_url);
    let (host, port) = endpoint
        .rsplit_once(':')
        .map(|(host, port)| (host.to_string(), port.parse().unwrap_or(4455)))
        .unwrap_or_else(|| (endpoint.to_string(), 4455));
    let client = tokio::time::timeout(
        std::time::Duration::from_millis(500),
        obws::Client::connect(host, port, Some(&settings.obs_websocket_password)),
    )
    .await
    .map_err(|_| "OBS connection timeout".to_string())?
    .map_err(|e| e.to_string())?;
    let stream = client
        .streaming()
        .status()
        .await
        .map_err(|e| e.to_string())?;
    let record = client
        .recording()
        .status()
        .await
        .map_err(|e| e.to_string())?;
    Ok(ObsStatus {
        is_streaming: stream.active,
        is_recording: record.active,
    })
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::Path;

    #[test]
    fn frontend_weather_contract_contains_serialized_fields() {
        let contract =
            fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("../src/contracts.ts"))
                .unwrap();
        for field in [
            "temp:",
            "symbol:",
            "precip:",
            "place_name:",
            "hourly:",
            "daily:",
        ] {
            assert!(contract.contains(field));
        }
    }

    #[test]
    fn usage_limits_contract_deserializes_provider_windows() {
        let snapshot: super::LimitsSnapshot = serde_json::from_str(
            r#"{
                "providers": {
                    "codex": {
                        "enabled": true,
                        "ok": true,
                        "planType": "plus",
                        "shortWindow": {
                            "label": "5H",
                            "usedPercent": 28,
                            "remainingPercent": 72,
                            "resetsAt": 1800000000
                        },
                        "longWindow": {
                            "label": "7D",
                            "usedPercent": 61,
                            "remainingPercent": 39,
                            "resetsAt": null
                        },
                        "rateLimitReachedType": null,
                        "error": null
                    },
                    "gemini": {
                        "enabled": false,
                        "ok": false,
                        "source": "disabled",
                        "error": null
                    }
                }
            }"#,
        )
        .unwrap();
        let codex = snapshot.providers.get("codex").unwrap();
        assert_eq!(codex.short_window.remaining_percent, Some(72.0));
        assert_eq!(codex.long_window.label, "7D");
        let gemini = snapshot.providers.get("gemini").unwrap();
        assert!(!gemini.enabled);
        assert!(gemini.short_window.remaining_percent.is_none());
    }
}
