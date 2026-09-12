//! Logistics gameplay action handlers.

use super::Game;
use crate::simulation::market;
use crate::state::{GameState, StateTransition};
use crate::ui::UiAction;

impl Game {
    pub(super) fn handle_refuel(&mut self, action: UiAction) -> Option<StateTransition> {
        match action {
            UiAction::Refuel => {
                if let GameState::Gameplay(gameplay) = &mut self.state {
                    match crate::simulation::ship::refuel(&mut gameplay.sim, &self.data.config) {
                        Ok(()) => self.notifications.success("Tanks topped off."),
                        Err(err) => self.notifications.warning(err),
                    }
                }
                None
            }

            _ => None,
        }
    }

    pub(super) fn handle_review_provisions(&mut self, action: UiAction) -> Option<StateTransition> {
        match action {
            UiAction::ReviewProvisions => None,

            _ => None,
        }
    }

    pub(super) fn handle_next_tutorial(&mut self, action: UiAction) -> Option<StateTransition> {
        match action {
            UiAction::NextTutorial => {
                if let GameState::Gameplay(gameplay) = &mut self.state {
                    if gameplay.sim.tutorial_step >= self.data.config.tutorial.guided_steps.len() {
                        gameplay.sim.tutorial_dismissed = true;
                        self.tutorial_open = false;
                    }
                }
                None
            }

            _ => None,
        }
    }

    pub(super) fn handle_skip_tutorial(&mut self, action: UiAction) -> Option<StateTransition> {
        match action {
            UiAction::SkipTutorial => {
                if let GameState::Gameplay(gameplay) = &mut self.state {
                    gameplay.sim.tutorial_dismissed = true;
                }
                self.tutorial_open = false;
                None
            }

            _ => None,
        }
    }

    pub(super) fn handle_buy_parts(&mut self, action: UiAction) -> Option<StateTransition> {
        match action {
            UiAction::BuyParts(amount) => {
                if let GameState::Gameplay(gameplay) = &mut self.state {
                    match crate::simulation::ship::buy_parts(
                        &mut gameplay.sim,
                        &self.data.config,
                        amount,
                    ) {
                        Ok(()) => self.notifications.success("Spare parts stocked."),
                        Err(err) => self.notifications.warning(err),
                    }
                }
                None
            }

            _ => None,
        }
    }

    pub(super) fn handle_purchase_component(
        &mut self,
        action: UiAction,
    ) -> Option<StateTransition> {
        match action {
            UiAction::PurchaseComponent(kind, id) => {
                self.purchase_component(kind, &id);
                None
            }

            _ => None,
        }
    }

    pub(super) fn handle_field_repair(&mut self, action: UiAction) -> Option<StateTransition> {
        match action {
            UiAction::FieldRepair(kind) => {
                if let GameState::Gameplay(gameplay) = &mut self.state {
                    match crate::simulation::ship::field_repair(
                        &mut gameplay.sim,
                        &self.data.config,
                        kind,
                    ) {
                        Ok(()) => self.notifications.success("Field repair complete."),
                        Err(err) => self.notifications.warning(err),
                    }
                }
                None
            }

            _ => None,
        }
    }

    pub(super) fn handle_full_repair(&mut self, action: UiAction) -> Option<StateTransition> {
        match action {
            UiAction::FullRepair => {
                if let GameState::Gameplay(gameplay) = &mut self.state {
                    match crate::simulation::ship::full_repair(&mut gameplay.sim, &self.data.config)
                    {
                        Ok(()) => self.notifications.success("Full refit complete."),
                        Err(err) => self.notifications.warning(err),
                    }
                }
                None
            }

            _ => None,
        }
    }

    pub(super) fn handle_install_salvage(&mut self, action: UiAction) -> Option<StateTransition> {
        match action {
            UiAction::InstallSalvage(id) => {
                if let GameState::Gameplay(gameplay) = &mut self.state {
                    match crate::simulation::ship::install_salvage(
                        &mut gameplay.sim,
                        &self.data,
                        &id,
                    ) {
                        Ok(()) => self.notifications.success("Salvage installed."),
                        Err(err) => self.notifications.warning(err),
                    }
                }
                None
            }

            _ => None,
        }
    }

    pub(super) fn handle_commission_ship(&mut self, action: UiAction) -> Option<StateTransition> {
        match action {
            UiAction::CommissionShip(id) => {
                if let GameState::Gameplay(gameplay) = &mut self.state {
                    match crate::simulation::ship::commission_ship(
                        &mut gameplay.sim,
                        &self.data,
                        &id,
                    ) {
                        Ok(()) => self.notifications.success("New ship commissioned."),
                        Err(err) => self.notifications.warning(err),
                    }
                }
                None
            }

            _ => None,
        }
    }

    pub(super) fn handle_buy(&mut self, action: UiAction) -> Option<StateTransition> {
        match action {
            UiAction::Buy(resource, amount) => {
                if let GameState::Gameplay(gameplay) = &mut self.state {
                    match market::buy(&mut gameplay.sim, resource, amount) {
                        Ok(receipt) => self.notifications.success(format!(
                            "Bought {amount} {} for {}cr · next {:.2}/u",
                            resource.label(),
                            receipt.total_credits,
                            receipt.market_price_after
                        )),
                        Err(err) => self.notifications.warning(err),
                    }
                }
                None
            }

            _ => None,
        }
    }

    pub(super) fn handle_sell(&mut self, action: UiAction) -> Option<StateTransition> {
        match action {
            UiAction::Sell(resource, amount) => {
                if let GameState::Gameplay(gameplay) = &mut self.state {
                    match market::sell(&mut gameplay.sim, resource, amount) {
                        Ok(receipt) => self.notifications.success(format!(
                            "Sold {amount} {} for {}cr · next {:.2}/u",
                            resource.label(),
                            receipt.total_credits,
                            receipt.market_price_after
                        )),
                        Err(err) => self.notifications.warning(err),
                    }
                }
                None
            }

            _ => None,
        }
    }

    pub(super) fn handle_toggle_delegation(&mut self, action: UiAction) -> Option<StateTransition> {
        match action {
            UiAction::ToggleDelegation(category) => {
                if let GameState::Gameplay(gameplay) = &mut self.state {
                    gameplay.sim.delegation.toggle(category);
                    self.notifications.info(format!(
                        "{}: {}.",
                        category.label(),
                        if gameplay.sim.delegation.is_delegated(category) {
                            "future events delegated"
                        } else {
                            "future events return to council review"
                        }
                    ));
                }
                None
            }

            _ => None,
        }
    }
}
