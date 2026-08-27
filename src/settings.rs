//! Player-facing display settings for the CRT monitor look. These are cosmetic
//! and live outside the sim save (their own persistence key), so they never
//! affect determinism (GDD §5.6).

use macroquad_toolkit::fx::CrtStyle;
use macroquad_toolkit::persistence::{load_json_key, save_json_key};
use serde::{Deserialize, Serialize};

use crate::state::sim::DelegationSettings;

/// Persistence key, separate from the campaign save slot.
pub const DISPLAY_KEY: &str = "display";
/// Persistence key for the default per-category council delegation (GDD §5.4)
/// applied to each new voyage.
pub const DELEGATION_KEY: &str = "delegation";
/// Persistence key for one-time onboarding flags (first-run welcome overlay).
pub const ONBOARDING_KEY: &str = "onboarding";

/// One-time onboarding progress. Lives outside the sim save so it survives
/// deleting a campaign — the welcome overlay is shown once per install, not
/// once per voyage. All fields `serde(default)` so an absent/partial blob reads
/// as "nothing seen yet".
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Onboarding {
    /// Whether the player has dismissed the first-run welcome overlay.
    pub welcome_seen: bool,
}

impl Onboarding {
    /// Loads onboarding flags, defaulting to "nothing seen" on a fresh install.
    pub fn load(game_name: &str) -> Self {
        load_json_key(game_name, ONBOARDING_KEY).unwrap_or_default()
    }

    /// Persists the onboarding flags.
    pub fn save(&self, game_name: &str) -> Result<(), String> {
        save_json_key(game_name, ONBOARDING_KEY, self)
    }
}

/// Load the persisted default delegation for new campaigns (all-council if
/// never set).
pub fn load_delegation(game_name: &str) -> DelegationSettings {
    load_json_key(game_name, DELEGATION_KEY).unwrap_or_default()
}

/// Persist the default delegation preferences.
pub fn save_delegation(delegation: &DelegationSettings, game_name: &str) -> Result<(), String> {
    save_json_key(game_name, DELEGATION_KEY, delegation)
}

/// Phosphor tube color for the CRT overlay.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum Phosphor {
    /// Warm amber (P3).
    #[default]
    Amber,
    /// Cool green (P1).
    Green,
}

/// User's CRT display preferences. All fields `serde(default)` so older or
/// partial blobs load cleanly.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct DisplaySettings {
    /// Master switch for the whole CRT overlay.
    pub crt_enabled: bool,
    /// Draw scanlines.
    pub scanlines: bool,
    /// Apply the subtle whole-screen flicker.
    pub flicker: bool,
    /// Phosphor tint of the overlay.
    pub phosphor: Phosphor,
    /// Master mix, deliberately restrained by default.
    pub audio_volume: f32,
    /// Underway engine-room ambience; cues remain available when disabled.
    pub ambience: bool,
    /// Whether the guided first-voyage tutorial is available.
    #[serde(default = "default_tutorial_enabled")]
    pub tutorial_enabled: bool,
}

fn default_tutorial_enabled() -> bool {
    true
}

impl Default for DisplaySettings {
    fn default() -> Self {
        Self {
            crt_enabled: true,
            scanlines: true,
            flicker: true,
            phosphor: Phosphor::Amber,
            audio_volume: 0.35,
            ambience: true,
            tutorial_enabled: true,
        }
    }
}

impl DisplaySettings {
    /// Loads saved preferences, falling back to defaults.
    pub fn load(game_name: &str) -> Self {
        load_json_key(game_name, DISPLAY_KEY).unwrap_or_default()
    }

    /// Persists the current preferences.
    pub fn save(&self, game_name: &str) -> Result<(), String> {
        save_json_key(game_name, DISPLAY_KEY, self)
    }

    /// Build the overlay style these settings describe. `crt_enabled` is honored
    /// separately by the caller (whether to draw at all).
    pub fn crt_style(&self) -> CrtStyle {
        let mut style = match self.phosphor {
            Phosphor::Amber => CrtStyle::amber(),
            Phosphor::Green => CrtStyle::green(),
        };
        // Ease the corner falloff and scanline darkening back from the toolkit
        // presets: the heavy vignette greyed the panels toward the edges and
        // washed the whole frame. A lighter tube keeps the CRT character while
        // letting the boosted palette read crisp and flat, corner to corner.
        style.vignette_alpha *= 0.4;
        style.scanline_alpha *= 0.7;
        if !self.scanlines {
            style.scanline_alpha = 0.0;
        }
        if !self.flicker {
            style.flicker_alpha = 0.0;
        }
        style
    }
}

#[cfg(test)]
mod tests;
