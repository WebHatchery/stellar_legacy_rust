use super::*;

#[test]
fn guide_requires_the_named_action_and_successful_state() {
    let data = GameData::load().unwrap();
    let mut sim = SimState::new_campaign(
        &data,
        "preservers",
        5,
        &crate::state::sim::founding_faction_ids(&data),
    );
    assert!(!completed(0, &UiAction::NextTutorial, &sim));
    assert!(!completed(
        0,
        &UiAction::SelectScreen(Screen::ShipBuilder),
        &sim
    ));
    assert!(completed(0, &UiAction::SelectScreen(Screen::Drydock), &sim));
    assert!(!completed(
        1,
        &UiAction::SelectCharter("missing".into()),
        &sim
    ));
    assert!(!completed(5, &UiAction::Launch, &sim));
    sim.toggle_pause();
    assert!(completed(6, &UiAction::TogglePause, &sim));
    assert!(!completed(7, &UiAction::TogglePause, &sim));
    sim.toggle_pause();
    assert!(completed(7, &UiAction::TogglePause, &sim));
}
