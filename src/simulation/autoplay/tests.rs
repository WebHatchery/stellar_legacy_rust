use super::*;

#[test]
fn autoplay_identifies_and_resolves_a_due_obligation_without_deadlock() {
    let mut data = GameData::load().unwrap();
    data.config.event_chance_base = 0.0;
    data.config.event_chance_cap = 0.0;
    data.config.dilemma_chance_per_generation = 0.0;
    let picks = crate::state::sim::founding_faction_ids(&data);
    let mut sim = SimState::new_campaign(&data, "preservers", 913, &picks);
    sim.resources.food = 10_000_000;
    let seed = data.events.get("sanctuary_berths_asked").unwrap().clone();
    event_resolver::apply_outcome(&mut sim, &data, &seed, 0);

    while sim.year() <= 46 {
        if let Some(pending) = sim.pending_event.clone() {
            let event = data.events.get(&pending.template_id).unwrap().clone();
            event_resolver::apply_outcome(&mut sim, &data, &event, 0);
        }
        advance_months(&mut sim, &data, 12);
    }
    if let Some(pending) = sim.pending_event.clone() {
        let event = data.events.get(&pending.template_id).unwrap().clone();
        event_resolver::apply_outcome(&mut sim, &data, &event, 0);
    }
    assert!(sim.due_obligations().is_empty());
    assert_eq!(
        sim.obligations[0].status,
        crate::state::sim::ObligationStatus::Fulfilled
    );
}

/// Economy rebalance soak (phase 5): fly a full early
/// campaign — six low-renown charters back to back on one dynasty — and prove
/// the repriced economy holds up end to end. Under the maintenance-heavy
/// autoplay policy (which spends surplus on training and repair), every
/// charter must remain flyable to completion, the dynasty must stay solvent
/// and unbroken across all six, and the accrued renown must clear the
/// top-tier gate (400) by the end — so a storied ship able to take the great
/// charters is earned across a normal early campaign, the prestige ladder
/// keeping pace with the credit one rather than either finishing first.
#[test]
fn an_early_campaign_flies_solvent_and_earns_its_renown() {
    let data = GameData::load().unwrap();
    let mut sim = SimState::new_campaign(
        &data,
        "wanderers",
        7,
        &crate::state::sim::founding_faction_ids(&data),
    );
    let seq = [
        "deep_vein_survey",
        "tarssen_relief",
        "coronal_tap",
        "hollow_fleet",
        "veiled_expanse_survey",
        "seedfall",
    ];
    let mut renown = 0.0f32;
    for id in seq {
        let dur = data.contracts.get(id).unwrap().target_duration_years;
        let cap = sim.year() + dur + 80;
        let out = play_mission(&mut sim, &data, id, cap);
        assert!(
            out.completed && !out.extinct,
            "charter '{id}' must stay flyable to completion at the repriced economy \
             (completed {}, extinct {}, year {})",
            out.completed,
            out.extinct,
            out.final_year
        );
        assert!(
            sim.resources.credits >= 0,
            "the dynasty must stay solvent flying '{id}' (credits {})",
            sim.resources.credits
        );
        renown += out.final_score * 100.0;
    }
    assert!(
        renown >= 400.0,
        "six early charters should earn past the top-tier renown gate (400); reached {renown:.0}"
    );
}

/// Soak test: fly the 340-year Deep Vein Survey end to end under the
/// autoplay policy. It must conclude with the charter completed and the
/// dynasty still alive, twelve-plus generations on. Per-year invariants are
/// asserted inside `play_mission`.
#[test]
fn deep_vein_survey_completes_with_a_living_dynasty() {
    let data = GameData::load().unwrap();
    let mut sim = SimState::new_campaign(
        &data,
        "preservers",
        2024,
        &crate::state::sim::founding_faction_ids(&data),
    );

    let outcome = play_mission(&mut sim, &data, "deep_vein_survey", 420);

    assert!(
        outcome.completed,
        "the 340-year survey should complete under autoplay (ended year {}, extinct {})",
        outcome.final_year, outcome.extinct
    );
    assert!(
        !outcome.extinct,
        "the dynasty should survive the survey with the autoplay policy"
    );
    assert!(
        sim.dynasty.generation >= 12,
        "340 years is 12+ successions; dynasty only reached generation {}",
        sim.dynasty.generation
    );
}

