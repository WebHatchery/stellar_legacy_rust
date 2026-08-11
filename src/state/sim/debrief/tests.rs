use super::*;

#[test]
fn a_metric_contributes_its_weight_only_when_it_meets_target() {
    let metric = |current, target, weight| DebriefMetric {
        name: "m".to_owned(),
        current,
        target,
        weight,
    };
    // Met exactly: the full weight.
    assert!((metric(0.9, 0.9, 0.3).contribution() - 0.3).abs() < 1e-6);
    // Half way there: half the weight.
    assert!((metric(0.45, 0.9, 0.3).contribution() - 0.15).abs() < 1e-6);
    // Overshooting does not earn more than the weight — the same clamp the
    // scorer applies, so the debrief's arithmetic matches the band shown.
    assert!((metric(2.0, 0.9, 0.3).contribution() - 0.3).abs() < 1e-6);
    // A zero target cannot be divided by; it counts as met.
    assert!((metric(0.0, 0.0, 0.25).contribution() - 0.25).abs() < 1e-6);
}

#[test]
fn the_report_summarizes_its_own_tallies() {
    let mut debrief = VoyageDebrief {
        milestones: vec![
            DebriefMilestone {
                name: "Departure burn complete".to_owned(),
                reached: true,
            },
            DebriefMilestone {
                name: "Halfway beacon passed".to_owned(),
                reached: true,
            },
            DebriefMilestone {
                name: "Colony charter signed".to_owned(),
                reached: false,
            },
        ],
        population_start: 900,
        population_end: 845,
        ..Default::default()
    };
    assert_eq!(debrief.milestones_reached(), (2, 3));
    assert_eq!(debrief.population_change(), -55);
    assert!(debrief.unpaid(), "a default report has paid nothing");
    debrief.payout.credits = 12_000;
    assert!(!debrief.unpaid());
}

#[test]
fn an_older_report_shape_loads_with_defaults() {
    // The record is `serde(default)` throughout so a save written by an
    // earlier build reads back as a sparse report rather than a hard error
    // that would cost the player the campaign.
    let debrief: VoyageDebrief =
        serde_json::from_str(r#"{"contract_name":"The Long Tow","score":0.62}"#).unwrap();
    assert_eq!(debrief.contract_name, "The Long Tow");
    assert!((debrief.score - 0.62).abs() < 1e-6);
    assert!(debrief.commanders.is_empty());
    assert_eq!(debrief.milestones_reached(), (0, 0));
}
