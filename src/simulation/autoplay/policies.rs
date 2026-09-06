//! Comparable policies use the same mission driver and authoritative services.
use super::*;
use crate::state::sim::ProjectStatus;
use std::collections::HashSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Policy {
    Legacy,
    NoProjects,
    Reactive,
    Prepared,
}

#[derive(Default)]
pub struct Metrics {
    pub months: u64,
    pub min_food: Option<i64>,
    pub min_parts: Option<i64>,
    pub critical_months: u64,
    pub occupied_slot_months: u64,
    pub blocked_ids: HashSet<u64>,
    pub queue_changes: u64,
    pub recoveries: u64,
    pub recovery_months: u64,
    pub max_recovery: u32,
    critical_since: Option<u32>,
    prior_queue: Vec<(u64, ProjectStatus)>,
}

impl Metrics {
    pub fn observe(&mut self, sim: &SimState, data: &GameData, advanced: bool) {
        self.min_food = Some(
            self.min_food
                .map_or(sim.resources.food, |v| v.min(sim.resources.food)),
        );
        self.min_parts = Some(
            self.min_parts
                .map_or(sim.ship.spare_parts, |v| v.min(sim.ship.spare_parts)),
        );
        let critical = sim.ship.hull_integrity <= data.config.survival.critical_warning_threshold
            || sim.ship.life_support <= data.config.survival.critical_warning_threshold
            || sim.resources.food == 0;
        if critical {
            self.critical_since.get_or_insert(sim.month_clock);
        } else if let Some(start) = self.critical_since.take() {
            let months = sim.month_clock.saturating_sub(start);
            self.recovery_months += months as u64;
            self.max_recovery = self.max_recovery.max(months);
            self.recoveries += 1;
        }
        if advanced {
            self.months += 1;
            self.critical_months += u64::from(critical);
            self.occupied_slot_months += sim.projects.active_count() as u64;
        }
        for job in &sim.projects.jobs {
            if job.is_waiting() && job.pause_reason.is_some() {
                self.blocked_ids.insert(job.sequence_id);
            }
        }
        let queue: Vec<_> = sim
            .projects
            .jobs
            .iter()
            .map(|j| (j.sequence_id, j.status))
            .collect();
        if queue != self.prior_queue {
            self.queue_changes += 1;
            self.prior_queue = queue;
        }
    }
}

pub fn act(sim: &mut SimState, data: &GameData, policy: Policy) {
    // All three policies can purchase emergency food; only ship-work differs.
    if sim.resources.food < data.config.low_food_threshold {
        let _ = market::buy(sim, TradeResource::Food, 1000);
    }
    if policy == Policy::NoProjects {
        survival::resume_after_warning(sim);
        return;
    }
    if sim.ship.life_support <= data.config.survival.critical_warning_threshold {
        let _ = survival::emergency_stabilise(sim, data);
    }
    survival::resume_after_warning(sim);
    let threshold = if policy == Policy::Prepared { 0.7 } else { 0.5 };
    if sim.ship.life_support < threshold {
        let _ = projects::queue_project(sim, data, "overhaul_life_support", None);
    }
    if sim.ship.hull_integrity < threshold {
        let _ = projects::queue_project(sim, data, "restore_hull", None);
    }
    for id in GameData::sorted_ids(&data.subsystems) {
        let Some(sub) = sim.subsystems.get(&id) else {
            continue;
        };
        let (condition, knowledge) = (sub.condition, sub.knowledge);
        let required = data.subsystems.get(&id).unwrap().repair_knowledge_required;
        let floor = if policy == Policy::Prepared {
            (required + 0.15).min(0.9)
        } else {
            required
        };
        if knowledge < floor {
            let _ =
                projects::queue_project(sim, data, "train_replacement_cohort", Some(id.clone()));
        }
        if condition < threshold {
            let _ = projects::queue_project(sim, data, "service_subsystem", Some(id));
        }
    }
    let suspended: Vec<_> = sim
        .projects
        .jobs
        .iter()
        .filter(|j| j.status == ProjectStatus::Paused)
        .map(|j| j.sequence_id)
        .collect();
    for id in suspended {
        let _ = projects::resume_project(sim, data, id);
    }
    if sim.issues.has_active("agriculture_blight") {
        let _ = projects::queue_project(
            sim,
            data,
            "sterilise_damaged_growing_systems",
            Some("agriculture".into()),
        );
    }
    if policy == Policy::Prepared && sim.projects.waiting_count() == 0 {
        let _ = projects::queue_project(
            sim,
            data,
            "establish_seed_programme",
            Some("agriculture".into()),
        );
        let _ = projects::queue_project(
            sim,
            data,
            "optimise_hydroponics",
            Some("agriculture".into()),
        );
    }
}