/// Soak the long-station charter shape (content-depth): the 480-year Deep
/// Camp spends most of its length parked on-station rather than in transit.
/// It must resolve legally (complete or lose the line), never run the clock
/// out mid-voyage, with all per-year invariants holding across 19+
/// generations.
#[test]
fn the_deep_camp_resolves_over_a_long_station_voyage() {
    let data = GameData::load().unwrap();
    let mut sim = SimState::new_campaign(
        &data,
        "preservers",
        51,
        &crate::state::sim::founding_faction_ids(&data),
    );

    let outcome = play_mission(&mut sim, &data, "the_deep_camp", 560);

    assert!(
        outcome.completed || outcome.extinct,
        "the deep camp should resolve, not run out the clock (year {}, completed {}, extinct {})",
        outcome.final_year,
        outcome.completed,
        outcome.extinct
    );
}

/// Soak the double-hop charter topology (content-depth round 3): the
/// 440-year Twin Survey has two Operation legs split by a second Travel
/// segment. The objective must accrue across both on-station stretches and
/// the run resolve legally, exercising a phase sequence no other charter uses.
#[test]
fn the_twin_survey_resolves_across_two_operation_legs() {
    let data = GameData::load().unwrap();
    let mut sim = SimState::new_campaign(
        &data,
        "adaptors",
        88,
        &crate::state::sim::founding_faction_ids(&data),
    );

    let outcome = play_mission(&mut sim, &data, "twin_survey", 520);

    assert!(
        outcome.completed || outcome.extinct,
        "the twin survey should resolve, not run out the clock (year {}, completed {}, extinct {})",
        outcome.final_year,
        outcome.completed,
        outcome.extinct
    );
}

/// Soak the return-dominant charter shape (content-depth charters round 4):
/// the 450-year Long Tow spends only 60 years on-station and 320 hauling the
/// prize home, so the hard stretch is the return, not the outbound or the
/// operation — a shape no other charter has. Assert the topology is genuinely
/// return-dominant, then fly it end to end and require a legal resolution.
#[test]
fn the_long_tow_resolves_over_a_return_dominant_voyage() {
    use crate::data::contracts::ContractPhase;

    let data = GameData::load().unwrap();
    let template = data.contracts.get("the_long_tow").unwrap();
    let return_years: u32 = template
        .phases
        .iter()
        .filter(|p| p.kind == ContractPhase::Return)
        .map(|p| p.years)
        .sum();
    assert!(
        return_years * 2 > template.target_duration_years,
        "the long tow's haul home must dominate the voyage: {return_years} of {} years",
        template.target_duration_years
    );

    let mut sim = SimState::new_campaign(
        &data,
        "wanderers",
        94,
        &crate::state::sim::founding_faction_ids(&data),
    );
    let outcome = play_mission(&mut sim, &data, "the_long_tow", 520);
    assert!(
        outcome.completed || outcome.extinct,
        "the long tow should resolve, not run out the clock (year {}, completed {}, extinct {})",
        outcome.final_year,
        outcome.completed,
        outcome.extinct
    );
}

/// Soak the outbound-dominant charter shape (content-depth charters round 5):
/// the 480-year Far Crossing spends 300 years on the outbound burn before a
/// 70-year survey and a 110-year charted return — the mirror of the Long
/// Tow, where the *getting there* is the trial. Assert the topology is
/// genuinely travel-dominant, then fly it end to end for a legal resolution.
#[test]
fn the_far_crossing_resolves_over_an_outbound_dominant_voyage() {
    use crate::data::contracts::ContractPhase;

    let data = GameData::load().unwrap();
    let template = data.contracts.get("the_far_crossing").unwrap();
    let travel_years: u32 = template
        .phases
        .iter()
        .filter(|p| p.kind == ContractPhase::Travel)
        .map(|p| p.years)
        .sum();
    assert!(
        travel_years * 2 > template.target_duration_years,
        "the far crossing's outbound burn must dominate the voyage: {travel_years} of {} years",
        template.target_duration_years
    );

    let mut sim = SimState::new_campaign(
        &data,
        "wanderers",
        77,
        &crate::state::sim::founding_faction_ids(&data),
    );
    let outcome = play_mission(&mut sim, &data, "the_far_crossing", 560);
    assert!(
        outcome.completed || outcome.extinct,
        "the far crossing should resolve, not run out the clock (year {}, completed {}, extinct {})",
        outcome.final_year,
        outcome.completed,
        outcome.extinct
    );
}

