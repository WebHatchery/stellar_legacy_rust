//! Underway ship-blueprint capture states.

use super::super::Game;
use crate::simulation::contract;
use crate::state::{GameplayState, Screen, SimState};

impl Game {
    /// A mid-mission demo state for the SHIP blueprint, optionally on a named hull
    /// class. Mixed subsystem tiers and wear exercise every highlight state (a
    /// proud tier-3 module, a failing one in alert-red, a mid one), a weapon is
    /// fitted, and a part sits in the salvage hold.
    pub(super) fn underway_blueprint_state(&self, hull: Option<&str>) -> crate::state::GameState {
        let mut sim = SimState::new_campaign(
            &self.data,
            "adaptors",
            0xC0FFEE,
            &crate::state::sim::founding_faction_ids(&self.data),
        );
        if let Some(hull) = hull {
            sim.ship.hull = hull.to_owned();
        }
        if let Some(template) = self.data.contracts.get("deep_vein_survey") {
            sim.contract = Some(contract::start_contract(template, &sim));
        }
        sim.ship.hull_integrity = 0.62;
        sim.ship.life_support = 0.74;
        sim.ship.fuel = 0.4;
        sim.ship.weapon = Some("mass_driver".to_owned());
        sim.ship.salvage = vec!["solar_sail".to_owned()];
        if let Some(s) = sim.subsystems.get_mut("agriculture") {
            s.tier = 3;
            s.condition = 0.95;
        }
        if let Some(s) = sim.subsystems.get_mut("medical_bay") {
            s.tier = 1;
            s.condition = 0.28;
        }
        if let Some(s) = sim.subsystems.get_mut("engineering_bay") {
            s.tier = 2;
            s.condition = 0.55;
        }
        let mut gameplay = GameplayState::new(sim);
        gameplay.screen = Screen::ShipBuilder;
        crate::state::GameState::Gameplay(Box::new(gameplay))
    }
}
