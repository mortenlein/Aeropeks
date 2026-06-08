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

#[cfg(test)]
mod tests {
    use super::{is_window_allowed, validate_pty_size, validate_volume};

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
}
