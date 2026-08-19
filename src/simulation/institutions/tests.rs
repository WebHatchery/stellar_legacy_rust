use super::*;
use crate::state::sim::founding_faction_ids;

fn campaign() -> (GameData, SimState) {
    let data = GameData::load().unwrap();
    let sim = SimState::new_campaign(&data, "preservers", 12, &founding_faction_ids(&data));
    (data, sim)
}

#[test]
fn an_apprentice_takes_the_post_and_materially_reduces_loss() {
    let (data, mut prepared) = campaign();
    let mut unprepared = prepared.clone();
    prepared.resources.credits = 10_000;
    designate_apprentice(&mut prepared, &data, "engineer").unwrap();
    let prepared_officer = prepared
        .crew
        .iter()
        .find(|c| c.archetype_id == "engineer")
        .unwrap()
        .clone();
    let unprepared_officer = unprepared
        .crew
        .iter()
        .find(|c| c.archetype_id == "engineer")
        .unwrap()
        .clone();
    let before = prepared.subsystems["engineering_bay"].knowledge;
    officer_departed(&mut prepared, &data, &prepared_officer);
    officer_departed(&mut unprepared, &data, &unprepared_officer);
    assert!(prepared
        .crew
        .iter()
        .any(|c| c.archetype_id == "engineer" && c.id != prepared_officer.id));
    assert!(
        prepared.subsystems["engineering_bay"].knowledge
            > unprepared.subsystems["engineering_bay"].knowledge
    );
    assert!(prepared.subsystems["engineering_bay"].knowledge < before);
}

#[test]
fn schools_persist_and_archives_preserve_named_methods() {
    let (data, mut sim) = campaign();
    sim.resources.credits = 10_000;
    establish_or_support_school(&mut sim, &data, "engineering_bay").unwrap();
    compile_archive(&mut sim, &data, "engineering_bay").unwrap();
    let officer = sim
        .crew
        .iter()
        .find(|c| c.archetype_id == "engineer")
        .unwrap()
        .clone();
    officer_departed(&mut sim, &data, &officer);
    assert_eq!(sim.subsystem_schools[0].subsystem_id, "engineering_bay");
    assert!(sim.procedure_archives[0]
        .preserved_experts
        .contains(&officer.name));
}

#[test]
fn school_support_slows_decay_and_the_benefit_ends_when_funding_lapses() {
    let (data, mut supported) = campaign();
    supported.resources.credits = 10_000;
    establish_or_support_school(&mut supported, &data, "medical_bay").unwrap();
    let mut lapsed = supported.clone();
    supported
        .subsystems
        .get_mut("medical_bay")
        .unwrap()
        .knowledge = 0.8;
    lapsed.subsystems.get_mut("medical_bay").unwrap().knowledge = 0.8;
    lapsed.month_clock = (data.config.crew.school_support_years + 1) * 12;
    crate::simulation::subsystems::transmit_knowledge(&mut supported, &data);
    crate::simulation::subsystems::transmit_knowledge(&mut lapsed, &data);
    assert!(
        supported.subsystems["medical_bay"].knowledge > lapsed.subsystems["medical_bay"].knowledge
    );
    let old_until = supported.subsystem_schools[0].supported_until_year;
    establish_or_support_school(&mut supported, &data, "medical_bay").unwrap();
    assert_eq!(
        supported.subsystem_schools[0].supported_until_year,
        old_until + data.config.crew.school_support_years,
        "every upkeep payment buys one complete support term"
    );
}

#[test]
fn council_can_grant_custody_to_a_non_dominant_people() {
    let (data, mut sim) = campaign();
    sim.resources.credits = 10_000;
    sim.resources.influence = 100;
    establish_or_support_school(&mut sim, &data, "engineering_bay").unwrap();
    compile_archive(&mut sim, &data, "engineering_bay").unwrap();
    let dominant = sim.dominant_faction_id().unwrap().to_owned();
    let chosen = sim
        .factions
        .iter()
        .find(|faction| faction.is_aboard() && faction.faction_id != dominant)
        .unwrap()
        .faction_id
        .clone();
    let approval_before = sim
        .factions
        .iter()
        .find(|faction| faction.faction_id == chosen)
        .unwrap()
        .approval;

    grant_custodianship(&mut sim, &data, "engineering_bay", &chosen).unwrap();

    assert_eq!(
        sim.subsystem_schools[0].custodian_faction_id.as_deref(),
        Some(chosen.as_str())
    );
    assert!(
        sim.factions
            .iter()
            .find(|faction| faction.faction_id == chosen)
            .unwrap()
            .approval
            > approval_before
    );
}

#[test]
fn a_people_not_aboard_cannot_receive_custody() {
    let (data, mut sim) = campaign();
    sim.resources.credits = 10_000;
    sim.resources.influence = 100;
    establish_or_support_school(&mut sim, &data, "engineering_bay").unwrap();
    compile_archive(&mut sim, &data, "engineering_bay").unwrap();
    let influence_before = sim.resources.influence;

    assert!(grant_custodianship(&mut sim, &data, "engineering_bay", "verdant_kin").is_err());
    assert_eq!(sim.resources.influence, influence_before);
    assert!(sim.subsystem_schools[0].custodian_faction_id.is_none());
}

#[test]
fn apprentices_succeed_on_both_retirement_and_death_and_survive_a_save() {
    let (data, mut retirement) = campaign();
    retirement.resources.credits = 10_000;
    designate_apprentice(&mut retirement, &data, "engineer").unwrap();
    retirement
        .crew
        .iter_mut()
        .find(|c| c.archetype_id == "engineer")
        .unwrap()
        .age = data.config.crew.retirement_age;
    crate::simulation::mortality::annual_aging(&mut retirement, &data);
    let successor = retirement
        .crew
        .iter()
        .find(|c| c.archetype_id == "engineer")
        .expect("apprentice takes the retired officer's post");
    assert_eq!(successor.age, 25);
    let saved = serde_json::to_string(&retirement).unwrap();
    let loaded: SimState = serde_json::from_str(&saved).unwrap();
    assert_eq!(
        loaded
            .crew
            .iter()
            .find(|c| c.archetype_id == "engineer")
            .unwrap()
            .name,
        successor.name
    );

    let (data, mut death) = campaign();
    death.resources.credits = 10_000;
    designate_apprentice(&mut death, &data, "engineer").unwrap();
    death
        .crew
        .iter_mut()
        .find(|c| c.archetype_id == "engineer")
        .unwrap()
        .age = data.config.member_max_age;
    let mut report = crate::simulation::tick::TickReport::default();
    crate::simulation::mortality::monthly_tick(&mut death, &data, &mut report);
    assert!(death
        .crew
        .iter()
        .any(|c| c.archetype_id == "engineer" && c.age == 25));
}
