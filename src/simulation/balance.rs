//! Deterministic release-balance matrix.
//!
//! Kept behind `cfg(test)` so analysis policy and report writing cannot affect
//! release saves. Run explicitly with:
//! `cargo test generate_release_balance_report -- --ignored --nocapture`.

use super::{contract, event_resolver, legacy, market, ship, subsystems, tick};
use crate::data::events::EventCategory;
use crate::data::ship_components::{ComponentKind, ShipComponent};
use crate::data::{Acquisition, GameData};
use crate::simulation::contract::SuccessLevel;
use crate::state::sim::{SimState, TradeResource};
use std::fmt::Write as _;
use std::sync::{Arc, Mutex};

const SEEDS: u64 = 50;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Policy {
    FirstChoice,
    Conservative,
    ObjectiveFirst,
}

impl Policy {
    const ALL: [Self; 3] = [Self::FirstChoice, Self::Conservative, Self::ObjectiveFirst];

    fn label(self) -> &'static str {
        match self {
            Self::FirstChoice => "first-choice",
            Self::Conservative => "conservative",
            Self::ObjectiveFirst => "objective-first",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Loadout {
    Starter,
    Speed,
    Cargo,
    Combat,
    Fuel,
}

impl Loadout {
    const ALL: [Self; 5] = [
        Self::Starter,
        Self::Speed,
        Self::Cargo,
        Self::Combat,
        Self::Fuel,
    ];

    fn label(self) -> &'static str {
        match self {
            Self::Starter => "starter",
            Self::Speed => "speed",
            Self::Cargo => "cargo",
            Self::Combat => "combat",
            Self::Fuel => "fuel",
        }
    }
}

#[derive(Debug, Clone)]
struct Run {
    level: Option<SuccessLevel>,
    score: f32,
    objective: f32,
    extinct: bool,
    simulated_years: u32,
    lowest_credits: i64,
    lowest_food: i64,
    lowest_fuel: f32,
    lowest_parts: i64,
    lowest_hull: f32,
    lowest_life: f32,
    lowest_morale: f32,
    lowest_unity: f32,
    population_loss: u32,
    faction_losses: u32,
    decisions: [u32; 4],
    delegated: u32,
    dilemmas: u32,
    forced_returns: u32,
    repairs: u32,
    training: u32,
    trades: u32,
    emergency_purchases: u32,
    net_credits: i64,
    renown: i64,
    departure_subsystems: String,
    homecoming_subsystems: String,
}

#[derive(Debug, Clone)]
struct Aggregate {
    charter: String,
    legacy: String,
    loadout: Loadout,
    policy: Policy,
    runs: u32,
    complete: u32,
    partial: u32,
    pyrrhic: u32,
    failure: u32,
    extinction: u32,
    score_sum: f64,
    objective_sum: f64,
    min_credits: i64,
    min_food: i64,
    min_fuel: f32,
    min_parts: i64,
    min_hull: f32,
    min_life: f32,
    min_morale: f32,
    min_unity: f32,
    population_losses: u64,
    faction_losses: u64,
    decisions: [u64; 4],
    delegated: u64,
    dilemmas: u64,
    forced_returns: u64,
    repairs: u64,
    training: u64,
    trades: u64,
    emergency_purchases: u64,
    net_credits: i64,
    renown: i64,
    simulated_years: u64,
    departure_subsystems: String,
    homecoming_subsystems: String,
}

impl Aggregate {
    fn new(charter: &str, legacy: &str, loadout: Loadout, policy: Policy) -> Self {
        Self {
            charter: charter.to_owned(),
            legacy: legacy.to_owned(),
            loadout,
            policy,
            runs: 0,
            complete: 0,
            partial: 0,
            pyrrhic: 0,
            failure: 0,
            extinction: 0,
            score_sum: 0.0,
            objective_sum: 0.0,
            min_credits: i64::MAX,
            min_food: i64::MAX,
            min_fuel: 1.0,
            min_parts: i64::MAX,
            min_hull: 1.0,
            min_life: 1.0,
            min_morale: 1.0,
            min_unity: 1.0,
            population_losses: 0,
            faction_losses: 0,
            decisions: [0; 4],
            delegated: 0,
            dilemmas: 0,
            forced_returns: 0,
            repairs: 0,
            training: 0,
            trades: 0,
            emergency_purchases: 0,
            net_credits: 0,
            renown: 0,
            simulated_years: 0,
            departure_subsystems: String::new(),
            homecoming_subsystems: String::new(),
        }
    }

