//! Global actions: persistence, navigation, and project controls.

use super::Game;
use crate::save;
use crate::state::{GameState, MenuState, StateTransition};
use crate::ui::UiAction;
use macroquad_toolkit::persistence::delete_slot;

impl Game {
    pub(super) fn apply_global_action(&mut self, action: UiAction) -> Option<StateTransition> {
        match action {
            action @ UiAction::SaveGame => self.handle_save_game(action),
            action @ UiAction::ToMenu => self.handle_to_menu(action),
            action @ UiAction::RetireVoyage => self.handle_retire_voyage(action),
            action @ UiAction::FileReport => self.handle_file_report(action),
            action @ UiAction::ChooseHomecomingRecovery(..) => {
                self.handle_choose_homecoming_recovery(action)
            }
            action @ UiAction::SelectScreen(..) => self.handle_select_screen(action),
            action @ UiAction::QueueProject { .. } => self.handle_queue_project(action),
            action @ UiAction::PauseProject(..) => self.handle_pause_project(action),
            action @ UiAction::ResumeProject(..) => self.handle_resume_project(action),
            action @ UiAction::MoveProject { .. } => self.handle_move_project(action),
            action @ UiAction::PreviewCancelProject(..) => {
                self.handle_preview_cancel_project(action)
            }
            action @ UiAction::ReviewCancelProject(..) => self.handle_review_cancel_project(action),
            action @ UiAction::CancelProject(..) => self.handle_cancel_project(action),
            action @ UiAction::DismissCancelProject => self.handle_dismiss_cancel_project(action),
            action @ UiAction::ReviewRecovery => self.handle_review_recovery(action),
            action @ UiAction::EmergencyStabilise => self.handle_emergency_stabilise(action),
            action @ UiAction::ResumeAfterWarning => self.handle_resume_after_warning(action),
            _ => None,
        }
    }

    fn handle_save_game(&mut self, action: UiAction) -> Option<StateTransition> {
        match action {
            UiAction::SaveGame => {
                if let GameState::Gameplay(gameplay) = &self.state {
                    match save::save_campaign(&self.data.config, &gameplay.sim) {
                        Ok(()) => self.notifications.success("Voyage saved."),
                        Err(err) => self.notifications.danger(format!("Save failed: {err}")),
                    }
                }
                None
            }

            _ => None,
        }
    }

    fn handle_to_menu(&mut self, action: UiAction) -> Option<StateTransition> {
        match action {
            UiAction::ToMenu => Some(StateTransition::ToMenu),

            _ => None,
        }
    }

    fn handle_retire_voyage(&mut self, action: UiAction) -> Option<StateTransition> {
        match action {
            UiAction::RetireVoyage => {
                // Clear the dead campaign so it can't be resumed (no autosave),
                // then return to the menu. The Chronicle persists separately.
                if let Err(err) =
                    delete_slot(&self.data.config.game_name, &self.data.config.save_slot)
                {
                    self.notifications
                        .warning(format!("Save clear failed: {err}"));
                }
                self.state = GameState::Menu(MenuState::new(save::save_exists(&self.data.config)));
                self.mission_started = None;
                self.last_mission_real_secs = None;
                self.notifications
                    .info("Voyage retired. The Chronicle remembers.");
                None
            }

            _ => None,
        }
    }

    fn handle_file_report(&mut self, action: UiAction) -> Option<StateTransition> {
        match action {
            UiAction::FileReport => {
                // The homecoming has been read. Clear the sealed report — that
                // alone dismisses the takeover — and land on the drydock board,
                // where the next charter is chosen. Autosave so a filed report
                // does not come back on the next load.
                if let GameState::Gameplay(gameplay) = &mut self.state {
                    if gameplay
                        .sim
                        .debrief
                        .as_ref()
                        .and_then(|report| report.recovery.as_ref())
                        .is_some_and(|recovery| !recovery.resolved)
                    {
                        self.notifications
                            .warning("Choose or defer the homecoming recovery before filing.");
                        return None;
                    }
                    gameplay.sim.debrief = None;
                    gameplay.screen = crate::state::Screen::Drydock;
                }
                if let GameState::Gameplay(gameplay) = &self.state {
                    if let Err(err) = save::save_campaign(&self.data.config, &gameplay.sim) {
                        self.notifications.danger(format!("Autosave failed: {err}"));
                    }
                }
                if crate::data::contracts::is_demo_build() {
                    self.state = GameState::Menu(MenuState::new(true));
                    self.notifications
                        .success("Demo complete. The Chronicle remembers your voyage.");
                }
                None
            }

            _ => None,
        }
    }

