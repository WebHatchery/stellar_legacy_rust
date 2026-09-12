//! Stateless simulation services (GDD §11). Each module receives state and
//! returns results; none of them touch UI or rendering.

pub mod advice;
pub mod approach;
#[allow(dead_code, unused_imports)]
pub mod autoplay {
    include!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/unit/simulation/autoplay.rs"
    ));
}
#[allow(dead_code, unused_imports)]
mod balance {
    include!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/unit/simulation/balance.rs"
    ));
}
pub mod command;
pub mod contract;
pub mod crew;
pub mod culture;
pub mod debrief;
pub mod event_resolver;
pub mod homecoming;
pub mod institutions;
pub mod issues;
pub mod legacy;
pub mod market;
pub mod memory;
pub mod mortality;
pub mod projects;
pub mod readiness;
pub mod ship;
pub mod subsystems;
pub mod succession;
pub mod survival;
pub mod tick;
