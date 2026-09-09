//! Agenda and survival-review action dispatch.

use super::Game;
use crate::simulation::{projects, survival};
use crate::state::GameState;
use crate::ui::UiAction;

/// Review ownership follows navigation; it must never leave an invisible clock hold.
pub(super) fn review_after_action(
    review: Option<u64>,
    cancellation: Option<u64>,
    action: &UiAction,
) -> (Option<u64>, Option<u64>) {
    match action {
        UiAction::PreviewCancelProject(id) => (Some(*id), None),
        UiAction::ReviewCancelProject(id) if review == Some(*id) => (review, Some(*id)),
        UiAction::DismissCancelProject
        | UiAction::CancelProject(_)
        | UiAction::SelectScreen(_)
        | UiAction::ReviewRecovery
        | UiAction::ReviewReturnHome
        | UiAction::FileReport
        | UiAction::RetireVoyage
        | UiAction::ToMenu => (None, None),
        _ => (review, cancellation),
    }
}

impl Game {
    pub(super) fn check_terminal_after_action(&mut self) {
        let outcome = if let GameState::Gameplay(gameplay) = &mut self.state {
            crate::simulation::readiness::refresh(&mut gameplay.sim, &self.data);
            survival::observe_air_warning(&mut gameplay.sim, &self.data);
            survival::check_and_record(&mut gameplay.sim, &self.data)
        } else {
            None
        };
        if let Some(outcome) = outcome {
            self.notifications.danger(outcome.reason.label());
        }
    }

    pub(super) fn apply_project_action(
        &mut self,
        action: UiAction,
    ) -> Option<crate::state::StateTransition> {
        match action {
            UiAction::QueueProject {
                project_id,
                target_id,
            } => {
                let result = if let GameState::Gameplay(gameplay) = &mut self.state {
                    projects::queue_project(&mut gameplay.sim, &self.data, &project_id, target_id)
                } else {
                    Err("No active campaign.".to_owned())
                };
                match result {
                    Ok(_) => self.notifications.success("Project added to the Agenda."),
                    Err(error) => self.notifications.warning(error),
                }
            }
            UiAction::PauseProject(sequence_id) => {
                let result = if let GameState::Gameplay(gameplay) = &mut self.state {
                    projects::pause_project(&mut gameplay.sim, &self.data, sequence_id)
                } else {
                    Err("No active campaign.".to_owned())
                };
                match result {
                    Ok(()) => self
                        .notifications
                        .info("Project paused; its slot is available."),
                    Err(error) => self.notifications.warning(error),
                }
            }
            UiAction::ResumeProject(sequence_id) => {
                let result = if let GameState::Gameplay(gameplay) = &mut self.state {
                    projects::resume_project(&mut gameplay.sim, &self.data, sequence_id)
                } else {
                    Err("No active campaign.".to_owned())
                };
                match result {
                    Ok(()) => self
                        .notifications
                        .success("Project resumed; restoration debt paid."),
                    Err(error) => self.notifications.warning(error),
                }
            }
            UiAction::MoveProject {
                sequence_id,
                direction,
            } => {
                if let GameState::Gameplay(gameplay) = &mut self.state {
                    let before = gameplay.sim.projects.waiting_position(sequence_id);
                    match projects::move_project(&mut gameplay.sim, sequence_id, direction) {
                        Err(error) => self.notifications.warning(error),
                        Ok(()) => {
                            if let Some(index) = gameplay
                                .sim
                                .projects
                                .jobs
                                .iter()
                                .position(|job| job.sequence_id == sequence_id)
                            {
                                self.presentation.selected_agenda.set(index);
                            }
                            let after = gameplay.sim.projects.waiting_position(sequence_id);
                            if after != before {
                                if let Some((position, count)) = after {
                                    let name = gameplay
                                        .sim
                                        .projects
                                        .find(sequence_id)
                                        .and_then(|job| self.data.projects.get(&job.project_id))
                                        .map_or("Project", |definition| definition.name.as_str());
                                    self.notifications.info(format!(
                                        "{name}: waiting position {position} of {count}."
                                    ));
                                }
                            }
                        }
                    }
                }
            }
            UiAction::CancelProject(sequence_id) => {
                let result = if let GameState::Gameplay(gameplay) = &mut self.state {
                    projects::cancel_project(&mut gameplay.sim, &self.data, sequence_id)
                } else {
                    Err("No active campaign.".to_owned())
                };
                match result {
                    Ok(refund) => self.notifications.info(format!(
                        "Project cancelled; {} returned to stores.",
                        format_refund(refund)
                    )),
                    Err(error) => self.notifications.warning(error),
                }
            }
            UiAction::ReviewRecovery => {
                if let GameState::Gameplay(gameplay) = &mut self.state {
                    survival::review_recovery(&mut gameplay.sim);
                    gameplay.screen = crate::state::Screen::Agenda;
                }
            }
            UiAction::EmergencyStabilise => {
                let result = if let GameState::Gameplay(gameplay) = &mut self.state {
                    survival::emergency_stabilise(&mut gameplay.sim, &self.data)
                } else {
                    Err("No active campaign.".to_owned())
                };
                match result {
                    Ok(()) => self.notifications.success("Emergency air stabilised."),
                    Err(error) => self.notifications.warning(error),
                }
            }
            UiAction::ResumeAfterWarning => {
                if let GameState::Gameplay(gameplay) = &mut self.state {
                    survival::resume_after_warning(&mut gameplay.sim);
                }
            }
            _ => {}
        }
        None
    }
}

#[cfg(test)]
mod tests;

fn format_refund(amounts: crate::state::sim::ProjectAmounts) -> String {
    let mut values = Vec::new();
    if amounts.credits > 0.0 {
        values.push(format!("{:.0}cr", amounts.credits));
    }
    if amounts.energy > 0.0 {
        values.push(format!("{:.0}en", amounts.energy));
    }
    if amounts.minerals > 0.0 {
        values.push(format!("{:.0}min", amounts.minerals));
    }
    if amounts.food > 0.0 {
        values.push(format!("{:.0} food", amounts.food));
    }
    if amounts.spare_parts > 0.0 {
        values.push(format!("{:.0} parts", amounts.spare_parts));
    }
    if values.is_empty() {
        "nothing".to_owned()
    } else {
        values.join(" · ")
    }
}
