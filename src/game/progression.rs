//! Monthly progression and its player-facing consequences.
use super::Game;
use crate::simulation::tick;
use crate::state::GameState;

impl Game {
    /// Advance the sim one month and react to what it produced (real-time loop
    /// §1): the sole tick driver now that time auto-advances. Called by the
    /// game-loop accumulator, once per elapsed month.
    pub(super) fn advance_one_month(&mut self) {
        let GameState::Gameplay(gameplay) = &mut self.state else {
            return;
        };
        let sim = &mut gameplay.sim;
        if sim.has_pending_decision() || sim.dynasty.extinct || sim.terminal.is_some() {
            return;
        }

        let report = tick::advance_months(sim, &self.data, 1);
        let completed = report.contract_completed.is_some();

        if report.decision_required {
            self.notifications.warning("The council must decide.");
            self.audio
                .cue(crate::audio::Cue::Council, self.display.audio_volume);
        }
        if report.dynasty_extinct {
            self.notifications.danger("The dynasty has ended.");
            self.audio
                .cue(crate::audio::Cue::GameOver, self.display.audio_volume);
        }
        if let Some(outcome) = report.terminal.as_ref() {
            self.notifications.danger(outcome.reason.label());
            self.audio
                .cue(crate::audio::Cue::GameOver, self.display.audio_volume);
        } else if report.critical_warning {
            self.notifications
                .warning("Life support critical. Recovery review paused the voyage.");
        }
        if report.leader_died {
            self.audio
                .cue(crate::audio::Cue::Succession, self.display.audio_volume);
        }
        if report.phase_changed.is_some() {
            self.audio
                .cue(crate::audio::Cue::Phase, self.display.audio_volume);
        }
        if let Some((score, level)) = report.contract_completed {
            self.audio
                .cue(crate::audio::Cue::Homecoming, self.display.audio_volume);
            self.conclude_contract(score, level);
        }

        // Reaching 100% docks the ship (real-time loop §4): the contract is now
        // cleared, so time auto-pauses. Land the player on the DRYDOCK board for
        // repairs / upgrades / the next charter.
        if completed {
            if let GameState::Gameplay(gameplay) = &mut self.state {
                gameplay.screen = crate::state::Screen::Drydock;
            }
            self.notifications.info("Docked for refit.");
        }
        self.check_achievements();
    }
}
