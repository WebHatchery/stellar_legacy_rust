//! The game-over capture: a real voyage flown until the ship strands, so the
//! terminal screen photographs a loss the simulation produced.

use super::*;
use crate::simulation::contract;
use crate::state::{GameplayState, SimState};

/// Years under way before the capture empties the tank.
const DRY_AFTER_YEARS: u32 = 12;

impl Game {
    /// Launch the founding charter on a Warp Coil (a drive with no scoop), fly
    /// it under the autoplay answers for a dozen years, then let the tank run
    /// dry mid-crossing. With no scoop and nothing in the salvage hold, the
    /// next Travel month records the stranded loss.
    pub(super) fn fly_to_stranding(&mut self) {
        use crate::simulation::{event_resolver, legacy, tick};

        let mut sim = SimState::new_campaign(
            &self.data,
            "preservers",
            0xC0FFEE,
            &crate::state::sim::founding_faction_ids(&self.data),
        );
        let Some(template) = self.data.contracts.get("founding_colony").cloned() else {
            return;
        };
        sim.ship.engine = "warp_coil".to_owned();
        sim.ship.fuel = 1.0;
        sim.contract = Some(contract::start_contract(&template, &sim));
        let limit = (template.target_duration_years.max(1) + 40) * 12;
        for month in 0..limit {
            if sim.pending_dilemma.is_some() {
                legacy::resolve_dilemma(&mut sim, &self.data, 0);
            }
            if let Some(pending) = sim.pending_event.clone() {
                match self.data.events.get(&pending.template_id).cloned() {
                    Some(t) => event_resolver::apply_outcome(&mut sim, &self.data, &t, 0),
                    None => sim.pending_event = None,
                }
            }
            if month >= DRY_AFTER_YEARS * 12 {
                sim.ship.fuel = 0.0;
            }
            let report = tick::advance_months(&mut sim, &self.data, 1);
            if report.terminal.is_some() || report.contract_completed.is_some() {
                break;
            }
        }
        self.state = crate::state::GameState::Gameplay(Box::new(GameplayState::new(sim)));
    }
}
