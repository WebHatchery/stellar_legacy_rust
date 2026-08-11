use super::*;
use crate::chronicle::ChronicleEntry;

fn entry(outcome: &str) -> ChronicleEntry {
    ChronicleEntry {
        completed_year: 60,
        contract_name: "c".into(),
        objective: "Mining".into(),
        legacy_id: "preservers".into(),
        leader_name: "l".into(),
        generation: 1,
        score: 0.95,
        outcome: outcome.into(),
        duration_years: 60,
    }
}

#[test]
fn definitions_are_stable_and_nonempty() {
    let defs = definitions();
    assert_eq!(defs.len(), 6);
    assert!(defs.iter().all(|a| !a.unlocked));
}

#[test]
fn fresh_campaign_unlocks_nothing() {
    let data = crate::data::GameData::load().unwrap();
    let sim = SimState::new_campaign(
        &data,
        "preservers",
        1,
        &crate::state::sim::founding_faction_ids(&data),
    );
    assert!(evaluate(&sim, &ChronicleStore::default()).is_empty());
}

#[test]
fn milestones_unlock_from_state_and_chronicle() {
    let data = crate::data::GameData::load().unwrap();
    let mut sim = SimState::new_campaign(
        &data,
        "preservers",
        1,
        &crate::state::sim::founding_faction_ids(&data),
    );
    sim.dynasty.generation = 5;
    sim.month_clock = 100 * 12;
    let chronicle = ChronicleStore {
        entries: vec![entry("Complete"), entry("Partial"), entry("Complete")],
    };

    let ids = evaluate(&sim, &chronicle);
    assert!(ids.contains(&"first_charter"));
    assert!(ids.contains(&"flawless"));
    assert!(ids.contains(&"long_line"));
    assert!(ids.contains(&"against_the_void"));
    assert!(ids.contains(&"storied_house")); // renown 285 >= 250
    assert!(!ids.contains(&"full_registry")); // only 3 recorded
}