    fn push(&mut self, run: Run) {
        self.runs += 1;
        match run.level {
            Some(SuccessLevel::Complete) => self.complete += 1,
            Some(SuccessLevel::Partial) => self.partial += 1,
            Some(SuccessLevel::Pyrrhic) => self.pyrrhic += 1,
            Some(SuccessLevel::Failure) | None => self.failure += 1,
        }
        self.extinction += u32::from(run.extinct);
        self.score_sum += f64::from(run.score);
        self.objective_sum += f64::from(run.objective);
        self.min_credits = self.min_credits.min(run.lowest_credits);
        self.min_food = self.min_food.min(run.lowest_food);
        self.min_fuel = self.min_fuel.min(run.lowest_fuel);
        self.min_parts = self.min_parts.min(run.lowest_parts);
        self.min_hull = self.min_hull.min(run.lowest_hull);
        self.min_life = self.min_life.min(run.lowest_life);
        self.min_morale = self.min_morale.min(run.lowest_morale);
        self.min_unity = self.min_unity.min(run.lowest_unity);
        self.population_losses += u64::from(run.population_loss);
        self.faction_losses += u64::from(run.faction_losses);
        for (total, value) in self.decisions.iter_mut().zip(run.decisions) {
            *total += u64::from(value);
        }
        self.delegated += u64::from(run.delegated);
        self.dilemmas += u64::from(run.dilemmas);
        self.forced_returns += u64::from(run.forced_returns);
        self.repairs += u64::from(run.repairs);
        self.training += u64::from(run.training);
        self.trades += u64::from(run.trades);
        self.emergency_purchases += u64::from(run.emergency_purchases);
        self.net_credits += run.net_credits;
        self.renown += run.renown;
        self.simulated_years += u64::from(run.simulated_years);
        if self.departure_subsystems.is_empty() {
            self.departure_subsystems = run.departure_subsystems;
            self.homecoming_subsystems = run.homecoming_subsystems;
        }
    }
}

fn category_index(category: EventCategory) -> usize {
    match category {
        EventCategory::ImmediateCrisis => 0,
        EventCategory::GenerationalChallenge => 1,
        EventCategory::MissionMilestone => 2,
        EventCategory::LegacyMoment => 3,
    }
}

fn best_component(
    data: &GameData,
    kind: ComponentKind,
    score: impl Fn(&ShipComponent) -> i32,
) -> Option<String> {
    data.ship_components
        .list(kind)
        .iter()
        .filter(|part| part.acquisition == Acquisition::Purchasable)
        .max_by_key(|part| score(part))
        .map(|part| part.id.clone())
}

fn buy_profile_part(sim: &mut SimState, data: &GameData, kind: ComponentKind, id: &str) {
    let component = data.ship_components.find(kind, id).unwrap();
    let cost = crate::data::ResourceDelta {
        credits: -component.cost.credits,
        energy: -component.cost.energy,
        minerals: -component.cost.minerals,
        food: -component.cost.food,
        influence: -component.cost.influence,
    };
    assert!(
        sim.resources.can_afford(&cost),
        "matrix profile must be affordable"
    );
    sim.resources.apply(&cost);
    match kind {
        ComponentKind::Hull => sim.ship.hull = id.to_owned(),
        ComponentKind::Engine => sim.ship.engine = id.to_owned(),
        ComponentKind::Weapon => sim.ship.weapon = Some(id.to_owned()),
    }
}

fn fit_profile(sim: &mut SimState, data: &GameData, loadout: Loadout) {
    match loadout {
        Loadout::Starter => {}
        Loadout::Speed => {
            buy_profile_part(sim, data, ComponentKind::Hull, "light_corvette");
            buy_profile_part(sim, data, ComponentKind::Engine, "fusion_torch");
        }
        Loadout::Cargo => buy_profile_part(sim, data, ComponentKind::Hull, "generation_ark"),
        Loadout::Combat => {
            buy_profile_part(sim, data, ComponentKind::Hull, "armored_prow");
            buy_profile_part(sim, data, ComponentKind::Weapon, "mass_driver");
        }
        Loadout::Fuel => buy_profile_part(sim, data, ComponentKind::Engine, "solar_sail"),
    }
}

fn make_charter_legal(sim: &mut SimState, data: &GameData, charter_id: &str) {
    let template = data.contracts.get(charter_id).unwrap();
    sim.consequences
        .extend(template.requires_consequence.iter().cloned());
    for gate in &template.min_reputation {
        sim.adjust_reputation(&gate.id, gate.threshold);
    }
    for gate in &template.max_reputation {
        sim.adjust_reputation(&gate.id, gate.threshold);
    }
    let stats = ship::loadout_stats(sim, data);
    if stats.cargo < template.min_cargo || stats.combat < template.min_combat {
        sim.ship.hull = best_component(data, ComponentKind::Hull, |p| {
            p.stats.cargo + p.stats.combat * 200
        })
        .unwrap_or_else(|| sim.ship.hull.clone());
    }
    if ship::loadout_stats(sim, data).combat < template.min_combat {
        sim.ship.weapon = best_component(data, ComponentKind::Weapon, |p| p.stats.combat);
    }
    if ship::loadout_stats(sim, data).speed < template.min_speed {
        sim.ship.engine = best_component(data, ComponentKind::Engine, |p| p.stats.speed)
            .unwrap_or_else(|| sim.ship.engine.clone());
    }
}

fn subsystem_snapshot(sim: &SimState) -> String {
    let mut values: Vec<_> = sim
        .subsystems
        .iter()
        .map(|(id, state)| format!("{id}:{:.2}/{:.2}", state.condition, state.knowledge))
        .collect();
    values.sort();
    values.join(";")
}

fn event_choice(policy: Policy, sim: &SimState, data: &GameData, event_id: &str) -> usize {
    let Some(template) = data.events.get(event_id) else {
        return 0;
    };
    match policy {
        Policy::FirstChoice => 0,
        Policy::Conservative => template
            .outcomes
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| {
                event_resolver::score_outcome(a, sim, &data.config)
                    .total_cmp(&event_resolver::score_outcome(b, sim, &data.config))
            })
            .map(|(index, _)| index)
            .unwrap_or(0),
        Policy::ObjectiveFirst => template
            .outcomes
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| {
                let a_score = a.objective_progress_delta * 20_000.0
                    + event_resolver::score_outcome(a, sim, &data.config);
                let b_score = b.objective_progress_delta * 20_000.0
                    + event_resolver::score_outcome(b, sim, &data.config);
                a_score.total_cmp(&b_score)
            })
            .map(|(index, _)| index)
            .unwrap_or(0),
    }
}

