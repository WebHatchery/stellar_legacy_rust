//! Cross-playthrough chronicle (GDD §7).
//!
//! Persists outside any save slot so it survives across playthroughs.
//! Records completed contracts and their legacy/command history. The heritage
//! module derives automatic new-dynasty bonuses from these entries.

use crate::state::sim::{CharterApproach, CommandPosture, HomecomingRecoveryRecord};
use macroquad_toolkit::persistence::{
    load_from_slot_with_migration, save_to_slot_with_version, slot_exists,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChronicleEntry {
    pub completed_year: u32,
    pub contract_name: String,
    pub objective: String,
    pub legacy_id: String,
    pub leader_name: String,
    pub generation: u32,
    pub score: f32,
    pub outcome: String,
    /// In-game years the mission ran (PLAN M4.7). Serde-default for old logs.
    #[serde(default)]
    pub duration_years: u32,
    /// The operating philosophy chosen for the voyage. Old Chronicle saves
    /// default to the neutral posture so the archive remains backward compatible.
    #[serde(default)]
    pub command_posture: CommandPosture,
    /// Mission-specific doctrine selected before launch. `None` means an older
    /// Chronicle entry predates charter approaches.
    #[serde(default)]
    pub charter_approach: Option<CharterApproach>,
    /// The between-voyages intervention recorded after the report was read.
    /// Old Chronicle entries have no recovery record.
    #[serde(default)]
    pub homecoming_recovery: Option<HomecomingRecoveryRecord>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ChronicleStore {
    pub entries: Vec<ChronicleEntry>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ChronicleStats {
    pub voyages: usize,
    pub completed: usize,
    pub years_flown: u32,
    pub average_score: f32,
}

impl ChronicleStore {
    pub fn load(game_name: &str, slot: &str, version: &str) -> Self {
        if !slot_exists(game_name, slot) {
            return Self::default();
        }
        load_from_slot_with_migration(game_name, slot, version, |_, value| {
            serde_json::from_value(value.get("data").cloned().unwrap_or(value))
                .map_err(|err| format!("chronicle migration failed: {err}"))
        })
        .unwrap_or_default()
    }

    pub fn save(&self, game_name: &str, slot: &str, version: &str) -> Result<(), String> {
        save_to_slot_with_version(game_name, slot, self, version)
    }

    pub fn record(&mut self, entry: ChronicleEntry) {
        self.entries.push(entry);
    }

    pub fn stats(&self) -> ChronicleStats {
        let voyages = self.entries.len();
        let total_score: f32 = self.entries.iter().map(|entry| entry.score).sum();
        ChronicleStats {
            voyages,
            completed: self
                .entries
                .iter()
                .filter(|entry| entry.outcome.eq_ignore_ascii_case("complete"))
                .count(),
            years_flown: self.entries.iter().map(|entry| entry.duration_years).sum(),
            average_score: if voyages == 0 {
                0.0
            } else {
                total_score / voyages as f32
            },
        }
    }
}
#[allow(dead_code, unused_imports)]
mod tests {
    include!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/unit/chronicle/tests.rs"
    ));
}
