//! Persistent state for the choice made after a voyage returns.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HomecomingFocus {
    Cohesion,
    Institutions,
    Obligation,
}

impl HomecomingFocus {
    pub fn label(self) -> &'static str {
        match self {
            Self::Cohesion => "COMMON LIFE",
            Self::Institutions => "LIVING CRAFT",
            Self::Obligation => "THE LEDGER",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HomecomingChoice {
    ReconcilePeople,
    PreserveCraft,
    HonorPromise,
    Defer,
}

impl HomecomingChoice {
    pub const ALL: [Self; 4] = [
        Self::ReconcilePeople,
        Self::PreserveCraft,
        Self::HonorPromise,
        Self::Defer,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::ReconcilePeople => "MEND THE COMMONS",
            Self::PreserveCraft => "PRESERVE THE CRAFT",
            Self::HonorPromise => "HONOR THE PROMISE",
            Self::Defer => "DEFER RECOVERY",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Self::ReconcilePeople => {
                "Spend political capital and treasury to restore morale, unity, and trust."
            }
            Self::PreserveCraft => {
                "Fund a school for the weakest discipline and carry its knowledge forward."
            }
            Self::HonorPromise => {
                "Pay down the oldest active duty and show its beneficiary the ledger still matters."
            }
            Self::Defer => {
                "Begin the next charter without repair; the unresolved wound will remain visible."
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct HomecomingRecovery {
    pub focus: HomecomingFocus,
    pub target_id: String,
    pub target_label: String,
    pub situation: String,
    pub resolved: bool,
    pub choice: Option<HomecomingChoice>,
}

impl Default for HomecomingRecovery {
    fn default() -> Self {
        Self {
            focus: HomecomingFocus::Cohesion,
            target_id: String::new(),
            target_label: String::new(),
            situation: String::new(),
            resolved: false,
            choice: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HomecomingRecoveryRecord {
    pub year: u32,
    pub focus: HomecomingFocus,
    pub choice: HomecomingChoice,
    pub target_label: String,
    pub note: String,
}

#[cfg(test)]
mod tests;
