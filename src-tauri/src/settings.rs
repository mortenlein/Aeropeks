use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};
use tauri::Manager;

use crate::platform::{read_secret, restore_secret, write_secret};

const PLEX_TOKEN_TARGET: &str = "Aeropeks/PlexToken";
const OBS_PASSWORD_TARGET: &str = "Aeropeks/ObsWebSocketPassword";
const GITHUB_TOKEN_TARGET: &str = "Aeropeks/GitHubToken";
const HA_TOKEN_TARGET: &str = "Aeropeks/HomeAssistantToken";
/// Retired with the Dreame cloud integration; only used to purge old credentials.
const RETIRED_DREAME_PASSWORD_TARGET: &str = "Aeropeks/DreamePassword";

fn default_accent_color() -> String {
    "#22c55e".to_string()
}

fn default_plex_url() -> String {
    "http://localhost:32400".to_string()
}

fn default_true() -> bool {
    true
}

fn default_reserve_screen_space() -> bool {
    true
}

fn default_hide_native_taskbar() -> bool {
    false
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TerminalShortcut {
    pub id: String,
    pub label: String,
    pub cmd: String,
    pub shortcut: String,
}

fn default_shortcuts() -> Vec<TerminalShortcut> {
    vec![
        TerminalShortcut {
            id: "local-pwsh".to_string(),
            label: "PowerShell 7".to_string(),
            cmd: String::new(),
            shortcut: "Alt+T".to_string(),
        },
        TerminalShortcut {
            id: "git-status".to_string(),
            label: "Git Status".to_string(),
            cmd: "git status".to_string(),
            shortcut: String::new(),
        },
    ]
}

/// A user-pinned website shown in the Shortcuts dropdown (max 8).
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PinnedShortcut {
    pub id: String,
    pub url: String,
    /// Display label; the hostname is shown when empty.
    #[serde(default)]
    pub name: String,
}

fn default_enabled() -> bool {
    true
}

/// Module with no extra config beyond on/off.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SimpleModule {
    #[serde(default = "default_enabled")]
    pub enabled: bool,
}

impl Default for SimpleModule {
    fn default() -> Self {
        Self { enabled: true }
    }
}

/// Module backed by a single Home Assistant entity.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct HaEntityModule {
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    #[serde(default)]
    pub entity_id: String,
}

impl Default for HaEntityModule {
    fn default() -> Self {
        Self {
            enabled: true,
            entity_id: String::new(),
        }
    }
}

/// Mower needs the lawn_mower entity plus an optional update entity that
/// doesn't follow the sensor-prefix naming convention.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MowerModule {
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    #[serde(default)]
    pub entity_id: String,
    #[serde(default)]
    pub update_entity_id: String,
}

impl Default for MowerModule {
    fn default() -> Self {
        Self {
            enabled: true,
            entity_id: String::new(),
            update_entity_id: String::new(),
        }
    }
}

/// Phone companion entities all share an HA device slug
/// (e.g. sensor.{slug}_battery_level, device_tracker.{slug}).
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PhoneModule {
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    #[serde(default)]
    pub device_slug: String,
}

