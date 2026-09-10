use super::*;
use crate::state::sim::{HomecomingChoice, HomecomingFocus, HomecomingRecovery};

#[test]
fn accounting_keeps_a_recorded_recovery_visible() {
    let mut report = VoyageDebrief::default();
    report.recovery = Some(HomecomingRecovery {
        focus: HomecomingFocus::Institutions,
        target_id: "water".to_owned(),
        target_label: "Water stewardship".to_owned(),
        situation: "The craft is fading.".to_owned(),
        resolved: true,
        choice: Some(HomecomingChoice::PreserveCraft),
    });

    let text = accounting(&report);
    assert!(text.contains("HOMECOMING RECOVERY"));
    assert!(text.contains("Recorded · LIVING CRAFT · PRESERVE THE CRAFT"));
    assert!(text.contains("Target: Water stewardship"));
}

#[test]
fn accounting_explains_a_pending_recovery() {
    let mut report = VoyageDebrief::default();
    report.recovery = Some(HomecomingRecovery {
        target_label: "Old promise".to_owned(),
        situation: "The promise still waits.".to_owned(),
        ..Default::default()
    });

    let text = recovery_accounting(&report);
    assert!(text.contains("Pending review"));
    assert!(text.contains("The promise still waits."));
}
