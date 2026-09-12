//! Embedded game data: config, content registries, and shared delta types.

pub mod config;
pub mod contracts;
pub mod crew;
pub mod events;
pub mod factions;
pub mod legacies;
pub mod projects;
pub mod ship_components;
pub mod subsystems;

use macroquad_toolkit::assets::TextureConfig;
use macroquad_toolkit::data_loader::{
    load_embedded_json, load_embedded_json_labeled, DataRegistry,
};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

use contracts::ContractTemplate;
use crew::{CrewArchetype, DynastyNamePools};
use events::EventTemplate;
use factions::FactionDef;
use legacies::LegacyDef;
use projects::ProjectDefinition;
use ship_components::ShipComponentCatalog;
use subsystems::SubsystemDef;

pub use config::*;

const GAME_CONFIG_JSON: &str =
    macroquad_toolkit::include_json_str!("../assets/data/game_config.json");
const TEXTURE_MANIFEST_JSON: &str =
    macroquad_toolkit::include_json_str!("../assets/data/texture_manifest.json");
const SHIP_COMPONENTS_JSON: &str =
    macroquad_toolkit::include_json_str!("../assets/ship_components.json");
/// Event templates, split per `family` (content-depth): one file per family
/// under `assets/events/` so no single content file grows unwieldy. Embedded via
/// `include_str!` (WASM-safe, same as before); merged into one registry at load
/// with a hard duplicate-id guard. Adding a new family = add one line here.
const EVENT_FILES: &[(&str, &str)] = &[
    (
        "biology_medical",
        macroquad_toolkit::include_json_str!("../assets/events/biology_medical.json"),
    ),
    (
        "comedy",
        macroquad_toolkit::include_json_str!("../assets/events/comedy.json"),
    ),
    (
        "diplomacy",
        macroquad_toolkit::include_json_str!("../assets/events/diplomacy.json"),
    ),
    (
        "engineering",
        macroquad_toolkit::include_json_str!("../assets/events/engineering.json"),
    ),
    (
        "ethics",
        macroquad_toolkit::include_json_str!("../assets/events/ethics.json"),
    ),
    (
        "exploration_first_contact",
        macroquad_toolkit::include_json_str!("../assets/events/exploration_first_contact.json"),
    ),
    (
        "legacy_drift",
        macroquad_toolkit::include_json_str!("../assets/events/legacy_drift.json"),
    ),
    (
        "mystery",
        macroquad_toolkit::include_json_str!("../assets/events/mystery.json"),
    ),
    (
        "obligations",
        macroquad_toolkit::include_json_str!("../assets/events/obligations.json"),
    ),
    (
        "science_anomaly",
        macroquad_toolkit::include_json_str!("../assets/events/science_anomaly.json"),
    ),
    (
        "survival",
        macroquad_toolkit::include_json_str!("../assets/events/survival.json"),
    ),
];
const LEGACIES_JSON: &str = macroquad_toolkit::include_json_str!("../assets/legacies.json");
const CONTRACTS_JSON: &str = macroquad_toolkit::include_json_str!("../assets/contracts.json");
const FACTIONS_JSON: &str = macroquad_toolkit::include_json_str!("../assets/factions.json");
const SUBSYSTEMS_JSON: &str = macroquad_toolkit::include_json_str!("../assets/subsystems.json");
const DYNASTY_NAMES_JSON: &str =
    macroquad_toolkit::include_json_str!("../assets/dynasty_names.json");
const CREW_ARCHETYPES_JSON: &str =
    macroquad_toolkit::include_json_str!("../assets/crew_archetypes.json");
const PROJECTS_JSON: &str = macroquad_toolkit::include_json_str!("../assets/projects.json");

/// How a fitting (ship component or subsystem version) is obtained. `Purchasable`
/// is bought in the drydock at its `cost`; `MissionReward` is never for sale —
/// it only reaches the ship as a granted part (`grant_component` /
/// `completion_reward`), dropping into the salvage hold to be installed. A part
/// may be granted whether it is purchasable or not, so a mission can hand over
/// either an ordinary catalog part (found early/free) or a unique one that can be
/// had no other way.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Acquisition {
    #[default]
    Purchasable,
    MissionReward,
}

impl Acquisition {
    /// True when the part can never be bought — it comes only from a mission.
    pub fn is_mission_only(self) -> bool {
        matches!(self, Acquisition::MissionReward)
    }
}

/// Signed per-resource change used by event outcomes, costs, and rewards.
/// Also doubles as an absolute amount set (e.g. starting resources).
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct ResourceDelta {
    pub credits: i64,
    pub energy: i64,
    pub minerals: i64,
    pub food: i64,
    pub influence: i64,
}

/// Per-year production rates for each tracked resource. Initialized with every
/// key present so colonization/component bonuses always have a slot to land in
/// (the original web build lost these bonuses to a missing-key bug — GDD §5.1).
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct ProductionRates {
    pub credits: f32,
    pub energy: f32,
    pub minerals: f32,
    pub food: f32,
    pub influence: f32,
}

/// Signed change to ship-condition stats.
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct ShipDelta {
    pub hull_integrity: f32,
    pub life_support: f32,
    pub fuel: f32,
    pub spare_parts: i32,
}

/// Signed change to colony-scale population stats.
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct PopulationDelta {
    pub count: i32,
    pub morale: f32,
    pub unity: f32,
    pub stability: f32,
    pub legacy_loyalty: f32,
    pub adaptation: f32,
    pub cultural_drift: f32,
}

