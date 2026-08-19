use super::*;

#[test]
fn known_effects_include_cultural_and_faction_consequences() {
    let data = crate::data::GameData::load().unwrap();
    let event = data.events.get("the_mascot_succession").unwrap();
    let election = event
        .outcomes
        .iter()
        .find(|outcome| outcome.id == "hold_an_election")
        .unwrap();
    let (text, _) = known_effects(election, None);

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
    let (wake_text, _) = known_effects(wake, None);
    assert!(wake_text.contains("objective -0.01%"));
    assert!(wake_text.contains("mercy reputation +2%"));

    let repair = voice
        .outcomes
        .iter()
        .find(|outcome| outcome.id == "mend_the_bank_and_return_her_to_cold")
        .unwrap();
    let (repair_text, _) = known_effects(repair, None);
    assert!(repair_text.contains("follow-up in 25y"));
}
