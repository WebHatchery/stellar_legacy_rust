use super::*;
use crate::data::GameData;

fn sim() -> SimState {
    let data = GameData::load().unwrap();
    SimState::new_campaign(
        &data,
        "preservers",
        91,
        &super::super::founding_faction_ids(&data),
    )
}

fn create() -> ObligationOperation {
    ObligationOperation::Create(ObligationCreate {
        authored_id: "sanctuary".to_owned(),
        title: "The Open Berth".to_owned(),
        source: "event:refugee_signal".to_owned(),
        beneficiary: "The Kestrel refugees".to_owned(),
        due_in_years: Some(20),
        resolution_event: "sanctuary_berths_due".to_owned(),
        visibility: ObligationVisibility::Public,
        material_stakes: "500 food".to_owned(),
        reputation_stakes: "Mercy and refugee trust".to_owned(),
    })
}

#[test]
fn creation_is_stable_serialized_state_and_old_saves_default_empty() {
    let mut sim = sim();
    sim.apply_obligation_operation(&create());
    assert_eq!(sim.obligations[0].id, "obligation-000001");
    let value = serde_json::to_value(&sim).unwrap();
    let back: SimState = serde_json::from_value(value.clone()).unwrap();
    assert_eq!(back.obligations, sim.obligations);
    let mut old = value;
    old.as_object_mut().unwrap().remove("obligations");
    old.as_object_mut().unwrap().remove("next_obligation_id");
    let migrated: SimState = serde_json::from_value(old).unwrap();
    assert!(migrated.obligations.is_empty());
}

#[test]
fn due_dates_and_all_resolutions_are_deterministic() {
    let mut sim = sim();
    sim.apply_obligation_operation(&create());
    sim.month_clock = 20 * 12;
    assert_eq!(sim.due_obligations()[0].authored_id, "sanctuary");
    sim.apply_obligation_operation(&ObligationOperation::Renegotiate {
        authored_id: "sanctuary".to_owned(),
        due_in_years: Some(5),
        note: "A smaller first convoy was accepted.".to_owned(),
    });
    assert_eq!(sim.obligations[0].due_year, Some(25));
    sim.apply_obligation_operation(&ObligationOperation::Fulfil {
        authored_id: "sanctuary".to_owned(),
        note: "The promised berths were opened.".to_owned(),
    });
    assert_eq!(sim.obligations[0].status, ObligationStatus::Fulfilled);

    sim.apply_obligation_operation(&create());
    sim.apply_obligation_operation(&ObligationOperation::Default {
        authored_id: "sanctuary".to_owned(),
        note: "The doors stayed shut.".to_owned(),
    });
    assert_eq!(sim.obligations[1].status, ObligationStatus::Defaulted);
}

#[test]
fn succession_transfers_owner_and_records_inheritance() {
    let mut sim = sim();
    sim.apply_obligation_operation(&create());
    let original = sim.obligations[0].responsible.clone();
    let heir = sim
        .dynasty
        .members
        .iter()
        .find(|m| !m.is_leader)
        .unwrap()
        .name
        .clone();
    for member in &mut sim.dynasty.members {
        member.is_leader = member.name == heir;
    }
    sim.inherit_obligations();
    assert_ne!(sim.obligations[0].responsible, original);
    assert_eq!(sim.obligations[0].successions_crossed, 1);
    assert_eq!(sim.obligations[0].history.len(), 2);
}