/// The 600-year Long Dark is allowed to end either way: completion or the
/// total loss of the line. This test only pins the invariants (asserted in
/// `play_mission`) and that the run terminates in one of the two legal
/// outcomes rather than running the clock out mid-voyage.
#[test]
fn the_long_dark_ends_in_completion_or_extinction() {
    let data = GameData::load().unwrap();
    let mut sim = SimState::new_campaign(
        &data,
        "wanderers",
        7,
        &crate::state::sim::founding_faction_ids(&data),
    );

    let outcome = play_mission(&mut sim, &data, "the_long_dark", 700);

    assert!(
        outcome.completed || outcome.extinct,
        "the long dark should resolve to completion or extinction, not run out the clock \
         (year {}, completed {}, extinct {})",
        outcome.final_year,
        outcome.completed,
        outcome.extinct
    );
}

/// Turning back at year 150 banks only part of the objective, and pay is
/// strictly proportional — a truncated run earns less than a full term (W2).
#[test]
fn aborting_at_year_150_reduces_the_pay() {
    use crate::simulation::contract::{advance_contract, jump_to_return, prorated_reward};

    let data = GameData::load().unwrap();
    let template = data.contracts.get("deep_vein_survey").unwrap().clone();

    // Fly the contract clock straight through — no economy needed to measure
    // the objective the timeline banks.
    let full_fraction = {
        let mut sim = SimState::new_campaign(
            &data,
            "preservers",
            2024,
            &crate::state::sim::founding_faction_ids(&data),
        );
        sim.contract = Some(start_contract(&template, &sim));
        let total = sim.contract.as_ref().unwrap().total_months();
        while sim.contract.as_ref().unwrap().months_elapsed < total {
            advance_contract(&mut sim, &data.config, 0, 0, 0, 0);
        }
        sim.contract.as_ref().unwrap().objective_fraction()
    };
    assert!(full_fraction >= 0.99, "a full term meets the objective");

    let abort_fraction = {
        let mut sim = SimState::new_campaign(
            &data,
            "preservers",
            2024,
            &crate::state::sim::founding_faction_ids(&data),
        );
        sim.contract = Some(start_contract(&template, &sim));
        while sim.contract.as_ref().unwrap().months_elapsed < 150 * 12 {
            advance_contract(&mut sim, &data.config, 0, 0, 0, 0);
        }
        assert!(jump_to_return(&mut sim), "turning back at year 150");
        let total = sim.contract.as_ref().unwrap().total_months();
        while sim.contract.as_ref().unwrap().months_elapsed < total {
            advance_contract(&mut sim, &data.config, 0, 0, 0, 0);
        }
        sim.contract.as_ref().unwrap().objective_fraction()
    };

    assert!(
        abort_fraction > 0.0 && abort_fraction < full_fraction,
        "aborting at year 150 banks some but not all of the objective: {abort_fraction} vs {full_fraction}"
    );
    let full_pay = prorated_reward(&template.reward, full_fraction);
    let abort_pay = prorated_reward(&template.reward, abort_fraction);
    assert!(
        abort_pay.credits < full_pay.credits,
        "a truncated mission pays less: {} vs {}",
        abort_pay.credits,
        full_pay.credits
    );
}

