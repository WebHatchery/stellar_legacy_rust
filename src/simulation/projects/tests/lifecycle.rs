use super::*;

#[test]
fn lost_expertise_preserves_investment_for_training_recovery() {
    let (data, mut sim) = campaign();
    let sub = sim.subsystems.get_mut("agriculture").unwrap();
    sub.condition = 0.5;
    sub.knowledge = 1.0;
    let id = queue_project(
        &mut sim,
        &data,
        "service_subsystem",
        Some("agriculture".into()),
    )
    .unwrap();
    advance_projects(&mut sim, &data);
    let escrow = sim.projects.find(id).unwrap().remaining_escrow;
    sim.subsystems.get_mut("agriculture").unwrap().knowledge = 0.0;
    advance_projects(&mut sim, &data);
    let job = sim.projects.find(id).unwrap();
    assert_eq!(job.status, ProjectStatus::Paused);
    assert_eq!(job.elapsed_months, 1);
    assert_eq!(job.remaining_escrow, escrow);
    sim.subsystems.get_mut("agriculture").unwrap().knowledge = 1.0;
    resume_project(&mut sim, &data, id).unwrap();
    advance_projects(&mut sim, &data);
    assert_eq!(sim.projects.find(id).unwrap().elapsed_months, 2);
}

#[test]
fn half_quarters_pivot_keeps_deliveries_and_refunds_only_unfinished_work() {
    let (data, mut sim) = campaign();
    sim.population.morale = 0.3;
    let id = queue_project(&mut sim, &data, "restore_crew_quarters", None).unwrap();
    for _ in 0..30 {
        advance_projects(&mut sim, &data);
    }
    assert_eq!(sim.projects.find(id).unwrap().delivered_stages, 5);
    let morale = sim.population.morale;
    assert!((morale - 0.33).abs() < 1e-5);
    pause_project(&mut sim, &data, id).unwrap();
    let food = queue_project(
        &mut sim,
        &data,
        "optimise_hydroponics",
        Some("agriculture".into()),
    )
    .unwrap();
    assert_eq!(
        sim.projects.find(food).unwrap().status,
        ProjectStatus::Running
    );
    let refund = cancel_project(&mut sim, &data, id).unwrap();
    assert!((refund.minerals - 20.0).abs() < 1e-5);
    assert_eq!(sim.population.morale, morale);
    assert!(cancel_project(&mut sim, &data, id).is_err());
}

#[test]
fn partial_stage_consumes_materials_and_completion_is_exactly_once() {
    let (data, mut sim) = campaign();
    sim.ship.hull_integrity = 0.5;
    let id = queue_project(&mut sim, &data, "restore_hull", None).unwrap();
    advance_projects(&mut sim, &data);
    let job = sim.projects.find(id).unwrap();
    assert_eq!(job.delivered_stages, 0);
    assert!(job.committed_cost.minerals > 0.0);
    assert!(refund_preview(job, &data).minerals < 64.0);
    for _ in 1..96 {
        advance_projects(&mut sim, &data);
    }
    let hull = sim.ship.hull_integrity;
    for _ in 0..10 {
        advance_projects(&mut sim, &data);
    }
    assert_eq!(sim.ship.hull_integrity, hull);
    assert!(sim.projects.find(id).unwrap().remaining_escrow.minerals < 1e-8);
}

#[test]
fn fractional_restoration_is_not_charged_again_through_rounding() {
    let (data, mut sim) = campaign();
    let before = sim.ship.spare_parts as f64;
    for _ in 0..100 {
        settle(
            &mut sim,
            ProjectAmounts {
                spare_parts: -0.01,
                ..Default::default()
            },
        )
        .unwrap();
    }
    assert!(
        (sim.ship.spare_parts as f64 + sim.projects.settlement_balance.spare_parts
            - (before - 1.0))
            .abs()
            < 1e-7
    );
    for _ in 0..100 {
        refund_to_stores(
            &mut sim,
            ProjectAmounts {
                spare_parts: 0.01,
                ..Default::default()
            },
        );
    }
    assert!(
        (sim.ship.spare_parts as f64 + sim.projects.settlement_balance.spare_parts - before).abs()
            < 1e-7
    );
    let _ = data;
}

#[test]
fn pause_grace_and_lifetime_cap_survive_repeated_resume() {
    let (data, mut sim) = campaign();
    sim.ship.hull_integrity = 0.5;
    let id = queue_project(&mut sim, &data, "restore_hull", None).unwrap();
    pause_project(&mut sim, &data, id).unwrap();
    for _ in 0..12 {
        age_paused_projects(&mut sim, &data);
    }
    assert!(!sim.projects.find(id).unwrap().restoration_debt.nonzero());
    for _ in 0..40 {
        age_paused_projects(&mut sim, &data);
        resume_project(&mut sim, &data, id).unwrap();
        pause_project(&mut sim, &data, id).unwrap();
    }
    let job = sim.projects.find(id).unwrap();
    assert!((job.lifetime_deterioration.minerals - 20.0).abs() < 1e-5);
    assert_eq!(job.stage_pause_months, vec![52, 52]);
}

#[test]
fn full_waiting_list_refuses_manual_pause_and_reorders_across_history() {
    let (data, mut sim) = campaign();
    sim.ship.hull_integrity = 0.5;
    let id = queue_project(&mut sim, &data, "restore_hull", None).unwrap();
    for seq in 100..104 {
        sim.projects.jobs.push(ProjectInstance::queued(
            seq,
            "restore_crew_quarters",
            None,
            0,
        ));
    }
    assert!(pause_project(&mut sim, &data, id).is_err());
    sim.projects.jobs[2].status = ProjectStatus::Completed;
    sim.projects.jobs[3].status = ProjectStatus::Paused;
    move_project(&mut sim, 102, -1).unwrap();
    assert_eq!(sim.projects.jobs[1].sequence_id, 102);
    assert_eq!(sim.projects.jobs[2].sequence_id, 101);
}

#[test]
fn food_material_deteriorates_and_unaffordable_resume_is_atomic() {
    let (data, mut sim) = campaign();
    let id = queue_project(
        &mut sim,
        &data,
        "establish_seed_programme",
        Some("agriculture".into()),
    )
    .unwrap();
    pause_project(&mut sim, &data, id).unwrap();
    for _ in 0..18 {
        age_paused_projects(&mut sim, &data);
    }
    assert!(sim.projects.find(id).unwrap().restoration_debt.food > 0.0);
    sim.resources.food = 0;
    let before = sim.projects.find(id).unwrap().remaining_escrow;
    assert!(resume_project(&mut sim, &data, id).is_err());
    assert_eq!(sim.projects.find(id).unwrap().remaining_escrow, before);
}
