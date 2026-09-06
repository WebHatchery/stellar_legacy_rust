//! Custodian Agenda commands, eligibility, staged delivery, and escrow.

use crate::data::projects::{ProjectDefinition, ProjectKind, ProjectTarget};
use crate::data::{GameData, PopulationDelta, ResourceDelta};
use crate::simulation::issues;
use crate::state::sim::{ProjectAmounts, ProjectInstance, ProjectStatus, SimState};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectEligibility {
    pub eligible: bool,
    pub reason: String,
}

impl ProjectEligibility {
    fn ready() -> Self {
        Self {
            eligible: true,
            reason: "Ready to queue.".to_owned(),
        }
    }

    fn blocked(reason: impl Into<String>) -> Self {
        Self {
            eligible: false,
            reason: reason.into(),
        }
    }
}

/// Read-only project eligibility shared by Agenda, readiness advice, and the
/// command service. It never considers queued jobs paid: only start rechecks
/// affordability and charges the authored budget.
pub fn eligibility(
    sim: &SimState,
    data: &GameData,
    definition: &ProjectDefinition,
    target_id: Option<&str>,
) -> ProjectEligibility {
    if sim.terminal.is_some() {
        return ProjectEligibility::blocked("The campaign has ended.");
    }
    if sim.has_pending_decision() {
        return ProjectEligibility::blocked("The council must finish its decision first.");
    }
    if let Some(capability) = &definition.requires_capability {
        if !sim.projects.has_capability(capability) {
            return ProjectEligibility::blocked(format!("Requires capability: {capability}."));
        }
    }
    if let Some(issue_id) = &definition.requires_issue {
        if !issues::issue_allows_project(sim, issue_id) {
            return ProjectEligibility::blocked("No matching active aftermath issue.");
        }
    }
    let target = match definition.target {
        ProjectTarget::Subsystem => {
            let Some(id) = target_id else {
                return ProjectEligibility::blocked("Choose a subsystem target.");
            };
            let Some(state) = sim.subsystems.get(id) else {
                return ProjectEligibility::blocked("That subsystem is not fitted aboard.");
            };
            let knowledge_floor = if definition.kind == ProjectKind::ServiceSubsystem {
                definition.knowledge_required.max(
                    data.subsystems
                        .get(id)
                        .map(|subsystem| subsystem.repair_knowledge_required)
                        .unwrap_or(0.0),
                )
            } else {
                definition.knowledge_required
            };
            if knowledge_floor > 0.0 && state.knowledge + f32::EPSILON < knowledge_floor {
                return ProjectEligibility::blocked(format!(
                    "Needs {:.0}% knowledge; this discipline has {:.0}%.",
                    knowledge_floor * 100.0,
                    state.knowledge * 100.0
                ));
            }
            Some((state.condition, state.knowledge))
        }
        ProjectTarget::Agriculture => {
            let state = sim.subsystems.get("agriculture");
            let Some(state) = state else {
                return ProjectEligibility::blocked("Agriculture is not fitted aboard.");
            };
            if definition.knowledge_required > 0.0
                && state.knowledge + f32::EPSILON < definition.knowledge_required
            {
                return ProjectEligibility::blocked(format!(
                    "Agriculture knowledge must reach {:.0}%.",
                    definition.knowledge_required * 100.0
                ));
            }
            Some((state.condition, state.knowledge))
        }
        ProjectTarget::Social | ProjectTarget::None => None,
    };
    match definition.kind {
        ProjectKind::ServiceSubsystem => {
            let Some((condition, _)) = target else {
                return ProjectEligibility::blocked("Choose a subsystem to service.");
            };
            if definition
                .target_condition_below
                .is_some_and(|floor| condition >= floor)
            {
                return ProjectEligibility::blocked("No useful service is needed on that module.");
            }
        }
        ProjectKind::RestoreHull => {
            if definition
                .target_condition_below
                .is_some_and(|floor| sim.ship.hull_integrity >= floor)
            {
                return ProjectEligibility::blocked("Hull is already above the service threshold.");
            }
        }
        ProjectKind::OverhaulLifeSupport => {
            if definition
                .target_condition_below
                .is_some_and(|floor| sim.ship.life_support >= floor)
            {
                return ProjectEligibility::blocked(
                    "Life support is already above the overhaul threshold.",
                );
            }
        }
        ProjectKind::TrainReplacementCohort => {
            let Some((_, knowledge)) = target else {
                return ProjectEligibility::blocked("Choose a discipline to teach.");
            };
            if knowledge >= 1.0 - f32::EPSILON {
                return ProjectEligibility::blocked("That discipline is already fully learned.");
            }
        }
        ProjectKind::OptimiseHydroponics => {
            if sim.projects.hydroponics_bonus
                >= data.config.projects.maximum_hydroponics_bonus - f32::EPSILON
            {
                return ProjectEligibility::blocked("Hydroponics is at its capped optimisation.");
            }
            if target.is_some_and(|(condition, _)| condition <= 0.2) {
                return ProjectEligibility::blocked(
                    "Agriculture must be working before optimisation.",
                );
            }
        }
        ProjectKind::RestoreCrewQuarters => {
            if sim.population.morale >= 0.95 && sim.population.unity >= 0.95 {
                return ProjectEligibility::blocked("The living decks need no recovery work.");
            }
        }
        ProjectKind::SteriliseGrowingSystems | ProjectKind::EstablishSeedProgramme => {}
    }
    ProjectEligibility::ready()
}

