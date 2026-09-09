//! Authored ship-work catalogue (GDD §5.7).
//!
//! The catalogue describes intent, cost, eligibility, and staged effects. Runtime
//! progress and accounting live in `state::sim::projects`; this module contains no
//! mutable campaign state.

use crate::data::ResourceDelta;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectKind {
    ServiceSubsystem,
    RestoreHull,
    OverhaulLifeSupport,
    TrainReplacementCohort,
    OptimiseHydroponics,
    RestoreCrewQuarters,
    SteriliseGrowingSystems,
    EstablishSeedProgramme,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectTarget {
    None,
    Subsystem,
    Agriculture,
    Social,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ProjectCost {
    pub credits: i64,
    pub energy: i64,
    pub minerals: i64,
    pub food: i64,
    pub influence: i64,
    pub spare_parts: i64,
}

impl ProjectCost {
    pub fn resource_delta(self) -> ResourceDelta {
        ResourceDelta {
            credits: -self.credits,
            energy: -self.energy,
            minerals: -self.minerals,
            food: -self.food,
            influence: -self.influence,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ProjectEffect {
    pub condition_gain: f32,
    pub hull_gain: f32,
    pub life_support_gain: f32,
    pub knowledge_gain: f32,
    pub food_production_bonus: f32,
    pub morale_recovery: f32,
    pub unity_recovery: f32,
    pub capability: Option<String>,
}

/// A data-authored project choice. Costs are positive amounts; the simulation
/// converts them into a signed deduction when work starts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectDefinition {
    pub id: String,
    pub name: String,
    pub description: String,
    pub kind: ProjectKind,
    pub target: ProjectTarget,
    pub duration_months: u32,
    #[serde(default = "default_stage_count")]
    pub stage_count: u32,
    #[serde(default)]
    pub divisible: bool,
    #[serde(default)]
    pub cost: ProjectCost,
    #[serde(default = "default_refundable")]
    pub refundable: ProjectCost,
    /// Knowledge floor used by service and preparation projects. Training has
    /// no floor so expertise loss never creates a circular recovery lock.
    #[serde(default)]
    pub knowledge_required: f32,
    /// Optional target condition ceiling: useful work is not offered on a sound
    /// target, and the target is rechecked when a queued job starts.
    #[serde(default)]
    pub target_condition_below: Option<f32>,
    #[serde(default)]
    pub requires_issue: Option<String>,
    #[serde(default)]
    pub requires_capability: Option<String>,
    #[serde(default)]
    pub effect: ProjectEffect,
}

fn default_stage_count() -> u32 {
    1
}

impl ProjectDefinition {
    pub fn stage_count(&self) -> u32 {
        self.stage_count.max(1)
    }

    pub fn validates(&self) -> Result<(), String> {
        if self.id.trim().is_empty() || self.name.trim().is_empty() {
            return Err("project ids and names cannot be empty".to_owned());
        }
        if self.duration_months == 0 {
            return Err(format!("project '{}' has no duration", self.id));
        }
        if self.stage_count == 0 {
            return Err(format!("project '{}' has no stages", self.id));
        }
        if !(0.0..=1.0).contains(&self.knowledge_required) {
            return Err(format!("project '{}' has invalid knowledge floor", self.id));
        }
        Ok(())
    }
}

fn default_refundable() -> ProjectCost {
    ProjectCost {
        credits: 1,
        energy: 1,
        minerals: 1,
        food: 1,
        influence: 0,
        spare_parts: 1,
    }
}
