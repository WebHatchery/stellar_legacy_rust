//! Heritage modifiers (GDD §7, PLAN item 7).
//!
//! The cross-playthrough [`ChronicleStore`](crate::chronicle::ChronicleStore)
//! outlives any single save. Its recorded contract scores accumulate into a
//! *renown* total, which places a new dynasty in a heritage tier that grants a
//! small head start. Deterministic (derived from the persisted Chronicle, no
//! RNG); applied once at campaign creation so within-campaign determinism holds.

use crate::chronicle::ChronicleStore;
use crate::data::{HeritageTier, ResourceDelta};
use crate::state::sim::SimState;

/// The heritage a new dynasty inherits from past voyages.
#[derive(Debug, Clone, PartialEq)]
pub struct Heritage {
    pub renown: i64,
    pub tier_name: String,
    pub credits: i64,
    pub influence: i64,
    pub tradition: i32,
}

impl Heritage {
    /// True when this tier actually grants something (i.e. not the base tier).
    pub fn has_bonus(&self) -> bool {
        self.credits != 0 || self.influence != 0 || self.tradition != 0
    }
}

/// Total renown across every recorded contract: each entry contributes its
/// success score scaled to points (a full success ≈ 100).
pub fn renown(chronicle: &ChronicleStore) -> i64 {
    chronicle
        .entries
        .iter()
        .map(|e| (e.score * 100.0).round() as i64)
        .sum::<i64>()
        .max(0)
}

/// Derive the heritage for a new campaign from the Chronicle and the configured
/// tier table (the highest tier whose `min_renown` the renown clears).
pub fn derive(chronicle: &ChronicleStore, tiers: &[HeritageTier]) -> Heritage {
    let renown = renown(chronicle);
    match tiers
        .iter()
        .filter(|t| renown >= t.min_renown)
        .max_by_key(|t| t.min_renown)
    {
        Some(t) => Heritage {
            renown,
            tier_name: t.name.clone(),
            credits: t.credits,
            influence: t.influence,
            tradition: t.tradition,
        },
        None => Heritage {
            renown,
            tier_name: "Founding".to_owned(),
            credits: 0,
            influence: 0,
            tradition: 0,
        },
    }
}

/// The first configured tier above the current renown, independent of input
/// ordering. `None` means the Chronicle has reached the highest inheritance.
pub fn next_tier(renown: i64, tiers: &[HeritageTier]) -> Option<&HeritageTier> {
    tiers
        .iter()
        .filter(|tier| tier.min_renown > renown)
        .min_by_key(|tier| tier.min_renown)
}

/// Grant the heritage bonus to a freshly created campaign.
pub fn apply(sim: &mut SimState, heritage: &Heritage) {
    sim.resources.apply(&ResourceDelta {
        credits: heritage.credits,
        influence: heritage.influence,
        ..Default::default()
    });
    sim.legacy.tradition_points += heritage.tradition;
    if heritage.has_bonus() {
        sim.push_log(format!(
            "Heritage of the {} line: the Chronicle steadies this founding.",
            heritage.tier_name
        ));
    }
}

#[cfg(test)]
mod tests;
