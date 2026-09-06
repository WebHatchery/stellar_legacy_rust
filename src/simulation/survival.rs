//! Authoritative vessel survival checks and the visible life-support rescue path.

use crate::data::{GameData, ResourceDelta};
use crate::state::sim::{SimState, TerminalOutcome, TerminalReason};

/// Evaluate terminal conditions without mutating state. The order keeps a
/// same-month vessel failure ahead of a later Homecoming reward.
pub fn terminal_reason(sim: &SimState, data: &GameData) -> Option<(TerminalReason, String)> {
    if sim.ship.hull_integrity <= 0.0 {
        return Some((
            TerminalReason::HullLoss,
            "Hull integrity reached zero after the authoritative mutation.".to_owned(),
        ));
    }
    if sim.population.count == 0 {
        return Some((
            TerminalReason::PopulationLoss,
            "No population remains aboard the vessel.".to_owned(),
        ));
    }
    if sim.survival.air_zero_months >= data.config.survival.air_grace_months.max(1) {
        return Some((
            TerminalReason::LifeSupportFailure,
            format!(
                "Life support remained at zero for {} simulation months.",
                sim.survival.air_zero_months
            ),
        ));
    }
    if sim.dynasty.extinct {
        return Some((
            TerminalReason::DynastyExtinction,
            "The founding dynasty has no eligible successor.".to_owned(),
        ));
    }
    None
}

/// Record a terminal result exactly once and freeze the voyage clock.
pub fn check_and_record(sim: &mut SimState, data: &GameData) -> Option<TerminalOutcome> {
    if let Some(existing) = &sim.terminal {
        return Some(existing.clone());
    }
    let (reason, evidence) = terminal_reason(sim, data)?;
    let outcome = TerminalOutcome {
        reason,
        month_clock: sim.month_clock,
        evidence,
    };
    sim.terminal = Some(outcome.clone());
    sim.survival.warning_active = false;
    sim.survival.warning_reviewed = true;
    sim.speed = crate::state::sim::GameSpeed::Paused;
    sim.push_log(format!("TERMINAL OUTCOME: {}.", reason.label()));
    Some(outcome)
}

/// Update the air countdown after the month's authoritative project work. A
/// critical reading pauses once and asks for recovery review; ordinary low air
/// never silently changes project choices.
pub fn update_air_warning(sim: &mut SimState, data: &GameData) -> bool {
    if sim.terminal.is_some() {
        return false;
    }
    if sim.ship.life_support <= 0.0 {
        sim.survival.air_zero_months = sim.survival.air_zero_months.saturating_add(1);
    } else {
        let recovered = sim.survival.air_zero_months > 0;
        sim.survival.air_zero_months = 0;
        if recovered {
            sim.survival.emergency_used = false;
            sim.survival.warning_active = false;
            sim.survival.warning_reviewed = false;
            sim.push_log("Life support rose above zero; the emergency countdown clears.");
        }
    }
    observe_air_warning(sim, data)
}

/// Observe event/action damage without charging another simulation month.
pub fn observe_air_warning(sim: &mut SimState, data: &GameData) -> bool {
    if sim.terminal.is_some() {
        return false;
    }
    if sim.ship.life_support > data.config.survival.critical_warning_threshold {
        sim.survival.warning_active = false;
        sim.survival.warning_reviewed = false;
        sim.survival.emergency_used = false;
    }
    let critical = sim.ship.life_support <= data.config.survival.critical_warning_threshold
        || sim.survival.air_zero_months > 0;
    if critical && !sim.survival.warning_active && !sim.survival.warning_reviewed {
        sim.survival.warning_active = true;
        sim.set_speed(crate::state::sim::GameSpeed::Paused);
        return true;
    }
    false
}

pub fn review_recovery(sim: &mut SimState) {
    sim.survival.warning_active = false;
    sim.survival.warning_reviewed = true;
}

pub fn resume_after_warning(sim: &mut SimState) {
    sim.survival.warning_active = false;
    sim.survival.warning_reviewed = true;
    if sim.speed == crate::state::sim::GameSpeed::Paused {
        sim.toggle_pause();
    }
}

/// One bounded stabilisation per air crisis. It is intentionally available at
/// zero expertise: recovery must not deadlock behind the training project.
pub fn emergency_stabilise(sim: &mut SimState, data: &GameData) -> Result<(), String> {
    if sim.terminal.is_some() {
        return Err("The vessel is already lost; no recovery action remains.".to_owned());
    }
    if sim.survival.emergency_used {
        return Err(
            "The emergency stabiliser has already been used in this air crisis.".to_owned(),
        );
    }
    let cfg = &data.config.survival;
    let cost = ResourceDelta {
        credits: -cfg.emergency_resource_cost.credits,
        energy: -cfg.emergency_resource_cost.energy,
        minerals: -cfg.emergency_resource_cost.minerals,
        food: -cfg.emergency_resource_cost.food,
        influence: -cfg.emergency_resource_cost.influence,
    };
    if !sim.resources.can_afford(&cost) {
        return Err("The disclosed emergency stabilisation stores are unavailable.".to_owned());
    }
    if sim.ship.spare_parts < cfg.emergency_parts_cost {
        return Err(format!(
            "Emergency stabilisation needs {} spare parts.",
            cfg.emergency_parts_cost
        ));
    }
    sim.resources.apply(&cost);
    sim.ship.spare_parts -= cfg.emergency_parts_cost;
    sim.ship.life_support = (sim.ship.life_support + cfg.emergency_air_gain).clamp(0.0, 1.0);
    sim.survival.emergency_used = true;
    sim.survival.air_zero_months = 0;
    sim.survival.warning_active = false;
    sim.survival.warning_reviewed = true;
    sim.push_log("Emergency stabilisation restored a thin, temporary breath to the ship.");
    Ok(())
}

/// Legacy migration hook: an old save with zero air has not yet had a chance
/// to receive the new warning, so it starts with the full grace period.
pub fn migrate_legacy(sim: &mut SimState) {
    if sim.terminal.is_none() && sim.dynasty.extinct && sim.survival.migration_notice.is_none() {
        sim.survival.migration_notice = Some(
            "This campaign was migrated after dynasty extinction; its terminal record is preserved."
                .to_owned(),
        );
    }
    if sim.terminal.is_none() && sim.ship.life_support <= 0.0 && sim.survival.air_zero_months == 0 {
        sim.survival.warning_active = true;
        sim.survival.migration_notice = Some(
            "This campaign was migrated with zero life support; the full emergency grace period applies."
                .to_owned(),
        );
        sim.speed = crate::state::sim::GameSpeed::Paused;
    }
}

#[cfg(test)]
mod tests;
