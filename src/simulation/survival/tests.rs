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
