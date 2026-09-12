use super::*;
use crate::data::events::IssueSpec;
use crate::data::GameData;
use crate::state::sim::IssueSeverity;

fn campaign() -> (GameData, crate::state::sim::SimState) {
    let data = GameData::load().expect("embedded data");
    let ids = crate::state::sim::founding_faction_ids(&data);
    (
        data.clone(),
        crate::state::sim::SimState::new_campaign(&data, "preservers", 8, &ids),
    )
}

#[test]
fn repeated_aftermath_merges_instead_of_duplicating() {
    let (_data, mut sim) = campaign();
    let spec = IssueSpec {
        food_production_penalty: 0.0,
        id: "crop".to_owned(),
        target: "agriculture".to_owned(),
        severity: IssueSeverity::Vulnerable,
        due_months: Some(4),
        recovery_project_ids: vec!["sterilise_damaged_growing_systems".to_owned()],
    };
    record_event_issue(&mut sim, &spec, "test");
    record_event_issue(
        &mut sim,
        &IssueSpec {
            severity: IssueSeverity::Critical,
            ..spec
        },
        "test",
    );
    assert_eq!(sim.issues.active.len(), 1);
    assert_eq!(sim.issues.active[0].severity, IssueSeverity::Critical);
}
