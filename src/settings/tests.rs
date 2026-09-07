use super::*;

#[test]
fn style_reflects_toggles() {
    let mut s = DisplaySettings::default();
    assert_eq!(s.crt_style().scanline_alpha, 0.0);
    s.scanlines = true;
    assert!(s.crt_style().scanline_alpha > 0.0);
    s.scanlines = false;
    s.flicker = false;
    let style = s.crt_style();
    assert_eq!(style.scanline_alpha, 0.0);
    assert_eq!(style.flicker_alpha, 0.0);
    // Vignette is unaffected by the scanline/flicker toggles.
    assert!(style.vignette_alpha > 0.0);
}

#[test]
fn green_phosphor_tints_differently() {
    let amber = DisplaySettings {
        phosphor: Phosphor::Amber,
        ..Default::default()
    };
    let green = DisplaySettings {
        phosphor: Phosphor::Green,
        ..Default::default()
    };
    assert_ne!(amber.crt_style().tint, green.crt_style().tint);
}

#[test]
fn partial_json_loads_with_defaults() {
    let s: DisplaySettings = serde_json::from_str(r#"{"scanlines": false}"#).unwrap();
    assert!(!s.scanlines);
    assert!(s.crt_enabled);
    assert_eq!(s.phosphor, Phosphor::Amber);
    assert!(s.tutorial_enabled);
}

#[test]
fn every_color_scheme_survives_a_settings_round_trip() {
    for phosphor in Phosphor::ALL {
        let settings = DisplaySettings {
            phosphor,
            ..Default::default()
        };
        let json = serde_json::to_string(&settings).unwrap();
        let restored: DisplaySettings = serde_json::from_str(&json).unwrap();
        assert_eq!(restored, settings);
    }
}

#[test]
fn old_terminal_preferences_keep_their_selected_scheme() {
    for (json, expected) in [
        (r#"{"phosphor":"Amber"}"#, Phosphor::Amber),
        (r#"{"phosphor":"Green"}"#, Phosphor::Green),
    ] {
        let settings: DisplaySettings = serde_json::from_str(json).unwrap();
        assert_eq!(settings.phosphor, expected);
    }
}
