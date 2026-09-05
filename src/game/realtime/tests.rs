use super::*;

#[test]
fn pause_preserves_decision_time_and_resume_only_counts_new_time() {
    let elapsed = elapsed_decision(12.0, 60.0, true);
    assert_eq!(elapsed, 12.0);
    assert_eq!(elapsed_decision(elapsed, 1.0, false), 13.0);
}

#[test]
fn repeated_event_occurrences_get_fresh_countdown_keys() {
    let data = GameData::load().unwrap();
    let mut sim = SimState::new_campaign(
        &data,
        "preservers",
        81,
        &crate::state::sim::founding_faction_ids(&data),
    );
    sim.pending_event = Some(crate::state::sim::PendingEvent {
        template_id: "repeat".into(),
        rolled_month_clock: 12,
    });
    let first = current_decision_key(&sim);
    sim.pending_event.as_mut().unwrap().rolled_month_clock = 24;
    assert_ne!(first, current_decision_key(&sim));
}