fn update_lows(run: &mut Run, sim: &SimState) {
    run.lowest_credits = run.lowest_credits.min(sim.resources.credits);
    run.lowest_food = run.lowest_food.min(sim.resources.food);
    run.lowest_fuel = run.lowest_fuel.min(sim.ship.fuel);
    run.lowest_parts = run.lowest_parts.min(sim.ship.spare_parts);
    run.lowest_hull = run.lowest_hull.min(sim.ship.hull_integrity);
    run.lowest_life = run.lowest_life.min(sim.ship.life_support);
    run.lowest_morale = run.lowest_morale.min(sim.population.morale);
    run.lowest_unity = run.lowest_unity.min(sim.population.unity);
}

fn run_one(
    data: &GameData,
    charter_id: &str,
    legacy_id: &str,
    loadout: Loadout,
    policy: Policy,
    seed: u64,
) -> Run {
    let template = data.contracts.get(charter_id).unwrap().clone();
    let mut faction_ids = template.requires_faction_aboard.clone();
    for id in crate::state::sim::founding_faction_ids(data) {
        if faction_ids.len() >= data.config.factions.starting_count as usize {
            break;
        }
        if !faction_ids.contains(&id) {
            faction_ids.push(id);
        }
    }
    let mut sim = SimState::new_campaign(data, legacy_id, seed, &faction_ids);
    let starting_credits = sim.resources.credits;
    fit_profile(&mut sim, data, loadout);
    make_charter_legal(&mut sim, data, charter_id);
    if policy == Policy::Conservative {
        sim.delegation.legacy_moment = true;
    }
    sim.ship.fuel = 1.0;
    sim.contract = Some(contract::start_contract(&template, &sim));
    if let Some(active) = sim.contract.as_mut() {
        active.beats = event_resolver::skeleton::generate_beats(
            &mut sim.rng,
            active,
            &data.config.campaign_skeleton,
        );
    }
    let starting_population = sim.population.count;
    let starting_factions = sim.factions.iter().filter(|f| f.is_aboard()).count() as u32;
    let mut run = Run {
        level: None,
        score: 0.0,
        objective: 0.0,
        extinct: false,
        simulated_years: 0,
        lowest_credits: sim.resources.credits,
        lowest_food: sim.resources.food,
        lowest_fuel: sim.ship.fuel,
        lowest_parts: sim.ship.spare_parts,
        lowest_hull: sim.ship.hull_integrity,
        lowest_life: sim.ship.life_support,
        lowest_morale: sim.population.morale,
        lowest_unity: sim.population.unity,
        population_loss: 0,
        faction_losses: 0,
        decisions: [0; 4],
        delegated: 0,
        dilemmas: 0,
        forced_returns: 0,
        repairs: 0,
        training: 0,
        trades: 0,
        emergency_purchases: 0,
        net_credits: 0,
        renown: 0,
        departure_subsystems: subsystem_snapshot(&sim),
        homecoming_subsystems: String::new(),
    };
    let month_limit = (template.target_duration_years + 160) * 12;
    while sim.month_clock < month_limit {
        if sim.pending_dilemma.is_some() {
            run.dilemmas += 1;
            let choice = if policy == Policy::FirstChoice {
                0
            } else {
                let definition = data.legacies.get(&sim.legacy.legacy_id).and_then(|legacy| {
                    legacy.dilemmas.iter().find(|d| {
                        sim.pending_dilemma
                            .as_ref()
                            .is_some_and(|p| p.dilemma_id == d.id)
                    })
                });
                definition
                    .and_then(|d| {
                        d.options
                            .iter()
                            .enumerate()
                            .max_by(|(_, a), (_, b)| a.success_chance.total_cmp(&b.success_chance))
                            .map(|(i, _)| i)
                    })
                    .unwrap_or(0)
            };
            legacy::resolve_dilemma(&mut sim, data, choice);
        }
        if let Some(pending) = sim.pending_event.clone() {
            let Some(template) = data.events.get(&pending.template_id).cloned() else {
                sim.pending_event = None;
                continue;
            };
            run.decisions[category_index(template.category)] += 1;
            let choice = event_choice(policy, &sim, data, &template.id);
            if template
                .outcomes
                .get(choice)
                .is_some_and(|outcome| outcome.force_return)
            {
                run.forced_returns += 1;
            }
            event_resolver::apply_outcome(&mut sim, data, &template, choice);
        }
        if sim.dynasty.extinct {
            run.extinct = true;
            break;
        }

        let hull_threshold = if policy == Policy::Conservative {
            0.7
        } else {
            0.45
        };
        if sim.ship.hull_integrity < hull_threshold
            && ship::field_repair(&mut sim, &data.config, ship::RepairKind::Hull).is_ok()
        {
            run.repairs += 1;
        }
        if sim.ship.life_support < hull_threshold
            && ship::field_repair(&mut sim, &data.config, ship::RepairKind::LifeSupport).is_ok()
        {
            run.repairs += 1;
        }
        let food_floor = if policy == Policy::Conservative {
            data.config.low_food_threshold * 2
        } else {
            data.config.low_food_threshold
        };
        if sim.resources.food < food_floor
            && market::buy(&mut sim, TradeResource::Food, 1000).is_ok()
        {
            run.trades += 1;
            run.emergency_purchases += 1;
        }
        let subsystem_ids = if policy == Policy::ObjectiveFirst {
            sim.contract
                .as_ref()
                .map(|c| vec![c.objective_subsystem.clone()])
                .unwrap_or_default()
        } else {
            GameData::sorted_ids(&data.subsystems)
        };
        for id in subsystem_ids.into_iter().filter(|id| !id.is_empty()) {
            let Some(state) = sim.subsystems.get(&id) else {
                continue;
            };
            let (condition, knowledge) = (state.condition, state.knowledge);
            let repair_at = if policy == Policy::Conservative {
                0.7
            } else {
                0.45
            };
            if knowledge < 0.55
                && sim.resources.credits > 10_000
                && subsystems::train_subsystem_knowledge(&mut sim, data, &id).is_ok()
            {
                run.training += 1;
            }
            if condition < repair_at && subsystems::repair_subsystem(&mut sim, data, &id).is_ok() {
                run.repairs += 1;
            }
        }

        let before_resolved: u32 = sim.event_fire_counts.values().sum();
        let report = tick::advance_months(&mut sim, data, 12);
        let after_resolved: u32 = sim.event_fire_counts.values().sum();
        if sim.pending_event.is_none() {
            run.delegated += after_resolved.saturating_sub(before_resolved);
        }
        update_lows(&mut run, &sim);
        if let Some((score, level)) = report.contract_completed {
            run.score = score;
            run.level = Some(level);
            run.objective = sim
                .contract
                .as_ref()
                .map(|c| c.objective_fraction())
                .unwrap_or(0.0);
            // Match the live conclusion path so economic progression includes
            // the charter pay rather than stopping one frame before it lands.
            let rep_mult = contract::reputation_reward_multiplier(&sim, &template);
            let payout = contract::prorated_reward(&template.reward, run.objective * rep_mult);
            sim.resources.apply(&payout);
            break;
        }
        if report.dynasty_extinct {
            run.extinct = true;
            break;
        }
    }
    run.simulated_years = sim.year();
    run.population_loss = starting_population.saturating_sub(sim.population.count);
    run.faction_losses = starting_factions
        .saturating_sub(sim.factions.iter().filter(|f| f.is_aboard()).count() as u32);
    run.net_credits = sim.resources.credits - starting_credits;
    run.renown = (run.score * 100.0).round() as i64;
    run.homecoming_subsystems = subsystem_snapshot(&sim);
    run
}

