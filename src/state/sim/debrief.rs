//! The record a voyage leaves behind: the beats worth remembering, captured as
//! they happen, and the sealed homecoming report the debrief screen reads.
//!
//! Both exist because the running state cannot answer "what happened on this
//! voyage" once it is over. The ship's log is trimmed to `log_limit` (200
//! lines), so a 300-year crossing has forgotten its own departure by the time
//! it docks; `sim.contract` is cleared the moment a charter concludes, taking
//! the metrics and milestones with it. So the notable beats accrue onto the
//! active contract while it flies, and completion seals a snapshot that
//! survives the contract's own end (and a save/quit in the middle of reading
//! it).

use crate::data::ResourceDelta;
use crate::state::sim::{dynasty::Reign, InstitutionRecord, Obligation};
use serde::{Deserialize, Serialize};

/// What kind of beat a highlight records — the debrief tags each line with
/// this, and the capture cap keeps the rarer kinds in preference to the common
/// one.
///
/// Captaincies are deliberately absent: the debrief's chain-of-command column
/// names every captain a charter passed through in more detail than a log line
/// could, and a full-length charter changes captains often enough that logging
/// each one crowded out the council's decisions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HighlightKind {
    /// A charter milestone came up on the timeline.
    Milestone,
    /// The council was asked, and answered — the player's own decision.
    Decision,
    /// The charter's phase turned: outbound, on station, homeward.
    Phase,
}

impl HighlightKind {
    /// Short tag shown at the head of the line in the debrief's log column.
    pub fn tag(self) -> &'static str {
        match self {
            HighlightKind::Milestone => "MARK",
            HighlightKind::Decision => "COUNCIL",
            HighlightKind::Phase => "LEG",
        }
    }
}

/// One remembered beat of a voyage, stamped with the month it happened in.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VoyageHighlight {
    pub year: u32,
    pub month: u32,
    pub kind: HighlightKind,
    pub text: String,
}

/// One success metric as it stood at the homecoming, with the target it was
/// scored against — the debrief shows the arithmetic behind the final band
/// rather than only its verdict.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DebriefMetric {
    pub name: String,
    pub current: f32,
    pub target: f32,
    pub weight: f32,
}

impl DebriefMetric {
    /// The metric's own contribution to the score, on the same
    /// `min(1, current/target) * weight` terms `contract::score_success` uses.
    pub fn contribution(&self) -> f32 {
        if self.target <= 0.0 {
            return self.weight;
        }
        (self.current / self.target).clamp(0.0, 1.0) * self.weight
    }
}

/// One charter milestone and whether the voyage actually reached it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DebriefMilestone {
    pub name: String,
    pub reached: bool,
}

/// The sealed homecoming report. Written when a charter concludes and cleared
/// when the player dismisses the debrief screen; serialized so quitting
/// mid-read and loading back returns to it rather than silently swallowing the
/// voyage's only summary.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct VoyageDebrief {
    pub contract_name: String,
    pub objective: String,
    /// `SuccessLevel::label()` for the band the score fell in.
    pub outcome: String,
    pub score: f32,
    pub began_year: u32,
    pub ended_year: u32,
    /// Full years the charter ran — its own clock, not the campaign's.
    pub duration_years: u32,
    /// Dynasty generations that turned over between launch and homecoming.
    pub generations: u32,
    /// What the charter actually paid, after the objective proration and the
    /// ship's reputation multiplier — the number the player can check against
    /// the writ they accepted.
    pub payout: ResourceDelta,
    pub metrics: Vec<DebriefMetric>,
    pub milestones: Vec<DebriefMilestone>,
    pub highlights: Vec<VoyageHighlight>,
    /// Every captain who held the chair between launch and homecoming, oldest
    /// first. Empty only on a save written before the reign roster existed.
    pub commanders: Vec<Reign>,
    /// Duties created, inherited, or resolved during this voyage.
    pub obligations: Vec<Obligation>,
    /// Appointments, losses, and preservation work recorded during the voyage.
    #[serde(default)]
    pub institutions: Vec<InstitutionRecord>,
    pub population_start: u32,
    pub population_end: u32,
    /// The authored homecoming prose for this outcome band, if the pool had a
    /// line for it.
    pub homecoming_line: Option<String>,
    /// The lasting capability a completed charter left the ship, if any.
    pub legacy_line: Option<String>,
}

impl Default for VoyageDebrief {
    fn default() -> Self {
        Self {
            contract_name: String::new(),
            objective: String::new(),
            outcome: String::new(),
            score: 0.0,
            began_year: 0,
            ended_year: 0,
            duration_years: 0,
            generations: 0,
            payout: ResourceDelta::default(),
            metrics: Vec::new(),
            milestones: Vec::new(),
            highlights: Vec::new(),
            commanders: Vec::new(),
            obligations: Vec::new(),
            institutions: Vec::new(),
            population_start: 0,
            population_end: 0,
            homecoming_line: None,
            legacy_line: None,
        }
    }
}

impl VoyageDebrief {
    /// Milestones reached over milestones offered — the headline the timeline
    /// column leads with.
    pub fn milestones_reached(&self) -> (usize, usize) {
        (
            self.milestones.iter().filter(|m| m.reached).count(),
            self.milestones.len(),
        )
    }

    /// Net head count across the voyage: positive if the ship came home with
    /// more people aboard than it left with.
    pub fn population_change(&self) -> i64 {
        self.population_end as i64 - self.population_start as i64
    }

    /// True when the charter paid nothing at all — a total default, which the
    /// payout column states outright rather than showing five zeroes.
    pub fn unpaid(&self) -> bool {
        let p = &self.payout;
        p.credits == 0 && p.energy == 0 && p.minerals == 0 && p.food == 0 && p.influence == 0
    }
}

#[cfg(test)]
mod tests;
