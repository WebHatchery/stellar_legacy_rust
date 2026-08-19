use super::*;

#[test]
fn loadout_sums_installed_component_stats() {
    let data = GameData::load().unwrap();
    let sim = SimState::new_campaign(
        &data,
        "preservers",
        3,
        &crate::state::sim::founding_faction_ids(&data),
    );
    // Founding loadout: colony_barge hull + ion_drive engine, no weapon.
    let stats = loadout_stats(&sim, &data);
    assert_eq!(stats.cargo, 200); // colony_barge
    assert_eq!(stats.speed, 2); // ion_drive
    assert_eq!(stats.combat, 0); // no weapon
}

#[test]
fn field_repair_patches_but_never_reaches_pristine() {
    let data = GameData::load().unwrap();
    let mut sim = SimState::new_campaign(
        &data,
        "preservers",
        1,
        &crate::state::sim::founding_faction_ids(&data),
    );
    sim.ship.hull_integrity = 0.3;
    sim.ship.spare_parts = 100;
    sim.resources.minerals = 100_000;

    for _ in 0..20 {
        let _ = field_repair(&mut sim, &data.config, RepairKind::Hull);
    }
    let ceiling = data.config.repair.field_ceiling;
    assert!(
        (sim.ship.hull_integrity - ceiling).abs() < 1e-4,
        "field repair tops out at the ceiling ({ceiling}), got {}",
        sim.ship.hull_integrity
    );
    assert!(sim.ship.hull_integrity < 1.0, "never pristine in the black");
    assert!(sim.ship.spare_parts < 100, "field repair spends parts");
}

#[test]
fn field_repair_delivers_the_condition_it_projects() {
    let data = GameData::load().unwrap();
    let mut sim = SimState::new_campaign(
        &data,
        "preservers",
        17,
        &crate::state::sim::founding_faction_ids(&data),
    );
    sim.ship.hull_integrity = 0.41;
    sim.ship.spare_parts = 100;
    sim.resources.minerals = 100_000;
    let target = field_repair_target(sim.ship.hull_integrity, &data.config);

    field_repair(&mut sim, &data.config, RepairKind::Hull).unwrap();

    assert_eq!(sim.ship.hull_integrity, target);
}

#[test]
fn field_repair_refused_without_parts() {
    let data = GameData::load().unwrap();
    let mut sim = SimState::new_campaign(
        &data,
        "preservers",
        1,
        &crate::state::sim::founding_faction_ids(&data),
    );
    sim.ship.hull_integrity = 0.3;
    sim.ship.spare_parts = 0;
    sim.resources.minerals = 100_000;
    assert!(field_repair(&mut sim, &data.config, RepairKind::Hull).is_err());
}

#[test]
fn full_repair_is_port_only_and_restores_everything() {
    use crate::simulation::contract::start_contract;
    let data = GameData::load().unwrap();
    let mut sim = SimState::new_campaign(
        &data,
        "preservers",
        1,
        &crate::state::sim::founding_faction_ids(&data),
    );
    sim.ship.hull_integrity = 0.3;
    sim.ship.life_support = 0.4;
    sim.ship.fuel = 0.2;
    sim.ship.spare_parts = 0;
    sim.resources.credits = 100_000;
    sim.resources.minerals = 100_000;

    // Underway: refused.
    let template = data.contracts.get("deep_vein_survey").unwrap().clone();
    sim.contract = Some(start_contract(&template, &sim));
    assert!(
        full_repair(&mut sim, &data.config).is_err(),
        "no full refit underway"
    );

    // In port: restores the ship to whole and tops parts back up.
    sim.contract = None;
    full_repair(&mut sim, &data.config).unwrap();
    assert_eq!(sim.ship.hull_integrity, 1.0);
    assert_eq!(sim.ship.life_support, 1.0);
    assert_eq!(sim.ship.fuel, 1.0);
    assert!(sim.ship.spare_parts >= data.config.repair.full_parts_restock);
}

