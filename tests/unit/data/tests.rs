// Data-layer tests, split by the content area each one guards.
//
// The derivations more than one area needs live here as shared fixtures.

use std::collections::HashSet;

use super::*;

mod charters {
    include!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/unit/data/tests/charters.rs"));
}
mod content {
    include!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/unit/data/tests/content.rs"));
}
mod economy {
    include!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/unit/data/tests/economy.rs"));
}
mod event_gates {
    include!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/unit/data/tests/event_gates.rs"));
}
mod parts {
    include!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/unit/data/tests/parts.rs"));
}
mod peoples {
    include!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/unit/data/tests/peoples.rs"));
}
mod records {
    include!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/unit/data/tests/records.rs"));
}
mod ship_systems {
    include!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/unit/data/tests/ship_systems.rs"));
}
mod skeleton {
    include!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/unit/data/tests/skeleton.rs"));
}
mod voice {
    include!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/unit/data/tests/voice.rs"));
}

/// Every event family that has authored events behind it. A beat or a bias that
/// names a family outside this set would draw from an empty pool.
fn authored_families(data: &GameData) -> HashSet<&String> {
    data.events.iter().map(|(_, e)| &e.family).collect()
}

/// Every reputation trait that something in the content actually nudges: an
/// event outcome, a charter's completion reward, or its abandonment penalty. A
/// gate naming a trait outside this set could never be met.
fn reputation_traits_produced(data: &GameData) -> HashSet<&String> {
    data.events
        .iter()
        .flat_map(|(_, e)| e.outcomes.iter())
        .flat_map(|o| o.reputation_deltas.iter().map(|r| &r.id))
        .chain(
            data.contracts
                .iter()
                .flat_map(|(_, c)| c.completion_reward.reputation_deltas.iter().map(|r| &r.id)),
        )
        .chain(
            data.contracts
                .iter()
                .flat_map(|(_, c)| c.abandonment.reputation_deltas.iter().map(|r| &r.id)),
        )
        .collect()
}
