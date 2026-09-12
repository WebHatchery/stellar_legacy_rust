//! Charter gameplay action handlers.

use super::Game;
use crate::simulation::{approach, contract, event_resolver};
use crate::state::{GameState, StateTransition};
use crate::ui::UiAction;
use macroquad::prelude::get_time;

impl Game {
    pub(super) fn handle_cancel_selection(&mut self, action: UiAction) -> Option<StateTransition> {
        match action {
            UiAction::CancelSelection => {
                if let GameState::Gameplay(g) = &mut self.state {
                    if g.sim.contract.is_none() {
                        g.sim.selected_charter = None;
                        g.sim.selected_charter_approach = Default::default();
                    }
                }
                self.description_scroll
                    .set(macroquad_toolkit::ui::ScrollArea::new());
                None
            }

            _ => None,
        }
    }

    pub(super) fn handle_select_charter(&mut self, action: UiAction) -> Option<StateTransition> {
        match action {
            UiAction::SelectCharter(id) => {
                self.description_scroll
                    .set(macroquad_toolkit::ui::ScrollArea::new());
                // Selecting a charter never starts it (W4) — it only puts it
                // under consideration on the PREP screen. Renown gates exactly
                // as before; re-selecting swaps the choice.
                let renown = crate::heritage::renown(&self.chronicle);
                if let (GameState::Gameplay(gameplay), Some(template)) =
                    (&mut self.state, self.data.contracts.get(&id))
                {
                    let sim = &mut gameplay.sim;
                    // Renown (cross-campaign) *and* the in-world gate (content-depth
                    // charters round 12: the peoples the writ needs aboard) must
                    // both clear before a charter can be put under consideration.
                    if sim.contract.is_none()
                        && crate::data::contracts::is_available_in_build(template)
                        && renown >= template.min_renown
                        && crate::simulation::contract::meets_in_world_gate(sim, template)
                        && crate::simulation::contract::meets_loadout_gate(
                            sim, &self.data, template,
                        )
                    {
                        sim.selected_charter = Some(id.clone());
                        sim.selected_charter_approach = Default::default();
                        sim.push_log(format!("Charter under consideration: {}", template.name));
                    }
                }
                None
            }

            _ => None,
        }
    }

    pub(super) fn handle_set_charter_approach(
        &mut self,
        action: UiAction,
    ) -> Option<StateTransition> {
        match action {
            UiAction::SetCharterApproach(selected) => {
                let mut warning = None;
                if let GameState::Gameplay(gameplay) = &mut self.state {
                    let sim = &mut gameplay.sim;
                    if sim.contract.is_some() {
                        warning = Some(
                            "A charter approach is fixed once the voyage launches.".to_owned(),
                        );
                    } else if let Some(id) = sim.selected_charter.as_deref() {
                        if let Some(template) = self.data.contracts.get(id) {
                            if let Some(reason) =
                                approach::unavailable_reason(sim, &self.data, template, selected)
                            {
                                warning = Some(reason);
                            } else {
                                sim.selected_charter_approach = selected;
                                sim.push_log(format!(
                                    "Charter approach selected: {}.",
                                    selected.label()
                                ));
                            }
                        }
                    }
                }
                if let Some(warning) = warning {
                    self.notifications.warning(warning);
                }
                None
            }

            _ => None,
        }
    }

    pub(super) fn handle_launch(&mut self, action: UiAction) -> Option<StateTransition> {
        match action {
            UiAction::Launch => {
                // The one and only path that starts a contract (W4).
                let mut launched = false;
                let mut launch_warning = None;
                if let GameState::Gameplay(gameplay) = &mut self.state {
                    let sim = &mut gameplay.sim;
                    let selected = if sim.contract.is_none() {
                        sim.selected_charter.clone()
                    } else {
                        None
                    };
                    if let Some(id) = selected {
                        if let Some(template) = self.data.contracts.get(&id) {
                            if let Some(reason) = approach::unavailable_reason(
                                sim,
                                &self.data,
                                template,
                                sim.selected_charter_approach,
                            ) {
                                launch_warning = Some(reason);
                            } else {
                                contract::default_obligation_conflicts(sim, template);
                                sim.contract = Some(contract::start_contract(template, sim));
                                for operation in &template.launch_obligation_operations {
                                    sim.apply_obligation_operation(operation);
                                }
                                // Lay out the seeded campaign skeleton at LAUNCH (W6).
                                if let Some(c) = sim.contract.as_mut() {
                                    c.beats = event_resolver::skeleton::generate_beats(
                                        &mut sim.rng,
                                        c,
                                        &self.data.config.campaign_skeleton,
                                    );
                                }
                                sim.selected_charter = None;
                                sim.push_log(format!(
                                    "LAUNCH. {} — {} years. May the line hold.",
                                    template.name, template.target_duration_years
                                ));
                                launched = true;
                            }
                        }
                    }
                }
                if launched {
                    // Start the cosmetic run timer for this mission (PLAN M4.7).
                    self.mission_started = Some(get_time());
                    self.last_mission_real_secs = None;
                    // The tab set changes under way (real-time loop §5): land on the
                    // active CONTRACT view since the DRYDOCK tab is now gone.
                    if let GameState::Gameplay(gameplay) = &mut self.state {
                        if !crate::state::Screen::UNDERWAY.contains(&gameplay.screen) {
                            gameplay.screen = crate::state::Screen::Contract;
                        }
                    }
                    self.notifications.success("Launched. The voyage begins.");
                } else if let Some(warning) = launch_warning {
                    self.notifications.warning(warning);
                } else {
                    self.notifications
                        .warning("Select a charter in port before launching.");
                }
                None
            }

            _ => None,
        }
    }
}
