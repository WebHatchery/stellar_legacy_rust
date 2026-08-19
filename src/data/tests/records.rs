//! Authored competing accounts remain complete and genuinely subordinate to
//! the authoritative outcome log.

use super::*;

#[test]
fn interpreted_decisions_have_a_fact_and_complete_perspectives() {
    let data = GameData::load().unwrap();
    let mut interpreted = 0;
    for (event_id, event) in data.events.iter() {
        for outcome in &event.outcomes {
            let Some(record) = &outcome.record else {
                continue;
            };
            interpreted += 1;
            assert!(
                event.requires_decision,
                "{event_id}/{} interprets an incident that was not a council decision",
                outcome.id
            );
            assert!(
                !outcome.log.trim().is_empty(),
                "{event_id}/{} needs an authoritative log fact",
                outcome.id
            );
            assert!(
                !record.official.trim().is_empty(),
                "{event_id}/{} has no official account",
                outcome.id
            );
            assert!(
                !record.dynasty.trim().is_empty(),
                "{event_id}/{} has no dynasty account",
                outcome.id
            );
            assert!(
                !record.affected.is_empty(),
                "{event_id}/{} has no affected-people account",
                outcome.id
            );
            for account in &record.affected {
                assert!(!account.people.trim().is_empty());
                assert!(!account.account.trim().is_empty());
            }
        }
    }
    assert!(
        interpreted >= 10,
        "the first Command Archive pass should ship a meaningful authored set"
    );
}
