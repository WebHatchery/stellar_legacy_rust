use super::*;
use crate::simulation::projects;
use crate::state::sim::ProjectStatus;

fn campaign() -> (crate::data::GameData, SimState) {
    let data = crate::data::GameData::load().unwrap();
    let mut sim = SimState::new_campaign(
        &data,
        "preservers",
        11,
        &crate::state::sim::founding_faction_ids(&data),
    );
    sim.contract = Some(crate::simulation::contract::start_contract(
        data.contracts.get("deep_vein_survey").unwrap(),
        &sim,
    ));
    (data, sim)
}

#[test]
fn investments_round_trip_with_paid_debt_and_fractional_change() {
    let (data, mut sim) = campaign();
    sim.ship.hull_integrity = 0.5;
    let id = projects::queue_project(&mut sim, &data, "restore_hull", None).unwrap();
    sim.month_clock = 1;
    projects::advance_projects(&mut sim, &data);
    projects::pause_project(&mut sim, &data, id).unwrap();
    for _ in 0..13 {
        projects::advance_projects(&mut sim, &data);
    }
    projects::resume_project(&mut sim, &data, id).unwrap();
    projects::pause_project(&mut sim, &data, id).unwrap();
    let value = serde_json::to_value(SaveData {
        version: data.config.version.clone(),
        sim: sim.clone(),
    })
    .unwrap();
    let loaded = migrate_save_value(Some(data.config.version.clone()), value, &data.config)
        .unwrap()
        .sim;
    assert_eq!(
        loaded.projects.find(id).unwrap().status,
        ProjectStatus::Paused
    );
    assert_eq!(
        loaded.projects.find(id).unwrap().stage_pause_months,
        vec![13, 13]
    );
    assert_eq!(
        loaded.projects.find(id).unwrap().remaining_escrow,
        sim.projects.find(id).unwrap().remaining_escrow
    );
    assert_eq!(
        loaded.projects.settlement_balance,
        sim.projects.settlement_balance
    );
}

#[test]
fn rejects_duplicate_unknown_and_inflated_investments() {
    let (data, mut sim) = campaign();
    sim.ship.hull_integrity = 0.5;
    let id = projects::queue_project(&mut sim, &data, "restore_hull", None).unwrap();
    let mut invalid = sim.clone();
    invalid.projects.jobs.push(invalid.projects.jobs[0].clone());
    assert!(validation::validate(&invalid, &data).is_err());
    invalid = sim.clone();
    invalid.projects.jobs[0].project_id = "missing".into();
    assert!(validation::validate(&invalid, &data).is_err());
    invalid = sim.clone();
    invalid
        .projects
        .find_mut(id)
        .unwrap()
        .remaining_escrow
        .minerals += 1.0;
    assert!(validation::validate(&invalid, &data).is_err());
}

#[test]
fn old_quarters_keep_their_original_single_delivery() {
    let (data, mut sim) = campaign();
    let id = projects::queue_project(&mut sim, &data, "restore_crew_quarters", None).unwrap();
    sim.projects.find_mut(id).unwrap().elapsed_months = 30;
    let value = serde_json::to_value(SaveData {
        version: "0.2.0".into(),
        sim,
    })
    .unwrap();
    let mut loaded = migrate_save_value(Some("0.2.0".into()), value, &data.config)
        .unwrap()
        .sim;
    assert!(loaded.projects.find(id).unwrap().legacy_single_delivery);
    projects::advance_projects(&mut loaded, &data);
    assert_eq!(loaded.projects.find(id).unwrap().delivered_stages, 0);
}

#[test]
fn legacy_terminal_states_pause_with_notice_and_unknown_versions_fail() {
    let (data, mut sim) = campaign();
    sim.ship.hull_integrity = 0.0;
    let value = serde_json::to_value(SaveData {
        version: "0.1.0".into(),
        sim,
    })
    .unwrap();
    let loaded = migrate_save_value(Some("0.1.0".into()), value.clone(), &data.config).unwrap();
    assert!(loaded.sim.survival.migration_notice.is_some());
    assert_eq!(loaded.sim.speed, crate::state::sim::GameSpeed::Paused);
    assert!(migrate_save_value(Some("99.0.0".into()), value, &data.config).is_err());
}
