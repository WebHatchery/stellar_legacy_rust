// Compare reserve changes with the same seeds and maintenance policy.
use super::*;

#[test]
fn smaller_departure_reserve_keeps_starter_and_long_voyages_viable() {
    let data = GameData::load().unwrap();
    let ids: Vec<_> = GameData::sorted_ids(&data.contracts)
        .into_iter()
        .filter(|id| data.contracts.get(id).unwrap().min_renown == 0 || id == "the_deep_camp")
        .collect();
    let mut results = Vec::new();
    for stock in [12_000, data.config.starting_resources.food] {
        let mut runs = 0;
        let mut good = 0;
        let mut extinct = 0;
        let mut end_food = 0;
        for id in &ids {
            for legacy in GameData::sorted_ids(&data.legacies) {
                for seed in 0..16 {
                    let mut sim = SimState::new_campaign(
                        &data,
                        &legacy,
                        seed,
                        &crate::state::sim::founding_faction_ids(&data),
                    );
                    sim.resources.food = stock;
                    let outcome = play_mission(
                        &mut sim,
                        &data,
                        id,
                        data.contracts.get(id).unwrap().target_duration_years + 160,
                    );
                    runs += 1;
                    good += usize::from(outcome.completed && outcome.final_score >= 0.45);
                    extinct += usize::from(outcome.extinct);
                    end_food += sim.resources.food;
                    assert!(
                        outcome.completed || outcome.extinct,
                        "{id}/{legacy}/{seed} stalled"
                    );
                }
            }
        }
        eprintln!("Starting food {stock}: {good}/{runs} score >= .45; {extinct} extinctions; mean ending food {}", end_food / runs as i64);
        results.push((runs, good, extinct));
    }
    assert!(
        results[1].1 + results[0].0 / 20 >= results[0].1,
        "reserve change loses over five percentage points of viable outcomes"
    );
    assert!(
        results[1].2 <= results[0].2 + results[0].0 / 100,
        "reserve change materially increases extinction"
    );
}