fn duplicate_job(sim: &SimState, project_id: &str, target_id: Option<&str>) -> bool {
    sim.projects.jobs.iter().any(|job| {
        job.project_id == project_id
            && job.target_id.as_deref() == target_id
            && matches!(
                job.status,
                ProjectStatus::Queued | ProjectStatus::Running | ProjectStatus::Paused
            )
    })
}

pub fn queue_project(
    sim: &mut SimState,
    data: &GameData,
    project_id: &str,
    target_id: Option<String>,
) -> Result<u64, String> {
    let Some(definition) = data.projects.get(project_id) else {
        return Err("Unknown Agenda project.".to_owned());
    };
    if sim.projects.waiting_count() >= data.config.projects.waiting_cap as usize {
        return Err(format!(
            "The waiting list is full ({}) — reorder, resume, or cancel a job first.",
            data.config.projects.waiting_cap
        ));
    }
    if duplicate_job(sim, project_id, target_id.as_deref()) {
        return Err("That exact project is already queued or underway.".to_owned());
    }
    let check = eligibility(sim, data, definition, target_id.as_deref());
    if !check.eligible {
        return Err(check.reason);
    }
    let sequence_id = sim.projects.next_sequence_id.saturating_add(1).max(1);
    sim.projects.next_sequence_id = sequence_id;
    sim.projects.jobs.push(ProjectInstance::queued(
        sequence_id,
        project_id,
        target_id,
        sim.month_clock,
    ));
    sim.push_log(format!("Agenda queued: {}.", definition.name));
    if sim.contract.is_some() {
        start_waiting_jobs(sim, data);
    }
    Ok(sequence_id)
}

pub fn start_waiting_jobs(sim: &mut SimState, data: &GameData) {
    if sim.contract.is_none() || sim.terminal.is_some() || sim.has_pending_decision() {
        return;
    }
    loop {
        if sim.projects.active_count() >= data.config.projects.concurrent_slots as usize {
            break;
        }
        let candidates: Vec<u64> = sim
            .projects
            .jobs
            .iter()
            .filter(|job| job.status == ProjectStatus::Queued)
            .map(|job| job.sequence_id)
            .collect();
        let mut started = false;
        for sequence_id in candidates {
            if try_start(sim, data, sequence_id) {
                started = true;
                break;
            }
        }
        if !started {
            break;
        }
    }
}

