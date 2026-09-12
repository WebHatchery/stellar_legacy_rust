use super::*;

#[test]
fn every_balance_policy_chooses_an_available_affordable_outcome() {
    let data = GameData::load().unwrap();
    let mut sim = SimState::new_campaign(
        &data,
        "preservers",
        7,
        &crate::state::sim::founding_faction_ids(&data),
    );
    sim.resources.credits = 0;
    sim.resources.energy = 0;
    sim.resources.minerals = 0;
    sim.resources.food = 0;
    sim.resources.influence = 0;

    for id in GameData::sorted_ids(&data.events) {
        let event = data.events.get(&id).unwrap();
        for policy in Policy::ALL {
            let choice = event_choice(policy, &sim, &data, &id);
            let outcome = &event.outcomes[choice];
            assert!(
                event_resolver::outcome_available(&sim, outcome)
                    && event_resolver::outcome_affordable(&sim, outcome),
                "{} chose unavailable/unaffordable outcome {} for {id}",
                policy.label(),
                outcome.id
            );
        }
    }
}

#[test]
#[ignore = "release analysis: deterministic full-voyage matrix"]
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
    let expected_cells = data.contracts.len() * data.legacies.len() * Loadout::ALL.len() * Policy::ALL.len();
    assert_eq!(aggregates.len(), expected_cells);
    assert!(aggregates.iter().all(|a| a.runs == SEEDS as u32));

    let output_dir = std::env::var_os("STELLAR_BALANCE_OUTPUT")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| {
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("target")
                .join("release-balance")
        });
    std::fs::create_dir_all(&output_dir).unwrap();
    std::fs::write(output_dir.join("balance_matrix.csv"), csv(&aggregates)).unwrap();
    std::fs::write(
        output_dir.join("balance_report.md"),
        markdown(&data, &aggregates),
    )
    .unwrap();
}
