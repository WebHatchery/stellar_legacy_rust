use super::*;

#[test]
fn the_council_lesson_must_finish_before_the_guide_offers_completion() {
    let data = crate::data::GameData::load().unwrap();
    let count = data.config.tutorial.guided_steps.len();
    for step in 0..count {
        assert!(!guide(&data, step).unwrap().complete);
    }
    let final_lesson = guide(&data, count - 1).unwrap();
    assert!(final_lesson.tip.contains("Commit"));
    let finished = guide(&data, count).unwrap();
    assert!(finished.complete);
    assert!(finished.tip.contains("FINISH"));
}
