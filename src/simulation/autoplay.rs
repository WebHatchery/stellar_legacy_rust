//! Automated full-mission playthrough harness (W1-rescale).
//!
//! The owner's primary playtest channel: a deterministic *policy player* that
//! starts a charter and flies it year by year with a fixed, dumb strategy —
//! resolve every council decision by first choice, patch the hull when it
//! slips, buy food when the stores run low — then reports how the voyage
//! ended. It exists to soak the whole content set (events, dilemmas,
//! succession, contract completion) across a generational voyage and catch any
//! invariant that escapes its range along the way.
//!
//! Test-only: it drives the same stateless services the game does, so a green
//! soak means a real campaign of that length stays internally consistent.

use crate::data::GameData;
use crate::simulation::contract::start_contract;
use crate::simulation::tick::advance_months;
use crate::simulation::{event_resolver, legacy, market, ship, subsystems};
use crate::state::sim::{SimState, TradeResource};

/// How a played-out mission ended.
#[derive(Debug, Clone, Copy)]
pub struct MissionOutcome {
    /// The charter reached its target duration and scored out.
    pub completed: bool,
    /// The dynasty ran out of heirs before the charter concluded.
    pub extinct: bool,
    /// The campaign year the run ended on.
    pub final_year: u32,
    /// Success score at completion (0.0 if the run never completed).
    pub final_score: f32,
}

/// Fly `contract_id` to its conclusion (or `max_years`, whichever comes first)
/// under a fixed policy, asserting every per-year invariant along the way.
///
/// Policy: refit and service affordable systems in port; resolve a pending
/// event by its first visible, affordable choice and a dilemma by its first
/// choice; field-repair the hull whenever it drops below half; buy food when
/// stores fall under the crisis threshold. Deterministic for a given
/// (sim, contract) pair — all randomness flows through `sim.rng`.
pub fn play_mission(
    sim: &mut SimState,
    data: &GameData,
    contract_id: &str,
    max_years: u32,
) -> MissionOutcome {
    let template = data
        .contracts
        .get(contract_id)
        .expect("autoplay contract id must resolve to a charter")
        .clone();
    // A maintenance-heavy policy uses the drydock window between charters.
    // Without this, it could launch a fresh mission with a degraded engineering
    // bay, burn more fuel than the nominal route budget, and coast for decades
    // despite holding ample port resources. That is not the player policy this
    // economy soak claims to represent.
    if sim.ship.hull_integrity < 1.0
        || sim.ship.life_support < 1.0
        || sim.ship.spare_parts < data.config.repair.full_parts_restock
    {
        let _ = ship::full_repair(sim, &data.config);
    }
    for id in crate::data::GameData::sorted_ids(&data.subsystems) {
        let required = data
            .subsystems
            .get(&id)
            .map(|definition| definition.repair_knowledge_required)
            .unwrap_or(1.0);
        while sim
            .subsystems
            .get(&id)
            .is_some_and(|state| state.knowledge < required)
            && sim.resources.credits > 20_000
        {
            if subsystems::train_subsystem_knowledge(sim, data, &id).is_err() {
                break;
            }
        }
        if sim
            .subsystems
            .get(&id)
            .is_some_and(|state| state.condition < 1.0)
        {
            let _ = subsystems::repair_subsystem(sim, data, &id);
        }
    }
    // Provision and launch explicitly (W4): top the tank in port, put the
    // charter under consideration, then commit — no silent contract start.
    sim.ship.fuel = 1.0;
    sim.selected_charter = Some(contract_id.to_owned());
    sim.contract = Some(start_contract(&template, sim));
    for operation in &template.launch_obligation_operations {
        sim.apply_obligation_operation(operation);
    }
    // Lay out the seeded campaign skeleton at LAUNCH (W6).
    if let Some(c) = sim.contract.as_mut() {
        c.beats = event_resolver::skeleton::generate_beats(
            &mut sim.rng,
            c,
            &data.config.campaign_skeleton,
        );
    }
    sim.selected_charter = None;

    let mut outcome = MissionOutcome {
        completed: false,
        extinct: false,
        final_year: sim.year(),
        final_score: 0.0,
    };

    let max_months = max_years * 12;
    // Once a faction has left the ship it must never reappear as Aboard (W7).
    let mut ever_lost: std::collections::HashSet<String> = std::collections::HashSet::new();
    while sim.month_clock < max_months {
        // Clear any blocking council decision by taking the first choice — the
        // same dumb policy the game's own soak has always used.
        if sim.pending_dilemma.is_some() {
            legacy::resolve_dilemma(sim, data, 0);
        }
        if let Some(pending) = sim.pending_event.clone() {
            match data.events.get(&pending.template_id).cloned() {
                Some(template) => {
                    let choice = event_resolver::available_outcome_indices(sim, &template)
                        .into_iter()
                        .find(|&index| {
                            event_resolver::outcome_affordable(sim, &template.outcomes[index])
                        })
                        .expect("every event needs an affordable fallback");
                    event_resolver::apply_outcome(sim, data, &template, choice);
                }
                None => sim.pending_event = None,
            }
        }
        if sim.dynasty.extinct {
            outcome.extinct = true;
            break;
        }

        // Standing orders: keep the hull off the floor and the galley stocked.
        // Both verbs refuse (harmlessly) when they can't be paid for.
        if sim.ship.hull_integrity < 0.5 {
            let _ = ship::field_repair(sim, &data.config, ship::RepairKind::Hull);
        }
        if sim.resources.food < data.config.low_food_threshold {
            let _ = market::buy(sim, TradeResource::Food, 1000);
        }
        // Keep the subsystems mended and their knowledge alive when it's cheap
        // and needed (W5) — train up before the experts die out, patch what
        // slips. Both verbs refuse harmlessly when they can't be paid for.
        for id in crate::data::GameData::sorted_ids(&data.subsystems) {
            let Some(sub) = sim.subsystems.get(&id) else {
                continue;
            };
            let (condition, knowledge) = (sub.condition, sub.knowledge);
            let required = data
                .subsystems
                .get(&id)
                .map(|d| d.repair_knowledge_required)
                .unwrap_or(1.0);
            if knowledge < required && sim.resources.credits > 20_000 {
                let _ = subsystems::train_subsystem_knowledge(sim, data, &id);
            }
            if condition < 0.5 {
                let _ = subsystems::repair_subsystem(sim, data, &id);
            }
        }

        // Fly a decade per step (hard-stops on the next decision either way), so
        // the dumb policy still resolves everything in order (real-time loop).
        let report = advance_months(sim, data, 120);
        outcome.final_year = sim.year();
        assert_year_invariants(sim);
        for faction in &sim.factions {
            if faction.is_aboard() {
                assert!(
                    !ever_lost.contains(&faction.faction_id),
                    "a lost faction returned to Aboard: {}",
                    faction.faction_id
                );
            } else {
                ever_lost.insert(faction.faction_id.clone());
            }
        }

        if let Some((score, _)) = report.contract_completed {
            outcome.completed = true;
            outcome.final_score = score;
            sim.contract = None;
            break;
        }
        if report.dynasty_extinct {
            outcome.extinct = true;
            break;
        }
    }

    outcome
}

