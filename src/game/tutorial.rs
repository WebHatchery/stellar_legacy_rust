//! Advance a lesson only after its requested action succeeds.
use super::*;
use crate::state::{sim::GameSpeed, Screen};

fn completed(step: usize, action: &UiAction, sim: &SimState) -> bool {
    match (step, action) {
        (0, UiAction::SelectScreen(Screen::Drydock)) => true,
        (1, UiAction::SelectCharter(id)) => sim.selected_charter.as_ref() == Some(id),
        (2, UiAction::ReviewProvisions) => sim.selected_charter.is_some(),
        (3, UiAction::SelectScreen(Screen::CrewDynasty)) => true,
        (4, UiAction::SelectScreen(Screen::ShipBuilder)) => true,
        (5, UiAction::Launch) => sim.contract.is_some(),
        (6, UiAction::SelectScreen(Screen::Agenda)) => true,
        (7, UiAction::QueueProject { .. }) => !sim.projects.jobs.is_empty(),
        (8, UiAction::TogglePause | UiAction::SetSpeed(GameSpeed::Paused)) => {
            sim.speed == GameSpeed::Paused
        }
        (9, UiAction::TogglePause | UiAction::SetSpeed(_)) => sim.speed != GameSpeed::Paused,
        (10, UiAction::ResolveEvent(_) | UiAction::ResolveDilemma(_)) => {
            !sim.has_pending_decision()
        }
        _ => false,
    }
}

impl Game {
    pub(super) fn track_tutorial(&mut self, action: &UiAction) {
        if !self.display.tutorial_enabled || !self.tutorial_open {
            return;
        }
        if let GameState::Gameplay(g) = &mut self.state {
            if !g.sim.tutorial_dismissed && completed(g.sim.tutorial_step, action, &g.sim) {
                g.sim.tutorial_step += 1;
            }
        }
    }

    pub(super) fn tutorial_holds_clock(&self) -> bool {
        self.display.tutorial_enabled
            && self.tutorial_open
            && matches!(&self.state, GameState::Gameplay(g) if !g.sim.tutorial_dismissed && g.sim.tutorial_step < 10)
    }
}

#[cfg(test)]
mod tests;
