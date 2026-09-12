// Faction tests, split by what they cover. Shared fixtures live here.

use super::*;
use crate::data::factions::FactionLossKind;
use crate::data::GameData;
use crate::state::sim::founding_faction_ids;

mod announce {
    include!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/unit/state/sim/factions/tests/announce.rs"));
}
mod condition {
    include!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/unit/state/sim/factions/tests/condition.rs"));
}
mod recruit {
    include!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/unit/state/sim/factions/tests/recruit.rs"));
}
mod roster {
    include!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/unit/state/sim/factions/tests/roster.rs"));
}
mod sentiment {
    include!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/unit/state/sim/factions/tests/sentiment.rs"));
}
mod spillover {
    include!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/unit/state/sim/factions/tests/spillover.rs"));
}

fn fs(id: &str, members: u32) -> FactionState {
    FactionState {
        faction_id: id.to_owned(),
        members,
        status: FactionStatus::Aboard,
        approval: default_approval(),
        mood_band: 0,
    }
}

fn armed(seed: u64) -> (GameData, SimState, Vec<String>) {
    let data = GameData::load().unwrap();
    let picks = founding_faction_ids(&data);
    let sim = SimState::new_campaign(&data, "preservers", seed, &picks);
    (data, sim, picks)
}
