use super::*;
use crate::data::GameData;
use crate::state::sim::SimState;

#[test]
fn failure_risk_matches_gdd_formula() {
    let data = GameData::load().unwrap();
    let mut sim = SimState::new_campaign(
        &data,
        "preservers",
        1,
        &crate::state::sim::founding_faction_ids(&data),
    );

    let calm = failure_risk(&sim, &data.config);
    assert_eq!(calm.total, 0);
    assert!(!calm.at_risk);

    sim.population.cultural_drift = 0.8; // +30
    sim.population.unity = 0.2; // +25
    sim.legacy.tradition_points = 10; // +35
    let dire = failure_risk(&sim, &data.config);
    assert_eq!(dire.total, 90);
    assert!(dire.at_risk);
    assert_eq!(dire.factors.len(), 3);
}

#[test]
fn legacy_specific_counters_only_threaten_their_own_legacy() {
    let data = GameData::load().unwrap();
    let mut sim = SimState::new_campaign(
        &data,
        "wanderers",
        2,
        &crate::state::sim::founding_faction_ids(&data),
    );
    // Adaptors' counters must not add risk to a Wanderer campaign.
    sim.legacy.body_horror_events = 10;
    sim.legacy.existential_dread = 1.0;
    assert_eq!(failure_risk(&sim, &data.config).total, 0);

    sim.legacy.piracy_reputation = 0.9;
    let risky = failure_risk(&sim, &data.config);
    assert_eq!(risky.total, data.config.failure_risk.piracy_points);
}

fn plain_option(chance: f32) -> DilemmaOption {
    DilemmaOption {
        id: "opt".into(),
        label: "opt".into(),
        success_chance: chance,
        success: DilemmaEffect::default(),
        failure: DilemmaEffect::default(),
        dominant_faction: String::new(),
        dominant_faction_odds: 0.0,
    }
}

#[test]
fn combat_lifts_wanderer_dilemma_odds_only() {
    let data = GameData::load().unwrap();
    // A Wanderer ship with a weapon installed beats its base odds.
    let mut wanderer = SimState::new_campaign(
        &data,
        "wanderers",
        4,
        &crate::state::sim::founding_faction_ids(&data),
    );
    wanderer.ship.weapon = Some("mass_driver".to_owned()); // combat 5
    let lifted = dilemma_odds(&wanderer, &data, &plain_option(0.65));
    assert!(lifted > 0.65, "combat should raise Wanderer odds: {lifted}");
    assert!(lifted <= data.config.ship.dilemma_odds_cap);

    // The same weapon does nothing for another legacy's dilemmas.
    let mut preserver = SimState::new_campaign(
        &data,
        "preservers",
        4,
        &crate::state::sim::founding_faction_ids(&data),
    );
    preserver.ship.weapon = Some("mass_driver".to_owned());
    assert_eq!(dilemma_odds(&preserver, &data, &plain_option(0.65)), 0.65);
}

#[test]
fn the_dominant_faction_backs_or_hinders_a_dilemma_gamble() {
    // Content-depth factions round 10: who runs the ship shifts the odds of a
    // defining gamble. A backed option reads higher only while its faction is
    // dominant; a hindered one reads lower.
    let data = GameData::load().unwrap();
    let mut sim = SimState::new_campaign(
        &data,
        "adaptors",
        4,
        &crate::state::sim::founding_faction_ids(&data),
    );
    let mut backed = plain_option(0.6);
    backed.dominant_faction = "ascension_circle".into();
    backed.dominant_faction_odds = 0.15;

    // Make the Ascension the sole (hence dominant) people: odds lift.
    sim.factions = vec![crate::state::sim::factions::FactionState {
        faction_id: "ascension_circle".into(),
        members: 1000,
        status: crate::state::sim::factions::FactionStatus::Aboard,
        approval: 0.5,
        mood_band: 0,
    }];
    let with = dilemma_odds(&sim, &data, &backed);
    assert!(with > 0.6, "the augmented back the augmentation: {with}");

    // A different dominant people: no lift.
    sim.factions[0].faction_id = "first_flame".into();
    assert_eq!(
        dilemma_odds(&sim, &data, &backed),
        0.6,
        "another people neither backs nor hinders it"
    );
}

#[test]
fn resolve_dilemma_applies_a_branch_and_updates_counters() {
    let data = GameData::load().unwrap();
    let mut sim = SimState::new_campaign(
        &data,
        "preservers",
        3,
        &crate::state::sim::founding_faction_ids(&data),
    );
    sim.pending_dilemma = Some(PendingDilemma {
        dilemma_id: "archive_purge".to_owned(),
        rolled_month_clock: sim.month_clock,
    });

    let tradition_before = sim.legacy.tradition_points;
    let log_len = sim.log.len();
    let text = resolve_dilemma(&mut sim, &data, 0).expect("dilemma must resolve");
    assert!(sim.pending_dilemma.is_none());
    assert_eq!(sim.log.len(), log_len + 1);
    assert!(!text.is_empty());
    // Option 0 ("protect the archive"): success grants +10 tradition,
    // failure costs food/morale but leaves tradition alone.
    let succeeded = sim.legacy.tradition_points != tradition_before;
    if succeeded {
        assert_eq!(sim.legacy.tradition_points, tradition_before + 10);
    }
}

#[test]
fn dilemma_resolution_is_deterministic_per_seed() {
    let data = GameData::load().unwrap();
    let mut runs = Vec::new();
    for _ in 0..2 {
        let mut sim = SimState::new_campaign(
            &data,
            "adaptors",
            99,
            &crate::state::sim::founding_faction_ids(&data),
        );
        sim.pending_dilemma = Some(PendingDilemma {
            dilemma_id: "gene_clinic".to_owned(),
            rolled_month_clock: 0,
        });
        resolve_dilemma(&mut sim, &data, 0);
        runs.push((
            sim.legacy.body_horror_events,
            sim.population.adaptation.to_bits(),
        ));
    }
    assert_eq!(runs[0], runs[1]);
}
