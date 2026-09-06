//! Dispatch for the Custodian/captain authority review.

use super::super::Game;
use crate::state::sim::AuthorityChoice;
use crate::state::GameState;
use crate::ui::UiAction;

impl Game {
    pub(crate) fn apply_authority_action(
        &mut self,
        action: UiAction,
    ) -> Option<crate::state::StateTransition> {
        match action {
            UiAction::SetPosture(posture) => {
                if let GameState::Gameplay(gameplay) = &mut self.state {
                    let sim = &mut gameplay.sim;
                    if !crate::simulation::command::posture_change_allowed(sim) {
                        let months = sim
                            .command_posture_locked_until
                            .saturating_sub(sim.month_clock);
                        self.notifications
                            .warning(format!("Command review locked for {months} more months."));
                    } else if sim.command_posture != posture {
                        if let Some(review) =
                            crate::state::sim::authority::posture_review(sim, posture)
                        {
                            let captain = review.captain.clone();
                            sim.authority.pending_review = Some(review);
                            sim.push_log(format!(
                                "Captain {captain} opened a mandate review over the proposed {} posture.",
                                posture.label()
                            ));
                            self.notifications
                                .warning("Captain review required before the posture can change.");
                        } else {
                            sim.command_posture = posture;
                            sim.command_posture_locked_until =
                                crate::simulation::command::next_review_month(sim);
                            sim.push_log(format!(
                                "The Custodian proposed and the captain ratified {} posture.",
                                posture.label()
                            ));
                            self.notifications
                                .success(format!("Posture ratified: {}", posture.label()));
                        }
                    }
                }
            }
            UiAction::ResolveAuthority(choice) => {
                if let GameState::Gameplay(gameplay) = &mut self.state {
                    let sim = &mut gameplay.sim;
                    let review = sim.authority.pending_review.clone();
                    let selected = crate::state::sim::authority::complete_review(sim, choice);
                    let captain = review
                        .as_ref()
                        .map(|review| review.captain.as_str())
                        .filter(|name| !name.is_empty())
                        .unwrap_or("the captain");
                    match (choice, selected) {
                        (AuthorityChoice::KeepCurrent, _) => {
                            sim.push_log(format!(
                                "{captain} objected to the posture change; the current policy remains in force."
                            ));
                            self.notifications
                                .info("Current posture retained under captain review.");
                        }
                        (_, Some(posture))
                            if crate::simulation::command::posture_change_allowed(sim) =>
                        {
                            sim.command_posture = posture;
                            sim.command_posture_locked_until =
                                crate::simulation::command::next_review_month(sim);
                            if choice == AuthorityChoice::EmergencyOverride {
                                sim.population.stability =
                                    (sim.population.stability - 0.03).clamp(0.0, 1.0);
                                sim.push_log(format!(
                                    "The Custodian invoked emergency authority over {captain}'s objection and set {} posture.",
                                    posture.label()
                                ));
                                self.notifications.warning(
                                    "Emergency posture override applied; stability cost recorded.",
                                );
                            } else {
                                sim.push_log(format!(
                                    "{captain} and the Custodian accepted the {} compromise posture.",
                                    posture.label()
                                ));
                                self.notifications
                                    .success(format!("Captain compromise: {}", posture.label()));
                            }
                        }
                        (_, Some(_)) => {
                            sim.push_log("The captain review expired after the posture lock changed; no cost was applied.");
                            self.notifications
                                .warning("Posture review became stale; nothing was changed.");
                        }
                        (_, None) => {
                            self.notifications
                                .warning("That authority response is no longer available.");
                        }
                    }
                }
            }
            _ => {}
        }
        None
    }
}
