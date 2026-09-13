//! Reject unsupported investments before accepting or replacing a save.
use crate::data::GameData;
use crate::state::sim::{ProjectAmounts, ProjectStatus, SimState};
use std::collections::HashSet;

fn amounts(value: ProjectAmounts) -> Result<(), String> {
    if value.values().iter().any(|v| !v.is_finite() || *v < -1e-7) {
        return Err("Invalid Agenda resource ledger.".into());
    }
    Ok(())
}

pub(super) fn validate(sim: &SimState, data: &GameData) -> Result<(), String> {
    amounts(sim.projects.settlement_balance)?;
    if sim
        .projects
        .settlement_balance
        .values()
        .iter()
        .any(|v| *v >= 1.0 + 1e-7)
    {
        return Err("Invalid fractional settlement balance.".into());
    }
    if !sim.projects.hydroponics_bonus.is_finite()
        || sim.projects.hydroponics_bonus < 0.0
        || sim.projects.hydroponics_bonus > data.config.projects.maximum_hydroponics_bonus + 1e-5
    {
        return Err("Invalid hydroponics capability strength.".into());
    }
    let mut ids = HashSet::new();
    for job in &sim.projects.jobs {
        validate_job(job, sim, data, &mut ids)?;
    }
    validate_capabilities(sim, data)?;
    validate_issues(sim, data)
}

fn validate_job(
    job: &crate::state::sim::ProjectInstance,
    sim: &SimState,
    data: &GameData,
    ids: &mut HashSet<u64>,
) -> Result<(), String> {
    if job.sequence_id == 0
        || !ids.insert(job.sequence_id)
        || job.sequence_id > sim.projects.next_sequence_id
    {
        return Err("Duplicate or invalid Agenda sequence ID.".into());
    }
    let def = crate::simulation::projects::definition_for(job, data)
        .ok_or_else(|| format!("Unknown saved project: {}", job.project_id))?;
    validate_job_target(job, &def, data)?;
    validate_job_progress(job, &def, sim.month_clock)?;
    validate_job_accounting(job, &def)
}

fn validate_job_target(
    job: &crate::state::sim::ProjectInstance,
    def: &crate::data::projects::ProjectDefinition,
    data: &GameData,
) -> Result<(), String> {
    if let Some(id) = &job.target_id {
        if data.subsystems.get(id).is_none() {
            return Err(format!("Unknown project target: {id}"));
        }
    }
    use crate::data::projects::ProjectTarget;
    if (def.target == ProjectTarget::Subsystem && job.target_id.is_none())
        || (def.target == ProjectTarget::Agriculture
            && job.target_id.as_deref() != Some("agriculture"))
    {
        return Err("Missing or mismatched project target.".into());
    }
    if job.legacy_single_delivery && job.project_id != "restore_crew_quarters" {
        return Err("Invalid legacy delivery contract.".into());
    }
    Ok(())
}

fn validate_job_progress(
    job: &crate::state::sim::ProjectInstance,
    def: &crate::data::projects::ProjectDefinition,
    month_clock: u32,
) -> Result<(), String> {
    if job.elapsed_months > def.duration_months
        || job.delivered_stages > def.stage_count()
        || job.delivered_months.len() != job.delivered_stages as usize
        || job.delivered_months.windows(2).any(|v| v[0] > v[1])
        || job.delivered_months.iter().any(|v| *v > month_clock)
        || job.stage_pause_months.len() > def.stage_count() as usize
        || job.stage_deterioration.len() > def.stage_count() as usize
    {
        return Err("Invalid project progress or delivery history.".into());
    }
    Ok(())
}

fn validate_job_accounting(
    job: &crate::state::sim::ProjectInstance,
    def: &crate::data::projects::ProjectDefinition,
) -> Result<(), String> {
    for value in [
        job.original_cost,
        job.remaining_escrow,
        job.committed_cost,
        job.restoration_debt,
        job.lifetime_deterioration,
    ] {
        amounts(value)?;
    }
    for value in &job.stage_deterioration {
        amounts(*value)?;
    }
    let original = job.original_cost.values();
    let escrow = job.remaining_escrow.values();
    let committed = job.committed_cost.values();
    let authored = ProjectAmounts::from_cost(def.cost.clone()).values();
    for i in 0..6 {
        if escrow[i] + committed[i] > original[i] + 1e-5
            || (job.status != ProjectStatus::Queued && (original[i] - authored[i]).abs() > 1e-5)
        {
            return Err("Project escrow exceeds its original investment.".into());
        }
    }
    if (job.status == ProjectStatus::Queued
        && (job.original_cost.nonzero() || job.elapsed_months != 0))
        || (job.status == ProjectStatus::Completed && job.delivered_stages != def.stage_count())
    {
        return Err("Project state contradicts its accounting.".into());
    }
    Ok(())
}

fn validate_capabilities(sim: &SimState, data: &GameData) -> Result<(), String> {
    let mut capabilities = HashSet::new();
    for id in &sim.projects.capabilities {
        if !capabilities.insert(id)
            || !data
                .projects
                .ids()
                .any(|key| data.projects.get(key).unwrap().effect.capability.as_ref() == Some(id))
        {
            return Err(format!("Unknown or duplicate saved capability: {id}"));
        }
    }
    Ok(())
}

fn validate_issues(sim: &SimState, data: &GameData) -> Result<(), String> {
    let mut active = HashSet::new();
    for issue in sim.issues.active.iter().chain(&sim.issues.resolved) {
        if !issue.food_production_penalty.is_finite()
            || !(0.0..=0.5).contains(&issue.food_production_penalty)
        {
            return Err("Invalid ongoing issue penalty.".into());
        }
        if data.subsystems.get(&issue.target).is_none()
            || issue
                .recovery_project_ids
                .iter()
                .any(|id| data.projects.get(id).is_none())
        {
            return Err(format!("Unsupported issue references: {}", issue.id));
        }
    }
    for issue in &sim.issues.active {
        if !active.insert(&issue.id) {
            return Err("Duplicate active issue ID.".into());
        }
    }
    Ok(())
}
