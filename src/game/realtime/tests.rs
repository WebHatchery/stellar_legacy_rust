use super::*;

#[test]
fn pause_preserves_decision_time_and_resume_only_counts_new_time() {
    let elapsed = elapsed_decision(12.0, 60.0, true);
    assert_eq!(elapsed, 12.0);
    assert_eq!(elapsed_decision(elapsed, 1.0, false), 13.0);
}
