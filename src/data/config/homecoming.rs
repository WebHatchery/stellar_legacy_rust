//! Tunables for the between-voyages recovery decision.

use serde::{Deserialize, Serialize};

/// Costs and bounded effects for the homecoming recovery choice. The recovery
/// reuses the campaign's ordinary resources and institutions; this section
/// only keeps the new intervention balance out of simulation code.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct HomecomingRecoveryConfig {
    pub reconcile_credits: i64,
    pub reconcile_influence: i64,
    pub reconcile_morale: f32,
    pub reconcile_unity: f32,
    pub reconcile_stability: f32,
    pub reconcile_approval: f32,
    pub preserve_knowledge_gain: f32,
    pub honor_credits: i64,
    pub honor_influence: i64,
    pub honor_approval: f32,
    pub defer_morale_loss: f32,
    pub defer_unity_loss: f32,
}
