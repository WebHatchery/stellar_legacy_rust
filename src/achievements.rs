//! Chronicle achievements (GDD §10). A small meta-progression layer over the
//! persistent Chronicle: milestones that unlock from the run and the
//! cross-playthrough record. Purely cosmetic — never touches the deterministic
//! sim — and persisted under their own key, separate from the campaign save.

use macroquad_toolkit::achievements::{Achievement, Achievements};
use macroquad_toolkit::persistence::{load_json_key, save_json_key};

use crate::chronicle::ChronicleStore;
use crate::state::sim::SimState;

const KEY: &str = "achievements";

/// The full definition list. Order is the display order.
pub fn definitions() -> Vec<Achievement> {
    vec![
        Achievement::new(
            "first_charter",
            "First Charter",
            "Complete your first contract.",
        ),
        Achievement::new(
            "flawless",
            "Flawless Voyage",
            "Complete a contract at the Complete band.",
        ),
        Achievement::new(
            "full_registry",
            "Full Registry",
            "Record five contracts in the Chronicle.",
        ),
        Achievement::new(
            "long_line",
            "The Long Line",
            "Steer a dynasty into its fifth generation.",
        ),
        Achievement::new(
            "against_the_void",
            "Against the Void",
            "Keep a voyage alive to year 100.",
        ),
        Achievement::new(
            "storied_house",
            "Storied House",
            "Amass 250 Chronicle renown.",
        ),
    ]
}

/// Load persisted unlock state, reconciled with the current definitions.
pub fn load(game_name: &str) -> Achievements {
    let mut achievements: Achievements = load_json_key(game_name, KEY).unwrap_or_default();
    achievements.sync_definitions(definitions());
    achievements
}

/// Persist the registry.
pub fn save(achievements: &Achievements, game_name: &str) -> Result<(), String> {
    save_json_key(game_name, KEY, achievements)
}

/// Which achievement ids the current state satisfies. Every condition is
/// derived from post-state (the sim plus the persistent Chronicle), so the
/// caller can simply unlock each on any state change; already-unlocked ids are
/// a no-op.
pub fn evaluate(sim: &SimState, chronicle: &ChronicleStore) -> Vec<&'static str> {
    let mut ids = Vec::new();
    let recorded = chronicle.entries.len();
    if recorded >= 1 {
        ids.push("first_charter");
    }
    if recorded >= 5 {
        ids.push("full_registry");
    }
    if chronicle.entries.iter().any(|e| e.outcome == "Complete") {
        ids.push("flawless");
    }
    if sim.dynasty.generation >= 5 {
        ids.push("long_line");
    }
    if sim.year() >= 100 {
        ids.push("against_the_void");
    }
    if crate::heritage::renown(chronicle) >= 250 {
        ids.push("storied_house");
    }
    ids
}

#[cfg(test)]
mod tests;
