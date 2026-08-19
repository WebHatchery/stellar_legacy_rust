use super::*;

fn campaign() -> (crate::data::GameData, SimState) {
    let data = crate::data::GameData::load().unwrap();
    let sim = SimState::new_campaign(
        &data,
        "preservers",
        21,
        &crate::state::sim::founding_faction_ids(&data),
    );
    (data, sim)
}

#[test]
fn primary_risk_names_the_exact_weakest_reserve_and_stays_quiet_when_sound() {
    let (data, mut sim) = campaign();
    let (label, score) = primary_risk(&sim, &data.config);
    assert_eq!(label, "ALL SYSTEMS SOUND");
    assert!(score >= 0.75);

    sim.resources.energy = data.config.low_energy_threshold / 5;
    let (label, score) = primary_risk(&sim, &data.config);
    assert_eq!(
        label,
        format!(
            "ENERGY {}/{}",
            sim.resources.energy, data.config.low_energy_threshold
        )
    );
    assert!((score - 0.2).abs() < 0.01);

    sim.resources.energy = data.config.low_energy_threshold;
    sim.resources.food = 0;
    let (label, score) = primary_risk(&sim, &data.config);
    assert_eq!(label, "FOOD 0.0Y");
    assert_eq!(score, 0.0);
}

#[test]
fn weakest_module_readout_does_not_invent_a_decline() {
    let (data, mut sim) = campaign();
    assert_eq!(weakest_module_readout(&sim, &data), "ALL MODULES SOUND");

    let subsystem = sim.subsystems.get_mut("life_support_habitat").unwrap();
    subsystem.condition = 0.42;
    let readout = weakest_module_readout(&sim, &data);
    assert!(readout.contains("42%"));
    assert!(!readout.contains('↓'));
}