fn try_start(sim: &mut SimState, data: &GameData, sequence_id: u64) -> bool {
    let Some(index) = sim
        .projects
        .jobs
        .iter()
        .position(|job| job.sequence_id == sequence_id && job.status == ProjectStatus::Queued)
    else {
        return false;
    };
    let (project_id, target_id) = {
        let job = &sim.projects.jobs[index];
        (job.project_id.clone(), job.target_id.clone())
    };
    let Some(definition) = data.projects.get(&project_id) else {
        return false;
    };
    let check = eligibility(sim, data, definition, target_id.as_deref());
    if !check.eligible {
        sim.projects.jobs[index].pause_reason = Some(check.reason);
        return false;
    }
    let cost = definition.cost.clone();
    let resource_cost = cost.clone().resource_delta();
    if !sim.resources.can_afford(&resource_cost) || sim.ship.spare_parts < cost.spare_parts {
        sim.projects.jobs[index].pause_reason = Some(format!(
            "Waiting for {}cr, {}min, {} food, and {} spare parts.",
            cost.credits, cost.minerals, cost.food, cost.spare_parts
        ));
        return false;
    }
    sim.resources.apply(&resource_cost);
    sim.ship.spare_parts -= cost.spare_parts;
    let amounts = ProjectAmounts::from_cost(cost);
    let job = &mut sim.projects.jobs[index];
    job.status = ProjectStatus::Running;
    job.started_month = Some(sim.month_clock);
    job.original_cost = amounts;
    job.remaining_escrow = amounts;
    job.pause_reason = None;
    sim.push_log(format!(
        "Agenda started: {}{}.",
        definition.name,
        target_id
            .as_deref()
            .map(|id| format!(" on {id}"))
            .unwrap_or_default()
    ));
    true
}

pub fn pause_project(sim: &mut SimState, sequence_id: u64) -> Result<(), String> {
    let Some(job) = sim.projects.find_mut(sequence_id) else {
        return Err("Unknown Agenda job.".to_owned());
    };
    if job.status != ProjectStatus::Running {
        return Err("Only a running project can be paused.".to_owned());
    }
    job.status = ProjectStatus::Paused;
    job.pause_reason = Some("Paused by the Custodian.".to_owned());
    sim.push_log(format!("Agenda paused job {}.", sequence_id));
    Ok(())
}

pub fn resume_project(sim: &mut SimState, data: &GameData, sequence_id: u64) -> Result<(), String> {
    if sim.contract.is_none() {
        return Err("Projects resume on the next voyage, not in port.".to_owned());
    }
    if sim.projects.active_count() >= data.config.projects.concurrent_slots as usize {
        return Err("Both Agenda slots are occupied.".to_owned());
    }
    let Some(index) = sim
        .projects
        .jobs
        .iter()
        .position(|job| job.sequence_id == sequence_id)
    else {
        return Err("Unknown Agenda job.".to_owned());
    };
    if sim.projects.jobs[index].status != ProjectStatus::Paused {
        return Err("Only a paused project can be resumed.".to_owned());
    }
    let (project_id, target_id, debt) = {
        let job = &sim.projects.jobs[index];
        (
            job.project_id.clone(),
            job.target_id.clone(),
            job.restoration_debt,
        )
    };
    let Some(definition) = data.projects.get(&project_id) else {
        return Err("The saved project definition is no longer available.".to_owned());
    };
    let check = eligibility(sim, data, definition, target_id.as_deref());
    if !check.eligible && !check.reason.starts_with("No useful service") {
        return Err(check.reason);
    }
    let cost = ProjectAmounts {
        credits: 0.0,
        energy: debt.energy,
        minerals: debt.minerals,
        food: debt.food,
        influence: 0.0,
        spare_parts: debt.spare_parts,
    };
    let resources = ResourceDelta {
        energy: -(cost.energy.ceil() as i64),
        minerals: -(cost.minerals.ceil() as i64),
        food: -(cost.food.ceil() as i64),
        ..Default::default()
    };
    if !sim.resources.can_afford(&resources)
        || sim.ship.spare_parts < cost.spare_parts.ceil() as i64
    {
        return Err(format!(
            "Resuming needs {} minerals and {} spare parts to restore paused work.",
            cost.minerals.ceil() as i64,
            cost.spare_parts.ceil() as i64
        ));
    }
    sim.resources.apply(&resources);
    sim.ship.spare_parts -= cost.spare_parts.ceil() as i64;
    let job = &mut sim.projects.jobs[index];
    job.remaining_escrow.spare_parts += debt.spare_parts;
    job.remaining_escrow.minerals += debt.minerals;
    job.remaining_escrow.energy += debt.energy;
    job.remaining_escrow.food += debt.food;
    job.status = ProjectStatus::Running;
    job.restoration_debt = ProjectAmounts::default();
    job.pause_reason = None;
    sim.push_log(format!(
        "Agenda resumed job {sequence_id}; restoration debt paid."
    ));
    Ok(())
}