impl Default for PhoneModule {
    fn default() -> Self {
        Self {
            enabled: true,
            device_slug: String::new(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct ModulesConfig {
    #[serde(default)]
    pub media: SimpleModule,
    #[serde(default)]
    pub weather: SimpleModule,
    #[serde(default)]
    pub usage_limits: SimpleModule,
    #[serde(default)]
    pub projects: SimpleModule,
    #[serde(default)]
    pub obs: SimpleModule,
    #[serde(default)]
    pub camera: HaEntityModule,
    #[serde(default)]
    pub vacuum: HaEntityModule,
    #[serde(default)]
    pub calendar: HaEntityModule,
    #[serde(default)]
    pub mower: MowerModule,
    #[serde(default)]
    pub phone: PhoneModule,
}

impl ModulesConfig {
    /// Seed for settings files written before the modules schema existed:
    /// these entity ids used to be hardcoded in main.rs.
    fn legacy(calendar_entity_id: String) -> Self {
        Self {
            camera: HaEntityModule {
                enabled: true,
                entity_id: "camera.garage".to_string(),
            },
            vacuum: HaEntityModule {
                enabled: true,
                entity_id: "vacuum.roberto".to_string(),
            },
            calendar: HaEntityModule {
                enabled: true,
                entity_id: calendar_entity_id,
            },
            mower: MowerModule {
                enabled: true,
                entity_id: "lawn_mower.a1_pro".to_string(),
                update_entity_id: "update.dreame_mower_a1_pro_update".to_string(),
            },
            phone: PhoneModule {
                enabled: true,
                device_slug: "pixel_9_pro_xl".to_string(),
            },
            ..Self::default()
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AppSettings {
    #[serde(default = "default_plex_url")]
    pub plex_url: String,
    #[serde(default)]
    pub plex_token: String,
    #[serde(default = "default_accent_color")]
    pub accent_color: String,
    #[serde(default = "default_shortcuts")]
    pub terminal_shortcuts: Vec<TerminalShortcut>,
    #[serde(default)]
    pub weather_location: String,
    #[serde(default)]
    pub weather_lat: Option<f64>,
    #[serde(default)]
    pub weather_lon: Option<f64>,
    #[serde(default)]
    pub obs_websocket_url: String,
    #[serde(default)]
    pub obs_websocket_password: String,
    #[serde(default)]
    pub github_token: String,
    #[serde(default)]
    pub usage_limits_url: String,
    /// Provider keys (e.g. "claude", "codex") hidden from the bar. Empty = show all.
    #[serde(default)]
    pub usage_hidden_providers: Vec<String>,
    #[serde(default = "default_true")]
    pub use_24h: bool,
    #[serde(default = "default_reserve_screen_space")]
    pub reserve_screen_space: bool,
    #[serde(default = "default_hide_native_taskbar")]
    pub hide_native_taskbar: bool,
    #[serde(default)]
    pub homeassistant_url: String,
    #[serde(default)]
    pub homeassistant_token: String,
    /// Seconds between bulk Home Assistant state fetches (clamped 5..=600).
    #[serde(default = "default_ha_poll_seconds")]
    pub homeassistant_poll_seconds: u32,
    #[serde(default)]
    pub pinned_shortcuts: Vec<PinnedShortcut>,
    #[serde(default)]
    pub modules: ModulesConfig,
}

fn default_ha_poll_seconds() -> u32 {
    30
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            plex_url: default_plex_url(),
            plex_token: String::new(),
            accent_color: default_accent_color(),
            terminal_shortcuts: default_shortcuts(),
            weather_location: String::new(),
            weather_lat: None,
            weather_lon: None,
            obs_websocket_url: String::new(),
            obs_websocket_password: String::new(),
            github_token: String::new(),
            usage_limits_url: String::new(),
            usage_hidden_providers: Vec::new(),
            use_24h: true,
            reserve_screen_space: true,
            hide_native_taskbar: false,
            homeassistant_url: String::new(),
            homeassistant_token: String::new(),
            homeassistant_poll_seconds: default_ha_poll_seconds(),
            pinned_shortcuts: Vec::new(),
            modules: ModulesConfig::default(),
        }
    }
}

impl AppSettings {
    pub fn without_secrets(&self) -> Self {
        let mut settings = self.clone();
        settings.plex_token.clear();
        settings.obs_websocket_password.clear();
        settings.github_token.clear();
        settings.homeassistant_token.clear();
        settings
    }
}

pub type SharedSettings = Arc<Mutex<AppSettings>>;

pub fn settings_path(handle: &tauri::AppHandle) -> Result<PathBuf, String> {
    let directory = handle
        .path()
        .home_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join(".aeropeks");
    fs::create_dir_all(&directory).map_err(|e| e.to_string())?;
    Ok(directory.join("settings.json"))
}

fn migrate_file(handle: &tauri::AppHandle, new_path: &Path) {
    if new_path.exists() {
        return;
    }
    if let Ok(old_dir) = handle.path().app_data_dir() {
        let old_path = old_dir.join("settings.json");
        if old_path.exists() {
            let _ = fs::rename(old_path, new_path);
        }
    }
}

/// Migrate pre-module settings files: seed the module config with the entity
/// ids that used to be hardcoded, and carry over the old flat calendar field.
fn migrate_legacy_modules(content: &str, settings: &mut AppSettings) {
    if content.contains("\"modules\"") {
        return;
    }
    let legacy_calendar = serde_json::from_str::<serde_json::Value>(content)
        .ok()
        .and_then(|value| {
            value
                .get("ha_calendar_entity_id")
                .and_then(|id| id.as_str())
                .map(String::from)
        })
        .unwrap_or_default();
    settings.modules = ModulesConfig::legacy(legacy_calendar);
}

pub fn load(handle: &tauri::AppHandle) -> AppSettings {
    let Ok(path) = settings_path(handle) else {
        return AppSettings::default();
    };
    migrate_file(handle, &path);

    let mut settings = match fs::read_to_string(&path) {
        Ok(content) => match serde_json::from_str::<AppSettings>(&content) {
            Ok(mut settings) => {
                migrate_legacy_modules(&content, &mut settings);
                settings
            }
            Err(_) => {
                let mut backup = path.clone();
                backup.set_extension("json.error");
                let _ = fs::rename(&path, backup);
                AppSettings::default()
            }
        },
        Err(_) => AppSettings::default(),
    };

    let plaintext_plex = settings.plex_token.clone();
    let plaintext_obs = settings.obs_websocket_password.clone();
    let plaintext_github = settings.github_token.clone();
    settings.plex_token = read_secret(PLEX_TOKEN_TARGET)
        .ok()
        .flatten()
        .unwrap_or(plaintext_plex);
    settings.obs_websocket_password = read_secret(OBS_PASSWORD_TARGET)
        .ok()
        .flatten()
        .unwrap_or(plaintext_obs);
    settings.github_token = read_secret(GITHUB_TOKEN_TARGET)
        .ok()
        .flatten()
        .unwrap_or(plaintext_github);
    settings.homeassistant_token = read_secret(HA_TOKEN_TARGET)
        .ok()
        .flatten()
        .unwrap_or_default();

    // One-time cleanup of the credential left behind by the removed Dreame integration.
    let _ = restore_secret(RETIRED_DREAME_PASSWORD_TARGET, None);

    if persist_secrets(&settings).is_ok() {
        let _ = write_file(&path, &settings);
    }
    settings
}

pub fn save(handle: &tauri::AppHandle, settings: &AppSettings) -> Result<(), String> {
    let previous_plex = read_secret(PLEX_TOKEN_TARGET)?;
    let previous_obs = read_secret(OBS_PASSWORD_TARGET)?;
    let previous_github = read_secret(GITHUB_TOKEN_TARGET)?;
    let previous_ha = read_secret(HA_TOKEN_TARGET)?;
    let path = settings_path(handle)?;

    let result = persist_secrets(settings).and_then(|_| write_file(&path, settings));
    if let Err(error) = result {
        let rollback_plex = restore_secret(PLEX_TOKEN_TARGET, previous_plex.as_deref());
        let rollback_obs = restore_secret(OBS_PASSWORD_TARGET, previous_obs.as_deref());
        let rollback_github = restore_secret(GITHUB_TOKEN_TARGET, previous_github.as_deref());
        let rollback_ha = restore_secret(HA_TOKEN_TARGET, previous_ha.as_deref());
        return match (rollback_plex, rollback_obs, rollback_github, rollback_ha) {
            (Ok(()), Ok(()), Ok(()), Ok(())) => Err(error),
            (plex, obs, github, ha) => Err(format!(
                "{error}; credential rollback failed: plex={plex:?}, obs={obs:?}, github={github:?}, ha={ha:?}"
            )),
        };
    }
    Ok(())
}

fn persist_secrets(settings: &AppSettings) -> Result<(), String> {
    write_secret(PLEX_TOKEN_TARGET, &settings.plex_token)?;
    write_secret(OBS_PASSWORD_TARGET, &settings.obs_websocket_password)?;
    write_secret(GITHUB_TOKEN_TARGET, &settings.github_token)?;
    write_secret(HA_TOKEN_TARGET, &settings.homeassistant_token)
}

fn write_file(path: &Path, settings: &AppSettings) -> Result<(), String> {
    let content = serde_json::to_string_pretty(&settings.without_secrets()).map_err(|e| e.to_string())?;
    let temporary = path.with_extension("json.tmp");
    let backup = path.with_extension("json.bak");
    fs::write(&temporary, content).map_err(|e| e.to_string())?;

    if backup.exists() {
        fs::remove_file(&backup).map_err(|e| e.to_string())?;
    }
    if path.exists() {
        fs::rename(path, &backup).map_err(|e| e.to_string())?;
    }
    if let Err(error) = fs::rename(&temporary, path) {
        if backup.exists() {
            let _ = fs::rename(&backup, path);
        }
        let _ = fs::remove_file(&temporary);
        return Err(error.to_string());
    }
    if backup.exists() {
        fs::remove_file(backup).map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{migrate_legacy_modules, write_file, AppSettings};
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn legacy_settings_get_seeded_with_previously_hardcoded_entities() {
        let content = r##"{"accent_color":"#000000","ha_calendar_entity_id":"calendar.test_cal"}"##;
        let mut settings: AppSettings = serde_json::from_str(content).unwrap();
        migrate_legacy_modules(content, &mut settings);

        assert_eq!(settings.modules.camera.entity_id, "camera.garage");
        assert_eq!(settings.modules.vacuum.entity_id, "vacuum.roberto");
        assert_eq!(settings.modules.mower.entity_id, "lawn_mower.a1_pro");
        assert_eq!(settings.modules.phone.device_slug, "pixel_9_pro_xl");
        assert_eq!(settings.modules.calendar.entity_id, "calendar.test_cal");
        assert!(settings.modules.media.enabled);
    }

    #[test]
    fn migration_leaves_module_aware_files_untouched() {
        let content = r##"{"modules":{"vacuum":{"enabled":false,"entity_id":"vacuum.other"}}}"##;
        let mut settings: AppSettings = serde_json::from_str(content).unwrap();
        migrate_legacy_modules(content, &mut settings);

        assert!(!settings.modules.vacuum.enabled);
        assert_eq!(settings.modules.vacuum.entity_id, "vacuum.other");
        assert_eq!(settings.modules.camera.entity_id, "");
    }

    // Secrets travel to the settings window over IPC by design (so they are
    // editable there); every persisted or broadcast copy must go through
    // without_secrets() first. This guards the value-stripping contract.
    #[test]
    fn serialized_settings_exclude_secrets() {
        let settings = AppSettings {
            plex_token: "plex-secret".to_string(),
            obs_websocket_password: "obs-secret".to_string(),
            ..AppSettings::default()
        };

        let serialized = serde_json::to_string(&settings.without_secrets()).unwrap();

        assert!(!serialized.contains("plex-secret"));
        assert!(!serialized.contains("obs-secret"));
    }

    #[test]
    fn public_settings_redact_secrets() {
        let settings = AppSettings {
            plex_token: "plex-secret".to_string(),
            obs_websocket_password: "obs-secret".to_string(),
            ..AppSettings::default()
        };

        let public = settings.without_secrets();

        assert!(public.plex_token.is_empty());
        assert!(public.obs_websocket_password.is_empty());
    }

    #[test]
    fn atomic_write_replaces_file_and_removes_work_files() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let directory = std::env::temp_dir().join(format!("aeropeks-settings-{unique}"));
        fs::create_dir_all(&directory).unwrap();
        let path = directory.join("settings.json");
        fs::write(&path, r##"{"accent_color":"#000000"}"##).unwrap();

        let settings = AppSettings {
            accent_color: "#123456".to_string(),
            plex_token: "not-on-disk".to_string(),
            ..AppSettings::default()
        };
        write_file(&path, &settings).unwrap();

        let persisted = fs::read_to_string(&path).unwrap();
        assert!(persisted.contains("#123456"));
        assert!(!persisted.contains("not-on-disk"));
        assert!(!path.with_extension("json.tmp").exists());
        assert!(!path.with_extension("json.bak").exists());
        fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn frontend_settings_contract_contains_serialized_fields() {
        let contract_path =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../src/contracts.ts");
        let contract = fs::read_to_string(contract_path).unwrap();
        for field in [
            "plex_url",
            "plex_token",
            "accent_color",
            "terminal_shortcuts",
            "weather_location",
            "weather_lat",
            "weather_lon",
            "obs_websocket_url",
            "obs_websocket_password",
            "usage_limits_url",
            "use_24h",
            "reserve_screen_space",
            "hide_native_taskbar",
            "homeassistant_url",
            "homeassistant_token",
            "homeassistant_poll_seconds",
            "pinned_shortcuts",
            "modules",
        ] {
            assert!(contract.contains(&format!("{field}:")));
        }
    }
}
