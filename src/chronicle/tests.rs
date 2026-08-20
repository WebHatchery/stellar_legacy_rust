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
        command_posture: crate::state::sim::CommandPosture::Steady,
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

#[test]
fn old_chronicle_entries_default_to_steady_and_new_ones_keep_their_posture() {
    let old_json = r#"{
        "completed_year": 40,
        "contract_name": "Old Writ",
        "objective": "Survey",
        "legacy_id": "preservers",
        "leader_name": "Old Captain",
        "generation": 1,
        "score": 0.8,
        "outcome": "Partial",
        "duration_years": 40
    }"#;
    let old: ChronicleEntry = serde_json::from_str(old_json).unwrap();
    assert_eq!(
        old.command_posture,
        crate::state::sim::CommandPosture::Steady
    );

    let mut civic = entry("Complete", 0.9, 60);
    civic.command_posture = crate::state::sim::CommandPosture::Civic;
    let encoded = serde_json::to_string(&civic).unwrap();
    let decoded: ChronicleEntry = serde_json::from_str(&encoded).unwrap();
    assert_eq!(
        decoded.command_posture,
        crate::state::sim::CommandPosture::Civic
    );
}
