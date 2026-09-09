use super::*;

fn paused_hull() -> (GameData, SimState, u64) {
    let (data, mut sim) = campaign();
    sim.ship.hull_integrity = 0.5;
    let id = queue_project(&mut sim, &data, "restore_hull", None).unwrap();
    pause_project(&mut sim, &data, id).unwrap();
    (data, sim, id)
}

#[test]
fn resume_preview_uses_saved_fractional_change_without_charging_stores() {
    let (data, mut sim, id) = paused_hull();
    sim.resources.minerals = 0;
    sim.projects.settlement_balance.minerals = 0.5;
    sim.projects.find_mut(id).unwrap().restoration_debt.minerals = 0.4;
    let before = serde_json::to_string(&sim).unwrap();
    for _ in 0..3 {
        resume_check(&sim, &data, id).unwrap();
    }
    assert_eq!(serde_json::to_string(&sim).unwrap(), before);
    resume_project(&mut sim, &data, id).unwrap();
    assert_eq!(sim.resources.minerals, 0);
    assert!((sim.projects.settlement_balance.minerals - 0.1).abs() < 1e-8);
    assert_eq!(
        sim.projects.find(id).unwrap().status,
        ProjectStatus::Running
    );
}

#[test]
fn unaffordable_resume_preview_matches_the_command_and_preserves_the_job() {
    let (data, mut sim, id) = paused_hull();
    sim.ship.spare_parts = 0;
    sim.projects
        .find_mut(id)
        .unwrap()
        .restoration_debt
        .spare_parts = 0.4;
    let before = serde_json::to_string(&sim).unwrap();
    let reason = resume_check(&sim, &data, id).unwrap_err();
    assert!(reason.contains("Insufficient stores"));
    assert_eq!(resume_project(&mut sim, &data, id).unwrap_err(), reason);
    assert_eq!(serde_json::to_string(&sim).unwrap(), before);
}

#[test]
fn resume_preview_accounts_for_port_and_occupied_slots() {
    let (data, mut sim, id) = paused_hull();
    let contract = sim.contract.take();
    assert!(resume_check(&sim, &data, id).unwrap_err().contains("port"));
    sim.contract = contract;
    for index in 0..data.config.projects.concurrent_slots {
        let mut other = ProjectInstance::queued(100 + index as u64, "restore_hull", None, 0);
        other.status = ProjectStatus::Running;
        sim.projects.jobs.push(other);
    }
    assert!(resume_check(&sim, &data, id)
        .unwrap_err()
        .contains("occupied"));
}

#[test]
fn pause_preview_explains_a_full_waiting_list_without_mutation() {
    let (data, mut sim, id) = paused_hull();
    resume_project(&mut sim, &data, id).unwrap();
    for index in 0..data.config.projects.waiting_cap {
        sim.projects.jobs.push(ProjectInstance::queued(
            100 + index as u64,
            "restore_hull",
            None,
            0,
        ));
    }
    let before = serde_json::to_string(&sim).unwrap();
    let reason = pause_check(&sim, &data, id).unwrap_err();
    assert!(reason.contains("Waiting list full"));
    assert_eq!(pause_project(&mut sim, &data, id).unwrap_err(), reason);
    assert_eq!(serde_json::to_string(&sim).unwrap(), before);
}