/// Every invariant that must hold at the end of any simulated year: 0-1
/// fractions stay in range, resources never go negative, and a living dynasty
/// always has someone at its head.
fn assert_year_invariants(sim: &SimState) {
    for fraction in [
        sim.population.morale,
        sim.population.unity,
        sim.population.stability,
        sim.population.legacy_loyalty,
        sim.population.adaptation,
        sim.population.cultural_drift,
        sim.ship.hull_integrity,
        sim.ship.life_support,
        sim.ship.fuel,
    ] {
        assert!(
            (0.0..=1.0).contains(&fraction),
            "0-1 sim fraction escaped its range: {fraction} at year {}",
            sim.year()
        );
    }
    assert!(sim.resources.food >= 0 && sim.resources.credits >= 0);
    if !sim.dynasty.extinct {
        assert!(
            sim.dynasty.leader().is_some(),
            "a living dynasty must always have a leader (year {})",
            sim.year()
        );
    }
    // W7: Aboard members always sum to the head count, and a faction that has
    // left the ship carries no members.
    let aboard_sum: u32 = sim
        .factions
        .iter()
        .filter(|f| f.is_aboard())
        .map(|f| f.members)
        .sum();
    assert_eq!(
        aboard_sum,
        sim.population.count,
        "faction members must sum to population.count (year {})",
        sim.year()
    );
    for faction in &sim.factions {
        if !faction.is_aboard() {
            assert_eq!(
                faction.members, 0,
                "a departed faction carries no members ({})",
                faction.faction_id
            );
        }
    }
    // W5: subsystem condition and knowledge stay 0-1 forever.
    for (id, sub) in &sim.subsystems {
        assert!(
            (0.0..=1.0).contains(&sub.condition),
            "subsystem {id} condition {} escaped 0-1 (year {})",
            sub.condition,
            sim.year()
        );
        assert!(
            (0.0..=1.0).contains(&sub.knowledge),
            "subsystem {id} knowledge {} escaped 0-1 (year {})",
            sub.knowledge,
            sim.year()
        );
    }
}

#[cfg(test)]
mod tests;
