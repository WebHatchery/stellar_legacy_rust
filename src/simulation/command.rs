//! Command posture effects: a small policy layer between the council's intent
//! and the deterministic voyage services.
//!
//! The UI chooses the posture; this module owns its numbers so event rolling,
//! objective work, fuel use, and annual social recovery stay consistent.

use crate::state::sim::{CommandPosture, SimState};

/// Event-pressure multiplier applied after the normal monthly chance is derived.
pub fn event_chance_factor(posture: CommandPosture) -> f32 {
    match posture {
        // The default must preserve the authored event cadence so old saves and
        // deterministic balance fixtures keep their established rhythm.
        CommandPosture::Steady => 1.0,
        CommandPosture::Expeditionary => 1.18,
        CommandPosture::Civic => 0.92,
    }
}

/// Objective-work multiplier for Operation months.
pub fn objective_factor(posture: CommandPosture) -> f32 {
    match posture {
        CommandPosture::Steady => 1.0,
        CommandPosture::Expeditionary => 1.12,
        CommandPosture::Civic => 0.92,
    }
}

/// Fuel-burn multiplier for Travel months.
pub fn fuel_burn_factor(posture: CommandPosture) -> f32 {
    match posture {
        CommandPosture::Steady => 1.0,
        CommandPosture::Expeditionary => 1.08,
        CommandPosture::Civic => 0.95,
    }
}

/// Whether the council can call a new posture review right now. Port is always
/// flexible; an underway change is a once-per-year strategic commitment.
pub fn posture_change_allowed(sim: &SimState) -> bool {
    sim.contract.is_none() || sim.month_clock >= sim.command_posture_locked_until
}

/// The next review date after an underway posture change.
pub fn next_review_month(sim: &SimState) -> u32 {
    if sim.contract.is_some() {
        sim.month_clock.saturating_add(12)
    } else {
        0
    }
}

/// Apply the posture's annual social consequences after the ordinary economy
/// and maintenance pass. The effects are intentionally small: posture should
/// steer a long campaign, not erase the consequences of starvation or collapse.
pub fn apply_annual_effects(sim: &mut SimState) {
    if sim.contract.is_none() {
        return;
    }
    let (morale, unity, stability, loyalty) = match sim.command_posture {
        CommandPosture::Steady => (0.0, 0.0, 0.0, 0.0),
        CommandPosture::Expeditionary => (-0.008, -0.004, 0.0, -0.004),
        CommandPosture::Civic => (0.015, 0.01, 0.01, 0.012),
    };
    sim.population.morale = (sim.population.morale + morale).clamp(0.0, 1.0);
    sim.population.unity = (sim.population.unity + unity).clamp(0.0, 1.0);
    sim.population.stability = (sim.population.stability + stability).clamp(0.0, 1.0);
    sim.population.legacy_loyalty = (sim.population.legacy_loyalty + loyalty).clamp(0.0, 1.0);
}

#[cfg(test)]
mod tests;