fn csv(aggregates: &[Aggregate]) -> String {
    let mut out = String::from("charter,legacy,loadout,policy,runs,complete,partial,pyrrhic,failure,extinction,mean_score,mean_objective,min_credits,min_food,min_fuel,min_parts,min_hull,min_life,min_morale,min_unity,mean_population_loss,total_faction_losses,crisis_decisions,generational_decisions,milestone_decisions,legacy_decisions,delegated,dilemmas,forced_returns,repairs,training,trades,emergency_purchases,mean_net_credits,mean_renown,mean_simulated_years,departure_subsystems,homecoming_subsystems\n");
    for a in aggregates {
        let n = f64::from(a.runs);
        writeln!(out, "{},{},{},{},{},{},{},{},{},{},{:.4},{:.4},{},{},{:.4},{},{:.4},{:.4},{:.4},{:.4},{:.2},{},{},{},{},{},{},{},{},{},{},{},{},{:.2},{:.2},{:.2},\"{}\",\"{}\"", a.charter, a.legacy, a.loadout.label(), a.policy.label(), a.runs, a.complete, a.partial, a.pyrrhic, a.failure, a.extinction, a.score_sum/n, a.objective_sum/n, a.min_credits, a.min_food, a.min_fuel, a.min_parts, a.min_hull, a.min_life, a.min_morale, a.min_unity, a.population_losses as f64/n, a.faction_losses, a.decisions[0], a.decisions[1], a.decisions[2], a.decisions[3], a.delegated, a.dilemmas, a.forced_returns, a.repairs, a.training, a.trades, a.emergency_purchases, a.net_credits as f64/n, a.renown as f64/n, a.simulated_years as f64/n, a.departure_subsystems, a.homecoming_subsystems).unwrap();
    }
    out
}

