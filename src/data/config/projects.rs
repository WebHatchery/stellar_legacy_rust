//! Shared tuning for the ship-work queue and vessel survival contract.

use crate::data::ResourceDelta;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ProjectTuning {
    pub concurrent_slots: u8,
    pub waiting_cap: u8,
    pub pause_grace_months: u32,
    pub pause_debt_fraction_per_month: f32,
    pub pause_debt_cap_fraction: f32,
    pub cancellation_refund_fraction: f32,
    pub maximum_hydroponics_bonus: f32,
    pub maintenance_condition_threshold: f32,
    pub maintenance_due_months: u32,
    pub overdue_hull_damage: f32,
    pub overdue_life_support_damage: f32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct SurvivalConfig {
    pub air_grace_months: u32,
    pub critical_warning_threshold: f32,
    pub emergency_air_gain: f32,
    pub emergency_resource_cost: ResourceDelta,
    pub emergency_parts_cost: i64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ReadinessConfig {
    pub strong_threshold: f32,
    pub stable_threshold: f32,
    pub vulnerable_threshold: f32,
    pub recovery_hysteresis: f32,
}
