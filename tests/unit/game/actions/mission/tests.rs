use super::*;
use crate::{data::GameData, state::Screen};

#[test]
fn leaving_the_return_review_releases_its_clock_hold_without_changing_speed() {
    let data = GameData::load().unwrap();
    let mut sim = SimState::new_campaign(
        &data,
        "preservers",
        5,
        &crate::state::sim::founding_faction_ids(&data),
    );
    assert!(!review_after_action(
        false,
        &UiAction::ReviewReturnHome,
        &sim
    ));
    sim.contract = Some(contract::start_contract(
        data.contracts.get("deep_vein_survey").unwrap(),
        &sim,
    ));
    let speed = sim.speed;
    let open = review_after_action(false, &UiAction::ReviewReturnHome, &sim);
    assert!(open);
    for exit in [
        UiAction::DismissReturnHome,
        UiAction::SelectScreen(Screen::Dashboard),
        UiAction::SelectScreen(Screen::Contract),
        UiAction::AbortMission,
        UiAction::ToMenu,
    ] {
        assert!(!review_after_action(open, &exit, &sim));
        assert_eq!(sim.speed, speed);
    }
    assert!(contract::jump_to_return(&mut sim));
    assert!(!review_after_action(
        false,
        &UiAction::ReviewReturnHome,
        &sim
    ));
}
