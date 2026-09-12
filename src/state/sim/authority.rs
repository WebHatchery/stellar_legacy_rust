//! The small, explicit division of authority between the Custodian and a
//! human captain.  The Custodian runs ordinary operations; the captain can
//! object to a strategic posture when the ship's condition makes that choice
//! materially contrary to the captain's priority.

use super::{CommandPosture, DynastyMember, SimState};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CaptainPriority {
    #[default]
    Steady,
    Civic,
    Expeditionary,
}

impl CaptainPriority {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Steady => "STEADY",
            Self::Civic => "CIVIC",
            Self::Expeditionary => "EXPEDITIONARY",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthorityChoice {
    KeepCurrent,
    AcceptCompromise,
    EmergencyOverride,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct MandateState {
    pub routine_operations: bool,
    pub emergency_override: bool,
}

impl Default for MandateState {
    fn default() -> Self {
        Self {
            routine_operations: true,
            // The power exists in the charter, but posture_review still gates
            // it behind a real morale or unity emergency.
            emergency_override: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthorityReview {
    pub proposed: CommandPosture,
    pub compromise: CommandPosture,
    pub captain: String,
    pub priority: CaptainPriority,
    pub reason: String,
    pub emergency_allowed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AuthorityState {
    pub mandate: MandateState,
    pub captain_priority: CaptainPriority,
    pub captain_name: String,
    pub pending_review: Option<AuthorityReview>,
}

impl Default for AuthorityState {
    fn default() -> Self {
        Self {
            mandate: MandateState::default(),
            captain_priority: CaptainPriority::Steady,
            captain_name: String::new(),
            pending_review: None,
        }
    }
}

/// Derive a captain's policy priority from authored identity fields. Unknown
/// or missing traits deliberately fall back to the neutral posture.
pub fn priority_for_member(member: Option<&DynastyMember>) -> CaptainPriority {
    let Some(member) = member else {
        return CaptainPriority::Steady;
    };
    let text = format!("{} {}", member.specialization, member.trait_name).to_lowercase();
    if ["engine", "scout", "bold", "frontier", "risk", "wander"]
        .iter()
        .any(|word| text.contains(word))
    {
        CaptainPriority::Expeditionary
    } else if [
        "medic", "care", "hearth", "keeper", "diplomat", "patient", "preserve",
    ]
    .iter()
    .any(|word| text.contains(word))
    {
        CaptainPriority::Civic
    } else {
        CaptainPriority::Steady
    }
}

/// Refresh only the captain-specific part of the contract after a handover.
/// The Custodian's mandate and history remain intact across succession.
pub fn refresh_captain(sim: &mut SimState) {
    let leader = sim.dynasty.leader();
    sim.authority.captain_name = leader.map(|member| member.name.clone()).unwrap_or_default();
    sim.authority.captain_priority = priority_for_member(leader);
}

/// Return a review when the current captain's documented priority objects to a
/// strategic posture under a concrete ship condition. Routine actions never
/// pass through this function.
pub fn posture_review(sim: &SimState, proposed: CommandPosture) -> Option<AuthorityReview> {
    if sim.contract.is_none() || proposed == sim.command_posture {
        return None;
    }
    let priority = sim.authority.captain_priority;
    let objection = match (priority, proposed) {
        (CaptainPriority::Civic, CommandPosture::Expeditionary)
            if sim.population.morale < 0.65 || sim.population.unity < 0.60 =>
        {
            Some("The captain objects that expeditionary tempo is being proposed while the crew is already carrying a social strain." )
        }
        (CaptainPriority::Expeditionary, CommandPosture::Civic)
            if sim.contract.as_ref().is_some_and(|contract| contract.objective_fraction() < 0.25) =>
        {
            Some("The captain objects that a civic slowdown would abandon the writ before its first quarter is complete." )
        }
        _ => None,
    }?;
    let compromise = match priority {
        CaptainPriority::Civic if proposed == CommandPosture::Expeditionary => {
            CommandPosture::Steady
        }
        CaptainPriority::Expeditionary if proposed == CommandPosture::Civic => {
            CommandPosture::Steady
        }
        _ => CommandPosture::Steady,
    };
    Some(AuthorityReview {
        proposed,
        compromise,
        captain: sim.authority.captain_name.clone(),
        priority,
        reason: objection.to_owned(),
        // An override is a narrow emergency power. A deeply distressed crew
        // makes the compromise the safer answer, so this remains uncommon.
        emergency_allowed: sim.authority.mandate.emergency_override
            && (sim.population.morale < 0.30 || sim.population.unity < 0.25),
    })
}

pub fn complete_review(sim: &mut SimState, choice: AuthorityChoice) -> Option<CommandPosture> {
    let review = sim.authority.pending_review.take()?;
    match choice {
        AuthorityChoice::KeepCurrent => None,
        AuthorityChoice::AcceptCompromise => Some(review.compromise),
        AuthorityChoice::EmergencyOverride if review.emergency_allowed => Some(review.proposed),
        AuthorityChoice::EmergencyOverride => None,
    }
}
#[allow(dead_code, unused_imports)]
mod tests {
    include!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/unit/state/sim/authority/tests.rs"
    ));
}
