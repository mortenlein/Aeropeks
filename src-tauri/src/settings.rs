use std::ffi::c_void;
use std::fs;
use std::path::{Path, PathBuf};
use std::ptr;
use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};
use tauri::Manager;
use windows::core::{PCWSTR, PWSTR};
use windows::Win32::Security::Credentials::{
    CredDeleteW, CredFree, CredReadW, CredWriteW, CREDENTIALW, CRED_PERSIST_LOCAL_MACHINE,
    CRED_TYPE_GENERIC,
};

const PLEX_TOKEN_TARGET: &str = "Aeropeks/PlexToken";
const OBS_PASSWORD_TARGET: &str = "Aeropeks/ObsWebSocketPassword";
const GITHUB_TOKEN_TARGET: &str = "Aeropeks/GitHubToken";
const DREAME_PASSWORD_TARGET: &str = "Aeropeks/DreamePassword";
const HA_TOKEN_TARGET: &str = "Aeropeks/HomeAssistantToken";

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

fn default_debug_inspector() -> bool {
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
            id: "ssh-home".to_string(),
            label: "SSH: Home Lab".to_string(),
            cmd: "ssh pi@homeserver.local".to_string(),
            shortcut: String::new(),
        },
        TerminalShortcut {
            id: "git-status".to_string(),
            label: "Git Status".to_string(),
            cmd: "git status".to_string(),
            shortcut: String::new(),
        },
    ]
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
    #[serde(default = "default_true")]
    pub use_24h: bool,
    #[serde(default = "default_reserve_screen_space")]
    pub reserve_screen_space: bool,
    #[serde(default = "default_hide_native_taskbar")]
    pub hide_native_taskbar: bool,
    #[serde(default = "default_debug_inspector")]
    pub debug_inspector: bool,
    #[serde(default)]
    pub dreame_username: String,
    #[serde(default)]
    pub dreame_password: String,
    #[serde(default)]
    pub dreame_device_id: String,
    #[serde(default)]
    pub homeassistant_url: String,
    #[serde(default)]
    pub homeassistant_token: String,
    #[serde(default)]
    pub ha_calendar_entity_id: String,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            plex_url: default_plex_url(),
            plex_token: String::new(),
            accent_color: default_accent_color(),
            terminal_shortcuts: default_shortcuts(),
            weather_location: "Oslo, Norge".to_string(),
            weather_lat: Some(59.9127),
            weather_lon: Some(10.7461),
            obs_websocket_url: String::new(),
            obs_websocket_password: String::new(),
            github_token: String::new(),
            usage_limits_url: String::new(),
            use_24h: true,
            reserve_screen_space: true,
            hide_native_taskbar: false,
            debug_inspector: false,
            dreame_username: String::new(),
            dreame_password: String::new(),
            dreame_device_id: String::new(),
            homeassistant_url: String::new(),
            homeassistant_token: String::new(),
            ha_calendar_entity_id: String::new(),
        }
    }
}

