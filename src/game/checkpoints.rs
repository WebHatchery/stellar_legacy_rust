//! Campaign checkpoints use real session time, independent of voyage speed.

use super::{current_decision_key, Game};
use crate::{save, state::GameState};
use macroquad_toolkit::persistence::AutoSaveManager;

#[derive(Default, PartialEq)]
struct Boundary {
    seed: u64,
    generation: u32,
    voyage: Option<(String, String)>,
    decision: Option<String>,
    homecoming: bool,
    ended: bool,
}

pub(super) struct Checkpoints {
    timer: AutoSaveManager,
    boundary: Option<Boundary>,
    failed: bool,
}

impl Checkpoints {
    pub(super) fn new(interval_seconds: f32) -> Self {
        Self {
            timer: AutoSaveManager::new(interval_seconds.max(1.0)),
            boundary: None,
            failed: false,
        }
    }
}

impl Game {
    pub(super) fn update_checkpoints(&mut self, dt: f32) {
        // Fixture captures must never overwrite a player's campaign.
        if self.instant_reveal {
            return;
        }
        let GameState::Gameplay(gameplay) = &self.state else {
            self.checkpoints.boundary = None;
            self.checkpoints.timer.reset_timer();
            return;
        };
        let sim = &gameplay.sim;
        let boundary = Boundary {
            seed: sim.seed,
            generation: sim.dynasty.generation,
            voyage: sim.contract.as_ref().map(|contract| {
                (
                    contract.template_id.clone(),
                    contract.phase.label().to_owned(),
                )
            }),
            decision: current_decision_key(sim),
            homecoming: sim.debrief.is_some(),
            ended: sim.terminal.is_some(),
        };
        let changed = self.checkpoints.boundary.as_ref() != Some(&boundary);
        self.checkpoints.boundary = Some(boundary);
        let result = if changed && !self.checkpoints.failed {
            self.checkpoints
                .timer
                .force(|| save::save_campaign(&self.data.config, sim))
                .map(|()| true)
        } else {
            self.checkpoints
                .timer
                .update(dt, true, || save::save_campaign(&self.data.config, sim))
        };
        match result {
            Ok(true) => {
                if self.checkpoints.failed {
                    self.notifications.success("Saving restored. Voyage saved.");
                }
                self.checkpoints.failed = false;
            }
            Ok(false) => {}
            Err(error) => self.checkpoint_failed(error),
        }
    }

    pub(super) fn persist_campaign(&mut self) -> Result<(), String> {
        if self.instant_reveal {
            return Ok(());
        }
        let GameState::Gameplay(gameplay) = &self.state else {
            return Ok(());
        };
        let result = self
            .checkpoints
            .timer
            .force(|| save::save_campaign(&self.data.config, &gameplay.sim));
        match &result {
            Ok(()) => self.checkpoints.failed = false,
            Err(error) => self.checkpoint_failed(error.clone()),
        }
        result
    }

    fn checkpoint_failed(&mut self, error: String) {
        eprintln!("Campaign checkpoint failed: {error}");
        if !self.checkpoints.failed {
            if let GameState::Gameplay(gameplay) = &mut self.state {
                gameplay.sim.push_log(format!(
                    "Save failed: {error}. Keep the game open for automatic retry, or use Utilities / Save game."
                ));
            }
        }
        self.checkpoints.failed = true;
        // The toolkit retains elapsed time on error; back off instead of
        // retrying a failing disk or browser store every rendered frame.
        self.checkpoints.timer.reset_timer();
        self.notifications.danger("Save failed. Keep game open.");
    }
}
