use super::*;
use crate::data::GameData;
use crate::state::sim::TerminalReason;

fn campaign() -> (GameData, crate::state::sim::SimState) {
    let data = GameData::load().expect("embedded data");
    let ids = crate::state::sim::founding_faction_ids(&data);
    let sim = crate::state::sim::SimState::new_campaign(&data, "preservers", 7, &ids);
    (data, sim)
}

#[test]
fn hull_loss_is_recorded_once() {
    let (data, mut sim) = campaign();
    sim.ship.hull_integrity = 0.0;
    let first = check_and_record(&mut sim, &data).expect("terminal");
    let second = check_and_record(&mut sim, &data).expect("same terminal");
    assert_eq!(first, second);
    assert_eq!(first.reason, TerminalReason::HullLoss);
}

#[test]
fn air_grace_allows_recovery_before_loss() {
    let (data, mut sim) = campaign();
    sim.ship.life_support = 0.0;
    for _ in 0..data.config.survival.air_grace_months - 1 {
        update_air_warning(&mut sim, &data);
    }
    assert!(check_and_record(&mut sim, &data).is_none());
    sim.ship.life_support = 0.2;
    update_air_warning(&mut sim, &data);
    assert_eq!(sim.survival.air_zero_months, 0);
    assert!(check_and_record(&mut sim, &data).is_none());
}

#[test]
fn population_loss_is_terminal_but_empty_stores_are_not() {
    let (data, mut sim) = campaign();
    sim.resources.food = 0;
    sim.ship.fuel = 0.0;
    assert!(check_and_record(&mut sim, &data).is_none());
    sim.population.count = 0;
    let result = check_and_record(&mut sim, &data).expect("population loss");
    assert_eq!(result.reason, TerminalReason::PopulationLoss);
}

#[test]
fn emergency_stabilisation_is_one_use_and_restores_air() {
    let (data, mut sim) = campaign();
    sim.ship.life_support = 0.0;
    sim.resources.energy = data.config.survival.emergency_resource_cost.energy;
    sim.resources.minerals = data.config.survival.emergency_resource_cost.minerals;
    sim.ship.spare_parts = data.config.survival.emergency_parts_cost;
    update_air_warning(&mut sim, &data);
    emergency_stabilise(&mut sim, &data).expect("emergency stores");
    assert!(sim.ship.life_support > 0.0);
    assert!(emergency_stabilise(&mut sim, &data).is_err());
}

#[test]
fn observing_damage_does_not_spend_air_grace() {
    let (data, mut sim) = campaign();
    sim.ship.life_support = 0.0;
    for month in 1..=12 {
        sim.month_clock = month;
        update_air_warning(&mut sim, &data);
        for _ in 0..3 {
            observe_air_warning(&mut sim, &data);
        }
        assert_eq!(sim.survival.air_zero_months, month);
        assert_eq!(check_and_record(&mut sim, &data).is_some(), month == 12);
    }
}

#[test]
fn monthly_driver_charges_one_air_month() {
    let (mut data, mut sim) = campaign();
    data.config.event_chance_base = 0.0;
    data.config.event_chance_cap = 0.0;
    sim.ship.life_support = 0.0;
    let template = data.contracts.get("deep_vein_survey").unwrap();
    sim.contract = Some(crate::simulation::contract::start_contract(template, &sim));
    let report = crate::simulation::tick::advance_months(&mut sim, &data, 1);
    assert_eq!(report.months_advanced, 1);
    assert_eq!(sim.survival.air_zero_months, 1);
    assert!(report.critical_warning);
    assert!(sim.terminal.is_none());
}

#[test]
fn neglected_air_really_loses_while_stabilisation_buys_time_for_overhaul() {
    let (mut data, mut neglected) = campaign();
    // Isolate the warned survival sequence from unrelated authored event choices.
    let ids: Vec<_> = data.events.ids().cloned().collect();
    for id in ids {
        data.events.remove(&id);
    }
    neglected.ship.life_support = 0.0;
    neglected.contract = Some(crate::simulation::contract::start_contract(
        data.contracts.get("deep_vein_survey").unwrap(),
        &neglected,
    ));
    let mut rescued = neglected.clone();
    emergency_stabilise(&mut rescued, &data).unwrap();
    let project = crate::simulation::projects::queue_project(
        &mut rescued,
        &data,
        "overhaul_life_support",
        None,
    )
    .unwrap();
    for _ in 0..12 {
        crate::simulation::tick::advance_months(&mut neglected, &data, 1);
    }
    assert_eq!(
        neglected.terminal.as_ref().unwrap().reason,
        TerminalReason::LifeSupportFailure
    );
    assert_eq!(neglected.month_clock, 12);
    for _ in 0..96 {
        if rescued.has_pending_decision() {
            break;
        }
        crate::simulation::tick::advance_months(&mut rescued, &data, 1);
    }
    assert!(rescued.terminal.is_none());
    assert_eq!(
        rescued.projects.find(project).unwrap().status,
        crate::state::sim::ProjectStatus::Completed
    );
    assert!(rescued.ship.life_support > data.config.survival.emergency_air_gain);
}
