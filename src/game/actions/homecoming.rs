//! Homecoming recovery action dispatch.

use super::Game;
use crate::simulation::homecoming;
use crate::state::GameState;

impl Game {
    pub(super) fn apply_homecoming_recovery(
        &mut self,
        choice: crate::state::sim::HomecomingChoice,
    ) {
        let result = if let GameState::Gameplay(gameplay) = &mut self.state {
            homecoming::apply_choice(&mut gameplay.sim, &self.data, choice)
        } else {
            Err("No active campaign.".to_owned())
        };
        match result {
            Ok(note) => {
                self.notifications.success(note);
                if let Err(error) = self.save_campaign() {
                    self.notifications
                        .danger(format!("Autosave failed: {error}"));
                }
            }
            Err(error) => self.notifications.warning(error),
        }
    }

    fn save_campaign(&mut self) -> Result<(), String> {
        let GameState::Gameplay(gameplay) = &self.state else {
            return Ok(());
        };
        crate::save::save_campaign(&self.data.config, &gameplay.sim)
    }
}
