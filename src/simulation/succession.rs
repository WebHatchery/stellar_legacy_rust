//! Dynasty generational renewal and leader succession (GDD §5.3).
//!
//! Aging and death are now continuous (see [`crate::simulation::mortality`]):
//! everyone ages a year on Founding Day and faces a monthly death roll. What
//! remains generational is *renewal* — every `generation_interval_years` a new
//! cohort of young members joins ([`process_generation`]) — and *succession*,
//! which the mortality tick drives through [`install_successor`] whenever the
//! leader's seat falls empty or a retirement-aged leader has an heir ready.

use crate::data::{GameConfig, GameData};
use crate::state::sim::{Dynasty, DynastyMember, SimState};

/// True while at least one non-leader member sits in the eligible heir band.
pub fn eligible_heir_exists(dynasty: &Dynasty, config: &GameConfig) -> bool {
    dynasty
        .members
        .iter()
        .any(|m| !m.is_leader && m.age >= config.heir_min_age && m.age <= config.heir_max_age)
}

/// The heir who would take the chair under an orderly handoff: the eligible
/// council designate first, otherwise the strongest eligible dynast. This is
/// the same plan `install_successor` follows before its emergency fallback.
pub fn planned_heir<'a>(dynasty: &'a Dynasty, config: &GameConfig) -> Option<&'a DynastyMember> {
    let eligible = |member: &&DynastyMember| {
        !member.is_leader && member.age >= config.heir_min_age && member.age <= config.heir_max_age
    };
    dynasty
        .designated_heir
        .and_then(|id| {
            dynasty
                .members
                .iter()
                .find(|member| member.id == id && eligible(member))
        })
        .or_else(|| {
            dynasty
                .members
                .iter()
                .filter(eligible)
                .max_by_key(|member| (member.leadership, member.id))
        })
}

/// Install a new leader (GDD §4 Select Heir): clear the current leader, then take
/// the council-designated heir if one is living and age-eligible, otherwise the
/// highest-leadership member in the ideal heir band, and failing that the best of
/// whoever remains — a ship is never left without a captain while anyone lives.
/// Returns the new leader's name (if any member remained) and whether the dynasty
/// is now extinct (no members at all). `year` stamps the handoff into the reign
/// roster, which outlives the living members the way the chair itself does.
pub fn install_successor(
    dynasty: &mut Dynasty,
    config: &GameConfig,
    year: u32,
) -> (Option<String>, bool) {
    let planned_id = planned_heir(dynasty, config).map(|member| member.id);
    // A handoff starts a new reign (content-depth campaign skeleton round 19).
    dynasty.leader_reign_years = 0;
    // Close the outgoing captaincy before the seat is cleared — after the loop
    // below there is no way to tell who was sitting in it.
    dynasty.end_reign(year);
    dynasty.long_reign_marked = false;
    for member in &mut dynasty.members {
        member.is_leader = false;
    }
    // Fallback when no one sits in the ideal band: the strongest survivor still
    // leads — an unusually young or old captain, but a captain. Ties break on id
    // so a given roster is deterministic.
    let best_any = || {
        dynasty
            .members
            .iter()
            .enumerate()
            .max_by_key(|(_, m)| (m.leadership, m.id))
            .map(|(i, _)| i)
    };
    let heir_index = planned_id
        .and_then(|id| dynasty.members.iter().position(|member| member.id == id))
        .or_else(best_any);
    dynasty.designated_heir = None;
    match heir_index {
        Some(i) => {
            dynasty.members[i].is_leader = true;
            dynasty.begin_reign(year);
            (Some(dynasty.members[i].name.clone()), false)
        }
        None => {
            dynasty.extinct = dynasty.members.is_empty();
            (None, dynasty.extinct)
        }
    }
}

/// Mark a new generation (GDD §5.3): advance the generation counter and return
/// the young adults who have come of age since the last one (births are yearly
/// now — see `mortality::annual_aging`; this only closes the generational ledger
/// and reports its tally for the coming-of-age line).
pub fn process_generation(sim: &mut SimState, _data: &GameData) -> u32 {
    sim.dynasty.generation += 1;
    sim.dynasty.years_since_generation = 0;
    let born = sim.dynasty.births_this_generation;
    sim.dynasty.births_this_generation = 0;
    born
}

#[cfg(test)]
mod tests;
