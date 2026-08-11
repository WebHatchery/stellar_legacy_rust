use super::*;
use crate::state::sim::founding_faction_ids;

fn campaign() -> (GameData, SimState) {
    let data = GameData::load().unwrap();
    let sim = SimState::new_campaign(&data, "preservers", 33, &founding_faction_ids(&data));
    (data, sim)
}

#[test]
fn advice_reacts_to_skill_faction_subsystem_reputation_and_duties() {
    let (data, mut sim) = campaign();
    let engineer = sim
        .crew
        .iter()
        .position(|c| c.archetype_id == "engineer")
        .unwrap();
    sim.crew[engineer].skill = 40;
    let novice = advice_for_post(&sim, &data, "engineer").unwrap().text;
    sim.crew[engineer].skill = 85;
    let expert = advice_for_post(&sim, &data, "engineer").unwrap().text;
    assert_ne!(novice, expert);
    sim.subsystems.get_mut("engineering_bay").unwrap().condition = 0.2;
    let strained = advice_for_post(&sim, &data, "engineer").unwrap().text;
    assert_ne!(expert, strained);
    sim.reputation.insert("resolve".to_owned(), 0.8);
    let reputed = advice_for_post(&sim, &data, "engineer").unwrap().text;
    assert_ne!(strained, reputed);
    assert!(reputed.contains(&sim.crew[engineer].name));
}

#[test]
fn vacant_posts_are_explicit_and_every_domain_selects_advisors() {
    let (data, mut sim) = campaign();
    sim.crew.retain(|c| c.archetype_id != "engineer");
    let vacant = advice_for_post(&sim, &data, "engineer").unwrap();
    assert!(vacant.officer_name.is_none());
    assert!(vacant.text.to_lowercase().contains("vacant"));
    let base = data.events.iter().next().unwrap().1.clone();
    for category in EventCategory::ALL {
        let mut event = base.clone();
        event.category = category;
        event.family.clear();
        event.advisor_posts.clear();
        assert_eq!(for_event(&sim, &data, &event).len(), 2);
    }
}
