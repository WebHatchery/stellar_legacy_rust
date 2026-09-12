//! Custodian Agenda commands, eligibility, staged delivery, and escrow.

use crate::data::projects::{ProjectDefinition, ProjectKind, ProjectTarget};
use crate::data::{GameData, ResourceDelta};
mod accounting;
mod effects;
use crate::simulation::issues;
use crate::state::sim::{ProjectAmounts, ProjectInstance, ProjectStatus, SimState};
pub use accounting::refund_preview;
use accounting::*;
use effects::deliver_stage;

/// Old quarters investments keep their original final-only delivery contract.
pub fn definition_for(job: &ProjectInstance, data: &GameData) -> Option<ProjectDefinition> {
    let mut definition = data.projects.get(&job.project_id)?.clone();
    if job.legacy_single_delivery {
        definition.stage_count = 1;
        definition.divisible = false;
    }
    Some(definition)
}

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
        ProjectKind::EstablishSeedProgramme => {
            if definition
                .effect
                .capability
                .as_deref()
                .is_some_and(|id| sim.projects.has_capability(id))
            {
                return ProjectEligibility::blocked("The seed programme is already available.");
            }
        }
        ProjectKind::SteriliseGrowingSystems => {}
    }
    ProjectEligibility::ready()
}

pub fn live_job<'a>(
    sim: &'a SimState,
    project_id: &str,
    target_id: Option<&str>,
) -> Option<&'a ProjectInstance> {
    sim.projects.jobs.iter().find(|job| {
        job.project_id == project_id
            && job.target_id.as_deref() == target_id
            && matches!(
                job.status,
                ProjectStatus::Queued | ProjectStatus::Running | ProjectStatus::Paused
            )
    })
}

