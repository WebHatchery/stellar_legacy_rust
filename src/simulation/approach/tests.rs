use super::*;
use crate::data::GameData;

#[test]
fn charter_approaches_keep_distinct_tradeoffs() {
    assert!(
        objective_factor(CharterApproach::ProveTheWrit)
            > objective_factor(CharterApproach::ProtectTheMargin)
    );
    assert!(
        event_chance_factor(CharterApproach::CarryThePeople)
            < event_chance_factor(CharterApproach::ProtectTheMargin)
    );
    assert_eq!(
        objective_factor(CharterApproach::ProtectTheMargin),
        1.0,
        "the default preserves authored charter pace"
    );
    assert!(
        fuel_burn_factor(CharterApproach::ProtectTheMargin)
            < fuel_burn_factor(CharterApproach::ProveTheWrit)
    );
    assert!(
        preserve_attrition_factor(CharterApproach::CarryThePeople)
            < preserve_attrition_factor(CharterApproach::ProtectTheMargin)
    );
}

#[test]
fn prove_the_writ_names_its_speed_requirement() {
    let data = GameData::load().unwrap();
    let sim = SimState::new_campaign(
        &data,
        "preservers",
        401,
        &crate::state::sim::founding_faction_ids(&data),
    );
    let template = data
        .contracts
        .get(crate::data::contracts::TUTORIAL_CONTRACT_ID)
        .unwrap();

    let reason = unavailable_reason(&sim, &data, template, CharterApproach::ProveTheWrit)
        .expect("the starting ship should not prove a writ at speed 2");
    assert!(reason.contains("Requires ship speed 4"));
}

#[test]
fn effect_summary_exposes_people_and_preserve_tradeoffs() {
    let summary = effect_summary(CharterApproach::CarryThePeople);
    assert!(summary.contains("EVENTS -12%"));
    assert!(summary.contains("PRESERVE LOSS -28%"));
    assert!(summary.contains("morale"));
}

#[test]
fn launch_snapshots_the_approach_and_applies_annual_people_effects() {
    let data = GameData::load().unwrap();
    let mut sim = SimState::new_campaign(
        &data,
        "preservers",
        402,
        &crate::state::sim::founding_faction_ids(&data),
    );
    sim.selected_charter_approach = CharterApproach::CarryThePeople;
    let template = data
        .contracts
        .get(crate::data::contracts::TUTORIAL_CONTRACT_ID)
        .unwrap();
    sim.contract = Some(crate::simulation::contract::start_contract(template, &sim));

    assert_eq!(
        sim.contract.as_ref().unwrap().approach,
        CharterApproach::CarryThePeople
    );
    let morale = sim.population.morale;
    annual_effects(&mut sim);
    assert!(sim.population.morale > morale);
}
