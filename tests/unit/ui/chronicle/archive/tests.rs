use super::*;
use crate::state::sim::{HomecomingChoice, HomecomingFocus, HomecomingRecoveryRecord};

#[test]
fn recovery_record_heading_keeps_the_ledger_scannable() {
    let record = HomecomingRecoveryRecord {
        year: 12,
        focus: HomecomingFocus::Obligation,
        choice: HomecomingChoice::HonorPromise,
        target_label: "The First Founding Compact".to_string(),
        note: "The promise was honored.".to_string(),
    };

    assert_eq!(
        recovery_record_heading(&record),
        "Year 12 · THE LEDGER · HONOR THE PROMISE"
    );
}