/// Availability to add new work, as distinct from a running job's continued
/// eligibility. A starting budget may be unavailable: unpaid work may wait.
pub fn queue_check(
    sim: &SimState,
    data: &GameData,
    definition: &ProjectDefinition,
    target_id: Option<&str>,
) -> ProjectEligibility {
    if sim.projects.waiting_count() >= data.config.projects.waiting_cap as usize {
        return ProjectEligibility::blocked(format!(
            "The waiting list is full ({}). Resume or cancel waiting work first.",
            data.config.projects.waiting_cap,
        ));
    }
    if live_job(sim, &definition.id, target_id).is_some() {
        return ProjectEligibility::blocked("That exact project is already queued or underway.");
    }
    eligibility(sim, data, definition, target_id)
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
    let check = queue_check(sim, data, definition, target_id.as_deref());
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

/// The interface and command use the same current-state availability check.
pub fn pause_check(sim: &SimState, data: &GameData, sequence_id: u64) -> Result<(), String> {
    if sim.projects.waiting_count() >= data.config.projects.waiting_cap as usize {
        return Err(
            "Waiting list full: resume or cancel a waiting job before pausing this project."
                .to_owned(),
        );
    }
    let Some(job) = sim.projects.find(sequence_id) else {
        return Err("Unknown Agenda job.".to_owned());
    };
    if job.status != ProjectStatus::Running {
        return Err("Only a running project can be paused.".to_owned());
    }
    Ok(())
}

pub fn pause_project(sim: &mut SimState, data: &GameData, sequence_id: u64) -> Result<(), String> {
    pause_check(sim, data, sequence_id)?;
    let job = sim.projects.find_mut(sequence_id).expect("checked job");
    job.status = ProjectStatus::Paused;
    job.pause_reason = Some("Paused by the Custodian.".to_owned());
    sim.push_log(format!("Agenda paused job {}.", sequence_id));
    Ok(())
}

pub fn resume_check(sim: &SimState, data: &GameData, sequence_id: u64) -> Result<(), String> {
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
    if !check.eligible {
        return Err(check.reason);
    }
    settlement_check(sim, ProjectAmounts::from_values(debt.values().map(|v| -v)))
}

pub fn resume_project(sim: &mut SimState, data: &GameData, sequence_id: u64) -> Result<(), String> {
    resume_check(sim, data, sequence_id)?;
    let debt = sim
        .projects
        .find(sequence_id)
        .expect("checked job")
        .restoration_debt;
    settle(sim, ProjectAmounts::from_values(debt.values().map(|v| -v)))?;
    let job = sim.projects.find_mut(sequence_id).expect("checked job");
    let original = job.original_cost.values();
    let committed = job.committed_cost.values();
    let mut escrow = job.remaining_escrow.values();
    for i in 0..6 {
        escrow[i] = (escrow[i] + debt.values()[i]).min((original[i] - committed[i]).max(0.0));
    }
    job.remaining_escrow = ProjectAmounts::from_values(escrow);
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
        (0..index)
            .rev()
            .find(|&i| sim.projects.jobs[i].is_waiting())
    } else {
        (index + 1..sim.projects.jobs.len()).find(|&i| sim.projects.jobs[i].is_waiting())
    };
    let Some(next) = next else {
        return Ok(());
    };
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
    let refund = refund_preview(job, data);
    refund_to_stores(sim, refund);
    let job = &mut sim.projects.jobs[index];
    job.status = ProjectStatus::Cancelled;
    job.stop_reason = Some("Cancelled by the Custodian.".to_owned());
    sim.push_log(format!(
        "Agenda cancelled job {sequence_id}; unused stores were refunded."
    ));
    Ok(refund)
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
            let Some(state) = sim.subsystems.get(id) else {
                return Some("The target is no longer fitted aboard.".to_owned());
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

/// Capture running eligibility before the annual economy or another delivery
/// changes expertise. Stable sequence IDs make simultaneous outcomes reproducible.
pub fn capture_month(sim: &SimState, data: &GameData) -> Vec<(u64, Option<String>)> {
    let mut snapshot: Vec<_> = sim
        .projects
        .jobs
        .iter()
        .filter(|job| job.status == ProjectStatus::Running)
        .map(|job| {
            (
                job.sequence_id,
                definition_for(job, data)
                    .and_then(|def| running_block_reason(sim, data, job, &def)),
            )
        })
        .collect();
    snapshot.sort_by_key(|(id, _)| *id);
    snapshot
}

pub fn advance_captured_month(
    sim: &mut SimState,
    data: &GameData,
    snapshot: Vec<(u64, Option<String>)>,
) {
    if sim.contract.is_none() || sim.terminal.is_some() || sim.has_pending_decision() {
        return;
    }
    age_paused_projects(sim, data);
    for (sequence_id, block_reason) in snapshot {
        let Some(index) = sim
            .projects
            .jobs
            .iter()
            .position(|job| job.sequence_id == sequence_id)
        else {
            continue;
        };
        let target_id = sim.projects.jobs[index].target_id.clone();
        let Some(definition) = definition_for(&sim.projects.jobs[index], data) else {
            continue;
        };
        if let Some(reason) = block_reason {
            if reason.starts_with("Required expertise") {
                let job = &mut sim.projects.jobs[index];
                job.status = ProjectStatus::Paused;
                job.pause_reason = Some(reason.clone());
                sim.push_log(format!("Agenda job {sequence_id} suspended: {reason}"));
                continue;
            }
            let refund = refund_preview(&sim.projects.jobs[index], data);
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
        commit_month(&mut sim.projects.jobs[index], duration);
        while sim.projects.jobs[index].delivered_stages < new_stages {
            let stage = sim.projects.jobs[index].delivered_stages + 1;
            deliver_stage(sim, data, index, &definition, stage, target_id.as_deref());
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
#[allow(dead_code, unused_imports)]
mod tests {
    include!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/unit/simulation/projects/tests.rs"
    ));
}

#[allow(unused_imports)]
pub(crate) use tests::advance_projects;
