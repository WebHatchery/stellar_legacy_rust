//! Session clock; screen navigation never changes its rate.
use super::*;

fn elapsed_decision(elapsed: f32, dt: f32, paused: bool) -> f32 {
    elapsed + if paused { 0.0 } else { dt.max(0.0) }
}

impl Game {
    /// The real-time driver (real-time loop §1/§2). While under way and unpaused,
    /// bank real seconds toward the next month and step the tick each time the
    /// per-month threshold is crossed, hard-stopping the moment a decision,
    /// completion, or extinction lands. While a decision blocks, freeze the clock
    /// and auto-resolve it once the countdown runs out. Docked, nothing advances.
    /// Skipped entirely in capture mode (deterministic screenshots).
    pub(super) fn update_realtime(&mut self, dt: f32) {
        if self.instant_reveal {
            return;
        }
        let (is_gameplay, key, can_advance, multiplier) = match &self.state {
            GameState::Gameplay(g) => {
                let key = current_decision_key(&g.sim);
                let can_advance = key.is_none()
                    && !g.sim.dynasty.extinct
                    && g.sim.contract.is_some()
                    && g.sim.speed != crate::state::sim::GameSpeed::Paused;
                (true, key, can_advance, g.sim.speed.multiplier())
            }
            _ => (false, None, false, 0.0),
        };
        if !is_gameplay {
            self.decision_key = None;
            self.month_accumulator = 0.0;
            return;
        }

        // Track the countdown clock, restarting it whenever the decision changes.
        if key != self.decision_key {
            self.decision_key = key.clone();
            self.decision_elapsed = 0.0;
        }

        if key.is_some() {
            // A decision blocks time; let the clock decide once it runs out.
            self.month_accumulator = 0.0;
            let paused = matches!(&self.state, GameState::Gameplay(g) if g.sim.speed == crate::state::sim::GameSpeed::Paused);
            self.decision_elapsed = elapsed_decision(self.decision_elapsed, dt, paused);
            let timeout = self.data.config.real_time.decision_timeout_secs;
            if self.decision_elapsed >= timeout {
                self.auto_resolve_decision();
            }
            return;
        }

        if !can_advance {
            self.month_accumulator = 0.0;
            return;
        }

        self.month_accumulator += dt * multiplier;
        let per_month = self.data.config.real_time.seconds_per_month.max(0.01);
        while self.month_accumulator >= per_month {
            self.month_accumulator -= per_month;
            self.advance_one_month();
            // Stop bursting months the instant something needs the player or the
            // voyage ended; the remainder is dropped so it doesn't fast-forward on
            // resume.
            let stop = match &self.state {
                GameState::Gameplay(g) => {
                    g.sim.has_pending_decision()
                        || g.sim.dynasty.extinct
                        || g.sim.contract.is_none()
                }
                _ => true,
            };
            if stop {
                self.month_accumulator = 0.0;
                break;
            }
        }
    }

    /// The cosmetic run timer's elapsed seconds (PLAN M4.7): live while a mission
    /// is active, frozen at the last mission's time while in port, and a fixed
    /// override in capture. Never feeds the deterministic sim.
    /// Real seconds left before the current blocking decision auto-resolves
    /// (real-time loop §2). Full timeout in capture (the clock never runs there);
    /// 0 when nothing is pending.
    pub(super) fn decision_remaining(&self, sim: &SimState) -> f32 {
        let timeout = self.data.config.real_time.decision_timeout_secs;
        if self.instant_reveal {
            return timeout;
        }
        if !sim.has_pending_decision() {
            return 0.0;
        }
        (timeout - self.decision_elapsed).max(0.0)
    }
}

#[cfg(test)]
mod tests;
