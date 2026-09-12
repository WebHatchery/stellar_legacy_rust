//! Menu actions: starting, resuming, and configuring a voyage.

use super::Game;
use crate::save;
use crate::state::{GameState, StateTransition};
use crate::ui::UiAction;
use macroquad_toolkit::persistence::delete_slot;
use macroquad_toolkit::rng;

impl Game {
    pub(super) fn apply_menu_action(&mut self, action: UiAction) -> Option<StateTransition> {
        match action {
            action @ UiAction::SelectLegacy(..) => self.handle_select_legacy(action),
            action @ UiAction::ToggleFaction(..) => self.handle_toggle_faction(action),
            action @ UiAction::StartNewGame => self.handle_start_new_game(action),
            action @ UiAction::ContinueGame => self.handle_continue_game(action),
            action @ UiAction::GoToNewGame => self.handle_go_to_new_game(action),
            action @ UiAction::BackToMainMenu => self.handle_back_to_main_menu(action),
            action @ UiAction::OpenHelp => self.handle_open_help(action),
            action @ UiAction::OpenSettings => self.handle_open_settings(action),
            action @ UiAction::ExitGame => self.handle_exit_game(action),
            action @ UiAction::DeleteSave => self.handle_delete_save(action),
            _ => None,
        }
    }

    fn handle_select_legacy(&mut self, action: UiAction) -> Option<StateTransition> {
        match action {
            UiAction::SelectLegacy(index) => {
                if let GameState::Menu(menu) = &mut self.state {
                    menu.selected_legacy = index.min(self.legacy_ids.len().saturating_sub(1));
                }
                None
            }

            _ => None,
        }
    }

    fn handle_toggle_faction(&mut self, action: UiAction) -> Option<StateTransition> {
        match action {
            UiAction::ToggleFaction(id) => {
                let starting = self.data.config.factions.starting_count as usize;
                if let GameState::Menu(menu) = &mut self.state {
                    // Toggling off is always allowed; toggling on is capped at
                    // the founding count (W7).
                    let already = menu.selected_factions.iter().any(|f| f == &id);
                    if already || menu.selected_factions.len() < starting {
                        menu.toggle_faction(&id);
                    }
                }
                None
            }

            _ => None,
        }
    }

    fn handle_start_new_game(&mut self, action: UiAction) -> Option<StateTransition> {
        match action {
            UiAction::StartNewGame => {
                let starting = self.data.config.factions.starting_count as usize;
                let selection = match &self.state {
                    GameState::Menu(menu) => {
                        let legacy = self
                            .legacy_ids
                            .get(menu.selected_legacy)
                            .cloned()
                            .unwrap_or_else(|| "preservers".to_owned());
                        Some((legacy, menu.selected_factions.clone()))
                    }
                    _ => None,
                };
                // Seed is random per campaign (from the wall-clock-seeded global
                // generator) unless `fixed_seed` is set for reproducible testing
                // (real-time loop follow-up). Determinism still holds *within* a
                // campaign because the seed is stored in the save (GDD §5.6).
                let seed = self.data.config.fixed_seed.unwrap_or_else(rng::random_u64);
                match selection {
                    Some((legacy_id, faction_ids)) if faction_ids.len() == starting => {
                        Some(StateTransition::NewCampaign {
                            legacy_id,
                            seed,
                            faction_ids,
                        })
                    }
                    Some(_) => {
                        self.notifications.warning(format!(
                            "Choose exactly {starting} founding factions to begin."
                        ));
                        None
                    }
                    None => None,
                }
            }

            _ => None,
        }
    }

    fn handle_continue_game(&mut self, action: UiAction) -> Option<StateTransition> {
        match action {
            UiAction::ContinueGame => Some(StateTransition::LoadCampaign),

            _ => None,
        }
    }

    fn handle_go_to_new_game(&mut self, action: UiAction) -> Option<StateTransition> {
        match action {
            UiAction::GoToNewGame => {
                if let GameState::Menu(menu) = &mut self.state {
                    menu.phase = crate::state::MenuPhase::NewGame;
                }
                // First time into the new-game picker, the orientation overlay
                // greets the commander over it (once per install, per the flag).
                if !self.onboarding.welcome_seen {
                    self.welcome_open = true;
                }
                None
            }

            _ => None,
        }
    }

    fn handle_back_to_main_menu(&mut self, action: UiAction) -> Option<StateTransition> {
        match action {
            UiAction::BackToMainMenu => {
                if let GameState::Menu(menu) = &mut self.state {
                    menu.phase = crate::state::MenuPhase::Main;
                }
                None
            }

            _ => None,
        }
    }

    fn handle_open_help(&mut self, action: UiAction) -> Option<StateTransition> {
        match action {
            UiAction::OpenHelp => {
                // The chrome row's way into the F2 overlay, for a screen with
                // no function keys to press.
                self.help_open = true;
                self.settings_open = false;
                None
            }

            _ => None,
        }
    }

    fn handle_open_settings(&mut self, action: UiAction) -> Option<StateTransition> {
        match action {
            UiAction::OpenSettings => {
                // Reuse the shared display/settings overlay (F1) from the menu.
                self.settings_open = true;
                self.presentation
                    .overlay_scroll
                    .set(macroquad_toolkit::ui::ScrollArea::new());
                self.help_open = false;
                None
            }

            _ => None,
        }
    }

    fn handle_exit_game(&mut self, action: UiAction) -> Option<StateTransition> {
        match action {
            UiAction::ExitGame => {
                // Native desktop quit; on WebGL there is no window to close.
                std::process::exit(0);
            }

            _ => None,
        }
    }

    fn handle_delete_save(&mut self, action: UiAction) -> Option<StateTransition> {
        match action {
            UiAction::DeleteSave => {
                match delete_slot(&self.data.config.game_name, &self.data.config.save_slot) {
                    Ok(()) => self.notifications.info("Save slot cleared."),
                    Err(err) => self.notifications.danger(format!("Delete failed: {err}")),
                }
                if let GameState::Menu(menu) = &mut self.state {
                    menu.save_exists = save::save_exists(&self.data.config);
                }
                None
            }

            // ---- Global ----
            _ => None,
        }
    }
}
