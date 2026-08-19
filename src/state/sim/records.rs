//! Persistent records of consequential council decisions.
//!
//! A record stores one authoritative deed plus interpretations. The fact is
//! never recomputed from the accounts, so conflicting memories cannot alter
//! simulation history (Release 6 — Command Archive).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AffectedAccount {
    /// Display name captured when the decision was made. It survives a later
    /// schism, merger, or content rename exactly as the contemporary record did.
    pub people: String,
    pub account: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DecisionRecord {
    pub year: u32,
    pub month: u32,
    pub event_id: String,
    pub event_title: String,
    pub outcome_id: String,
    pub outcome_label: String,
    /// The authoritative mechanical history, copied from the outcome log.
    pub fact: String,
    /// Captain who held the first chair when the deed was entered.
    pub captain: String,
    #[serde(default)]
    pub official_account: String,
    #[serde(default)]
    pub dynasty_account: String,
    #[serde(default)]
    pub affected_accounts: Vec<AffectedAccount>,
}
