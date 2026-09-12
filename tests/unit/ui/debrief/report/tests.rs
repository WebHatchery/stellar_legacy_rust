use super::*;
use crate::state::sim::{CharterApproach, HomecomingChoice, HomecomingFocus, HomecomingRecovery};

#[test]
fn accounting_keeps_the_approach_effects_with_the_sealed_report() {
    let report = VoyageDebrief {
        approach: CharterApproach::CarryThePeople,
        ..Default::default()
    };

    let text = accounting(&report);
    assert!(text.contains("CHARTER APPROACH\nCARRY THE PEOPLE"));
    assert!(text.contains("WORK -8% · EVENTS -12% · FUEL -4%"));
    assert!(text.contains("ANNUAL PEOPLE +1.0% morale"));
}

#[test]
fn accounting_keeps_a_recorded_recovery_visible() {
    let report = VoyageDebrief {
        recovery: Some(HomecomingRecovery {
            focus: HomecomingFocus::Institutions,
            target_id: "water".to_owned(),
            target_label: "Water stewardship".to_owned(),
            situation: "The craft is fading.".to_owned(),
            resolved: true,
            choice: Some(HomecomingChoice::PreserveCraft),
        }),
        ..Default::default()
    };

    let text = accounting(&report);
    assert!(text.contains("HOMECOMING RECOVERY"));
    assert!(text.contains("Recorded · LIVING CRAFT · PRESERVE THE CRAFT"));
    assert!(text.contains("Target: Water stewardship"));
}

#[test]
fn accounting_explains_a_pending_recovery() {
    let report = VoyageDebrief {
        recovery: Some(HomecomingRecovery {
            target_label: "Old promise".to_owned(),
            situation: "The promise still waits.".to_owned(),
            ..Default::default()
        }),
        ..Default::default()
    };

    let text = recovery_accounting(&report);
    assert!(text.contains("Pending review"));
    assert!(text.contains("The promise still waits."));
}
