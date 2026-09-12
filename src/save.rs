//! Save slots and migration (GDD §7): local persistence only, no server.

use crate::data::GameConfig;
use crate::simulation::survival;
use crate::state::sim::SimState;
use macroquad_toolkit::persistence::{
    load_from_slot_with_migration, quarantine_slot, save_to_slot_with_version, slot_exists,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
mod validation;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveData {
    pub version: String,
    pub sim: SimState,
}

pub fn save_exists(config: &GameConfig) -> bool {
    slot_exists(&config.game_name, &config.save_slot)
}

pub fn save_campaign(config: &GameConfig, sim: &SimState) -> Result<(), String> {
    let save = SaveData {
        version: config.version.clone(),
        sim: sim.clone(),
    };
    save_to_slot_with_version(&config.game_name, &config.save_slot, &save, &config.version)
}

pub fn load_campaign(config: &GameConfig) -> Result<SimState, String> {
    let loaded: SaveData = match load_from_slot_with_migration(
        &config.game_name,
        &config.save_slot,
        &config.version,
        |version, value| migrate_save_value(version, value, config),
    )
    .and_then(|save: SaveData| {
        validation::validate(&save.sim, &crate::data::GameData::load()?)?;
        Ok(save)
    }) {
        Ok(save) => save,
        Err(error) => {
            let preserved = quarantine_slot(&config.game_name, &config.save_slot)
                .map(|slot| format!(" Preserved as {slot}."))
                .unwrap_or_else(|quarantine| format!(" Could not preserve it: {quarantine}."));
            return Err(format!("Save could not be loaded: {error}.{preserved}"));
        }
    };
    Ok(loaded.sim)
}

/// v0.2.0 adds Agenda, issue, and survival state. New fields are serde-defaulted
/// so v0.1.0 saves preserve their existing resources and obligations while the
/// next safe simulation update derives new warnings and maintenance notices.
pub fn migrate_save_value(
    detected_version: Option<String>,
    value: Value,
    config: &GameConfig,
) -> Result<SaveData, String> {
    let payload = value.get("data").cloned().unwrap_or(value);
    match serde_json::from_value::<SaveData>(payload) {
        Ok(mut save) => {
            let version = detected_version.as_deref().unwrap_or(&save.version);
            if !["0.1.0", "0.2.0", config.version.as_str()].contains(&version)
                || !["0.1.0", "0.2.0", config.version.as_str()].contains(&save.version.as_str())
            {
                return Err(format!("Unsupported save version: {version}"));
            }
            if version == "0.2.0" {
                for job in &mut save.sim.projects.jobs {
                    if job.project_id == "restore_crew_quarters" {
                        job.legacy_single_delivery = true;
                    }
                }
            }
            if version == "0.1.0" {
                survival::migrate_legacy(&mut save.sim);
            }
            validation::validate(&save.sim, &crate::data::GameData::load()?)?;
            save.version = config.version.clone();
            Ok(save)
        }
        Err(err) => Err(format!(
            "Unsupported save format {detected_version:?}: {err}"
        )),
    }
}
#[allow(dead_code, unused_imports)]
mod tests {
    include!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/unit/save/tests.rs"
    ));
}