#[test]
fn salvage_field_install_is_gated_by_crew_and_part() {
    use crate::simulation::contract::start_contract;
    let data = GameData::load().unwrap();
    let mut sim = SimState::new_campaign(
        &data,
        "preservers",
        1,
        &crate::state::sim::founding_faction_ids(&data),
    );
    sim.ship.spare_parts = 100;
    sim.resources.minerals = 100_000;
    // Underway.
    let template = data.contracts.get("deep_vein_survey").unwrap().clone();
    sim.contract = Some(start_contract(&template, &sim));

    // A field-installable weapon in the hold, with a skilled engineer aboard
    // (the founding crew includes an engineer), installs underway.
    let eng = sim
        .crew
        .iter_mut()
        .find(|c| c.archetype_id == "engineer")
        .unwrap();
    eng.skill = data.config.field_install.skill_required + 5;
    sim.ship.salvage.push("mass_driver".to_owned());
    assert_eq!(
        install_eligibility(&sim, &data, "mass_driver"),
        InstallEligibility::Ready
    );
    install_salvage(&mut sim, &data, "mass_driver").unwrap();
    assert_eq!(sim.ship.weapon.as_deref(), Some("mass_driver"));
    assert!(!sim.ship.salvage.iter().any(|s| s == "mass_driver"));

    // A hull is not field-installable — it must wait for a drydock.
    sim.ship.salvage.push("generation_ark".to_owned());
    assert_eq!(
        install_eligibility(&sim, &data, "generation_ark"),
        InstallEligibility::NeedsDrydock
    );
    assert!(install_salvage(&mut sim, &data, "generation_ark").is_err());

    // With no skilled engineer, even a modular part can't be fitted underway.
    sim.crew.retain(|c| c.archetype_id != "engineer");
    sim.ship.salvage.push("flak_screen".to_owned());
    assert_eq!(
        install_eligibility(&sim, &data, "flak_screen"),
        InstallEligibility::NeedsEngineer
    );
}

#[test]
fn salvage_installs_freely_in_port() {
    let data = GameData::load().unwrap();
    let mut sim = SimState::new_campaign(
        &data,
        "preservers",
        1,
        &crate::state::sim::founding_faction_ids(&data),
    );
    sim.contract = None; // in port
    sim.ship.spare_parts = 0;
    sim.resources.minerals = 0;
    // Even a hull installs in the drydock, with no crew or consumables.
    sim.crew.clear();
    sim.ship.salvage.push("generation_ark".to_owned());
    assert_eq!(
        install_eligibility(&sim, &data, "generation_ark"),
        InstallEligibility::Ready
    );
    install_salvage(&mut sim, &data, "generation_ark").unwrap();
    assert_eq!(sim.ship.hull, "generation_ark");
    assert!(sim.ship.salvage.is_empty());
}

#[test]
fn granted_component_lands_in_the_salvage_hold() {
    use crate::simulation::event_resolver::apply_outcome;
    let data = GameData::load().unwrap();
    let mut sim = SimState::new_campaign(
        &data,
        "wanderers",
        1,
        &crate::state::sim::founding_faction_ids(&data),
    );
    let template = data.events.get("derelict_encounter").unwrap().clone();
    let idx = template
        .outcomes
        .iter()
        .position(|o| o.grant_component.is_some())
        .expect("derelict_encounter grants a salvage part");
    apply_outcome(&mut sim, &data, &template, idx);
    assert!(
        !sim.ship.salvage.is_empty(),
        "boarding a derelict fills the salvage hold"
    );
}

#[test]
fn commission_refits_and_lifts_hope_but_keeps_the_people() {
    use crate::simulation::contract::start_contract;
    let data = GameData::load().unwrap();
    let mut sim = SimState::new_campaign(
        &data,
        "preservers",
        1,
        &crate::state::sim::founding_faction_ids(&data),
    );
    sim.resources.credits = 100_000;
    sim.resources.minerals = 100_000;
    sim.ship.hull_integrity = 0.3;
    sim.ship.life_support = 0.4;
    sim.ship.spare_parts = 0;
    sim.population.morale = 0.4;
    let drift_before = sim.population.cultural_drift;

    // Underway: refused.
    let template = data.contracts.get("deep_vein_survey").unwrap().clone();
    sim.contract = Some(start_contract(&template, &sim));
    assert!(commission_ship(&mut sim, &data, "generation_ark").is_err());

    // In port: swaps hull, full refit, morale lift; the people don't reset.
    sim.contract = None;
    commission_ship(&mut sim, &data, "generation_ark").unwrap();
    assert_eq!(sim.ship.hull, "generation_ark");
    assert_eq!(sim.ship.hull_integrity, 1.0);
    assert_eq!(sim.ship.life_support, 1.0);
    assert!(sim.ship.spare_parts >= data.config.repair.full_parts_restock);
    assert!(sim.population.morale > 0.4, "a fresh hull lifts hope");
    assert_eq!(
        sim.population.cultural_drift, drift_before,
        "commissioning a ship never resets who the people have become"
    );
}

