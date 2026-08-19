use super::*;
use crate::data::GameData;

#[test]
fn new_campaign_is_deterministic_for_same_seed() {
    let data = GameData::load().unwrap();
    let a = SimState::new_campaign(
        &data,
        "preservers",
        42,
        &crate::state::sim::founding_faction_ids(&data),
    );
    let b = SimState::new_campaign(
        &data,
        "preservers",
        42,
        &crate::state::sim::founding_faction_ids(&data),
    );
    let names_a: Vec<_> = a.dynasty.members.iter().map(|m| m.name.clone()).collect();
    let names_b: Vec<_> = b.dynasty.members.iter().map(|m| m.name.clone()).collect();
    assert_eq!(names_a, names_b);
    assert_eq!(a.dynasty.leader().unwrap().age, 45);
}

#[test]
fn resource_pool_clamps_at_zero_and_checks_affordability() {
    let mut pool = ResourcePool {
        credits: 100,
        ..Default::default()
    };
    let cost = crate::data::ResourceDelta {
        credits: -150,
        ..Default::default()
    };
    assert!(!pool.can_afford(&cost));
    pool.apply(&cost);
    assert_eq!(pool.credits, 0);
}

#[test]
fn sim_state_round_trips_through_serde() {
    let data = GameData::load().unwrap();
    let sim = SimState::new_campaign(
        &data,
        "wanderers",
        7,
        &crate::state::sim::founding_faction_ids(&data),
    );
    let json = serde_json::to_string(&sim).unwrap();
    let back: SimState = serde_json::from_str(&json).unwrap();
    assert_eq!(back.dynasty.members.len(), sim.dynasty.members.len());
    assert_eq!(back.legacy.legacy_id, "wanderers");
    // The reign roster is part of the saved state — a campaign that loses it
    // loses every captaincy before the sitting one.
    assert_eq!(back.dynasty.reigns.len(), sim.dynasty.reigns.len());
    assert_eq!(back.dynasty.reigns[0].name, sim.dynasty.reigns[0].name);
}

#[test]
fn an_unread_homecoming_survives_a_save_and_load() {
    // The debrief is a full-screen takeover the player dismisses by hand.
    // Quitting while it is up and loading back must return to it — a
    // voyage's only summary should not be lost to closing the window.
    let data = GameData::load().unwrap();
    let mut sim = SimState::new_campaign(
        &data,
        "preservers",
        11,
        &crate::state::sim::founding_faction_ids(&data),
    );
    sim.debrief = Some(debrief::VoyageDebrief {
        contract_name: "Salvage Writ: The Long Tow".to_owned(),
        outcome: "Partial".to_owned(),
        score: 0.62,
        duration_years: 450,
        highlights: vec![debrief::VoyageHighlight {
            year: 22,
            month: 7,
            kind: debrief::HighlightKind::Milestone,
            text: "Departure burn complete".to_owned(),
        }],
        commanders: vec![sim.dynasty.reigns[0].clone()],
        ..Default::default()
    });

    let json = serde_json::to_string(&sim).unwrap();
    let back: SimState = serde_json::from_str(&json).unwrap();
    let report = back.debrief.expect("the unread report came back");
    assert_eq!(report.contract_name, "Salvage Writ: The Long Tow");
    assert_eq!(report.outcome, "Partial");
    assert_eq!(report.duration_years, 450);
    assert_eq!(report.highlights.len(), 1);
    assert_eq!(report.highlights[0].kind, debrief::HighlightKind::Milestone);
    assert_eq!(report.commanders.len(), 1);

    // …and a filed report stays filed.
    sim.debrief = None;
    let json = serde_json::to_string(&sim).unwrap();
    let back: SimState = serde_json::from_str(&json).unwrap();
    assert!(back.debrief.is_none());
}

#[test]
fn active_contract_forecasts_follow_mission_month_boundaries() {
    let data = GameData::load().unwrap();
    let sim = SimState::new_campaign(
        &data,
        "preservers",
        13,
        &crate::state::sim::founding_faction_ids(&data),
    );
    let template = data.contracts.get("founding_colony").unwrap();
    let mut active = crate::simulation::contract::start_contract(template, &sim);

    assert_eq!(active.mission_months_remaining(), active.total_months());
    assert_eq!(active.next_phase_eta(), Some((active.phases[0].kind, 1)));

    active.months_elapsed = 1;
    (active.phase_index, active.phase) = active.phase_at(active.months_elapsed);
    assert_eq!(
        active.next_phase_eta(),
        Some((active.phases[1].kind, active.phases[0].years * 12))
    );

    let (milestone, eta) = active.next_milestone_eta().unwrap();
    let target_month = (active.total_months() as f32 * milestone.progress_threshold).ceil() as u32;
    assert_eq!(eta, target_month - active.months_elapsed);

    active.months_elapsed = active.total_months() + 6;
    assert_eq!(active.mission_months_remaining(), 0);
}
