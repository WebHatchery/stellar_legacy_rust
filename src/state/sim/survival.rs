//! Serializable vessel-loss and emergency-recovery state.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TerminalReason {
    DynastyExtinction,
    HullLoss,
    PopulationLoss,
    LifeSupportFailure,
}

impl TerminalReason {
    pub fn label(self) -> &'static str {
        match self {
            Self::DynastyExtinction => "DYNASTY EXTINCTION",
            Self::HullLoss => "VESSEL LOST · HULL FAILURE",
            Self::PopulationLoss => "VESSEL LOST · NO POPULATION",
            Self::LifeSupportFailure => "VESSEL LOST · LIFE SUPPORT FAILURE",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TerminalOutcome {
    pub reason: TerminalReason,
    pub month_clock: u32,
    pub evidence: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct SurvivalState {
    pub air_zero_months: u32,
    pub emergency_used: bool,
    pub warning_active: bool,
    pub warning_reviewed: bool,
    pub migration_notice: Option<String>,
}
