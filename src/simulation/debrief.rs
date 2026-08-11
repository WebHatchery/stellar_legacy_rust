//! Remembering a voyage, and sealing the report it comes home with.
//!
//! Two halves of one job. `remember` is called from the tick, the succession
//! path, and the event resolver as beats happen — the running record, kept on
//! the active contract. `seal` is called once, at the moment a charter
//! concludes, and snapshots everything the homecoming screen needs before
//! `sim.contract` is cleared out from under it.

use crate::data::{GameData, ResourceDelta};
use crate::simulation::contract::SuccessLevel;
use crate::state::sim::debrief::{DebriefMetric, DebriefMilestone, HighlightKind, VoyageDebrief};
use crate::state::sim::SimState;

/// Remember a beat of the voyage under way. A no-op in port — there is no
/// voyage to recap — so callers need not check for a contract themselves.
pub fn remember(sim: &mut SimState, data: &GameData, kind: HighlightKind, text: impl Into<String>) {
    let (year, month) = (sim.year(), sim.month());
    let limit = data.config.voyage_highlight_limit;
    if let Some(contract) = sim.contract.as_mut() {
        contract.push_highlight(year, month, kind, text, limit);
    }
}

/// Seal the homecoming report for the charter that just concluded. Must be
/// called while `sim.contract` is still set — it reads the metrics, milestones,
/// and remembered beats straight off it.
///
/// `payout` is what the charter actually paid after proration and reputation,
/// not the writ's headline reward; `homecoming_line` and `legacy_line` are the
/// authored prose already pushed to the log, repeated here so the debrief can
/// lead with them instead of asking the player to go find them.
pub fn seal(
    sim: &SimState,
    score: f32,
    level: SuccessLevel,
    payout: ResourceDelta,
    homecoming_line: Option<String>,
    legacy_line: Option<String>,
) -> Option<VoyageDebrief> {
    let contract = sim.contract.as_ref()?;
    let ended_year = sim.year();
    // The charter's own clock, not the campaign's: a fuel stall stretches the
    // wall-clock years without advancing the contract, and the writ was flown
    // against the latter.
    let duration_years = contract.months_elapsed / 12;
    let began_year = contract.began_year;

    let commanders = sim
        .dynasty
        .reigns
        .iter()
        .filter(|reign| reign.overlaps(began_year, ended_year))
        .cloned()
        .collect();
    let obligations = sim
        .obligations
        .iter()
        .filter(|obligation| {
            obligation.created_year >= began_year
                || obligation
                    .history
                    .iter()
                    .any(|entry| entry.year >= began_year)
        })
        .cloned()
        .collect();
    let institutions = sim
        .institution_records
        .iter()
        .filter(|record| record.year >= began_year && record.year <= ended_year)
        .cloned()
        .collect();

    Some(VoyageDebrief {
        contract_name: contract.name.clone(),
        objective: contract.objective.label().to_owned(),
        outcome: level.label().to_owned(),
        score,
        began_year,
        ended_year,
        duration_years,
        generations: sim
            .dynasty
            .generation
            .saturating_sub(contract.began_generation),
        payout,
        metrics: contract
            .metrics
            .iter()
            .map(|m| DebriefMetric {
                name: m.name.clone(),
                current: m.current,
                target: m.target,
                weight: m.weight,
            })
            .collect(),
        milestones: contract
            .milestones
            .iter()
            .map(|m| DebriefMilestone {
                name: m.name.clone(),
                reached: m.reached,
            })
            .collect(),
        highlights: contract.highlights.clone(),
        commanders,
        obligations,
        institutions,
        population_start: contract.starting_population,
        population_end: sim.population.count,
        homecoming_line,
        legacy_line,
    })
}

#[cfg(test)]
mod tests;
