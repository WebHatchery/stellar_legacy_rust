use super::*;
use crate::data::GameData;
use crate::state::sim::SimState;

fn fresh(seed: u64) -> (GameData, SimState) {
    let data = GameData::load().unwrap();
    let sim = SimState::new_campaign(
        &data,
        "preservers",
        seed,
        &crate::state::sim::founding_faction_ids(&data),
    );
    (data, sim)
}

#[test]
fn campaign_starts_with_the_configured_posts_filled() {
    let (data, sim) = fresh(1);
    assert_eq!(sim.crew.len(), data.config.crew.starting_posts.len());
    for post in &data.config.crew.starting_posts {
        let holder = post_holder(&sim, post).expect("starting post must be filled");
        let archetype = data.crew_archetypes.iter().find(|a| &a.id == post).unwrap();
        assert!((archetype.skill_min..=archetype.skill_max).contains(&holder.skill));
    }
}

#[test]
fn recruit_fills_a_vacancy_and_charges_the_treasury() {
    let (data, mut sim) = fresh(2);
    let credits_before = sim.resources.credits;
    recruit(&mut sim, &data, "medic").expect("medic post starts vacant");
    assert!(post_holder(&sim, "medic").is_some());
    assert_eq!(
        sim.resources.credits,
        credits_before - data.config.crew.recruit_cost_credits
    );
    assert!(recruit(&mut sim, &data, "medic").is_err(), "post now held");
    assert!(recruit(&mut sim, &data, "warlock").is_err(), "unknown post");
}

#[test]
fn recruit_fails_when_broke() {
    let (data, mut sim) = fresh(3);
    sim.resources.credits = 0;
    assert!(recruit(&mut sim, &data, "medic").is_err());
    assert!(post_holder(&sim, "medic").is_none());
}

#[test]
fn train_raises_skill_and_caps_at_the_archetype_max() {
    let (data, mut sim) = fresh(4);
    let before = post_holder(&sim, "engineer").unwrap().skill;
    train(&mut sim, &data, "engineer").unwrap();
    let archetype = data
        .crew_archetypes
        .iter()
        .find(|a| a.id == "engineer")
        .unwrap();
    let after = post_holder(&sim, "engineer").unwrap().skill;
    assert_eq!(
        after,
        (before + data.config.crew.train_skill_gain).min(archetype.skill_max)
    );

    sim.crew
        .iter_mut()
        .find(|c| c.archetype_id == "engineer")
        .unwrap()
        .skill = archetype.skill_max;
    assert!(train(&mut sim, &data, "engineer").is_err(), "maxed out");
    assert!(train(&mut sim, &data, "medic").is_err(), "vacant post");
}

#[test]
fn crew_skills_multiply_production() {
    let (data, mut sim) = fresh(5);
    let mult = production_multipliers(&sim, &data);
    // Founding agronomist grants a food bonus (0.005/skill, skill >= 40).
    assert!(mult.food >= 1.2);
    // Nobody boosts minerals until a scientist is hired.
    assert!((mult.minerals - 1.0).abs() < f32::EPSILON);

    sim.resources.credits = 100_000;
    recruit(&mut sim, &data, "scientist").unwrap();
    assert!(production_multipliers(&sim, &data).minerals > 1.0);
}

#[test]
fn medic_reduces_famine_losses_and_security_steadies_unity() {
    let (data, mut sim) = fresh(6);
    assert_eq!(famine_loss_reduction(&sim, &data), 0.0);
    sim.resources.credits = 100_000;
    recruit(&mut sim, &data, "medic").unwrap();
    assert!(famine_loss_reduction(&sim, &data) > 0.0);

    recruit(&mut sim, &data, "security_chief").unwrap();
    sim.population.unity = 0.9;
    assert_eq!(unity_recovery(&sim, &data), 0.0, "no effect above ceiling");
    sim.population.unity = 0.3;
    assert!(unity_recovery(&sim, &data) > 0.0);
}

#[test]
fn officers_retire_when_aging_past_their_term() {
    let (data, mut sim) = fresh(7);
    let posts_before = sim.crew.len();
    // One year short of retirement: Founding Day tips them over, and they
    // stand down (a vacancy, not a death).
    sim.crew[0].age = data.config.crew.retirement_age;
    crate::simulation::mortality::annual_aging(&mut sim, &data);
    assert_eq!(
        sim.crew.len(),
        posts_before - 1,
        "the over-age officer retires"
    );
    for member in &sim.crew {
        assert!(member.age <= data.config.crew.retirement_age);
    }
}
