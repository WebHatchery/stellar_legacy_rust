//! Serializable Agenda state: queued ship work, staged deliveries, and escrow.

use crate::data::projects::ProjectCost;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectStatus {
    #[default]
    Queued,
    Running,
    Paused,
    Completed,
    Stopped,
    Cancelled,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ProjectAmounts {
    pub credits: f64,
    pub energy: f64,
    pub minerals: f64,
    pub food: f64,
    pub influence: f64,
    pub spare_parts: f64,
}

impl ProjectAmounts {
    pub fn from_cost(cost: ProjectCost) -> Self {
        Self {
            credits: cost.credits as f64,
            energy: cost.energy as f64,
            minerals: cost.minerals as f64,
            food: cost.food as f64,
            influence: cost.influence as f64,
            spare_parts: cost.spare_parts as f64,
        }
    }

    pub fn values(self) -> [f64; 6] {
        [
            self.credits,
            self.energy,
            self.minerals,
            self.food,
            self.influence,
            self.spare_parts,
        ]
    }

    pub fn from_values(v: [f64; 6]) -> Self {
        Self {
            credits: v[0],
            energy: v[1],
            minerals: v[2],
            food: v[3],
            influence: v[4],
            spare_parts: v[5],
        }
    }

    pub fn nonzero(self) -> bool {
        [
            self.credits,
            self.energy,
            self.minerals,
            self.food,
            self.influence,
            self.spare_parts,
        ]
        .into_iter()
        .any(|amount| amount > f64::EPSILON)
    }
}

/// A single queued or delivered intention. Costs are charged into escrow when
/// it starts; only the remaining recoverable escrow can be refunded.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectInstance {
    #[serde(default)]
    pub legacy_single_delivery: bool,
    pub sequence_id: u64,
    pub project_id: String,
    pub target_id: Option<String>,
    pub status: ProjectStatus,
    pub queued_month: u32,
    pub started_month: Option<u32>,
    pub elapsed_months: u32,
    pub delivered_stages: u32,
    pub delivered_months: Vec<u32>,
    /// Pause history for each authored stage. Delivered stages stop ageing, but
    /// resuming does not erase the unfinished stages' grace history.
    #[serde(default)]
    pub stage_pause_months: Vec<u32>,
    /// Material deterioration accumulated by each unfinished stage.
    #[serde(default)]
    pub stage_deterioration: Vec<ProjectAmounts>,
    pub original_cost: ProjectAmounts,
    pub remaining_escrow: ProjectAmounts,
    pub committed_cost: ProjectAmounts,
    pub paused_months: u32,
    pub restoration_debt: ProjectAmounts,
    pub lifetime_deterioration: ProjectAmounts,
    pub pause_reason: Option<String>,
    pub stop_reason: Option<String>,
}

impl ProjectInstance {
    pub fn queued(
        sequence_id: u64,
        project_id: &str,
        target_id: Option<String>,
        month: u32,
    ) -> Self {
        Self {
            legacy_single_delivery: false,
            sequence_id,
            project_id: project_id.to_owned(),
            target_id,
            status: ProjectStatus::Queued,
            queued_month: month,
            started_month: None,
            elapsed_months: 0,
            delivered_stages: 0,
            delivered_months: Vec::new(),
            stage_pause_months: Vec::new(),
            stage_deterioration: Vec::new(),
            original_cost: ProjectAmounts::default(),
            remaining_escrow: ProjectAmounts::default(),
            committed_cost: ProjectAmounts::default(),
            paused_months: 0,
            restoration_debt: ProjectAmounts::default(),
            lifetime_deterioration: ProjectAmounts::default(),
            pause_reason: None,
            stop_reason: None,
        }
    }

    pub fn is_waiting(&self) -> bool {
        matches!(self.status, ProjectStatus::Queued | ProjectStatus::Paused)
    }

    pub fn progress(&self, duration_months: u32) -> f32 {
        (self.elapsed_months as f32 / duration_months.max(1) as f32).clamp(0.0, 1.0)
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct ProjectState {
    pub jobs: Vec<ProjectInstance>,
    pub capabilities: Vec<String>,
    pub next_sequence_id: u64,
    pub hydroponics_bonus: f32,
    /// Fractional change retained when settling against integer ship stores.
    pub settlement_balance: ProjectAmounts,
}

impl ProjectState {
    pub fn has_capability(&self, id: &str) -> bool {
        self.capabilities.iter().any(|capability| capability == id)
    }

    pub fn active_count(&self) -> usize {
        self.jobs
            .iter()
            .filter(|job| job.status == ProjectStatus::Running)
            .count()
    }

    pub fn waiting_count(&self) -> usize {
        self.jobs.iter().filter(|job| job.is_waiting()).count()
    }

    pub fn find(&self, sequence_id: u64) -> Option<&ProjectInstance> {
        self.jobs.iter().find(|job| job.sequence_id == sequence_id)
    }

    pub fn find_mut(&mut self, sequence_id: u64) -> Option<&mut ProjectInstance> {
        self.jobs
            .iter_mut()
            .find(|job| job.sequence_id == sequence_id)
    }
}
