//! UiAction dispatch: the pure-view UI returns intents (`UiAction`); this
//! module interprets each one against the sim, applies persistence side
//! effects, and surfaces notifications (CODE_STANDARDS §7). Split out of
//! `game.rs` so the state-machine core stays lean.

mod authority;
mod completion;
mod decision;
mod gameplay;
mod global;
mod homecoming;
mod menu;
mod mission;
mod projects;

use super::Game;
use crate::data::ship_components::ComponentKind;
use crate::state::{GameState, StateTransition};
use crate::ui::UiAction;

impl Game {
    pub(super) fn apply_action(&mut self, action: UiAction) -> Option<StateTransition> {
        let (review, cancellation) = if matches!(self.state, GameState::Gameplay(_)) {
            projects::review_after_action(
                self.project_cancel_confirm.get(),
                self.presentation.project_cancellation.get(),
                &action,
            )
        } else {
            (None, None)
        };
        self.project_cancel_confirm.set(review);
        self.presentation.project_cancellation.set(cancellation);
        self.abort_confirm.set(match &self.state {
            GameState::Gameplay(gameplay) => {
                mission::review_after_action(self.abort_confirm.get(), &action, &gameplay.sim)
            }
            _ => false,
        });
        if Self::is_menu_action(&action) {
            return self.apply_menu_action(action);
        }
        if Self::is_global_action(&action) {
            return self.apply_global_action(action);
        }
        self.apply_gameplay_action(action)
    }

    fn is_menu_action(action: &UiAction) -> bool {
        matches!(
            action,
            UiAction::SelectLegacy(_)
                | UiAction::ToggleFaction(_)
                | UiAction::StartNewGame
                | UiAction::ContinueGame
                | UiAction::GoToNewGame
                | UiAction::BackToMainMenu
                | UiAction::OpenHelp
                | UiAction::OpenSettings
                | UiAction::ExitGame
                | UiAction::DeleteSave
        )
    }

    fn is_global_action(action: &UiAction) -> bool {
        matches!(
            action,
            UiAction::SaveGame
                | UiAction::ToMenu
                | UiAction::RetireVoyage
                | UiAction::FileReport
                | UiAction::ChooseHomecomingRecovery(_)
                | UiAction::SelectScreen(_)
                | UiAction::QueueProject { .. }
                | UiAction::PauseProject(_)
                | UiAction::ResumeProject(_)
                | UiAction::MoveProject { .. }
                | UiAction::PreviewCancelProject(_)
                | UiAction::ReviewCancelProject(_)
                | UiAction::CancelProject(_)
                | UiAction::DismissCancelProject
                | UiAction::ReviewRecovery
                | UiAction::EmergencyStabilise
                | UiAction::ResumeAfterWarning
        )
    }

    /// Run a subsystem verb (W5) against the sim and surface its result. Keeps
    /// the three dispatch arms thin (ground rule 2).
    fn subsystem_verb(
        &mut self,
        verb: fn(
            &mut crate::state::sim::SimState,
            &crate::data::GameData,
            &str,
        ) -> Result<(), String>,
        id: &str,
        ok_msg: &str,
    ) {
        if let GameState::Gameplay(gameplay) = &mut self.state {
            match verb(&mut gameplay.sim, &self.data, id) {
                Ok(()) => self.notifications.success(ok_msg.to_owned()),
                Err(err) => self.notifications.warning(err),
            }
        }
    }

    fn purchase_component(&mut self, kind: ComponentKind, id: &str) {
        let GameState::Gameplay(gameplay) = &mut self.state else {
            return;
        };
        let Some(component) = self.data.ship_components.find(kind, id) else {
            return;
        };
        let sim = &mut gameplay.sim;

        // Loadout changes are a drydock job (PLAN M4.6): only in port.
        if sim.contract.is_some() {
            self.notifications
                .warning("Loadout changes wait for port — you're underway.");
            return;
        }

        let cost = crate::data::ResourceDelta {
            credits: -component.cost.credits,
            energy: -component.cost.energy,
            minerals: -component.cost.minerals,
            food: -component.cost.food,
            influence: -component.cost.influence,
        };
        if !sim.resources.can_afford(&cost) {
            self.notifications.warning("The treasury cannot cover it.");
            return;
        }
        sim.resources.apply(&cost);
        match kind {
            ComponentKind::Hull => sim.ship.hull = component.id.clone(),
            ComponentKind::Engine => sim.ship.engine = component.id.clone(),
            ComponentKind::Weapon => sim.ship.weapon = Some(component.id.clone()),
        }
        sim.push_log(format!("Refit complete: {} installed.", component.name));
        self.notifications
            .success(format!("{} installed.", component.name));
    }
}
