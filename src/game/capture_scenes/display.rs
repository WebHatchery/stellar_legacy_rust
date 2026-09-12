//! Display-preference setup for deterministic capture scenes.

use crate::settings::{DisplaySettings, Phosphor};

pub(super) fn prepare<'a>(scene: &'a str, display: &mut DisplaySettings) -> &'a str {
    // Compose text and UI preferences, for example
    // people_officers_scale_125_text_150.
    let scene = if let Some((base, percent)) = scene.rsplit_once("_text_") {
        if let Ok(percent) = percent.parse::<f32>() {
            if percent.is_finite() {
                display.text_scale = (percent / 100.0).clamp(0.75, 1.5);
            }
        }
        base
    } else {
        scene
    };
    if let Some((scene, percent)) = scene.rsplit_once("_scale_") {
        if let Ok(percent) = percent.parse::<f32>() {
            display.ui_scale = macroquad_toolkit::ui::sanitize_ui_scale(percent / 100.0);
        }
        return scene;
    }
    match scene {
        "settings_95" => {
            display.ui_scale = 0.95;
            "settings"
        }
        "settings_small" => {
            display.ui_scale = 0.75;
            "settings"
        }
        "settings_large" => {
            display.ui_scale = 1.5;
            "settings"
        }
        "settings_text" => {
            display.text_scale = 1.5;
            "settings"
        }
        "settings_slate" => {
            display.phosphor = Phosphor::Slate;
            "settings"
        }
        _ => scene,
    }
}
