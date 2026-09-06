use super::*;
use crate::simulation::autoplay::policies::{Metrics, Policy};
use crate::state::sim::TerminalReason;

#[test]
fn compare_ship_work_policies_on_the_reserve_cohort() {
    let data = GameData::load().unwrap();
    let charters: Vec<_> = GameData::sorted_ids(&data.contracts)
        .into_iter()
        .filter(|id| data.contracts.get(id).unwrap().min_renown == 0 || id == "the_deep_camp")
        .collect();
    let mut csv = String::from("policy,charter,legacy,seed,completed,terminal,food_min,parts_min,critical_months,months,occupied_slot_months,blocked_jobs,queue_transitions,recoveries,recovery_months,max_recovery_months\n");
    let mut totals = Vec::new();
    for policy in [Policy::NoProjects, Policy::Reactive, Policy::Prepared] {
        let (mut completed, mut terminal, mut timed_out) = (0, [0usize; 4], 0);
        let (mut food, mut parts) = (i64::MAX, i64::MAX);
        let (
            mut critical,
            mut months,
            mut occupied,
            mut blocked,
            mut changes,
            mut recoveries,
            mut recovery_months,
            mut max_recovery,
        ) = (0u64, 0u64, 0u64, 0usize, 0u64, 0u64, 0u64, 0u32);
        for id in &charters {
            for legacy in GameData::sorted_ids(&data.legacies) {
                for seed in 0..16 {
                    let mut sim = SimState::new_campaign(
                        &data,
                        &legacy,
                        seed,
                        &crate::state::sim::founding_faction_ids(&data),
                    );
                    let mut metrics = Metrics::default();
                    let outcome = play_with_policy(
                        &mut sim,
                        &data,
                        id,
                        data.contracts.get(id).unwrap().target_duration_years + 160,
                        policy,
                        &mut metrics,
                    );
                    let reason = sim.terminal.as_ref().map(|t| t.reason);
                    completed += usize::from(outcome.completed);
                    if let Some(reason) = reason {
                        terminal[match reason {
                            TerminalReason::DynastyExtinction => 0,
                            TerminalReason::HullLoss => 1,
                            TerminalReason::PopulationLoss => 2,
                            TerminalReason::LifeSupportFailure => 3,
                        }] += 1;
                    } else if !outcome.completed {
                        timed_out += 1;
                    }
                    food = food.min(metrics.min_food.unwrap_or(sim.resources.food));
                    parts = parts.min(metrics.min_parts.unwrap_or(sim.ship.spare_parts));
                    critical += metrics.critical_months;
                    months += metrics.months;
                    occupied += metrics.occupied_slot_months;
                    blocked += metrics.blocked_ids.len();
                    changes += metrics.queue_changes;
                    recoveries += metrics.recoveries;
                    recovery_months += metrics.recovery_months;
                    max_recovery = max_recovery.max(metrics.max_recovery);
                    csv.push_str(&format!("{policy:?},{id},{legacy},{seed},{},{reason:?},{},{},{},{},{},{},{},{},{},{}\n", outcome.completed,
                        metrics.min_food.unwrap_or(0), metrics.min_parts.unwrap_or(0), metrics.critical_months, metrics.months,
                        metrics.occupied_slot_months, metrics.blocked_ids.len(), metrics.queue_changes, metrics.recoveries, metrics.recovery_months, metrics.max_recovery));
                }
            }
        }
        eprintln!("SHIP_WORK {policy:?}: completed={completed}; terminal[dynasty,hull,population,air]={terminal:?}; timeout={timed_out}; min_food={food}; min_parts={parts}; critical_months={critical}; months={months}; occupied_slot_months={occupied}; blocked_jobs={blocked}; queue_transitions={changes}; recoveries={recoveries}; recovery_months={recovery_months}; max_recovery={max_recovery}");
        totals.push((completed, terminal.iter().sum::<usize>(), critical));
    }
    if let Ok(path) = std::env::var("STELLAR_SHIP_WORK_REPORT") {
        std::fs::write(path, csv).unwrap();
    }
    assert!(
        totals[2].0 > totals[0].0 || totals[2].1 < totals[0].1 || totals[2].2 < totals[0].2,
        "Prepared play must improve completion, losses, or critical exposure over neglect."
    );
}
