//! Core gameplay action handlers.

use super::Game;
use crate::simulation::contract;
use crate::state::{GameState, StateTransition};
use crate::ui::UiAction;

impl Game {
    pub(super) fn handle_toggle_pause(&mut self, action: UiAction) -> Option<StateTransition> {
        match action {
            UiAction::TogglePause => {
                if let GameState::Gameplay(gameplay) = &mut self.state {
                    gameplay.sim.toggle_pause();
                }
                None
            }

            _ => None,
        }
    }

    pub(super) fn handle_set_speed(&mut self, action: UiAction) -> Option<StateTransition> {
        match action {
            UiAction::SetSpeed(step) => {
                if let GameState::Gameplay(gameplay) = &mut self.state {
                    gameplay.sim.set_speed(step);
                }
                None
            }

            _ => None,
        }
    }

    pub(super) fn handle_set_posture(&mut self, action: UiAction) -> Option<StateTransition> {
        match action {
            UiAction::SetPosture(posture) => {
                self.apply_authority_action(UiAction::SetPosture(posture))
            }

            _ => None,
        }
    }

    pub(super) fn handle_resolve_authority(&mut self, action: UiAction) -> Option<StateTransition> {
        match action {
            UiAction::ResolveAuthority(choice) => {
                self.apply_authority_action(UiAction::ResolveAuthority(choice))
            }

            _ => None,
        }
    }

    pub(super) fn handle_review_return_home(
        &mut self,
        action: UiAction,
    ) -> Option<StateTransition> {
        match action {
            UiAction::ReviewReturnHome => {
                self.presentation.utilities.set(false);
                None
            }

            _ => None,
        }
    }

    pub(super) fn handle_dismiss_return_home(
        &mut self,
        action: UiAction,
    ) -> Option<StateTransition> {
        match action {
            UiAction::DismissReturnHome => None,

            _ => None,
        }
    }

    pub(super) fn handle_abort_mission(&mut self, action: UiAction) -> Option<StateTransition> {
        match action {
            UiAction::AbortMission => {
                if let GameState::Gameplay(gameplay) = &mut self.state {
                    let sim = &mut gameplay.sim;
                    // The council turns the ship for home; pay will be prorated
                    // to whatever objective progress was banked (W2).
                    if contract::jump_to_return(sim) {
                        sim.push_log("The council votes to turn back.");
                        self.notifications
                            .warning("Turning for home — pay prorated to the objective banked.");
                    }
                }
                None
            }

            _ => None,
        }
    }
}
