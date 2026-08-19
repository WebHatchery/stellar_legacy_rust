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

#[test]
fn training_label_projects_the_capped_skill_gain() {
    assert_eq!(
        training_label(81, 100, 10, 400, 400),
        "TRAIN TO SK 91 · 400CR"
    );
    assert_eq!(
        training_label(96, 100, 10, 400, 400),
        "TRAIN TO SK 100 · 400CR"
    );
    assert_eq!(training_label(100, 100, 10, 400, 400), "MASTERED");
    assert_eq!(training_label(50, 100, 10, 400, 399), "NEED 400 CR");
}