pub fn move_project(sim: &mut SimState, sequence_id: u64, direction: i32) -> Result<(), String> {
    let Some(index) = sim
        .projects
        .jobs
        .iter()
        .position(|job| job.sequence_id == sequence_id && job.is_waiting())
    else {
        return Err("Only waiting Agenda jobs can be reordered.".to_owned());
    };
    let next = if direction < 0 {
        index.checked_sub(1)
    } else {
        Some(index + 1)
    };
    let Some(next) = next.filter(|index| *index < sim.projects.jobs.len()) else {
        return Ok(());
    };
    if !sim.projects.jobs[next].is_waiting() {
        return Ok(());
    }
    sim.projects.jobs.swap(index, next);
    Ok(())
}

pub fn cancel_project(
    sim: &mut SimState,
    data: &GameData,
    sequence_id: u64,
) -> Result<ProjectAmounts, String> {
    let Some(index) = sim
        .projects
        .jobs
        .iter()
        .position(|job| job.sequence_id == sequence_id)
    else {
        return Err("Unknown Agenda job.".to_owned());
    };
    let job = &sim.projects.jobs[index];
    if matches!(
        job.status,
        ProjectStatus::Completed | ProjectStatus::Stopped | ProjectStatus::Cancelled
    ) {
        return Err("That project has already ended.".to_owned());
    }
    let refund = if job.status == ProjectStatus::Queued {
        ProjectAmounts::default()
    } else {
        multiply_refund(
            job.remaining_escrow,
            data.config.projects.cancellation_refund_fraction,
        )
    };
    refund_to_stores(sim, refund);
    let job = &mut sim.projects.jobs[index];
    job.status = ProjectStatus::Cancelled;
    job.stop_reason = Some("Cancelled by the Custodian.".to_owned());
    sim.push_log(format!(
        "Agenda cancelled job {sequence_id}; unused stores were refunded."
    ));
    Ok(refund)
}

fn multiply_refund(amounts: ProjectAmounts, fraction: f32) -> ProjectAmounts {
    let f = fraction.clamp(0.0, 1.0) as f64;
    ProjectAmounts {
        credits: amounts.credits * f,
        energy: amounts.energy * f,
        minerals: amounts.minerals * f,
        food: amounts.food * f,
        influence: 0.0,
        spare_parts: amounts.spare_parts * f,
    }
}

fn refund_to_stores(sim: &mut SimState, refund: ProjectAmounts) {
    sim.resources.apply(&ResourceDelta {
        credits: refund.credits.floor() as i64,
        energy: refund.energy.floor() as i64,
        minerals: refund.minerals.floor() as i64,
        food: refund.food.floor() as i64,
        influence: 0,
    });
    sim.ship.spare_parts += refund.spare_parts.floor() as i64;
}