/// A full mission fires its seeded campaign beats (W6): flown to completion
/// under the first-choice policy, every scheduled beat fires by mission end
/// — the campaign skeleton actually plays out.
#[test]
fn a_full_mission_fires_its_campaign_beats() {
    let data = GameData::load().unwrap();
    let picks = crate::state::sim::founding_faction_ids(&data);
    let mut sim = SimState::new_campaign(&data, "preservers", 2024, &picks);
    let template = data.contracts.get("deep_vein_survey").unwrap().clone();
    sim.contract = Some(start_contract(&template, &sim));
    if let Some(c) = sim.contract.as_mut() {
        c.beats = crate::simulation::event_resolver::skeleton::generate_beats(
            &mut sim.rng,
            c,
            &data.config.campaign_skeleton,
        );
    }
    // Absurd food so the run never dies to famine — we only care about beats.
    sim.resources.food = 100_000_000;
    let total = sim.contract.as_ref().unwrap().beats.len();
    assert_eq!(total, 17, "17 beats for a 340-yr charter");

    // Fly the whole voyage, resolving every decision by first choice; keep
    // the contract intact on completion so the beats can be inspected.
    for _ in 0..5000 {
        if sim.pending_dilemma.is_some() {
            legacy::resolve_dilemma(&mut sim, &data, 0);
        }
        if let Some(pending) = sim.pending_event.clone() {
            match data.events.get(&pending.template_id).cloned() {
                Some(t) => event_resolver::apply_outcome(&mut sim, &data, &t, 0),
                None => sim.pending_event = None,
            }
        }
        if sim.dynasty.extinct {
            break;
        }
        let report = advance_months(&mut sim, &data, 120);
        if report.contract_completed.is_some() {
            break;
        }
    }

    let fired = sim
        .contract
        .as_ref()
        .map(|c| c.beats.iter().filter(|b| b.fired).count())
        .unwrap_or(0);
    assert!(
        fired == total,
        "all scheduled beats fire by mission end (fired {fired}/{total})"
    );
}

/// A launch on a dry tank strands the ship in transit (W4): it stalls, so
/// after the same calendar span its contract has advanced measurably less
/// than a fully-fuelled run's, and it logged stalled months.
#[test]
fn an_under_fuelled_launch_stalls_and_falls_behind() {
    let mut data = GameData::load().unwrap();
    data.config.event_chance_base = 0.0;
    data.config.event_chance_cap = 0.0;
    data.config.dilemma_chance_per_generation = 0.0;
    // Threshold beats fire independent of event chance and would block this
    // resolve-nothing timeline loop; clear them (content-depth rounds 2-3),
    // and switch off the round-5 dead-air backstop for the same reason.
    data.config.campaign_skeleton.drift_beats.clear();
    data.config.campaign_skeleton.adaptation_beats.clear();
    data.config.campaign_skeleton.crisis_beats.clear();
    data.config.campaign_skeleton.dead_air_years = 0;
    let picks = crate::state::sim::founding_faction_ids(&data);
    let template = data.contracts.get("deep_vein_survey").unwrap().clone();

    // Returns (calendar months, contract months, stalled months).
    let run = |fuel: f32| -> (u32, u32, u32) {
        let mut sim = SimState::new_campaign(&data, "preservers", 7, &picks);
        sim.contract = Some(start_contract(&template, &sim));
        sim.resources.food = 10_000_000;
        sim.ship.fuel = fuel;
        while sim.month_clock < 50 * 12 {
            // Clear any beat-forced decision so the calendar keeps moving
            // uninterrupted — this test measures fuel/stall, not choices.
            sim.pending_event = None;
            sim.pending_dilemma = None;
            advance_months(&mut sim, &data, 120);
        }
        (
            sim.month_clock,
            sim.contract.as_ref().unwrap().months_elapsed,
            sim.stalled_months,
        )
    };

    let (fuelled_cal, fuelled_con, fuelled_stall) = run(1.0);
    let (dry_cal, dry_con, dry_stall) = run(0.0);

    assert_eq!(fuelled_stall, 0, "a full tank never stalls in transit");
    assert_eq!(
        fuelled_cal, fuelled_con,
        "a fuelled voyage's calendar keeps pace with its contract clock"
    );
    assert!(dry_stall > 0, "a dry launch strands the ship");
    assert!(
        dry_cal > dry_con,
        "the dry run's calendar outran its contract clock: {dry_cal} > {dry_con}"
    );
    assert_eq!(
        dry_cal - dry_con,
        dry_stall,
        "the calendar/contract gap is exactly the stalled months"
    );
}
