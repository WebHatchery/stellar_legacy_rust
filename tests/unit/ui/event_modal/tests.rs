use super::*;

#[test]
fn fuel_preview_matches_resolution_at_tank_limits_and_with_protection() {
    let data = crate::data::GameData::load().unwrap();
    let factions = crate::state::sim::founding_faction_ids(&data);
    let mut sim = crate::state::SimState::new_campaign(&data, "preservers", 42, &factions);
    let event = data.events.get("relativity_pocket").unwrap();
    for fuel in [0.0, 0.5, 1.0] {
        sim.ship.fuel = fuel;
        for index in 0..event.outcomes.len() {
            let preview =
                crate::simulation::event_resolver::outcome_fuel_preview(&sim, &data, event, index)
                    .unwrap();
            let mut resolved = sim.clone();
            crate::simulation::event_resolver::apply_outcome(&mut resolved, &data, event, index);
            assert_eq!(preview, (fuel, resolved.ship.fuel));
            let (text, _) = known_effects(&event.outcomes[index], None, Some(preview));
            assert!(text.contains("fuel tank"));
            assert!(!text.contains("20000%"));
            assert!(!text.contains("18000%"));
        }
    }
    let def = data
        .subsystems
        .iter()
        .map(|(_, def)| def)
        .find(|def| {
            !def.buffers_family.is_empty()
                && def
                    .tier_stats(1)
                    .is_some_and(|tier| tier.severity_reduction > 0.0)
        })
        .unwrap();
    let state = sim.subsystems.get_mut(&def.id).unwrap();
    state.tier = 1;
    state.condition = 1.0;
    sim.ship.fuel = 0.5;
    let mut protected = event.clone();
    protected.family = def.buffers_family.clone();
    protected.outcomes[0].ship_delta.fuel = -0.2;
    let preview =
        crate::simulation::event_resolver::outcome_fuel_preview(&sim, &data, &protected, 0)
            .unwrap();
    assert!(preview.1 > 0.3);
    crate::simulation::event_resolver::apply_outcome(&mut sim, &data, &protected, 0);
    assert_eq!(preview.1, sim.ship.fuel);
}

#[test]
fn known_effects_include_cultural_and_faction_consequences() {
    let data = crate::data::GameData::load().unwrap();
    let event = data.events.get("the_mascot_succession").unwrap();
    let election = event
        .outcomes
        .iter()
        .find(|outcome| outcome.id == "hold_an_election")
        .unwrap();
    let (text, _) = known_effects(election, None, None);

    assert!(text.contains("adapt +2%"));
    assert!(text.contains("meridian accord approval +4%"));
    assert!(text.contains("future consequence"));
}

#[test]
fn known_effects_keep_small_objective_changes_and_timed_payoffs_visible() {
    let data = crate::data::GameData::load().unwrap();
    let voice = data.events.get("the_voice_under_glass").unwrap();
    let wake = voice
        .outcomes
        .iter()
        .find(|outcome| outcome.id == "wake_her_into_the_crew")
        .unwrap();
    let (wake_text, _) = known_effects(wake, None, None);
    assert!(wake_text.contains("objective -0.01%"));
    assert!(wake_text.contains("mercy reputation +2%"));

    let repair = voice
        .outcomes
        .iter()
        .find(|outcome| outcome.id == "mend_the_bank_and_return_her_to_cold")
        .unwrap();
    let (repair_text, _) = known_effects(repair, None, None);
    assert!(repair_text.contains("follow-up in 25y"));
}

#[test]
fn known_effects_name_the_custodians_change_as_ai_empathy() {
    let data = crate::data::GameData::load().unwrap();
    let event = data.events.get("the_spare_calculation").unwrap();
    let humane = event
        .outcomes
        .iter()
        .find(|outcome| outcome.id == "teach_it_the_crew")
        .unwrap();
    let (text, _) = known_effects(humane, None, None);
    assert!(text.contains("AI empathy +9%"));
    assert!(!text.contains("custodian empathy reputation"));
}