/// Age deliberately paused work by one simulation month. Called only for an
/// active voyage month, so global pause, port, and closed-app time never accrue.
pub fn age_paused_projects(sim: &mut SimState, data: &GameData) {
    let ids: Vec<u64> = sim
        .projects
        .jobs
        .iter()
        .filter(|job| job.status == ProjectStatus::Paused)
        .map(|job| job.sequence_id)
        .collect();
    for sequence_id in ids {
        let Some(index) = sim
            .projects
            .jobs
            .iter()
            .position(|job| job.sequence_id == sequence_id)
        else {
            continue;
        };
        let (project_id, delivered, paused) = {
            let job = &sim.projects.jobs[index];
            (
                job.project_id.clone(),
                job.delivered_stages,
                job.paused_months,
            )
        };
        let Some(definition) = data.projects.get(&project_id) else {
            continue;
        };
        let job = &mut sim.projects.jobs[index];
        job.paused_months = paused.saturating_add(1);
        let stage_count = definition.stage_count() as usize;
        if job.stage_pause_months.len() < stage_count {
            job.stage_pause_months.resize(stage_count, 0);
        }
        if job.stage_deterioration.len() < stage_count {
            job.stage_deterioration
                .resize(stage_count, ProjectAmounts::default());
        }
        for stage_index in delivered as usize..stage_count {
            job.stage_pause_months[stage_index] =
                job.stage_pause_months[stage_index].saturating_add(1);
            if job.stage_pause_months[stage_index] <= data.config.projects.pause_grace_months {
                continue;
            }
            let stage_fraction = 1.0 / stage_count as f64;
            let parts_budget = job.original_cost.spare_parts * stage_fraction;
            let minerals_budget = job.original_cost.minerals * stage_fraction;
            let stage = &mut job.stage_deterioration[stage_index];
            let parts_room = (parts_budget * data.config.projects.pause_debt_cap_fraction as f64
                - stage.spare_parts)
                .max(0.0);
            let minerals_room = (minerals_budget
                * data.config.projects.pause_debt_cap_fraction as f64
                - stage.minerals)
                .max(0.0);
            let parts = (parts_budget * data.config.projects.pause_debt_fraction_per_month as f64)
                .min(parts_room);
            let minerals = (minerals_budget
                * data.config.projects.pause_debt_fraction_per_month as f64)
                .min(minerals_room);
            stage.spare_parts += parts;
            stage.minerals += minerals;
            job.restoration_debt.spare_parts += parts;
            job.restoration_debt.minerals += minerals;
            job.lifetime_deterioration.spare_parts += parts;
            job.lifetime_deterioration.minerals += minerals;
            job.remaining_escrow.spare_parts = (job.remaining_escrow.spare_parts - parts).max(0.0);
            job.remaining_escrow.minerals = (job.remaining_escrow.minerals - minerals).max(0.0);
        }
    }
}

fn running_block_reason(
    sim: &SimState,
    data: &GameData,
    job: &ProjectInstance,
    definition: &ProjectDefinition,
) -> Option<String> {
    let target = job.target_id.as_deref();
    match definition.kind {
        ProjectKind::ServiceSubsystem | ProjectKind::EstablishSeedProgramme => {
            let id = target.unwrap_or("agriculture");
            let state = sim.subsystems.get(id)?;
            let knowledge_floor = if definition.kind == ProjectKind::ServiceSubsystem {
                definition.knowledge_required.max(
                    data.subsystems
                        .get(id)
                        .map(|subsystem| subsystem.repair_knowledge_required)
                        .unwrap_or(0.0),
                )
            } else {
                definition.knowledge_required
            };
            if knowledge_floor > 0.0 && state.knowledge + f32::EPSILON < knowledge_floor {
                Some(format!(
                    "Required expertise fell below {:.0}%; project suspended.",
                    knowledge_floor * 100.0
                ))
            } else if definition.kind == ProjectKind::ServiceSubsystem
                && state.condition >= 1.0 - f32::EPSILON
            {
                Some(
                    "The target was already fully repaired; no duplicate service was delivered."
                        .to_owned(),
                )
            } else {
                None
            }
        }
        ProjectKind::TrainReplacementCohort => target
            .and_then(|id| sim.subsystems.get(id))
            .and_then(|state| {
                (state.knowledge >= 1.0 - f32::EPSILON).then_some(
                    "The discipline reached its knowledge cap before completion.".to_owned(),
                )
            }),
        ProjectKind::RestoreHull => (sim.ship.hull_integrity >= 1.0 - f32::EPSILON)
            .then_some("The hull reached full integrity before completion.".to_owned()),
        ProjectKind::OverhaulLifeSupport => (sim.ship.life_support >= 1.0 - f32::EPSILON)
            .then_some("Life support reached full integrity before completion.".to_owned()),
        _ => None,
    }
}