#[test]
fn commission_needs_the_full_price() {
    let data = GameData::load().unwrap();
    let mut sim = SimState::new_campaign(
        &data,
        "preservers",
        1,
        &crate::state::sim::founding_faction_ids(&data),
    );
    sim.contract = None;
    sim.resources.credits = 0;
    sim.resources.minerals = 0;
    assert!(commission_ship(&mut sim, &data, "generation_ark").is_err());
}

#[test]
fn loadout_effects_add_production_bonus() {
    let data = GameData::load().unwrap();
    let mut sim = SimState::new_campaign(
        &data,
        "preservers",
        3,
        &crate::state::sim::founding_faction_ids(&data),
    );
    let credits = sim.resources.credits;
    let minerals = sim.resources.minerals;

    apply_loadout_effects(&mut sim, &data);

    let stats = loadout_stats(&sim, &data);
    assert_eq!(
        sim.resources.credits,
        credits + stats.speed as i64 * data.config.ship.credits_per_speed
    );
    assert!(sim.resources.minerals > minerals, "cargo yields minerals");
}

#[test]
fn buying_parts_is_port_only_and_charges_per_part() {
    use crate::simulation::contract::start_contract;
    let data = GameData::load().unwrap();
    let mut sim = SimState::new_campaign(
        &data,
        "preservers",
        1,
        &crate::state::sim::founding_faction_ids(&data),
    );
    sim.resources.credits = 10_000;
    let parts_before = sim.ship.spare_parts;

    // Underway: refused.
    let template = data.contracts.get("deep_vein_survey").unwrap().clone();
    sim.contract = Some(start_contract(&template, &sim));
    assert!(buy_parts(&mut sim, &data.config, 10).is_err());

    // In port: parts land, credits leave at the configured price.
    sim.contract = None;
    buy_parts(&mut sim, &data.config, 10).unwrap();
    assert_eq!(sim.ship.spare_parts, parts_before + 10);
    assert_eq!(
        sim.resources.credits,
        10_000 - 10 * data.config.provisioning.part_cost_credits
    );

    // Zero or unaffordable orders are refused whole.
    assert!(buy_parts(&mut sim, &data.config, 0).is_err());
    sim.resources.credits = 5;
    assert!(buy_parts(&mut sim, &data.config, 10).is_err());
}

#[test]
fn refuel_is_port_only_and_charges_by_the_missing_fraction() {
    use crate::simulation::contract::start_contract;
    let data = GameData::load().unwrap();
    let mut sim = SimState::new_campaign(
        &data,
        "preservers",
        1,
        &crate::state::sim::founding_faction_ids(&data),
    );
    sim.resources.credits = 100_000;
    sim.ship.fuel = 0.5;

    // Underway: refused.
    let template = data.contracts.get("deep_vein_survey").unwrap().clone();
    sim.contract = Some(start_contract(&template, &sim));
    assert!(
        refuel(&mut sim, &data.config).is_err(),
        "no refuel underway"
    );

    // In port: tops to full and charges missing × cost/point × 100.
    sim.contract = None;
    let before = sim.resources.credits;
    refuel(&mut sim, &data.config).unwrap();
    assert_eq!(sim.ship.fuel, 1.0);
    let expected =
        (data.config.provisioning.fuel_cost_credits_per_point as f32 * 0.5 * 100.0).ceil() as i64;
    assert_eq!(before - sim.resources.credits, expected);

    // Already full: refused.
    assert!(refuel(&mut sim, &data.config).is_err());
}

#[test]
fn a_complete_refit_cannot_charge_for_no_work() {
    let data = GameData::load().unwrap();
    let mut sim = SimState::new_campaign(
        &data,
        "preservers",
        52,
        &crate::state::sim::founding_faction_ids(&data),
    );
    sim.ship.hull_integrity = 1.0;
    sim.ship.life_support = 1.0;
    sim.ship.fuel = 1.0;
    sim.ship.spare_parts = data.config.repair.full_parts_restock;
    sim.resources.credits = 100_000;
    sim.resources.minerals = 100_000;
    assert!(!full_repair_needed(&sim, &data.config));
    let before = sim.resources;
    assert!(full_repair(&mut sim, &data.config).is_err());
    assert_eq!(sim.resources.credits, before.credits);
    assert_eq!(sim.resources.minerals, before.minerals);

    sim.ship.fuel = 0.99;
    assert!(full_repair_needed(&sim, &data.config));
    full_repair(&mut sim, &data.config).unwrap();
    assert_eq!(sim.ship.fuel, 1.0);
}
