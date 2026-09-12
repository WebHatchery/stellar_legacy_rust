use super::*;
use crate::data::GameData;

#[test]
fn death_chance_rises_with_age_and_is_certain_at_the_cap() {
    let data = GameData::load().unwrap();
    let cfg = &data.config.mortality;
    let max = data.config.member_max_age;
    let young = monthly_death_chance(20, cfg, max);
    let onset = monthly_death_chance(cfg.onset_age, cfg, max);
    let old = monthly_death_chance(cfg.onset_age + cfg.doubling_years as u32, cfg, max);
    assert!(young < onset, "the young are far safer than the old");
    assert!(old > onset, "risk climbs past the onset age");
    assert!(
        onset >= cfg.monthly_base_chance,
        "onset age carries the base risk"
    );
    assert_eq!(
        monthly_death_chance(max, cfg, max),
        1.0,
        "certain at the cap"
    );
    assert_eq!(monthly_death_chance(max + 5, cfg, max), 1.0);
}

#[test]
fn founding_day_ages_everyone_by_a_year() {
    let data = GameData::load().unwrap();
    let mut sim = crate::state::sim::SimState::new_campaign(
        &data,
        "preservers",
        1,
        &crate::state::sim::founding_faction_ids(&data),
    );
    let dyn_before: Vec<u32> = sim.dynasty.members.iter().map(|m| m.age).collect();
    let crew_before: Vec<u32> = sim.crew.iter().map(|c| c.age).collect();
    annual_aging(&mut sim, &data);
    for (member, before) in sim.dynasty.members.iter().zip(&dyn_before) {
        assert_eq!(member.age, before + 1);
    }
    for (officer, before) in sim.crew.iter().zip(&crew_before) {
        assert_eq!(officer.age, before + 1);
    }
}

#[test]
fn sustained_plenty_fills_the_cradles_faster_than_lean() {
    // Content-depth provisioning round 19: a ship long in plenty lifts the yearly
    // renewal — the positive pole of the chronic-hunger death toll.
    let mut data = GameData::load().unwrap();
    // Decisive plenty: fed, the boosted chance clears 1.0 and every open cradle
    // fills; lean, only the base chance applies.
    data.config.mortality.annual_birth_chance = 0.3;
    data.config.sustained_plenty_birth_bonus = 4.0;
    let base = crate::state::sim::SimState::new_campaign(
        &data,
        "preservers",
        7,
        &crate::state::sim::founding_faction_ids(&data),
    );
    let target = data.config.mortality.dynasty_target_size as usize;

    // Thin the line to two (the minimum that can renew) with a sound home.
    let mut fed = base.clone();
    fed.dynasty.members.truncate(2);
    if let Some(h) = fed.subsystems.get_mut("life_support_habitat") {
        h.condition = 1.0;
    }
    let mut lean = fed.clone();

    fed.fat_food_years = data.config.chronic_hunger_years.max(1);
    lean.fat_food_years = 0;
    annual_aging(&mut fed, &data);
    annual_aging(&mut lean, &data);

    assert_eq!(
        fed.dynasty.members.len(),
        target,
        "sustained plenty fills every open cradle"
    );
    assert!(
        fed.dynasty.members.len() > lean.dynasty.members.len(),
        "and faster than a lean ship raises its young"
    );
}

#[test]
fn a_leader_dying_in_office_is_flagged_for_the_succession_beat() {
    // The succession beat (campaign-skeleton round 18) keys on a sitting
    // leader falling — so the monthly tick must flag it, and only for a death
    // in office, not a routine survival.
    let data = GameData::load().unwrap();
    let mut sim = crate::state::sim::SimState::new_campaign(
        &data,
        "preservers",
        4,
        &crate::state::sim::founding_faction_ids(&data),
    );

    // No one due to die: the flag stays down.
    let mut report = crate::simulation::tick::TickReport::default();
    monthly_tick(&mut sim, &data, &mut report);
    assert!(!report.leader_died, "no death in office means no flag");

    // Force the sitting leader to certain death (age at the cap), leaving an
    // eligible heir among the founders.
    let max_age = data.config.member_max_age;
    for member in &mut sim.dynasty.members {
        if member.is_leader {
            member.age = max_age;
        }
    }
    let mut report = crate::simulation::tick::TickReport::default();
    monthly_tick(&mut sim, &data, &mut report);
    assert!(
        report.leader_died,
        "the leader's death in office is flagged"
    );
    assert!(!report.dynasty_extinct, "an heir carries the line on");
    assert!(sim.dynasty.leader().is_some(), "the seat is filled at once");
}

#[test]
fn a_long_hunger_raises_the_death_roll() {
    // Content-depth provisioning round 18: a sustained lean past
    // `chronic_hunger_years` adds a monthly death toll on top of the age curve.
    let mut data = GameData::load().unwrap();
    // Isolate the hunger toll: no accident floor, a decisive hunger bonus.
    data.config.mortality.monthly_accident_chance = 0.0;
    data.config.chronic_hunger_death_bonus = 1.0;
    let mut sim = crate::state::sim::SimState::new_campaign(
        &data,
        "preservers",
        3,
        &crate::state::sim::founding_faction_ids(&data),
    );
    // Young members (below the age onset) so any death is the hunger's doing,
    // and a wrecked bay so the toll lands in full.
    for member in &mut sim.dynasty.members {
        member.age = 25;
    }
    if let Some(bay) = sim.subsystems.get_mut("medical_bay") {
        bay.condition = 0.0;
    }
    let founders = sim.dynasty.members.len();

    // Well-fed: the young are safe.
    sim.lean_food_years = 0;
    let mut report = crate::simulation::tick::TickReport::default();
    monthly_tick(&mut sim, &data, &mut report);
    assert_eq!(
        sim.dynasty.members.len(),
        founders,
        "a fed ship's young do not die of hunger"
    );

    // A hunger years past the threshold: it takes even the young.
    sim.lean_food_years = data.config.chronic_hunger_years.max(1);
    let mut report = crate::simulation::tick::TickReport::default();
    monthly_tick(&mut sim, &data, &mut report);
    assert!(
        sim.dynasty.members.len() < founders,
        "a long hunger thins the roster on top of the age curve"
    );
}
