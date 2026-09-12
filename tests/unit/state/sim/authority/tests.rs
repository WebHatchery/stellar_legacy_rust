use super::*;
use crate::data::GameData;
use crate::simulation::contract;

#[test]
fn unknown_captain_traits_use_a_steady_fallback() {
    let member = DynastyMember {
        id: 1,
        name: "Unknown Captain".to_owned(),
        age: 40,
        leadership: 50,
        specialization: String::new(),
        trait_name: String::new(),
        is_leader: true,
    };
    assert_eq!(priority_for_member(Some(&member)), CaptainPriority::Steady);
}

#[test]
fn civic_captain_can_object_to_expeditionary_posture() {
    let data = GameData::load().unwrap();
    let factions = crate::state::sim::founding_faction_ids(&data);
    let mut sim = SimState::new_campaign(&data, "preservers", 41, &factions);
    sim.contract = Some(contract::start_contract(
        data.contracts.get("founding_colony").unwrap(),
        &sim,
    ));
    sim.population.morale = 0.5;
    sim.authority.captain_name = "Captain Vale".to_owned();
    sim.authority.captain_priority = CaptainPriority::Civic;

    let review = posture_review(&sim, CommandPosture::Expeditionary).unwrap();
    assert_eq!(review.compromise, CommandPosture::Steady);
    assert!(!review.emergency_allowed);
}

#[test]
fn review_completion_is_one_shot_and_keeps_current_posture_when_declined() {
    let data = GameData::load().unwrap();
    let factions = crate::state::sim::founding_faction_ids(&data);
    let mut sim = SimState::new_campaign(&data, "preservers", 42, &factions);
    sim.contract = Some(contract::start_contract(
        data.contracts.get("founding_colony").unwrap(),
        &sim,
    ));
    sim.authority.pending_review = posture_review(&sim, CommandPosture::Expeditionary);
    assert!(complete_review(&mut sim, AuthorityChoice::KeepCurrent).is_none());
    assert!(sim.authority.pending_review.is_none());
}

#[test]
fn emergency_override_requires_a_real_shipwide_crisis() {
    let data = GameData::load().unwrap();
    let factions = crate::state::sim::founding_faction_ids(&data);
    let mut sim = SimState::new_campaign(&data, "preservers", 43, &factions);
    sim.contract = Some(contract::start_contract(
        data.contracts.get("founding_colony").unwrap(),
        &sim,
    ));
    sim.authority.captain_name = "Captain Vale".to_owned();
    sim.authority.captain_priority = CaptainPriority::Civic;
    sim.population.morale = 0.2;
    sim.authority.pending_review = posture_review(&sim, CommandPosture::Expeditionary);
    assert!(
        sim.authority
            .pending_review
            .as_ref()
            .unwrap()
            .emergency_allowed
    );
    assert_eq!(
        complete_review(&mut sim, AuthorityChoice::EmergencyOverride),
        Some(CommandPosture::Expeditionary)
    );
}
