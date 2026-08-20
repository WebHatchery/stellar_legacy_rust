use super::*;

#[test]
fn priced_subsystem_actions_name_the_missing_store() {
    assert_eq!(priced_action_label("TRAIN", 600, 599, "cr"), "NEED 600cr");
    assert_eq!(priced_action_label("CUSTODY", 25, 24, "inf"), "NEED 25inf");
    assert_eq!(
        priced_action_label("ARCHIVE", 1000, 1500, "cr"),
        "ARCHIVE (1000cr)"
    );
}
