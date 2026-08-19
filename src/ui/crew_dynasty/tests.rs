use super::*;

#[test]
fn priced_crew_actions_explain_when_credits_are_missing() {
    assert_eq!(priced_action_label("TRAIN", 400, 399), "NEED 400 CR");
    assert_eq!(priced_action_label("TRAIN", 400, 400), "TRAIN (400 CR)");
    assert_eq!(
        priced_action_label("APPRENTICE", 600, 12_000),
        "APPRENTICE (600 CR)"
    );
}
