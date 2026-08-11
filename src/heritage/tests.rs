use super::*;
use crate::chronicle::ChronicleEntry;

fn tiers() -> Vec<HeritageTier> {
    vec![
        HeritageTier {
            min_renown: 0,
            name: "Founding".into(),
            credits: 0,
            influence: 0,
            tradition: 0,
        },
        HeritageTier {
            min_renown: 100,
            name: "Remembered".into(),
            credits: 500,
            influence: 0,
            tradition: 5,
        },
        HeritageTier {
            min_renown: 250,
            name: "Storied".into(),
            credits: 1500,
            influence: 100,
            tradition: 15,
        },
    ]
}

fn entry(score: f32) -> ChronicleEntry {
    ChronicleEntry {
        completed_year: 60,
        contract_name: "c".into(),
        objective: "Mining".into(),
        legacy_id: "preservers".into(),
        leader_name: "l".into(),
        generation: 1,
        score,
        outcome: "Complete".into(),
        duration_years: 60,
    }
}

#[test]
fn empty_chronicle_is_founding_tier() {
    let store = ChronicleStore::default();
    let h = derive(&store, &tiers());
    assert_eq!(h.tier_name, "Founding");
    assert!(!h.has_bonus());
}

#[test]
fn renown_accumulates_and_selects_highest_cleared_tier() {
    let store = ChronicleStore {
        entries: vec![entry(0.95), entry(0.9), entry(0.85)],
    };
    assert_eq!(renown(&store), 270);
    let h = derive(&store, &tiers());
    assert_eq!(h.tier_name, "Storied");
    assert!(h.has_bonus());
}

#[test]
fn apply_grants_the_bonus() {
    let data = crate::data::GameData::load().unwrap();
    let mut sim = SimState::new_campaign(
        &data,
        "preservers",
        7,
        &crate::state::sim::founding_faction_ids(&data),
    );
    let credits = sim.resources.credits;
    let tradition = sim.legacy.tradition_points;
    let h = Heritage {
        renown: 300,
        tier_name: "Storied".into(),
        credits: 1500,
        influence: 100,
        tradition: 15,
    };
    apply(&mut sim, &h);
    assert_eq!(sim.resources.credits, credits + 1500);
    assert_eq!(sim.legacy.tradition_points, tradition + 15);
}
