//! Homecoming capture scenes covering the report and recovery boundary.

use super::*;
use crate::simulation::contract;
use crate::state::{GameplayState, SimState};

impl Game {
    /// Fly a charter end to end under the autoplay policy and conclude it, so
    /// the HOMECOMING debrief photographs a report the simulation actually
    /// produced — real marks, real council decisions, real captains — rather
    /// than a hand-built stub that could drift from what players see.
    ///
    /// Deliberately stops one step short of `autoplay::play_mission`, which
    /// clears `sim.contract` on completion; the report must be sealed while the
    /// concluded charter is still there to read.
    pub(super) fn fly_to_homecoming(&mut self, contract_id: &str) {
        use crate::simulation::{event_resolver, legacy, tick};

        let mut sim = SimState::new_campaign(
            &self.data,
            "preservers",
            0xC0FFEE,
            &crate::state::sim::founding_faction_ids(&self.data),
        );
        let Some(template) = self.data.contracts.get(contract_id).cloned() else {
            return;
        };
        sim.resources.credits = 20_000;
        let _ =
            crate::simulation::institutions::designate_apprentice(&mut sim, &self.data, "engineer");
        let _ = crate::simulation::institutions::establish_or_support_school(
            &mut sim,
            &self.data,
            "engineering_bay",
        );
        let _ = crate::simulation::institutions::compile_archive(
            &mut sim,
            &self.data,
            "engineering_bay",
        );
        sim.ship.fuel = 1.0;
        if let Some(event) = self.data.events.get("sanctuary_berths_asked").cloned() {
            event_resolver::apply_outcome(&mut sim, &self.data, &event, 0);
        }
        sim.contract = Some(contract::start_contract(&template, &sim));
        if let Some(c) = sim.contract.as_mut() {
            c.beats = event_resolver::skeleton::generate_beats(
                &mut sim.rng,
                c,
                &self.data.config.campaign_skeleton,
            );
        }

        let mut concluded = None;
        let limit = template.target_duration_years.max(1) * 12 + 240;
        for _ in 0..limit {
            // Answer whatever the council is asked by taking the first option —
            // the same policy the soak tests use. Every answer is remembered, so
            // the voyage log fills with genuine decisions.
            if sim.pending_dilemma.is_some() {
                legacy::resolve_dilemma(&mut sim, &self.data, 0);
            }
            if let Some(pending) = sim.pending_event.clone() {
                match self.data.events.get(&pending.template_id).cloned() {
                    Some(t) => event_resolver::apply_outcome(&mut sim, &self.data, &t, 0),
                    None => sim.pending_event = None,
                }
            }
            if sim.dynasty.extinct {
                break;
            }
            // The same standing orders `autoplay::play_mission` flies under:
            // keep the hull off the floor and the galley stocked. Without them a
            // 450-year charter starves, and the capture would advertise a death
            // march as the typical homecoming. Both verbs refuse harmlessly when
            // they cannot be paid for.
            if sim.ship.hull_integrity < 0.5 {
                let _ = crate::simulation::ship::field_repair(
                    &mut sim,
                    &self.data.config,
                    crate::simulation::ship::RepairKind::Hull,
                );
            }
            if sim.resources.food < self.data.config.low_food_threshold {
                let _ = crate::simulation::market::buy(
                    &mut sim,
                    crate::state::sim::TradeResource::Food,
                    1000,
                );
            }
            let report = tick::advance_months(&mut sim, &self.data, 1);
            if let Some(result) = report.contract_completed {
                concluded = Some(result);
                break;
            }
            if report.dynasty_extinct {
                break;
            }
        }

        self.state = crate::state::GameState::Gameplay(Box::new(GameplayState::new(sim)));
        if let Some((score, level)) = concluded {
            self.conclude_contract(score, level);
        }
    }
}
