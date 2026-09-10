//! Stateless simulation services (GDD §11). Each module receives state and
//! returns results; none of them touch UI or rendering.

pub mod advice;
pub mod approach;
#[cfg(test)]
pub mod autoplay;
#[cfg(test)]
mod balance;
pub mod command;
pub mod contract;
pub mod crew;
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
