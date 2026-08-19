use super::*;

fn entry(outcome: &str, score: f32, years: u32) -> ChronicleEntry {
    ChronicleEntry {
        completed_year: years,
        contract_name: "Test Writ".to_owned(),
        objective: "Survey".to_owned(),
        legacy_id: "preservers".to_owned(),
        leader_name: "The Test Captain".to_owned(),
        generation: 1,
        score,
        outcome: outcome.to_owned(),
        duration_years: years,
    }
}

#[test]
fn stats_summarize_the_persisted_voyages_without_rounding_the_record() {
    let chronicle = ChronicleStore {
        entries: vec![
            entry("Complete", 0.95, 40),
            entry("Partial", 0.70, 60),
            entry("Complete", 0.85, 80),
        ],
    };

    let stats = chronicle.stats();
    assert_eq!(stats.voyages, 3);
    assert_eq!(stats.completed, 2);
    assert_eq!(stats.years_flown, 180);
    assert!((stats.average_score - 2.5 / 3.0).abs() < f32::EPSILON);
}

#[test]
fn an_empty_chronicle_has_zeroed_stats() {
    assert_eq!(
        ChronicleStore::default().stats(),
        ChronicleStats {
            voyages: 0,
            completed: 0,
            years_flown: 0,
            average_score: 0.0,
        }
    );
}
