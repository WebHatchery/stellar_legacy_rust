use super::*;

#[test]
fn recovery_choices_keep_a_visible_deferral() {
    assert_eq!(HomecomingChoice::ALL.len(), 4);
    assert_eq!(HomecomingChoice::Defer.label(), "DEFER RECOVERY");
    assert!(HomecomingChoice::Defer
        .description()
        .contains("unresolved wound"));
}

#[test]
fn recovery_briefs_default_to_unresolved() {
    let recovery = HomecomingRecovery::default();
    assert!(!recovery.resolved);
    assert!(recovery.choice.is_none());
}
