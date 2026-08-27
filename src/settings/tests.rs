use super::*;

#[test]
fn style_reflects_toggles() {
    let mut s = DisplaySettings::default();
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
