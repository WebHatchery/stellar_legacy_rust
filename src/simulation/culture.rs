//! Bounded culture carried by each ship compartment.
//!
//! A compartment has one authored descriptor, one local people, one remembered
//! institutional moment, and one current grievance. The local record is derived
//! from factions, schools, archives, and condition; it is not a second resident
//! simulation and never stacks unlimited traits.

use crate::data::subsystems::{CompartmentDescriptor, SubsystemDef};
use crate::data::{GameData, PopulationDelta};
use crate::state::sim::SimState;

pub fn descriptor<'a>(
    data: &'a GameData,
    subsystem_id: &str,
    descriptor_id: &str,
) -> Option<&'a CompartmentDescriptor> {
    data.subsystems
        .get(subsystem_id)
        .and_then(|def| def.culture_descriptor(descriptor_id))
}

pub fn decay_multiplier(sim: &SimState, data: &GameData, subsystem_id: &str) -> f32 {
    let Some(state) = sim.subsystems.get(subsystem_id) else {
        return 1.0;
    };
    if state.culture.custodian_faction_id.is_none() {
        return 1.0;
    }
    descriptor(data, subsystem_id, &state.culture.descriptor_id)
        .map_or(1.0, |culture| culture.decay_multiplier.clamp(0.5, 1.5))
}

pub fn effect_summary(descriptor: &CompartmentDescriptor) -> String {
    format!(
        "MORALE {:+.1}% · UNITY {:+.1}% · STABILITY {:+.1}% · CRAFT {:+.1}%/y · DECAY ×{:.2}",
        descriptor.morale_per_year * 100.0,
        descriptor.unity_per_year * 100.0,
        descriptor.stability_per_year * 100.0,
        descriptor.knowledge_per_year * 100.0,
        descriptor.decay_multiplier
    )
}

fn institution_memory(sim: &SimState, subsystem_id: &str) -> Option<String> {
    sim.institution_records
        .iter()
        .rev()
        .find(|record| record.discipline == subsystem_id)
        .map(|record| {
            format!(
                "Y{} · {} · {}",
                record.year,
                record.kind.label(),
                record.subject
            )
        })
}

fn descriptor_id_for_custodian(
    def: &SubsystemDef,
    data: &GameData,
    faction_id: Option<&str>,
) -> String {
    let ideology = faction_id
        .and_then(|id| data.factions.get(id))
        .map_or(0.0, |faction| faction.ideology);
    def.culture_descriptor_for_ideology(ideology)
        .map(|descriptor| descriptor.id.clone())
        .or_else(|| {
            def.default_culture_descriptor()
                .map(|descriptor| descriptor.id.clone())
        })
        .unwrap_or_default()
}

fn grievance(
    sim: &SimState,
    data: &GameData,
    def: &SubsystemDef,
    subsystem_id: &str,
    custodian_id: Option<&str>,
) -> Option<String> {
    if custodian_id.is_none() {
        return Some("No aboard people currently holds this discipline.".to_owned());
    }
    let state = sim.subsystems.get(subsystem_id)?;
    let neglect_line = data.config.factions.neglect_condition_threshold;
    if neglect_line > 0.0 && state.condition < neglect_line {
        return Some(format!(
            "The {} decks are neglected at {:.0}% condition.",
            def.name,
            state.condition * 100.0
        ));
    }
    if state.knowledge + f32::EPSILON < def.repair_knowledge_required {
        return Some(format!(
            "The {} craft has thinned below repair knowledge.",
            def.name
        ));
    }
    None
}

/// Reconcile each local record with the faction and institution state that
/// already exists. Calling this repeatedly is deterministic and idempotent.
pub fn refresh(sim: &mut SimState, data: &GameData) {
    for subsystem_id in GameData::sorted_ids(&data.subsystems) {
        let Some(def) = data.subsystems.get(&subsystem_id) else {
            continue;
        };
        let custodian_id = sim
            .culture_custodian_faction_id(data, &subsystem_id)
            .map(str::to_owned);
        let descriptor_id = descriptor_id_for_custodian(def, data, custodian_id.as_deref());
        let memory = institution_memory(sim, &subsystem_id).or_else(|| {
            custodian_id.as_deref().map(|faction_id| {
                let name = data
                    .factions
                    .get(faction_id)
                    .map_or(faction_id, |faction| faction.log_name.as_str());
                format!("Founding hands placed {} in {}'s care.", def.name, name)
            })
        });
        let current_custodian = sim
            .subsystems
            .get(&subsystem_id)
            .and_then(|state| state.culture.custodian_faction_id.clone());
        let current_grievance = grievance(sim, data, def, &subsystem_id, custodian_id.as_deref());
        if let Some(state) = sim.subsystems.get_mut(&subsystem_id) {
            if current_custodian.as_deref() != custodian_id.as_deref()
                || state.culture.descriptor_id.is_empty()
                || def
                    .culture_descriptor(&state.culture.descriptor_id)
                    .is_none()
            {
                state.culture.custodian_faction_id = custodian_id.clone();
                state.culture.descriptor_id = descriptor_id;
            }
            if memory.is_some() {
                state.culture.remembered_event = memory;
            }
            state.culture.grievance = current_grievance;
        }
    }
}

/// Apply the current descriptor once per economic year, with every effect read
/// from the authored data and no accumulation beyond the live compartment.
pub fn annual_effects(sim: &mut SimState, data: &GameData) {
    refresh(sim, data);
    let mut people = PopulationDelta::default();
    let mut knowledge_changes = Vec::new();
    for subsystem_id in GameData::sorted_ids(&data.subsystems) {
        let Some(state) = sim.subsystems.get(&subsystem_id) else {
            continue;
        };
        if state.culture.custodian_faction_id.is_none() {
            continue;
        }
        let Some(culture) = descriptor(data, &subsystem_id, &state.culture.descriptor_id) else {
            continue;
        };
        people.morale += culture.morale_per_year;
        people.unity += culture.unity_per_year;
        people.stability += culture.stability_per_year;
        knowledge_changes.push((subsystem_id, culture.knowledge_per_year));
    }
    for (subsystem_id, change) in knowledge_changes {
        if let Some(state) = sim.subsystems.get_mut(&subsystem_id) {
            state.knowledge = (state.knowledge + change).clamp(0.0, 1.0);
        }
    }
    sim.population.apply(&people);
}

#[cfg(test)]
mod tests;
