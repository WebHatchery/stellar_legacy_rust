use super::*;
use crate::data::GameData;
use crate::state::sim::{
    debrief::VoyageDebrief, HomecomingChoice, ObligationCreate, ObligationOperation,
    ObligationVisibility, SimState,
};

fn campaign(data: &GameData) -> SimState {
    let factions = crate::state::sim::founding_faction_ids(data);
    SimState::new_campaign(data, "preservers", 401, &factions)
}

fn with_report(mut sim: SimState, data: &GameData) -> SimState {
    let plan = build_plan(&sim, data);
    sim.debrief = Some(VoyageDebrief {
        recovery: Some(plan),
        ..Default::default()
    });
    sim
}

#[test]
fn due_promises_take_priority_in_the_recovery_brief() {
    let data = GameData::load().unwrap();
    let mut sim = campaign(&data);
    sim.apply_obligation_operation(&ObligationOperation::Create(ObligationCreate {
        authored_id: "return_duty".to_owned(),
        title: "Return the surveyors".to_owned(),
        source: "test".to_owned(),
        beneficiary: "Harrowlight Station".to_owned(),
        due_in_years: Some(1),
        resolution_event: String::new(),
        visibility: ObligationVisibility::Public,
        material_stakes: "A safe return".to_owned(),
        reputation_stakes: "Reliability".to_owned(),
    }));
    let plan = build_plan(&sim, &data);
    assert_eq!(plan.focus, HomecomingFocus::Obligation);
    assert_eq!(plan.target_label, "Return the surveyors");
}

#[test]
fn reconciliation_spends_the_bill_and_records_a_campaign_fact() {
    let data = GameData::load().unwrap();
    let mut sim = campaign(&data);
    sim.population.unity = 0.35;
    sim = with_report(sim, &data);
    let before = sim.resources.credits;
    let note = apply_choice(&mut sim, &data, HomecomingChoice::ReconcilePeople).unwrap();
    assert!(sim.resources.credits < before);
    assert!(note.contains("commons grant"));
    assert_eq!(sim.homecoming_recovery_history.len(), 1);
    assert_eq!(
        sim.debrief
            .as_ref()
            .unwrap()
            .recovery
            .as_ref()
            .unwrap()
            .choice,
        Some(HomecomingChoice::ReconcilePeople)
    );
}

#[test]
fn deferral_is_visible_and_has_a_small_social_cost() {
    let data = GameData::load().unwrap();
    let mut sim = campaign(&data);
    sim.population.unity = 0.35;
    sim = with_report(sim, &data);
    let before = sim.population.unity;
    apply_choice(&mut sim, &data, HomecomingChoice::Defer).unwrap();
    assert!(sim.population.unity < before);
    assert!(sim
        .homecoming_recovery_history
        .last()
        .unwrap()
        .note
        .contains("deferred"));
}
