//! Ship-subsystem runtime state (W5): per-subsystem tier, condition, and the
//! institutional knowledge that gates its repair. Knowledge is a per-subsystem
//! aggregate carried by the population — not per-crew, not per-faction — and
//! dies with the people unless the education subsystem transmits it forward.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::data::GameData;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CompartmentCulture {
    /// Authored descriptor id from the subsystem catalog.
    #[serde(default)]
    pub descriptor_id: String,
    /// The aboard people currently acting as this compartment's local custodian.
    #[serde(default)]
    pub custodian_faction_id: Option<String>,
    /// The latest authored or institutional moment this compartment remembers.
    #[serde(default)]
    pub remembered_event: Option<String>,
    /// Current bounded complaint derived from condition, craft, and custody.
    #[serde(default)]
    pub grievance: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubsystemState {
    /// 0..=3 (0 is the ship's baseline; 1..=3 are drydock upgrades).
    pub tier: u32,
    /// 0-1 physical condition; decays yearly, restored by repair.
    pub condition: f32,
    /// 0-1 institutional knowledge for THIS subsystem.
    pub knowledge: f32,
    /// Compact local culture derived from the existing factions and institutions.
    #[serde(default)]
    pub culture: CompartmentCulture,
}

/// One runtime entry per catalog subsystem: baseline tier 0, whole condition,
/// and the founding knowledge stock (W5).
pub fn build_founding_subsystems(
    data: &GameData,
    founding_faction_ids: &[String],
) -> HashMap<String, SubsystemState> {
    let start = data.config.subsystems.knowledge_start;
    data.subsystems
        .ids()
        .map(|id| {
            let definition = data.subsystems.get(id).expect("catalog subsystem id");
            let custodian_faction_id = founding_faction_ids
                .iter()
                .find(|faction_id| {
                    data.factions
                        .get(faction_id)
                        .is_some_and(|faction| faction.tended_subsystem == *id)
                })
                .cloned()
                .or_else(|| founding_faction_ids.first().cloned());
            let ideology = custodian_faction_id
                .as_deref()
                .and_then(|faction_id| data.factions.get(faction_id))
                .map_or(0.0, |faction| faction.ideology);
            let descriptor_id = definition
                .culture_descriptor_for_ideology(ideology)
                .map(|descriptor| descriptor.id.clone())
                .unwrap_or_default();
            (
                id.clone(),
                SubsystemState {
                    tier: 0,
                    condition: 1.0,
                    knowledge: start,
                    culture: CompartmentCulture {
                        descriptor_id,
                        custodian_faction_id,
                        remembered_event: Some(format!(
                            "Founding hands set the {} compartment to work.",
                            definition.name
                        )),
                        grievance: None,
                    },
                },
            )
        })
        .collect()
}