/// Advance all running projects by one month, delivering each newly reached
/// stage in stable sequence order. A newly started job intentionally waits for
/// the next call before earning its first month.
pub fn advance_projects(sim: &mut SimState, data: &GameData) {
    if sim.contract.is_none() || sim.terminal.is_some() || sim.has_pending_decision() {
        return;
    }
    start_waiting_jobs(sim, data);
    age_paused_projects(sim, data);
    let ids: Vec<u64> = sim
        .projects
        .jobs
        .iter()
        .filter(|job| job.status == ProjectStatus::Running)
        .map(|job| job.sequence_id)
        .collect();
    for sequence_id in ids {
        let Some(index) = sim
            .projects
            .jobs
            .iter()
            .position(|job| job.sequence_id == sequence_id)
        else {
            continue;
        };
        let (project_id, target_id) = {
            let job = &sim.projects.jobs[index];
            (job.project_id.clone(), job.target_id.clone())
        };
        let Some(definition) = data.projects.get(&project_id) else {
            continue;
        };
        if let Some(reason) = running_block_reason(sim, data, &sim.projects.jobs[index], definition)
        {
            let refund = multiply_refund(
                sim.projects.jobs[index].remaining_escrow,
                data.config.projects.cancellation_refund_fraction,
            );
            refund_to_stores(sim, refund);
            let job = &mut sim.projects.jobs[index];
            job.status = ProjectStatus::Stopped;
            job.stop_reason = Some(reason.clone());
            sim.push_log(format!("Agenda job {sequence_id} stopped: {reason}"));
            continue;
        }
        let elapsed = sim.projects.jobs[index].elapsed_months.saturating_add(1);
        let stage_count = definition.stage_count();
        let duration = definition.duration_months.max(1);
        let due = ((elapsed as u64 * stage_count as u64) / duration as u64) as u32;
        let new_stages = due.min(stage_count);
        sim.projects.jobs[index].elapsed_months = elapsed;
        while sim.projects.jobs[index].delivered_stages < new_stages {
            let stage = sim.projects.jobs[index].delivered_stages + 1;
            deliver_stage(sim, data, index, definition, stage, target_id.as_deref());
        }
        if elapsed >= duration {
            let job = &mut sim.projects.jobs[index];
            job.status = ProjectStatus::Completed;
            job.elapsed_months = duration;
            sim.push_log(format!("Agenda completed: {}.", definition.name));
            if let Some(issue_id) = &definition.requires_issue {
                issues::resolve_issue(sim, issue_id, &format!("{} completed", definition.name));
            }
        }
    }
    start_waiting_jobs(sim, data);
}

