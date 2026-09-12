use super::*;
use crate::data::GameData;
use crate::state::sim::SimState;

#[test]
fn a_vacated_seat_hands_off_to_the_best_eligible_heir() {
    let data = GameData::load().unwrap();
    let mut sim = SimState::new_campaign(
        &data,
        "preservers",
        1,
        &crate::state::sim::founding_faction_ids(&data),
    );

    // The leader dies: clear the flag, then install a successor.
    for member in &mut sim.dynasty.members {
        member.is_leader = false;
    }
    let year = sim.year();
    let (new_leader, extinct) = install_successor(&mut sim.dynasty, &data.config, year);
    assert!(!extinct, "the founding dynasty is not extinct");
    assert!(new_leader.is_some(), "a founding heir stands ready");
    let leader = sim.dynasty.leader().expect("a leader was installed");
    assert!(leader.age >= data.config.heir_min_age && leader.age <= data.config.heir_max_age);
}

#[test]
fn an_orderly_handoff_cannot_reinstall_the_outgoing_captain() {
    let data = GameData::load().unwrap();
    let mut sim = SimState::new_campaign(
        &data,
        "preservers",
        2,
        &crate::state::sim::founding_faction_ids(&data),
    );
    let outgoing = sim.dynasty.leader().unwrap().id;
    assert!(planned_heir(&sim.dynasty, &data.config).is_some());

    let year = sim.year();
    let (new_leader, extinct) = install_successor(&mut sim.dynasty, &data.config, year);

    assert!(!extinct);
    assert!(new_leader.is_some());
    assert_ne!(sim.dynasty.leader().unwrap().id, outgoing);
}

#[test]
fn designated_heir_takes_precedence_over_best_leadership() {
    let data = GameData::load().unwrap();
    let mut sim = SimState::new_campaign(
        &data,
        "preservers",
        12,
        &crate::state::sim::founding_faction_ids(&data),
    );

    // Designate the weakest eligible member instead of the strongest.
    let eligible: Vec<(u32, u32)> = sim
        .dynasty
        .members
        .iter()
        .filter(|m| {
            !m.is_leader && m.age >= data.config.heir_min_age && m.age <= data.config.heir_max_age
        })
        .map(|m| (m.id, m.leadership))
        .collect();
    let weakest = eligible
        .iter()
        .min_by_key(|(_, leadership)| *leadership)
        .map(|(id, _)| *id)
        .expect("founding dynasty has eligible members");
    sim.dynasty.designated_heir = Some(weakest);

    assert_eq!(
        planned_heir(&sim.dynasty, &data.config).map(|member| member.id),
        Some(weakest),
        "the visible orderly plan matches the handoff"
    );

    let year = sim.year();
    let (new_leader, _) = install_successor(&mut sim.dynasty, &data.config, year);
    assert!(new_leader.is_some());
    assert_eq!(
        sim.dynasty.leader().map(|l| l.id),
        Some(weakest),
        "the designated heir must inherit even with lower leadership"
    );
    assert!(sim.dynasty.designated_heir.is_none(), "consumed on use");
}

#[test]
fn an_ineligible_designate_does_not_hide_the_ready_heir() {
    let data = GameData::load().unwrap();
    let mut sim = SimState::new_campaign(
        &data,
        "preservers",
        13,
        &crate::state::sim::founding_faction_ids(&data),
    );
    let too_young_id = {
        let too_young = sim
            .dynasty
            .members
            .iter_mut()
            .find(|member| !member.is_leader)
            .unwrap();
        too_young.age = data.config.heir_min_age.saturating_sub(1);
        too_young.id
    };
    sim.dynasty.designated_heir = Some(too_young_id);

    let planned = planned_heir(&sim.dynasty, &data.config).expect("another heir is eligible");
    assert_ne!(planned.id, too_young_id);
    assert!(planned.age >= data.config.heir_min_age);
}

#[test]
fn marking_a_generation_advances_the_counter_and_reports_its_births() {
    let data = GameData::load().unwrap();
    let mut sim = SimState::new_campaign(
        &data,
        "adaptors",
        9,
        &crate::state::sim::founding_faction_ids(&data),
    );
    let before = sim.dynasty.members.len();
    // Births are yearly now; the generation mark only closes the ledger.
    sim.dynasty.births_this_generation = 4;

    let reported = process_generation(&mut sim, &data);

    assert_eq!(
        reported, 4,
        "the accumulated coming-of-age tally is reported"
    );
    assert_eq!(sim.dynasty.births_this_generation, 0, "the tally resets");
    assert_eq!(
        sim.dynasty.members.len(),
        before,
        "the mark itself adds no one"
    );
    assert_eq!(sim.dynasty.generation, 2);
}
