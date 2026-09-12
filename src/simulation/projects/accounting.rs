//! Escrow and refund accounting for ship projects.

use super::*;
pub(super) fn multiply_refund(amounts: ProjectAmounts, fraction: f32) -> ProjectAmounts {
    let f = fraction.clamp(0.0, 1.0) as f64;
    ProjectAmounts {
        credits: amounts.credits * f,
        energy: amounts.energy * f,
        minerals: amounts.minerals * f,
        food: amounts.food * f,
        influence: amounts.influence * f,
        spare_parts: amounts.spare_parts * f,
    }
}

/// Settle exact amounts against whole-unit stores, retaining the change in the
/// saved ledger. Positive deltas refund; negative deltas pay restoration.
pub(super) fn settle(sim: &mut SimState, delta: ProjectAmounts) -> Result<(), String> {
    let (resources, parts, balance) = settlement_plan(sim, delta)?;
    sim.resources.apply(&resources);
    sim.ship.spare_parts += parts;
    sim.projects.settlement_balance = balance;
    Ok(())
}

pub(super) fn settlement_check(sim: &SimState, delta: ProjectAmounts) -> Result<(), String> {
    settlement_plan(sim, delta).map(|_| ())
}

fn settlement_plan(
    sim: &SimState,
    delta: ProjectAmounts,
) -> Result<(ResourceDelta, i64, ProjectAmounts), String> {
    let balance = sim.projects.settlement_balance.values();
    let values = delta.values();
    let mut whole = [0i64; 6];
    let mut change = [0.0; 6];
    for i in 0..6 {
        let exact = values[i] + balance[i];
        whole[i] = (exact + 1e-9).floor() as i64;
        change[i] = (exact - whole[i] as f64).max(0.0);
    }
    let resources = ResourceDelta {
        credits: whole[0],
        energy: whole[1],
        minerals: whole[2],
        food: whole[3],
        influence: whole[4],
    };
    if !sim.resources.can_afford(&resources) || sim.ship.spare_parts + whole[5] < 0 {
        return Err("Insufficient stores for the displayed restoration cost (fractional change is retained).".to_owned());
    }
    Ok((resources, whole[5], ProjectAmounts::from_values(change)))
}

pub(super) fn refund_to_stores(sim: &mut SimState, refund: ProjectAmounts) {
    settle(sim, refund).expect("a nonnegative refund is always affordable");
}

pub fn refund_preview(job: &ProjectInstance, data: &GameData) -> ProjectAmounts {
    if job.status == ProjectStatus::Queued {
        return ProjectAmounts::default();
    }
    let Some(definition) = data.projects.get(&job.project_id) else {
        return ProjectAmounts::default();
    };
    let mask = ProjectAmounts::from_cost(definition.refundable.clone()).values();
    let mut amounts = multiply_refund(
        job.remaining_escrow,
        data.config.projects.cancellation_refund_fraction,
    )
    .values();
    for i in 0..6 {
        if mask[i] == 0.0 {
            amounts[i] = 0.0;
        }
    }
    ProjectAmounts::from_values(amounts)
}

/// Commit labour/materials every month, including unfinished delivery stages.
pub(super) fn commit_month(job: &mut ProjectInstance, duration: u32) {
    let budget = job.original_cost.values();
    let mut committed = job.committed_cost.values();
    let mut escrow = job.remaining_escrow.values();
    let progress = job.elapsed_months.min(duration) as f64 / duration as f64;
    for i in 0..6 {
        let due = budget[i] * progress;
        escrow[i] = (escrow[i] - (due - committed[i]).max(0.0)).max(0.0);
        committed[i] = due;
    }
    job.committed_cost = ProjectAmounts::from_values(committed);
    job.remaining_escrow = ProjectAmounts::from_values(escrow);
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
        let (_project_id, delivered, paused) = {
            let job = &sim.projects.jobs[index];
            (
                job.project_id.clone(),
                job.delivered_stages,
                job.paused_months,
            )
        };
        let Some(definition) = definition_for(&sim.projects.jobs[index], data) else {
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
            let budget = job.original_cost.values();
            let mut stage = job.stage_deterioration[stage_index].values();
            let mut debt = job.restoration_debt.values();
            let mut lifetime = job.lifetime_deterioration.values();
            let mut escrow = job.remaining_escrow.values();
            for i in [1, 2, 3, 5] {
                let stage_budget = budget[i] / stage_count as f64;
                let room = (stage_budget * data.config.projects.pause_debt_cap_fraction as f64
                    - stage[i])
                    .max(0.0);
                let loss = (stage_budget
                    * data.config.projects.pause_debt_fraction_per_month as f64)
                    .min(room);
                stage[i] += loss;
                debt[i] += loss;
                lifetime[i] += loss;
                escrow[i] = (escrow[i] - loss).max(0.0);
            }
            job.stage_deterioration[stage_index] = ProjectAmounts::from_values(stage);
            job.restoration_debt = ProjectAmounts::from_values(debt);
            job.lifetime_deterioration = ProjectAmounts::from_values(lifetime);
            job.remaining_escrow = ProjectAmounts::from_values(escrow);
        }
    }
}
