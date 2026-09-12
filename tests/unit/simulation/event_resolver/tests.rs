// Resolver tests, split by what the gate or outcome under test keys on.
// The one shared fixture lives here.

use super::*;
use crate::data::events::EventCategory;
use crate::data::GameData;
use crate::state::sim::SimState;

mod aftermath {
    include!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/unit/simulation/event_resolver/tests/aftermath.rs"));
}
mod chains {
    include!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/unit/simulation/event_resolver/tests/chains.rs"));
}
mod charters {
    include!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/unit/simulation/event_resolver/tests/charters.rs"));
}
mod complications {
    include!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/unit/simulation/event_resolver/tests/complications.rs"));
}
mod gates {
    include!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/unit/simulation/event_resolver/tests/gates.rs"));
}
mod outcome {
    include!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/unit/simulation/event_resolver/tests/outcome.rs"));
}
mod peoples {
    include!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/unit/simulation/event_resolver/tests/peoples.rs"));
}
mod provisioning {
    include!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/unit/simulation/event_resolver/tests/provisioning.rs"));
}
mod reputation {
    include!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/unit/simulation/event_resolver/tests/reputation.rs"));
}
mod scheduled {
    include!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/unit/simulation/event_resolver/tests/scheduled.rs"));
}

fn impact_cfg() -> RealTimeConfig {
    RealTimeConfig {
        seconds_per_month: 5.0,
        decision_timeout_secs: 30.0,
        impact_variance: 0.4,
        impact_min_magnitude_for_range: 20,
    }
}
