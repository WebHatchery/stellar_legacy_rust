//! Mission-specific charter approach effects.
//!
//! The approach is chosen in port and remains fixed for one writ. This module
//! owns its gates and numbers so the live tick, forecasts, and UI read one
//! deterministic policy instead of each inventing a slightly different bill.

use crate::data::{contracts::ContractTemplate, GameData, PopulationDelta};
use crate::state::sim::{CharterApproach, SimState};

const PROVE_SPEED_REQUIREMENT: i32 = 4;

pub fn objective_factor(approach: CharterApproach) -> f32 {
    match approach {
        // The default preserves authored charter baselines for old saves,
        // fixtures, and captains who make no extra doctrine commitment.
        CharterApproach::ProtectTheMargin => 1.0,
        CharterApproach::ProveTheWrit => 1.12,
        CharterApproach::CarryThePeople => 0.92,
    }
}

pub fn event_chance_factor(approach: CharterApproach) -> f32 {
    match approach {
        CharterApproach::ProtectTheMargin => 1.0,
        CharterApproach::ProveTheWrit => 1.12,
        CharterApproach::CarryThePeople => 0.88,
    }
}

pub fn fuel_burn_factor(approach: CharterApproach) -> f32 {
    match approach {
        CharterApproach::ProtectTheMargin => 1.0,
        CharterApproach::ProveTheWrit => 1.06,
        CharterApproach::CarryThePeople => 0.96,
    }
}

pub fn preserve_attrition_factor(approach: CharterApproach) -> f32 {
    match approach {
        CharterApproach::ProtectTheMargin => 1.0,
        CharterApproach::ProveTheWrit => 1.10,
        CharterApproach::CarryThePeople => 0.72,
    }
}

pub fn annual_population_delta(approach: CharterApproach) -> PopulationDelta {
    match approach {
        CharterApproach::ProtectTheMargin => PopulationDelta::default(),
        CharterApproach::ProveTheWrit => PopulationDelta {
            morale: -0.006,
            unity: -0.003,
            legacy_loyalty: -0.003,
            ..Default::default()
        },
        CharterApproach::CarryThePeople => PopulationDelta {
            morale: 0.010,
            unity: 0.008,
            stability: 0.006,
            legacy_loyalty: 0.004,
            ..Default::default()
        },
    }
}

pub fn annual_effects(sim: &mut SimState) {
    let Some(approach) = sim.contract.as_ref().map(|contract| contract.approach) else {
        return;
    };
    sim.population.apply(&annual_population_delta(approach));
}

pub fn choice_available(
    sim: &SimState,
    data: &GameData,
    template: &ContractTemplate,
    approach: CharterApproach,
) -> bool {
    unavailable_reason(sim, data, template, approach).is_none()
}

pub fn unavailable_reason(
    sim: &SimState,
    data: &GameData,
    _template: &ContractTemplate,
    approach: CharterApproach,
) -> Option<String> {
    if approach == CharterApproach::ProveTheWrit {
        let speed = crate::simulation::ship::loadout_stats(sim, data).speed;
        if speed < PROVE_SPEED_REQUIREMENT {
            return Some(format!(
                "Requires ship speed {PROVE_SPEED_REQUIREMENT}; current rating {speed}."
            ));
        }
    }
    None
}

/// Signed modifier line for an approach. Neutral terms are omitted, so an
/// approach with no trade-offs returns an empty string and callers draw nothing.
pub fn effect_summary(approach: CharterApproach) -> String {
    let people = annual_population_delta(approach);
    let pct =
        |label: &str, value: f32| (value.round() != 0.0).then(|| format!("{label} {value:+.0}%"));
    let tenth = |label: &str, value: f32| {
        ((value * 10.0).round() != 0.0).then(|| format!("{value:+.1}% {label}"))
    };
    let ship: Vec<String> = [
        pct("WORK", (objective_factor(approach) - 1.0) * 100.0),
        pct("EVENTS", (event_chance_factor(approach) - 1.0) * 100.0),
        pct("FUEL", (fuel_burn_factor(approach) - 1.0) * 100.0),
    ]
    .into_iter()
    .flatten()
    .collect();
    let annual: Vec<String> = [
        tenth("morale", people.morale * 100.0),
        tenth("unity", people.unity * 100.0),
        pct(
            "PRESERVE LOSS",
            (preserve_attrition_factor(approach) - 1.0) * 100.0,
        ),
    ]
    .into_iter()
    .flatten()
    .collect();
    let mut lines = Vec::new();
    if !ship.is_empty() {
        lines.push(ship.join(" · "));
    }
    if !annual.is_empty() {
        lines.push(format!("ANNUAL PEOPLE {}", annual.join(" · ")));
    }
    lines.join("\n")
}
#[allow(dead_code, unused_imports)]
mod tests {
    include!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/unit/simulation/approach/tests.rs"
    ));
}
