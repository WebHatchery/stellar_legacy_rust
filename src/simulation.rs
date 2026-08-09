//! Stateless simulation services (GDD §11). Each module receives state and
//! returns results; none of them touch UI or rendering.

#[cfg(test)]
pub mod autoplay;
#[cfg(test)]
mod balance;
pub mod contract;
pub mod crew;
pub mod debrief;
pub mod event_resolver;
pub mod institutions;
pub mod legacy;
pub mod market;
pub mod mortality;
pub mod ship;
pub mod subsystems;
pub mod succession;
pub mod tick;
