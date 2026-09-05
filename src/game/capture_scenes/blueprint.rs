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

impl Game {
    pub(super) fn seed_drydock_ship(&mut self, scene: &str) {
        self.ship_modules_tab.set(scene == "ship_modules");
        self.ship_preview.set((0, -100.0));
        match scene {
            "ship" | "ship_refitted" => {
                let mut sim = SimState::new_campaign(
                    &self.data,
                    "preservers",
                    0xC0FFEE,
                    &crate::state::sim::founding_faction_ids(&self.data),
                );
                if scene == "ship_refitted" {
                    sim.ship.hull = "generation_ark".to_owned();
                    sim.ship.engine = "warp_coil".to_owned();
                    sim.ship.weapon = Some("mass_driver".to_owned());
                    sim.subsystems.get_mut("agriculture").unwrap().tier = 3;
                }
                // Seed a salvage hold so the SALVAGE HOLD strip shows (M4.4), incl.
                // a mission-reward part so its MISSION REWARD tag + install state show.
                sim.ship.salvage = vec![
                    "mass_driver".to_owned(),
                    "solar_sail".to_owned(),
                    "singularity_lance".to_owned(),
                ];
                let mut gameplay = GameplayState::new(sim);
                gameplay.screen = Screen::ShipBuilder;
                self.state = crate::state::GameState::Gameplay(Box::new(gameplay));
            }
            "ship_modules" => {
                // The SHIP tab's MODULES sub-tab: subsystem version ladders. Vary
                // the fitted tiers so passed / installed / next-to-buy rows all show.
                let mut sim = SimState::new_campaign(
                    &self.data,
                    "preservers",
                    0xC0FFEE,
                    &crate::state::sim::founding_faction_ids(&self.data),
                );
                for (id, tier) in [
                    ("agriculture", 3),
                    ("engineering_bay", 3),
                    ("medical_bay", 2),
                    ("life_support_habitat", 3),
                    ("security", 0),
                ] {
                    if let Some(s) = sim.subsystems.get_mut(id) {
                        s.tier = tier;
                    }
                }
                // One mission-reward version recovered (engineering's Nanolathe
                // Forge → INSTALL · RECOVERED), one still locked (life support's
                // Voidsealed Biosphere → MISSION REWARD).
                sim.ship.unlocked_fittings = vec!["nanolathe_forge".to_owned()];
                let mut gameplay = GameplayState::new(sim);
                gameplay.screen = Screen::ShipBuilder;
                self.ship_modules_tab.set(true);
                self.state = crate::state::GameState::Gameplay(Box::new(gameplay));
            }
            _ => {}
        }
    }
}
