use super::*;
use crate::data::GameData;
use crate::state::sim::ProjectStatus;

pub(crate) fn advance_projects(sim: &mut SimState, data: &GameData) {
    let snapshot = capture_month(sim, data);
    advance_captured_month(sim, data, snapshot);
}

fn campaign() -> (GameData, SimState) {
    let data = GameData::load().expect("embedded data");
    let ids = crate::state::sim::founding_faction_ids(&data);
    let mut sim = SimState::new_campaign(&data, "preservers", 11, &ids);
    let charter = data.contracts.ids().next().cloned().expect("charter");
    let template = data.contracts.get(&charter).unwrap();
    sim.contract = Some(crate::simulation::contract::start_contract(template, &sim));
    (data, sim)
}

#[test]
fn queued_training_charges_only_when_started_and_completes_once() {
    let (data, mut sim) = campaign();
    let before = sim.resources.credits;
    let id = queue_project(
        &mut sim,
        &data,
        "train_replacement_cohort",
        Some("agriculture".to_owned()),
    )
    .expect("queue");
    assert_eq!(
        sim.resources.credits,
        before
            - data
                .projects
                .get("train_replacement_cohort")
                .unwrap()
                .cost
                .credits
    );
    let duration = data
        .projects
        .get("train_replacement_cohort")
        .unwrap()
        .duration_months;
    for _ in 0..duration {
        advance_projects(&mut sim, &data);
    }
    let job = sim.projects.find(id).unwrap();
    assert_eq!(job.status, ProjectStatus::Completed);
    assert_eq!(job.delivered_stages, 2);
    let knowledge = sim.subsystems.get("agriculture").unwrap().knowledge;
    assert!(knowledge > data.config.subsystems.knowledge_start);
}

#[test]
fn paused_work_ages_after_grace_and_cancellation_refunds_remaining_escrow() {
    let (data, mut sim) = campaign();
    sim.ship.hull_integrity = 0.7;
    let id = queue_project(&mut sim, &data, "restore_hull", None).unwrap();
    pause_project(&mut sim, &data, id).unwrap();
    for _ in 0..data.config.projects.pause_grace_months + 2 {
        age_paused_projects(&mut sim, &data);
    }
    let debt = sim.projects.find(id).unwrap().restoration_debt.minerals;
    assert!(debt > 0.0);
    let refund = cancel_project(&mut sim, &data, id).unwrap();
    assert!(refund.minerals < 80.0 * data.config.projects.cancellation_refund_fraction as f64);
}

mod controls {
    include!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/unit/simulation/projects/tests/controls.rs"));
}
mod lifecycle {
    include!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/unit/simulation/projects/tests/lifecycle.rs"));
}
