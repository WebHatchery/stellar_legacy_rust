//! Automatic decision handling. A timeout follows authored priorities and
//! captain authority, so it records who acted instead of inventing a random
//! player conviction.

use super::super::Game;
use crate::simulation::{event_resolver, legacy};
use crate::state::sim::AuthorityChoice;
use crate::state::GameState;

impl Game {
    pub(crate) fn auto_resolve_decision(&mut self) {
        let GameState::Gameplay(gameplay) = &mut self.state else {
            return;
        };
        let sim = &mut gameplay.sim;
        if sim.authority.pending_review.is_some() {
            let captain = sim.authority.captain_name.clone();
            if let Some(posture) = crate::state::sim::authority::complete_review(
                sim,
                AuthorityChoice::AcceptCompromise,
            ) {
                sim.command_posture = posture;
                sim.command_posture_locked_until =
                    crate::simulation::command::next_review_month(sim);
                sim.push_log(format!(
                    "Captain {captain} accepted the compromise posture when the review timer expired."
                ));
            } else {
                sim.push_log(format!(
                    "Captain {captain} retained the current posture when the review timer expired."
                ));
            }
            self.notifications
                .warning("Captain review timed out; the legal compromise was recorded.");
            return;
        }
        if let Some(pending) = sim.pending_event.clone() {
            match self.data.events.get(&pending.template_id).cloned() {
                Some(template) => {
                    let label = event_resolver::auto_resolve(sim, &self.data, &template);
                    let actor = sim
                        .dynasty
                        .leader()
                        .map(|leader| leader.name.as_str())
                        .unwrap_or("the acting office");
                    sim.push_log(format!(
                        "Captain {actor} exercised the standing mandate after the decision timer expired: {label}."
                    ));
                }
                None => sim.pending_event = None,
            }
            self.notifications
                .warning("Captain fallback applied after the decision timer expired.");
            self.check_achievements();
            return;
        }
        if sim.pending_dilemma.is_some() {
            let pick = legacy::pending_dilemma_def(sim, &self.data)
                .map(|dilemma| {
                    dilemma
                        .options
                        .iter()
                        .enumerate()
                        .max_by(|(_, left), (_, right)| {
                            legacy::dilemma_odds(sim, &self.data, left)
                                .total_cmp(&legacy::dilemma_odds(sim, &self.data, right))
                        })
                        .map(|(index, _)| index)
                        .unwrap_or(0)
                })
                .unwrap_or(0);
            legacy::resolve_dilemma(sim, &self.data, pick);
            let actor = sim
                .dynasty
                .leader()
                .map(|leader| leader.name.as_str())
                .unwrap_or("the acting office");
            sim.push_log(format!(
                "Captain {actor} took the highest-probability legacy response after the decision timer expired."
            ));
            self.notifications
                .warning("Captain fallback applied after the decision timer expired.");
            self.check_achievements();
        }
    }
}