impl AppSettings {
    pub fn without_secrets(&self) -> Self {
        let mut settings = self.clone();
        settings.plex_token.clear();
        settings.obs_websocket_password.clear();
        settings.github_token.clear();
        settings.dreame_password.clear();
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

pub fn load(handle: &tauri::AppHandle) -> AppSettings {
    let Ok(path) = settings_path(handle) else {
        return AppSettings::default();
    };
    migrate_file(handle, &path);

    let mut settings = match fs::read_to_string(&path) {
        Ok(content) => match serde_json::from_str::<AppSettings>(&content) {
            Ok(settings) => settings,
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
    settings.dreame_password = read_secret(DREAME_PASSWORD_TARGET)
        .ok()
        .flatten()
        .unwrap_or_default();
    settings.homeassistant_token = read_secret(HA_TOKEN_TARGET)
        .ok()
        .flatten()
        .unwrap_or_default();

    if persist_secrets(&settings).is_ok() {
        let _ = write_file(&path, &settings);
    }
    settings
}

pub fn save(handle: &tauri::AppHandle, settings: &AppSettings) -> Result<(), String> {
    let previous_plex = read_secret(PLEX_TOKEN_TARGET)?;
    let previous_obs = read_secret(OBS_PASSWORD_TARGET)?;
    let previous_github = read_secret(GITHUB_TOKEN_TARGET)?;
    let previous_dreame = read_secret(DREAME_PASSWORD_TARGET)?;
    let previous_ha = read_secret(HA_TOKEN_TARGET)?;
    let path = settings_path(handle)?;

    let result = persist_secrets(settings).and_then(|_| write_file(&path, settings));
    if let Err(error) = result {
        let rollback_plex = restore_secret(PLEX_TOKEN_TARGET, previous_plex.as_deref());
        let rollback_obs = restore_secret(OBS_PASSWORD_TARGET, previous_obs.as_deref());
        let rollback_github = restore_secret(GITHUB_TOKEN_TARGET, previous_github.as_deref());
        let rollback_dreame = restore_secret(DREAME_PASSWORD_TARGET, previous_dreame.as_deref());
        let rollback_ha = restore_secret(HA_TOKEN_TARGET, previous_ha.as_deref());
        return match (rollback_plex, rollback_obs, rollback_github, rollback_dreame, rollback_ha) {
            (Ok(()), Ok(()), Ok(()), Ok(()), Ok(())) => Err(error),
            (plex, obs, github, dreame, ha) => Err(format!(
                "{error}; credential rollback failed: plex={plex:?}, obs={obs:?}, github={github:?}, dreame={dreame:?}, ha={ha:?}"
            )),
        };
    }
    Ok(())
}

fn persist_secrets(settings: &AppSettings) -> Result<(), String> {
    write_secret(PLEX_TOKEN_TARGET, &settings.plex_token)?;
    write_secret(OBS_PASSWORD_TARGET, &settings.obs_websocket_password)?;
    write_secret(GITHUB_TOKEN_TARGET, &settings.github_token)?;
    write_secret(DREAME_PASSWORD_TARGET, &settings.dreame_password)?;
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

fn wide(value: &str) -> Vec<u16> {
    value.encode_utf16().chain(Some(0)).collect()
}

fn write_secret(target: &str, secret: &str) -> Result<(), String> {
    let mut target = wide(target);
    let mut username = wide("Aeropeks");
    let mut blob = secret.as_bytes().to_vec();
    let credential = CREDENTIALW {
        Type: CRED_TYPE_GENERIC,
        TargetName: PWSTR(target.as_mut_ptr()),
        CredentialBlobSize: blob.len() as u32,
        CredentialBlob: blob.as_mut_ptr(),
        Persist: CRED_PERSIST_LOCAL_MACHINE,
        UserName: PWSTR(username.as_mut_ptr()),
        ..Default::default()
    };
    unsafe { CredWriteW(&credential, 0).map_err(|e| e.to_string()) }
}

fn restore_secret(target: &str, secret: Option<&str>) -> Result<(), String> {
    match secret {
        Some(secret) => write_secret(target, secret),
        None => {
            let target = wide(target);
            unsafe {
                CredDeleteW(PCWSTR(target.as_ptr()), CRED_TYPE_GENERIC, 0)
                    .or_else(|error| {
                        if error.code().0 as u32 == 1168 {
                            Ok(())
                        } else {
                            Err(error)
                        }
                    })
                    .map_err(|e| e.to_string())
            }
        }
    }
}

fn read_secret(target: &str) -> Result<Option<String>, String> {
    let target = wide(target);
    let mut credential = ptr::null_mut::<CREDENTIALW>();
    unsafe {
        if let Err(error) = CredReadW(
            PCWSTR(target.as_ptr()),
            CRED_TYPE_GENERIC,
            0,
            &mut credential,
        ) {
            if error.code().0 as u32 == 1168 {
                return Ok(None);
            }
            return Err(error.to_string());
        }
        if credential.is_null() {
            return Ok(None);
        }
        let value = {
            let credential_ref = &*credential;
            if credential_ref.CredentialBlobSize == 0 {
                Some(String::new())
            } else if credential_ref.CredentialBlob.is_null() {
                None
            } else {
                let bytes = std::slice::from_raw_parts(
                    credential_ref.CredentialBlob,
                    credential_ref.CredentialBlobSize as usize,
                );
                String::from_utf8(bytes.to_vec()).ok()
            }
        };
        CredFree(credential.cast::<c_void>());
        Ok(value)
    }
}

#[cfg(test)]
mod tests {
    use super::{write_file, AppSettings};
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn serialized_settings_exclude_secrets() {
        let settings = AppSettings {
            plex_token: "plex-secret".to_string(),
            obs_websocket_password: "obs-secret".to_string(),
            ..AppSettings::default()
        };

        let serialized = serde_json::to_string(&settings).unwrap();

        assert!(!serialized.contains("plex-secret"));
        assert!(!serialized.contains("obs-secret"));
        assert!(!serialized.contains("plex_token"));
        assert!(!serialized.contains("obs_websocket_password"));
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
            "debug_inspector",
            "homeassistant_url",
            "homeassistant_token",
        ] {
            assert!(contract.contains(&format!("{field}:")));
        }
    }
}
