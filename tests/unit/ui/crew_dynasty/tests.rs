use super::*;

#[test]
fn priced_crew_actions_explain_when_credits_are_missing() {
    assert_eq!(
        priced_action_label("TRAIN", 400, 399),
        "TRAIN · NEED 400 CR"
    );
    assert_eq!(priced_action_label("TRAIN", 400, 400), "TRAIN · 400 CR");
    assert_eq!(
        priced_action_label("APPRENTICE", 600, 12_000),
        "APPRENTICE · 600 CR"
    );
}

#[test]
fn training_label_names_the_price_or_the_mastery() {
    assert_eq!(training_label(81, 100, 400, 400), "TRAIN · 400 CR");
    assert_eq!(training_label(100, 100, 400, 400), "MASTERED");
    assert_eq!(training_label(50, 100, 400, 399), "TRAIN · NEED 400 CR");
}
