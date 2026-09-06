//! Deterministic continuity callbacks for the Custodian's archive.
//!
//! Reigns and obligation histories already preserve the facts needed to
//! remember a handover. This module selects one relevant duty and formats a
//! compact callback without inventing private thoughts or repeating the full
//! ship log.

use crate::state::sim::{Dynasty, Obligation, Reign, SimState};

/// Format the handover memory that should be surfaced after a captain changes.
/// The selected obligation is stable for a given save: active duties with the
/// most succession crossings win, then earlier creation and authored id.
pub fn succession_callback(sim: &SimState, outgoing: &str, incoming: &str) -> String {
    let priority = sim.authority.captain_priority.label();
    let duty = relevant_obligation(sim, incoming);
    let provenance = outgoing_reign(&sim.dynasty, outgoing).is_some();
    match (provenance, duty) {
        (true, Some(obligation)) => format!(
            "Custodian archive: Captain {outgoing} recorded \"{}\" in Year {} for {}. Captain {incoming} now carries it after {} succession{}; current priority is {priority}.",
            obligation.title,
            obligation.created_year,
            obligation.beneficiary,
            obligation.successions_crossed,
            if obligation.successions_crossed == 1 { "" } else { "s" },
        ),
        _ => format!(
            "Custodian archive: Captain {outgoing} handed the commission to Captain {incoming}. The new priority is {priority}; no active duty crossed this handover.",
        ),
    }
}

/// Choose a surviving obligation from the persistent record rather than the
/// trimmed general log. A missing or old-save provenance stays neutral.
pub fn relevant_obligation<'a>(sim: &'a SimState, incoming: &str) -> Option<&'a Obligation> {
    sim.obligations
        .iter()
        .filter(|obligation| {
            obligation.status.is_active()
                && obligation.responsible == incoming
                && obligation.successions_crossed > 0
        })
        .min_by_key(|obligation| {
            (
                std::cmp::Reverse(obligation.successions_crossed),
                obligation.created_year,
                obligation.authored_id.as_str(),
            )
        })
}

/// Return the outgoing captain's durable reign record when it is available.
/// This helper makes old saves explicit: an empty reign roster is an archival
/// gap, not permission to fabricate a name or a memory.
pub fn outgoing_reign<'a>(dynasty: &'a Dynasty, outgoing: &str) -> Option<&'a Reign> {
    dynasty
        .reigns
        .iter()
        .rev()
        .find(|reign| reign.name == outgoing)
}

#[cfg(test)]
mod tests;