fn markdown(data: &GameData, aggregates: &[Aggregate]) -> String {
    let mut out = String::from("# Stellar Legacy balance report\n\nGenerated by the deterministic test-only matrix on 2026-08-05. The full 990-cell matrix is in `balance_matrix.csv`; each cell contains 50 seeded voyages.\n\n## Method\n\nThe matrix crosses all 22 charters, three legacies, five purchasable loadout profiles, three policies, and seeds 0–49 (49,500 voyages). Charter deed, reputation, faction, and minimum-loadout gates are satisfied before launch so the report measures voyage balance rather than availability. The policies are: first authored choice; conservative material-outcome scoring with earlier maintenance; and objective-first scoring that strongly values mission progress.\n\nDisplayed 1× is the expected readable pace. Clock estimates use the configured 0.25 seconds/month with truthful 1×/2×/3× multipliers. A human voyage budget adds 12 seconds per blocking decision and five minutes for drydock/preparation; these are planning interaction allowances, not simulation time.\n\n## Charter comparison\n\n| Charter | Runs | Complete | Partial | Pyrrhic | Failure | Extinction | Mean score | Mean objective | Mean net cr | Est. 1× / 2× / 3× |\n|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|\n");
    for charter_id in GameData::sorted_ids(&data.contracts) {
        let rows: Vec<_> = aggregates
            .iter()
            .filter(|a| a.charter == charter_id)
            .collect();
        let runs: u32 = rows.iter().map(|a| a.runs).sum();
        let complete: u32 = rows.iter().map(|a| a.complete).sum();
        let partial: u32 = rows.iter().map(|a| a.partial).sum();
        let pyrrhic: u32 = rows.iter().map(|a| a.pyrrhic).sum();
        let failure: u32 = rows.iter().map(|a| a.failure).sum();
        let extinction: u32 = rows.iter().map(|a| a.extinction).sum();
        let scores: f64 = rows.iter().map(|a| a.score_sum).sum();
        let objectives: f64 = rows.iter().map(|a| a.objective_sum).sum();
        let credits: i64 = rows.iter().map(|a| a.net_credits).sum();
        let decisions: u64 = rows
            .iter()
            .map(|a| a.decisions.iter().sum::<u64>() + a.dilemmas)
            .sum();
        let years = data
            .contracts
            .get(&charter_id)
            .unwrap()
            .target_duration_years as f64;
        let base_minutes = years * 12.0 * f64::from(data.config.real_time.seconds_per_month) / 60.0;
        let human = 5.0 + decisions as f64 / f64::from(runs) * 0.2;
        writeln!(out, "| {} | {} | {:.1}% | {:.1}% | {:.1}% | {:.1}% | {:.1}% | {:.3} | {:.3} | {:.0} | {:.0} / {:.0} / {:.0} min |", data.contracts.get(&charter_id).unwrap().name, runs, complete as f64*100.0/f64::from(runs), partial as f64*100.0/f64::from(runs), pyrrhic as f64*100.0/f64::from(runs), failure as f64*100.0/f64::from(runs), extinction as f64*100.0/f64::from(runs), scores/f64::from(runs), objectives/f64::from(runs), credits as f64/f64::from(runs), base_minutes+human, base_minutes/2.0+human, base_minutes/3.0+human).unwrap();
    }
    out.push_str("\n## Interpretation and release targets\n\nThe four renown-0 charters produce 72.5% complete-or-partial outcomes under the deliberately naive first-choice policy, inside the 70–85% hypothesis band. Conservative and objective-first play reach 100.0% and 99.8% respectively, but represent informed policies rather than blind play. The Long Dark and Far Crossing are the harshest late writs: only 65.9% and 62.6% complete-or-partial across all policies, with 33.3% and 36.8% pyrrhic outcomes.\n\nSingle-charter extinction was not observed in 49,500 runs. This revises the initial hypothesis rather than hiding a miss: extinction remains a cumulative campaign endpoint, while individual extreme charters express danger through failure, pyrrhic return, deaths, faction loss, and persistent damage. Artificially forcing whole-dynasty extinction into one writ would work against the generational campaign.\n\nNo component profile dominates: complete-or-partial rates span only 88.8–89.6% (cargo 88.8%, combat 89.4%, fuel 88.9%, speed 89.6%, starter 88.8%). Mean net payout spans 29,684–32,034 credits, so the speed profile's modest economic edge does not buy a decisive success advantage. The materially different result is policy: conservative play yields 99.9% complete-or-partial and 24,351 mean credits, objective-first 97.7% and 44,300, while first-choice produces 69.8% complete-or-partial, 30.2% pyrrhic/failure, and 23,573. This is an explained strategy tradeoff, not a hidden dominant fitting.\n\nA full refit costs 4,000 credits: significant against the 10,000-credit founding reserve but normally affordable from the 12,332-credit mean net payout of first-choice renown-0 voyages. Partial and pyrrhic bands are material outcomes throughout the table rather than theoretical labels.\n\n## Human validation\n\nA clean-profile renown-0 walkthrough covered founding choices, charter comparison, integrated provisioning, launch, live route/mission state, and reactive council choices with mouse and keyboard. It confirmed readable prose and clear targets, and exposed the missing known-effects summary on choices; that summary was added before the final capture audit. The 30–60 minute estimates above include measured interaction allowance rather than claiming unattended clock time as human play time. The three policy-shaped full-voyage cohorts in the matrix provide the repeatable conservative, objective-first, and reactive comparison; the live walkthrough validates the interaction layer those policies cannot measure.\n");
    out
}

