use super::*;
use crate::data::GameData;
use crate::state::sim::{ObligationStatus, ObligationVisibility};

#[test]
fn callback_uses_the_persistent_duty_record() {
    let data = GameData::load().unwrap();
    let factions = crate::state::sim::founding_faction_ids(&data);
    let mut sim = SimState::new_campaign(&data, "preservers", 7, &factions);
    let outgoing = sim.dynasty.leader().unwrap().name.clone();
    let incoming = "Captain Ilyan Vale".to_owned();
    sim.obligations.push(crate::state::sim::Obligation {
        id: "obligation-1".to_owned(),
        authored_id: "first-watch".to_owned(),
        title: "Keep the archive open".to_owned(),
        source: "founding".to_owned(),
        creator: "Captain Vale".to_owned(),
        responsible: incoming.clone(),
        beneficiary: "the next generation".to_owned(),
        created_year: 12,
        due_year: None,
        resolution_event: String::new(),
        visibility: ObligationVisibility::Public,
        status: ObligationStatus::Pending,
        stakes: crate::state::sim::ObligationStakes {
            material: "none".to_owned(),
            reputation: "memory".to_owned(),
        },
        successions_crossed: 2,
        history: Vec::new(),
    });

    let text = succession_callback(&sim, &outgoing, &incoming);
    assert!(text.contains("Keep the archive open"));
    assert!(text.contains("after 2 successions"));
}

#[test]
fn old_save_without_reign_history_has_no_fabricated_outgoing_record() {
    let data = GameData::load().unwrap();
    let factions = crate::state::sim::founding_faction_ids(&data);
    let sim = SimState::new_campaign(&data, "preservers", 8, &factions);
    let mut dynasty = sim.dynasty.clone();
    dynasty.reigns.clear();
    assert!(outgoing_reign(&dynasty, "Unknown").is_none());
}