    fn handle_choose_homecoming_recovery(&mut self, action: UiAction) -> Option<StateTransition> {
        match action {
            UiAction::ChooseHomecomingRecovery(choice) => {
                self.apply_homecoming_recovery(choice);
                None
            }

            _ => None,
        }
    }

    fn handle_select_screen(&mut self, action: UiAction) -> Option<StateTransition> {
        match action {
            UiAction::SelectScreen(screen) => {
                if let GameState::Gameplay(gameplay) = &mut self.state {
                    gameplay.screen = screen;
                }
                if screen != crate::state::Screen::Subsystems {
                    self.custody_picker = None;
                }
                if screen != crate::state::Screen::Chronicle {
                    self.obligation_detail = None;
                }
                None
            }

            _ => None,
        }
    }

    fn handle_queue_project(&mut self, action: UiAction) -> Option<StateTransition> {
        match action {
            action @ UiAction::QueueProject { .. } => self.apply_project_action(action),

            _ => None,
        }
    }

    fn handle_pause_project(&mut self, action: UiAction) -> Option<StateTransition> {
        match action {
            action @ UiAction::PauseProject(_) => self.apply_project_action(action),

            _ => None,
        }
    }

    fn handle_resume_project(&mut self, action: UiAction) -> Option<StateTransition> {
        match action {
            action @ UiAction::ResumeProject(_) => self.apply_project_action(action),

            _ => None,
        }
    }

    fn handle_move_project(&mut self, action: UiAction) -> Option<StateTransition> {
        match action {
            action @ UiAction::MoveProject { .. } => self.apply_project_action(action),

            _ => None,
        }
    }

    fn handle_preview_cancel_project(&mut self, action: UiAction) -> Option<StateTransition> {
        match action {
            action @ UiAction::PreviewCancelProject(_) => self.apply_project_action(action),

            _ => None,
        }
    }

    fn handle_review_cancel_project(&mut self, action: UiAction) -> Option<StateTransition> {
        match action {
            action @ UiAction::ReviewCancelProject(_) => self.apply_project_action(action),

            _ => None,
        }
    }

    fn handle_cancel_project(&mut self, action: UiAction) -> Option<StateTransition> {
        match action {
            action @ UiAction::CancelProject(_) => self.apply_project_action(action),

            _ => None,
        }
    }

    fn handle_dismiss_cancel_project(&mut self, action: UiAction) -> Option<StateTransition> {
        match action {
            UiAction::DismissCancelProject => {
                self.apply_project_action(UiAction::DismissCancelProject)
            }

            _ => None,
        }
    }

    fn handle_review_recovery(&mut self, action: UiAction) -> Option<StateTransition> {
        match action {
            UiAction::ReviewRecovery => self.apply_project_action(UiAction::ReviewRecovery),

            _ => None,
        }
    }

    fn handle_emergency_stabilise(&mut self, action: UiAction) -> Option<StateTransition> {
        match action {
            UiAction::EmergencyStabilise => self.apply_project_action(UiAction::EmergencyStabilise),

            _ => None,
        }
    }

    fn handle_resume_after_warning(&mut self, action: UiAction) -> Option<StateTransition> {
        match action {
            UiAction::ResumeAfterWarning => self.apply_project_action(UiAction::ResumeAfterWarning),

            _ => None,
        }
    }
}