#[derive(Debug, Clone)]
pub struct GameData {
    pub config: GameConfig,
    pub ship_components: ShipComponentCatalog,
    pub events: DataRegistry<EventTemplate>,
    pub legacies: DataRegistry<LegacyDef>,
    pub projects: DataRegistry<ProjectDefinition>,
    pub contracts: DataRegistry<ContractTemplate>,
    pub factions: DataRegistry<FactionDef>,
    pub subsystems: DataRegistry<SubsystemDef>,
    pub dynasty_names: DynastyNamePools,
    pub crew_archetypes: Vec<CrewArchetype>,
    pub texture_manifest: Vec<TextureConfig>,
}

impl GameData {
    pub fn load() -> Result<Self, String> {
        let events = Self::load_events()?;
        let projects = Self::load_projects()?;
        Self::validate_project_references(&projects, &events)?;
        Ok(Self {
            config: load_embedded_json_labeled("game_config", GAME_CONFIG_JSON)?,
            ship_components: load_embedded_json_labeled("ship_components", SHIP_COMPONENTS_JSON)?,
            events,
            legacies: DataRegistry::from_embedded_json(LEGACIES_JSON, "id")?,
            projects,
            contracts: DataRegistry::from_embedded_json(CONTRACTS_JSON, "id")?,
            factions: DataRegistry::from_embedded_json(FACTIONS_JSON, "id")?,
            subsystems: DataRegistry::from_embedded_json(SUBSYSTEMS_JSON, "id")?,
            dynasty_names: load_embedded_json_labeled("dynasty_names", DYNASTY_NAMES_JSON)?,
            crew_archetypes: load_embedded_json_labeled("crew_archetypes", CREW_ARCHETYPES_JSON)?,
            texture_manifest: load_embedded_json(TEXTURE_MANIFEST_JSON)?,
        })
    }

    fn load_projects() -> Result<DataRegistry<ProjectDefinition>, String> {
        let registry: DataRegistry<ProjectDefinition> =
            DataRegistry::from_embedded_json(PROJECTS_JSON, "id")?;
        for id in registry.ids() {
            registry
                .get(id)
                .ok_or_else(|| format!("project registry lost '{id}'"))?
                .validates()
                .map_err(|error| format!("projects.json: {error}"))?;
        }
        Ok(registry)
    }

    fn validate_project_references(
        projects: &DataRegistry<ProjectDefinition>,
        events: &DataRegistry<EventTemplate>,
    ) -> Result<(), String> {
        let capabilities: HashSet<&str> = projects
            .iter()
            .filter_map(|(_, project)| project.effect.capability.as_deref())
            .collect();
        let issue_ids: HashSet<&str> = events
            .iter()
            .flat_map(|(_, event)| {
                event
                    .outcomes
                    .iter()
                    .filter_map(|outcome| outcome.issue.as_ref())
            })
            .map(|issue| issue.id.as_str())
            .collect();
        for (_, project) in projects.iter() {
            if let Some(capability) = &project.requires_capability {
                if !capabilities.contains(capability.as_str()) {
                    return Err(format!(
                        "projects.json: '{}' requires unknown capability '{}'",
                        project.id, capability
                    ));
                }
            }
            if let Some(issue) = &project.requires_issue {
                if !issue_ids.contains(issue.as_str()) {
                    return Err(format!(
                        "projects.json: '{}' requires unknown issue '{}'",
                        project.id, issue
                    ));
                }
            }
        }
        for (_, event) in events.iter() {
            for outcome in &event.outcomes {
                for capability in &outcome.requires.requires_capabilities {
                    if !capabilities.contains(capability.as_str()) {
                        return Err(format!(
                            "event '{}': outcome '{}' requires unknown capability '{}'",
                            event.id, outcome.id, capability
                        ));
                    }
                }
                for issue in &outcome.resolves_issues {
                    if !issue_ids.contains(issue.as_str()) {
                        return Err(format!(
                            "event '{}': outcome '{}' resolves unknown issue '{}'",
                            event.id, outcome.id, issue
                        ));
                    }
                }
                if let Some(issue) = &outcome.issue {
                    for project_id in &issue.recovery_project_ids {
                        if !projects.contains(project_id) {
                            return Err(format!(
                                "event '{}': issue '{}' names unknown recovery project '{}'",
                                event.id, issue.id, project_id
                            ));
                        }
                    }
                }
            }
        }
        Ok(())
    }

    /// Merge the per-family event files into one registry. Fails loudly on a
    /// duplicate id *across* files — a single file makes a collision obvious, but
    /// two files can each define the same id and `merge` would silently drop one.
    fn load_events() -> Result<DataRegistry<EventTemplate>, String> {
        let mut merged: DataRegistry<EventTemplate> = DataRegistry::new();
        for (family, json) in EVENT_FILES {
            let part = DataRegistry::<EventTemplate>::from_embedded_json(json, "id")
                .map_err(|e| format!("events/{family}.json: {e}"))?;
            for id in part.ids() {
                if merged.contains(id) {
                    return Err(format!(
                        "duplicate event id '{id}' across event files (redefined in events/{family}.json)"
                    ));
                }
            }
            merged.merge(part);
        }
        Ok(merged)
    }

    /// Registry ids sorted for deterministic iteration (`DataRegistry` is
    /// hash-map backed, so raw iteration order is unstable — never feed it
    /// to the seeded RNG unsorted).
    pub fn sorted_ids<T: Clone>(registry: &DataRegistry<T>) -> Vec<String> {
        let mut ids: Vec<String> = registry.ids().cloned().collect();
        ids.sort();
        ids
    }
}
#[allow(dead_code, unused_imports)]
mod tests {
    include!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/unit/data/tests.rs"
    ));
}
