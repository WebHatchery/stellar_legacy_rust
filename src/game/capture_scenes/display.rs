//! Display-preference setup for deterministic capture scenes.

use crate::settings::{DisplaySettings, Phosphor};

pub(super) fn prepare<'a>(scene: &'a str, display: &mut DisplaySettings) -> &'a str {
    // Compose a text preference, for example people_officers_text_150.
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
    match scene {
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