fn deliver_stage(
    sim: &mut SimState,
    data: &GameData,
    index: usize,
    definition: &ProjectDefinition,
    stage: u32,
    target_id: Option<&str>,
) {
    let stages = definition.stage_count() as f64;
    let slice = ProjectAmounts {
        credits: definition.cost.credits as f64 / stages,
        energy: definition.cost.energy as f64 / stages,
        minerals: definition.cost.minerals as f64 / stages,
        food: definition.cost.food as f64 / stages,
        influence: definition.cost.influence as f64 / stages,
        spare_parts: definition.cost.spare_parts as f64 / stages,
    };
    let job = &mut sim.projects.jobs[index];
    job.delivered_stages = stage;
    job.delivered_months.push(sim.month_clock);
    job.committed_cost.credits += slice.credits;
    job.committed_cost.energy += slice.energy;
    job.committed_cost.minerals += slice.minerals;
    job.committed_cost.food += slice.food;
    job.committed_cost.influence += slice.influence;
    job.committed_cost.spare_parts += slice.spare_parts;
    job.remaining_escrow.credits = (job.remaining_escrow.credits - slice.credits).max(0.0);
    job.remaining_escrow.energy = (job.remaining_escrow.energy - slice.energy).max(0.0);
    job.remaining_escrow.minerals = (job.remaining_escrow.minerals - slice.minerals).max(0.0);
    job.remaining_escrow.food = (job.remaining_escrow.food - slice.food).max(0.0);
    job.remaining_escrow.influence = (job.remaining_escrow.influence - slice.influence).max(0.0);
    job.remaining_escrow.spare_parts =
        (job.remaining_escrow.spare_parts - slice.spare_parts).max(0.0);

    if definition.divisible || stage == definition.stage_count() {
        apply_effect(
            sim,
            data,
            definition,
            target_id,
            stage == definition.stage_count(),
        );
    }
}

fn apply_effect(
    sim: &mut SimState,
    data: &GameData,
    definition: &ProjectDefinition,
    target_id: Option<&str>,
    final_stage: bool,
) {
    let stages = if definition.divisible {
        definition.stage_count() as f32
    } else {
        1.0
    };
    let effect = &definition.effect;
    match definition.kind {
        ProjectKind::ServiceSubsystem | ProjectKind::SteriliseGrowingSystems => {
            if let Some(id) = target_id.or(Some("agriculture")) {
                if let Some(state) = sim.subsystems.get_mut(id) {
                    state.condition =
                        (state.condition + effect.condition_gain / stages).clamp(0.0, 1.0);
                }
                let maintenance_id = format!("maintenance:{id}");
                if sim.subsystems.get(id).is_some_and(|state| {
                    state.condition > data.config.projects.maintenance_condition_threshold
                }) {
                    issues::resolve_issue(sim, &maintenance_id, &definition.name);
                }
            }
        }
        ProjectKind::RestoreHull => {
            sim.ship.hull_integrity =
                (sim.ship.hull_integrity + effect.hull_gain / stages).clamp(0.0, 1.0);
        }
        ProjectKind::OverhaulLifeSupport => {
            sim.ship.life_support =
                (sim.ship.life_support + effect.life_support_gain / stages).clamp(0.0, 1.0);
        }
        ProjectKind::TrainReplacementCohort => {
            if let Some(id) = target_id {
                if let Some(state) = sim.subsystems.get_mut(id) {
                    state.knowledge =
                        (state.knowledge + effect.knowledge_gain / stages).clamp(0.0, 1.0);
                }
            }
        }
        ProjectKind::OptimiseHydroponics => {
            sim.projects.hydroponics_bonus = (sim.projects.hydroponics_bonus
                + effect.food_production_bonus / stages)
                .min(data.config.projects.maximum_hydroponics_bonus);
        }
        ProjectKind::RestoreCrewQuarters => {
            sim.population.apply(&PopulationDelta {
                morale: effect.morale_recovery / stages,
                unity: effect.unity_recovery / stages,
                ..Default::default()
            });
        }
        ProjectKind::EstablishSeedProgramme => {
            if final_stage {
                if let Some(capability) = &effect.capability {
                    if !sim.projects.has_capability(capability) {
                        sim.projects.capabilities.push(capability.clone());
                        sim.push_log(format!("Capability completed: {capability}."));
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests;
