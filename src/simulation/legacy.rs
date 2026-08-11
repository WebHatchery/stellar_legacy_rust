//! Legacy dilemmas and the failure-risk formula (GDD §5.5).
//!
//! Dilemmas fire on generation boundaries: each new generation confronts the
//! legacy's defining tension. Their success/failure branches update the real
//! tracked counters on `LegacyTrack`, which in turn feed the failure-risk
//! score surfaced on the Crew & Dynasty screen.

use crate::data::legacies::{DilemmaDef, DilemmaEffect, DilemmaOption};
use crate::data::{GameConfig, GameData};
use crate::state::sim::{PendingDilemma, SimState};

/// Roll whether the new generation faces a legacy dilemma. Called from the
/// tick on generation boundaries only; returns the pending dilemma without
/// applying anything (dilemmas always block — they are the player's defining
/// choice and are never delegated).
pub fn roll_dilemma(sim: &mut SimState, data: &GameData) -> Option<PendingDilemma> {
    if !sim.rng.chance(data.config.dilemma_chance_per_generation) {
        return None;
    }
    let legacy = data.legacies.get(&sim.legacy.legacy_id)?;
    if legacy.dilemmas.is_empty() {
        return None;
    }
    let dilemma = &legacy.dilemmas[sim.rng.below(legacy.dilemmas.len())];
    Some(PendingDilemma {
        dilemma_id: dilemma.id.clone(),
        rolled_month_clock: sim.month_clock,
    })
}

/// Look up the sim's pending dilemma definition in the loaded data.
pub fn pending_dilemma_def<'a>(sim: &SimState, data: &'a GameData) -> Option<&'a DilemmaDef> {
    let pending = sim.pending_dilemma.as_ref()?;
    data.legacies
        .get(&sim.legacy.legacy_id)?
        .dilemmas
        .iter()
        .find(|d| d.id == pending.dilemma_id)
}

/// Effective success chance for a dilemma option: the base chance plus a
/// combat bonus on Wanderer dilemmas (firepower backs the confrontation —
/// GDD combat → wanderer odds), capped by config. Shown honestly in the modal
/// and used for the roll (Pillar 3).
pub fn dilemma_odds(sim: &SimState, data: &GameData, option: &DilemmaOption) -> f32 {
    let combat_bonus = if sim.legacy.legacy_id == "wanderers" {
        let combat = crate::simulation::ship::loadout_stats(sim, data).combat;
        combat as f32 * data.config.ship.combat_dilemma_odds_per_point
    } else {
        0.0
    };
    // Who runs the ship can back or hinder a defining gamble (content-depth
    // factions round 10): while the named faction is dominant, its craft (or its
    // resistance) shifts the option's odds — the augmented back an augmentation,
    // the makers a risky repair, the arbiters drag on summary justice.
    let faction_bonus = if !option.dominant_faction.is_empty()
        && sim.dominant_faction_id() == Some(option.dominant_faction.as_str())
    {
        option.dominant_faction_odds
    } else {
        0.0
    };
    (option.success_chance + combat_bonus + faction_bonus)
        .clamp(0.0, data.config.ship.dilemma_odds_cap)
}

/// Resolve the pending dilemma with the chosen option: roll the option's
/// (combat-adjusted) success chance on the sim RNG, apply the winning branch
/// (including the legacy counters), log it, and clear the pending state.
/// Returns the log line that was recorded.
pub fn resolve_dilemma(sim: &mut SimState, data: &GameData, option_index: usize) -> Option<String> {
    let dilemma = pending_dilemma_def(sim, data)?.clone();
    let option = dilemma.options.get(option_index)?;

    let chance = dilemma_odds(sim, data, option);
    let succeeded = sim.rng.chance(chance);
    let effect = if succeeded {
        option.success.clone()
    } else {
        option.failure.clone()
    };

    apply_dilemma_effect(sim, &effect);
    let text = if effect.log.is_empty() {
        format!("{}: {}", dilemma.title, option.label)
    } else {
        effect.log.clone()
    };
    sim.push_log(text.clone());
    sim.pending_dilemma = None;
    Some(text)
}

fn apply_dilemma_effect(sim: &mut SimState, effect: &DilemmaEffect) {
    sim.resources.apply(&effect.resource_delta);
    sim.ship.apply(&effect.ship_delta);
    sim.population.apply(&effect.population_delta);

    let track = &mut sim.legacy;
    track.tradition_points += effect.tradition_points;
    track.body_horror_events += effect.body_horror_events;
    track.existential_dread = (track.existential_dread + effect.existential_dread).clamp(0.0, 1.0);
    track.piracy_reputation = (track.piracy_reputation + effect.piracy_reputation).clamp(0.0, 1.0);
}

/// One contributing factor of the failure-risk score, for honest UI display
/// (Pillar 3: only show numbers that are real).
#[derive(Debug, Clone)]
pub struct RiskFactor {
    pub label: &'static str,
    pub points: i32,
}

#[derive(Debug, Clone, Default)]
pub struct FailureRisk {
    pub total: i32,
    pub at_risk: bool,
    pub factors: Vec<RiskFactor>,
}

/// The §5.5 failure-risk formula. Cultural drift and unity threaten every
/// legacy; the legacy-specific counters only threaten the legacy whose
/// failure condition they belong to.
pub fn failure_risk(sim: &SimState, config: &GameConfig) -> FailureRisk {
    let fr = &config.failure_risk;
    let mut risk = FailureRisk::default();
    let mut add = |label, points| risk.factors.push(RiskFactor { label, points });

    if sim.population.cultural_drift > fr.drift_threshold {
        add("Cultural drift runs high", fr.drift_points);
    }
    if sim.population.unity < fr.unity_threshold {
        add("Unity has frayed", fr.unity_points);
    }
    match sim.legacy.legacy_id.as_str() {
        "preservers" => {
            if sim.legacy.tradition_points < fr.tradition_threshold {
                add("Tradition nears extinction", fr.tradition_points);
            }
        }
        "adaptors" => {
            if sim.legacy.body_horror_events >= fr.body_horror_threshold {
                add("The modifications have a cost", fr.body_horror_points);
            }
            if sim.legacy.existential_dread > fr.dread_threshold {
                add("Existential dread spreads", fr.dread_points);
            }
        }
        "wanderers" if sim.legacy.piracy_reputation > fr.piracy_threshold => {
            add("Piracy invites reprisal", fr.piracy_points);
        }
        _ => {}
    }

    risk.total = risk.factors.iter().map(|f| f.points).sum();
    risk.at_risk = risk.total > fr.at_risk_threshold;
    risk
}

#[cfg(test)]
mod tests;
