use super::*;

#[test]
fn every_subsystem_has_two_bounded_culture_descriptors() {
    let data = GameData::load().unwrap();
    for id in GameData::sorted_ids(&data.subsystems) {
        let definition = data.subsystems.get(&id).unwrap();
        assert!(
            definition.culture_descriptors.len() >= 2,
            "{id} needs at least two culture descriptors"
        );
        for descriptor in &definition.culture_descriptors {
            assert!((0.5..=1.5).contains(&descriptor.decay_multiplier));
            assert!(descriptor.morale_per_year.abs() <= 0.02);
            assert!(descriptor.unity_per_year.abs() <= 0.02);
            assert!(descriptor.stability_per_year.abs() <= 0.02);
            assert!(descriptor.knowledge_per_year.abs() <= 0.02);
        }
    }
}

#[test]
fn founding_culture_derives_a_local_people_for_each_compartment() {
    let data = GameData::load().unwrap();
    let factions = crate::state::sim::founding_faction_ids(&data);
    let sim = SimState::new_campaign(&data, "preservers", 91, &factions);

    for id in GameData::sorted_ids(&data.subsystems) {
        let state = sim.subsystems.get(&id).unwrap();
        assert!(state.culture.custodian_faction_id.is_some());
        assert!(!state.culture.descriptor_id.is_empty());
        assert!(state.culture.remembered_event.is_some());
    }
}

#[test]
fn an_older_save_reconciles_missing_compartment_culture_on_load() {
    let data = GameData::load().unwrap();
    let factions = crate::state::sim::founding_faction_ids(&data);
    let sim = SimState::new_campaign(&data, "preservers", 93, &factions);
    let mut value = serde_json::to_value(&sim).unwrap();
    for state in value
        .get_mut("subsystems")
        .and_then(serde_json::Value::as_object_mut)
        .unwrap()
        .values_mut()
    {
        state.as_object_mut().unwrap().remove("culture");
    }

    let mut back: SimState = serde_json::from_value(value).unwrap();
    assert!(back
        .subsystems
        .values()
        .all(|state| state.culture.descriptor_id.is_empty()));
    refresh(&mut back, &data);
    assert!(back
        .subsystems
        .values()
        .all(|state| !state.culture.descriptor_id.is_empty()));
}

#[test]
fn a_culture_descriptor_changes_people_and_craft_without_stacking() {
    let data = GameData::load().unwrap();
    let factions = crate::state::sim::founding_faction_ids(&data);
    let mut sim = SimState::new_campaign(&data, "preservers", 92, &factions);
    let id = "security";
    let descriptor = data
        .subsystems
        .get(id)
        .unwrap()
        .culture_descriptors
        .get(1)
        .unwrap();
    sim.subsystems.get_mut(id).unwrap().culture.descriptor_id = descriptor.id.clone();
    let morale = sim.population.morale;
    let knowledge = sim.subsystems.get(id).unwrap().knowledge;

    annual_effects(&mut sim, &data);

    assert!((sim.population.morale - morale - descriptor.morale_per_year).abs() < 1e-6);
    assert!(
        (sim.subsystems.get(id).unwrap().knowledge - knowledge - descriptor.knowledge_per_year)
            .abs()
            < 1e-6
    );
}
