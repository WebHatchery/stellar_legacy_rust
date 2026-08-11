use super::*;
use crate::data::GameData;
use crate::simulation::contract::start_contract;
use crate::state::sim::{founding_faction_ids, SimState};

#[test]
fn beats_are_deterministic_and_one_per_twenty_years() {
    let data = GameData::load().unwrap();
    let picks = founding_faction_ids(&data);
    let template = data.contracts.get("deep_vein_survey").unwrap().clone();

    let cfg = &data.config.campaign_skeleton;
    let schedule = || {
        let mut sim = SimState::new_campaign(&data, "preservers", 99, &picks);
        let contract = start_contract(&template, &sim);
        generate_beats(&mut sim.rng, &contract, cfg)
    };
    let a = schedule();
    let b = schedule();

    // A 340-year charter spans 17 twenty-year windows → 17 beats.
    assert_eq!(
        a.len(),
        17,
        "one beat per full 20 years of a 340-yr charter"
    );
    let flat = |v: &[CampaignBeat]| -> Vec<(u32, String)> {
        v.iter()
            .map(|x| (x.month_clock, x.family.clone()))
            .collect()
    };
    assert_eq!(flat(&a), flat(&b), "same seed replays the same schedule");

    // Beats are ordered, skip the opening window, and only draw families the
    // config declares (phase pools + any-phase + both era pools).
    let valid: std::collections::HashSet<&str> = cfg
        .travel_pool
        .iter()
        .chain(&cfg.operation_pool)
        .chain(&cfg.return_pool)
        .chain(&cfg.any_pool)
        .chain(&cfg.early_pool)
        .chain(&cfg.mid_pool)
        .chain(&cfg.late_pool)
        .map(String::as_str)
        .collect();
    for beat in &a {
        assert!(
            beat.month_clock >= cfg.skip_months,
            "no beat before the skip window"
        );
        assert!(valid.contains(beat.family.as_str()));
    }
}

#[test]
fn a_charter_beat_bias_shapes_its_seeded_campaign() {
    // Content-depth charters round 7: a charter's beat_families ride in every
    // window's draw, so the mission biases the campaign it generates. A heavy
    // bias must visibly dominate the schedule vs the same charter unbiased.
    let data = GameData::load().unwrap();
    let picks = founding_faction_ids(&data);
    let cfg = &data.config.campaign_skeleton;
    let template = data.contracts.get("deep_vein_survey").unwrap().clone();

    let count_family = |beat_families: Vec<String>| -> usize {
        let mut sim = SimState::new_campaign(&data, "preservers", 3, &picks);
        let mut contract = start_contract(&template, &sim);
        contract.beat_families = beat_families;
        generate_beats(&mut sim.rng, &contract, cfg)
            .iter()
            .filter(|b| b.family == "comedy")
            .count()
    };

    // A charter with no bias draws comedy only from the shared any_pool.
    let baseline = count_family(Vec::new());
    // The same charter biased hard toward comedy fills up with it.
    let biased = count_family(vec!["comedy".to_string(); 30]);
    assert!(
        biased > baseline,
        "the charter's beat bias should weight its campaign toward that family \
         (biased {biased} vs baseline {baseline})"
    );
}

#[test]
fn era_layering_tints_the_ends_of_a_voyage() {
    let data = GameData::load().unwrap();
    let cfg = &data.config.campaign_skeleton;
    // Founding-, mid-, and homecoming-era pools must be authored for the
    // layering to mean anything, and must be real event families.
    assert!(!cfg.early_pool.is_empty() && !cfg.late_pool.is_empty());
    assert!(
        !cfg.mid_pool.is_empty(),
        "the deep middle needs its own tint"
    );
    for fam in cfg
        .early_pool
        .iter()
        .chain(&cfg.mid_pool)
        .chain(&cfg.late_pool)
    {
        assert!(
            data.events.iter().any(|(_, e)| &e.family == fam),
            "era family '{fam}' has no events"
        );
    }
}
