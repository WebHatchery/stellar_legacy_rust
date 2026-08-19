use super::*;

fn active_campaign() -> (GameData, SimState) {
    let data = GameData::load().unwrap();
    let mut sim = SimState::new_campaign(
        &data,
        "preservers",
        42,
        &crate::state::sim::founding_faction_ids(&data),
    );
    let template = data.contracts.get("founding_colony").unwrap();
    sim.contract = Some(crate::simulation::contract::start_contract(template, &sim));
    (data, sim)
}

#[test]
fn mission_clock_status_only_calls_a_dry_travel_leg_stalled() {
    let (data, mut sim) = active_campaign();
    sim.ship.fuel = 1.0;
    let (status, stalled) = mission_clock_status(&sim, &data);
    assert!(!stalled);
    assert!(status.starts_with("RUNNING"));

    sim.ship.fuel = 0.0;
    let (status, stalled) = mission_clock_status(&sim, &data);
    assert!(stalled);
    assert!(status.starts_with("STALLED"));
    assert!(status.contains("FUEL TO ADVANCE"));

    let contract = sim.contract.as_mut().unwrap();
    contract.months_elapsed = contract
        .phases
        .iter()
        .take_while(|phase| phase.kind == ContractPhase::Travel)
        .map(|phase| phase.years * 12)
        .sum::<u32>();
    contract.phase = ContractPhase::Operation;
    contract.phase_index = contract
        .phases
        .iter()
        .position(|phase| phase.kind == ContractPhase::Operation)
        .unwrap();
    let (status, stalled) = mission_clock_status(&sim, &data);
    assert!(!stalled);
    assert_eq!(status, "RUNNING · CURRENT LEG NEEDS NO FUEL");
}
