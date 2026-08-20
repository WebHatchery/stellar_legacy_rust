use super::*;
use crate::state::sim::CommandPosture;

#[test]
fn expeditionary_posture_buys_speed_with_risk() {
    assert!(
        objective_factor(CommandPosture::Expeditionary) > objective_factor(CommandPosture::Steady)
    );
    assert!(
        event_chance_factor(CommandPosture::Expeditionary)
            > event_chance_factor(CommandPosture::Steady)
    );
    assert!(
        fuel_burn_factor(CommandPosture::Expeditionary) > fuel_burn_factor(CommandPosture::Steady)
    );
}

#[test]
fn civic_posture_is_slower_but_more_socially_gentle() {
    assert!(objective_factor(CommandPosture::Civic) < objective_factor(CommandPosture::Steady));
    assert!(
        event_chance_factor(CommandPosture::Civic)
            < event_chance_factor(CommandPosture::Expeditionary)
    );

    let data = crate::data::GameData::load().unwrap();
    let legacy = crate::data::GameData::sorted_ids(&data.legacies)[0].clone();
    let factions = crate::state::sim::founding_faction_ids(&data);
    let mut sim = crate::state::sim::SimState::new_campaign(&data, &legacy, 7, &factions);
    let contract_id = crate::data::GameData::sorted_ids(&data.contracts)[0].clone();
    sim.command_posture = CommandPosture::Civic;
    sim.contract = Some(crate::simulation::contract::start_contract(
        data.contracts.get(&contract_id).unwrap(),
        &sim,
    ));
    sim.population.morale = 0.5;
    apply_annual_effects(&mut sim);
    assert!((sim.population.morale - 0.515).abs() < 0.0001);
    assert!((sim.population.unity - 0.71).abs() < 0.0001);
}

#[test]
fn annual_posture_effects_stop_in_port() {
    let data = crate::data::GameData::load().unwrap();
    let legacy = crate::data::GameData::sorted_ids(&data.legacies)[0].clone();
    let factions = crate::state::sim::founding_faction_ids(&data);
    let mut sim = crate::state::sim::SimState::new_campaign(&data, &legacy, 7, &factions);
    sim.command_posture = CommandPosture::Civic;
    sim.population.morale = 0.5;
    apply_annual_effects(&mut sim);
    assert_eq!(sim.population.morale, 0.5);
}

#[test]
fn annual_posture_review_leaves_a_legible_log_entry() {
    let data = crate::data::GameData::load().unwrap();
    let legacy = crate::data::GameData::sorted_ids(&data.legacies)[0].clone();
    let factions = crate::state::sim::founding_faction_ids(&data);
    let mut sim = crate::state::sim::SimState::new_campaign(&data, &legacy, 7, &factions);
    let contract_id = crate::data::GameData::sorted_ids(&data.contracts)[0].clone();
    sim.contract = Some(crate::simulation::contract::start_contract(
        data.contracts.get(&contract_id).unwrap(),
        &sim,
    ));
    sim.command_posture = CommandPosture::Expeditionary;

    apply_annual_effects(&mut sim);

    assert!(sim
        .log
        .last()
        .is_some_and(|entry| entry.text.contains("Expeditionary posture")));
}

#[test]
fn underway_posture_reviews_are_once_per_year() {
    let data = crate::data::GameData::load().unwrap();
    let legacy = crate::data::GameData::sorted_ids(&data.legacies)[0].clone();
    let factions = crate::state::sim::founding_faction_ids(&data);
    let mut sim = crate::state::sim::SimState::new_campaign(&data, &legacy, 7, &factions);
    let contract_id = crate::data::GameData::sorted_ids(&data.contracts)[0].clone();
    sim.contract = Some(crate::simulation::contract::start_contract(
        data.contracts.get(&contract_id).unwrap(),
        &sim,
    ));
    sim.month_clock = 24;
    assert!(posture_change_allowed(&sim));
    sim.command_posture_locked_until = next_review_month(&sim);
    assert_eq!(sim.command_posture_locked_until, 36);
    assert!(!posture_change_allowed(&sim));
    sim.month_clock = 36;
    assert!(posture_change_allowed(&sim));
}
