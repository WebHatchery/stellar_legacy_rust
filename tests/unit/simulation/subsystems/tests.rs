// Subsystem tests, split by what the module under test is being asked to do.
// The two shared fixtures live here.

use super::*;
use crate::state::sim::founding_faction_ids;

mod effects {
    include!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/unit/simulation/subsystems/tests/effects.rs"));
}
mod life_support {
    include!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/unit/simulation/subsystems/tests/life_support.rs"));
}
mod verbs {
    include!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/unit/simulation/subsystems/tests/verbs.rs"));
}

fn campaign(seed: u64) -> (GameData, SimState) {
    let data = GameData::load().unwrap();
    let picks = founding_faction_ids(&data);
    let sim = SimState::new_campaign(&data, "preservers", seed, &picks);
    (data, sim)
}

fn data_swing() -> f32 {
    GameData::load()
        .unwrap()
        .config
        .subsystems
        .engineering_decay_swing
}