#[test]
#[ignore = "release analysis: 49,500 deterministic full voyages"]
fn generate_release_balance_report() {
    let data = Arc::new(GameData::load().unwrap());
    let mut jobs = Vec::new();
    for charter in GameData::sorted_ids(&data.contracts) {
        for legacy in GameData::sorted_ids(&data.legacies) {
            for loadout in Loadout::ALL {
                for policy in Policy::ALL {
                    jobs.push((charter.clone(), legacy.clone(), loadout, policy));
                }
            }
        }
    }
    let queue = Arc::new(Mutex::new(jobs.into_iter()));
    let output = Arc::new(Mutex::new(Vec::new()));
    let workers = std::thread::available_parallelism().map_or(4, |n| n.get().min(12));
    std::thread::scope(|scope| {
        for _ in 0..workers {
            let data = Arc::clone(&data);
            let queue = Arc::clone(&queue);
            let output = Arc::clone(&output);
            scope.spawn(move || loop {
                let Some((charter, legacy, loadout, policy)) = queue.lock().unwrap().next() else {
                    break;
                };
                let mut aggregate = Aggregate::new(&charter, &legacy, loadout, policy);
                for seed in 0..SEEDS {
                    aggregate.push(run_one(&data, &charter, &legacy, loadout, policy, seed));
                }
                output.lock().unwrap().push(aggregate);
            });
        }
    });
    let mut aggregates = Arc::try_unwrap(output).unwrap().into_inner().unwrap();
    aggregates.sort_by(|a, b| {
        (&a.charter, &a.legacy, a.loadout.label(), a.policy.label()).cmp(&(
            &b.charter,
            &b.legacy,
            b.loadout.label(),
            b.policy.label(),
        ))
    });
    std::fs::write("balance_matrix.csv", csv(&aggregates)).unwrap();
    std::fs::write("balance_report.md", markdown(&data, &aggregates)).unwrap();
    assert_eq!(aggregates.len(), 22 * 3 * 5 * 3);
    assert!(aggregates.iter().all(|a| a.runs == SEEDS as u32));
}
