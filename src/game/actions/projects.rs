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
            } => self.queue_project(project_id, target_id),
            UiAction::PauseProject(id) => self.pause_project(id),
            UiAction::ResumeProject(id) => self.resume_project(id),
            UiAction::MoveProject {
                sequence_id,
                direction,
            } => self.move_project(sequence_id, direction),
            UiAction::CancelProject(id) => self.cancel_project(id),
            UiAction::ReviewRecovery => self.review_recovery(),
            UiAction::EmergencyStabilise => self.emergency_stabilise(),
            UiAction::ResumeAfterWarning => self.resume_after_warning(),
            _ => {}
        }
        None
    }

    fn queue_project(&mut self, project_id: String, target_id: Option<String>) {
        let result = if let GameState::Gameplay(gameplay) = &mut self.state {
            projects::queue_project(&mut gameplay.sim, &self.data, &project_id, target_id)
        } else {
            Err("No active campaign.".to_owned())
        };
        match result {
            Ok(id) => {
                self.select_queued_project(id);
                self.presentation.mobile_section.borrow_mut().clear();
                self.presentation.utilities.set(false);
                self.project_cancel_confirm.set(None);
                self.presentation.project_cancellation.set(None);
                let name = self
                    .data
                    .projects
                    .get(&project_id)
                    .map_or(project_id.as_str(), |definition| definition.name.as_str());
                self.notifications
                    .success(format!("{name} added to the Agenda."));
            }
            Err(error) => self.notifications.warning(error),
        }
    }

    fn select_queued_project(&mut self, id: u64) {
        if let GameState::Gameplay(gameplay) = &mut self.state {
            gameplay.screen = crate::state::Screen::Agenda;
            if let Some(index) = gameplay
                .sim
                .projects
                .jobs
                .iter()
                .position(|job| job.sequence_id == id)
            {
                self.presentation.selected_agenda.set(index);
                let mut scroll = self.agenda_scroll.get();
                scroll.set_offset(index as f32 * 66.0);
                self.agenda_scroll.set(scroll);
            }
        }
    }

    fn pause_project(&mut self, id: u64) {
        let result = if let GameState::Gameplay(gameplay) = &mut self.state {
            projects::pause_project(&mut gameplay.sim, &self.data, id)
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

    fn resume_project(&mut self, id: u64) {
        let result = if let GameState::Gameplay(gameplay) = &mut self.state {
            projects::resume_project(&mut gameplay.sim, &self.data, id)
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

    fn move_project(&mut self, id: u64, direction: i32) {
        let mut selected = None;
        let mut notice = None;
        let mut error = None;
        if let GameState::Gameplay(gameplay) = &mut self.state {
            let before = gameplay.sim.projects.waiting_position(id);
            match projects::move_project(&mut gameplay.sim, id, direction) {
                Err(message) => error = Some(message),
                Ok(()) => {
                    selected = gameplay
                        .sim
                        .projects
                        .jobs
                        .iter()
                        .position(|job| job.sequence_id == id);
                    let after = gameplay.sim.projects.waiting_position(id);
                    if after != before {
                        if let Some((position, count)) = after {
                            let name = gameplay
                                .sim
                                .projects
                                .find(id)
                                .and_then(|job| self.data.projects.get(&job.project_id))
                                .map_or("Project", |definition| definition.name.as_str());
                            notice =
                                Some(format!("{name}: waiting position {position} of {count}."));
                        }
                    }
                }
            }
        }
        if let Some(index) = selected {
            self.presentation.selected_agenda.set(index);
        }
        if let Some(message) = error {
            self.notifications.warning(message);
        }
        if let Some(message) = notice {
            self.notifications.info(message);
        }
    }

    fn cancel_project(&mut self, id: u64) {
        let result = if let GameState::Gameplay(gameplay) = &mut self.state {
            projects::cancel_project(&mut gameplay.sim, &self.data, id)
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

    fn review_recovery(&mut self) {
        if let GameState::Gameplay(gameplay) = &mut self.state {
            survival::review_recovery(&mut gameplay.sim);
            gameplay.screen = crate::state::Screen::Agenda;
        }
    }

    fn emergency_stabilise(&mut self) {
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

    fn resume_after_warning(&mut self) {
        if let GameState::Gameplay(gameplay) = &mut self.state {
            survival::resume_after_warning(&mut gameplay.sim);
        }
    }
}
#[allow(dead_code, unused_imports)]
mod tests {
    include!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/unit/game/actions/projects/tests.rs"
    ));
}
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
