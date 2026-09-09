use super::*;

#[test]
fn incoming_decisions_do_not_change_the_record_being_read() {
    let presentation = Presentation::default();
    assert_eq!(presentation.record_index(0), None);
    assert_eq!(presentation.record_index(3), Some(2));
    assert_eq!(presentation.record_index(4), Some(2));
    presentation.selected_record.set(Some(0));
    assert_eq!(presentation.record_index(5), Some(0));
}

#[test]
fn stale_selection_falls_back_to_the_latest_available_record() {
    let presentation = Presentation::default();
    presentation.selected_record.set(Some(8));
    assert_eq!(presentation.record_index(2), Some(1));
    assert_eq!(presentation.record_index(3), Some(1));
}
