use crate::settings::{DisplaySettings, Phosphor};

pub(super) fn prepare<'a>(scene: &'a str, display: &mut DisplaySettings) -> &'a str {
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
