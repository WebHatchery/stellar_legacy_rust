//! Authoritative vessel survival checks and the visible life-support rescue path.

use crate::data::contracts::ContractPhase;
use crate::data::{GameData, ResourceDelta};
use crate::simulation::{ship, subsystems};
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
    if let Some(evidence) = stranded(sim, data) {
        return Some((TerminalReason::Stranded, evidence));
    }
    None
}

/// The one way a sound, crewed ship is lost: it cannot move and cannot be
/// mended to move. That means a Travel leg with a dry tank, no fuel coming in
/// from the scoops, and nothing aboard that could change it — no salvaged
/// scoop-bearing part that can be fitted now, and no engineering repair that
/// would bring a fouled scoop back. A lost captain, a hungry year or a calm in
/// port is never this; a lucky event cannot be counted on, so it is not a fix.
/// Returns the evidence line when stranded.
pub fn stranded(sim: &SimState, data: &GameData) -> Option<String> {
    let contract = sim.contract.as_ref()?;
    let (_, phase) = contract.phase_at(contract.months_elapsed + 1);
    if phase != ContractPhase::Travel || sim.ship.fuel > 0.0 {
        return None;
    }
    let stats = ship::loadout_stats(sim, data);
    let regen_factor = subsystems::engineering_fuel_regen_factor(sim, data);
    if stats.fuel_regen > 0 && regen_factor > 0.0 {
        return None;
    }
    // Scoops fitted but fouled: a mendable engineering bay clears them.
    if stats.fuel_regen > 0 && subsystems::repair_blocker(sim, data, "engineering_bay").is_none() {
        return None;
    }
    // No scoop at all: a salvaged part that carries one, fittable right now.
    let fittable_scoop = sim.ship.salvage.iter().any(|id| {
        data.ship_components
            .find_any(id)
            .is_some_and(|(_, part)| part.stats.fuel_regen > 0)
            && ship::install_eligibility(sim, data, id) == ship::InstallEligibility::Ready
    });
    if fittable_scoop {
        return None;
    }
    let drive = data
        .ship_components
        .find_any(&sim.ship.engine)
        .map_or("drive", |(_, part)| part.name.as_str());
    Some(if stats.fuel_regen > 0 {
        format!(
            "The tank ran dry mid-crossing. The engineering bay is too far gone to clear the {drive}'s scoops, and no one aboard can mend it."
        )
    } else {
        format!(
            "The tank ran dry mid-crossing. The {drive} gathers no fuel of its own, and nothing in the salvage hold could be fitted to take it."
        )
    })
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
pub fn emergency_availability(sim: &SimState, data: &GameData) -> Result<(), String> {
    if sim.terminal.is_some() {
        return Err("The vessel is already lost; no recovery action remains.".to_owned());
    }
    if sim.survival.emergency_used {
        return Err(
            "The emergency stabiliser has already been used in this air crisis.".to_owned(),
        );
    }
    if sim.ship.life_support >= 1.0 {
        return Err("Air is already at 100%; emergency stores are not needed.".to_owned());
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
    Ok(())
}

pub fn emergency_stabilise(sim: &mut SimState, data: &GameData) -> Result<(), String> {
    emergency_availability(sim, data)?;
    let cfg = &data.config.survival;
    let cost = ResourceDelta {
        credits: -cfg.emergency_resource_cost.credits,
        energy: -cfg.emergency_resource_cost.energy,
        minerals: -cfg.emergency_resource_cost.minerals,
        food: -cfg.emergency_resource_cost.food,
        influence: -cfg.emergency_resource_cost.influence,
    };
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
    if sim.terminal.is_none() && (sim.ship.hull_integrity <= 0.0 || sim.population.count == 0) {
        sim.survival.migration_notice = Some("This legacy campaign has zero hull or population. It will end under the new survival rules when evaluation resumes.".to_owned());
        sim.speed = crate::state::sim::GameSpeed::Paused;
        sim.survival.warning_active = true;
        return;
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
#[allow(dead_code, unused_imports)]
mod tests {
    include!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/unit/simulation/survival/tests.rs"
    ));
}
