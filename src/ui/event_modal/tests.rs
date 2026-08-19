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
