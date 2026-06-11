use tauri::Window;

pub fn is_window_allowed(label: &str, allowed: &[&str]) -> bool {
    allowed.contains(&label)
}

pub fn require_window(window: &Window, allowed: &[&str]) -> Result<(), String> {
    if is_window_allowed(window.label(), allowed) {
        return Ok(());
    }

    Err(format!(
        "window '{}' is not authorized for this operation",
        window.label()
    ))
}

pub fn validate_volume(volume: f32) -> Result<(), String> {
    if volume.is_finite() && (0.0..=1.0).contains(&volume) {
        Ok(())
    } else {
        Err("volume must be between 0 and 1".to_string())
    }
}

pub fn validate_pty_size(rows: u16, cols: u16) -> Result<(), String> {
    if (1..=300).contains(&rows) && (1..=500).contains(&cols) {
        Ok(())
    } else {
        Err("invalid terminal dimensions".to_string())
    }
}

fn is_ha_object_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 100
        && value
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_')
}

/// HA entity ids ("domain.object_id") end up interpolated into REST URL paths,
/// so the charset must stay strictly [a-z0-9_] with a single dot separator.
pub fn validate_ha_entity_id(entity_id: &str) -> Result<(), String> {
    match entity_id.split_once('.') {
        Some((domain, object_id)) if is_ha_object_id(domain) && is_ha_object_id(object_id) => {
            Ok(())
        }
        _ => Err(format!("invalid Home Assistant entity id '{entity_id}'")),
    }
}

/// Device slugs are combined into entity ids (sensor.{slug}_battery_level).
pub fn validate_ha_slug(slug: &str) -> Result<(), String> {
    if is_ha_object_id(slug) {
        Ok(())
    } else {
        Err(format!("invalid Home Assistant device slug '{slug}'"))
    }
}

#[cfg(test)]
mod tests {
    use super::{
        is_window_allowed, validate_ha_entity_id, validate_ha_slug, validate_pty_size,
        validate_volume,
    };

    #[test]
    fn authorization_is_exact_and_deny_by_default() {
        assert!(is_window_allowed("main", &["main"]));
        assert!(!is_window_allowed("settings", &["main"]));
        assert!(!is_window_allowed("main-dev", &["main"]));
        assert!(!is_window_allowed("main", &[]));
    }

    #[test]
    fn volume_validation_rejects_invalid_values() {
        assert!(validate_volume(-0.1).is_err());
        assert!(validate_volume(1.1).is_err());
        assert!(validate_volume(f32::NAN).is_err());
        assert!(validate_volume(0.5).is_ok());
    }

    #[test]
    fn pty_dimensions_are_bounded() {
        assert!(validate_pty_size(0, 80).is_err());
        assert!(validate_pty_size(24, 0).is_err());
        assert!(validate_pty_size(24, 80).is_ok());
    }

    #[test]
    fn ha_entity_ids_are_strictly_validated() {
        assert!(validate_ha_entity_id("vacuum.roberto").is_ok());
        assert!(validate_ha_entity_id("lawn_mower.a1_pro").is_ok());
        assert!(validate_ha_entity_id("camera.garage").is_ok());
        assert!(validate_ha_entity_id("").is_err());
        assert!(validate_ha_entity_id("no_dot").is_err());
        assert!(validate_ha_entity_id("two.dots.here").is_err());
        assert!(validate_ha_entity_id("vacuum.Roberto").is_err());
        assert!(validate_ha_entity_id("vacuum.rob/../../admin").is_err());
        assert!(validate_ha_entity_id("vacuum.rob?x=1").is_err());
    }

    #[test]
    fn ha_slugs_are_strictly_validated() {
        assert!(validate_ha_slug("pixel_9_pro_xl").is_ok());
        assert!(validate_ha_slug("").is_err());
        assert!(validate_ha_slug("has.dot").is_err());
        assert!(validate_ha_slug("has space").is_err());
    }
}
