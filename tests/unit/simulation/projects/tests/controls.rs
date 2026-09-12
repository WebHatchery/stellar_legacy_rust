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

#[test]
fn reordering_waiting_work_skips_active_and_ended_jobs_without_charging() {
    let (data, mut sim, id) = paused_hull();
    let mut running = ProjectInstance::queued(100, "restore_hull", None, 0);
    running.status = ProjectStatus::Running;
    sim.projects.jobs.push(running);
    sim.projects.jobs.push(ProjectInstance::queued(
        101,
        "overhaul_life_support",
        None,
        0,
    ));
    let mut ended = ProjectInstance::queued(102, "restore_hull", None, 0);
    ended.status = ProjectStatus::Completed;
    sim.projects.jobs.push(ended);
    let before = serde_json::to_string(&sim.resources).unwrap();
    move_project(&mut sim, 101, -1).unwrap();
    assert_eq!(sim.projects.waiting_position(101), Some((1, 2)));
    assert_eq!(sim.projects.waiting_position(id), Some((2, 2)));
    assert_eq!(sim.projects.jobs[1].sequence_id, 100);
    assert_eq!(sim.projects.jobs[3].sequence_id, 102);
    assert_eq!(serde_json::to_string(&sim.resources).unwrap(), before);
    let ordered = serde_json::to_string(&sim.projects).unwrap();
    move_project(&mut sim, 101, -1).unwrap();
    assert_eq!(serde_json::to_string(&sim.projects).unwrap(), ordered);
    // Reordering does not change the resumed project's eligibility.
    assert!(resume_check(&sim, &data, id).is_ok());
}

#[test]
fn catalogue_queue_check_rejects_live_duplicates_but_allows_ended_work() {
    let (data, mut sim, id) = paused_hull();
    let definition = data.projects.get("restore_hull").unwrap();
    for status in [
        ProjectStatus::Running,
        ProjectStatus::Paused,
        ProjectStatus::Queued,
    ] {
        sim.projects.find_mut(id).unwrap().status = status;
        let before = serde_json::to_string(&sim).unwrap();
        let check = queue_check(&sim, &data, definition, None);
        assert!(!check.eligible);
        assert_eq!(
            live_job(&sim, "restore_hull", None).unwrap().sequence_id,
            id
        );
        assert_eq!(
            queue_project(&mut sim, &data, "restore_hull", None).unwrap_err(),
            check.reason
        );
        assert_eq!(serde_json::to_string(&sim).unwrap(), before);
    }
    for status in [
        ProjectStatus::Cancelled,
        ProjectStatus::Stopped,
        ProjectStatus::Completed,
    ] {
        sim.projects.find_mut(id).unwrap().status = status;
        assert!(live_job(&sim, "restore_hull", None).is_none());
        assert!(queue_check(&sim, &data, definition, None).eligible);
    }
}

#[test]
fn queue_check_matches_capacity_rejection_and_keeps_unpaid_work_available() {
    let (data, mut sim) = campaign();
    sim.ship.hull_integrity = 0.5;
    sim.contract = None;
    sim.resources.minerals = 0;
    sim.resources.credits = 0;
    sim.ship.spare_parts = 0;
    let definition = data.projects.get("restore_hull").unwrap();
    assert!(queue_check(&sim, &data, definition, None).eligible);
    let id = queue_project(&mut sim, &data, "restore_hull", None).unwrap();
    assert_eq!(sim.projects.find(id).unwrap().status, ProjectStatus::Queued);
    assert_eq!(sim.resources.minerals, 0);
    assert_eq!(sim.resources.credits, 0);
    assert_eq!(sim.ship.spare_parts, 0);
    for index in 1..data.config.projects.waiting_cap {
        sim.projects.jobs.push(ProjectInstance::queued(
            100 + index as u64,
            "other",
            None,
            0,
        ));
    }
    let definition = data.projects.get("overhaul_life_support").unwrap();
    let check = queue_check(&sim, &data, definition, None);
    assert!(!check.eligible && check.reason.contains("waiting list is full"));
    assert_eq!(
        queue_project(&mut sim, &data, "overhaul_life_support", None).unwrap_err(),
        check.reason
    );
}
