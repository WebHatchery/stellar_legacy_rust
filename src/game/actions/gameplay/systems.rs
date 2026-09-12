//! Systems gameplay action handlers.

use super::Game;
use crate::simulation::{institutions, subsystems};
use crate::state::{GameState, StateTransition};
use crate::ui::UiAction;

impl Game {
    pub(super) fn handle_recruit_faction_group(
        &mut self,
        action: UiAction,
    ) -> Option<StateTransition> {
        match action {
            UiAction::RecruitFactionGroup(id) => {
                if let GameState::Gameplay(gameplay) = &mut self.state {
                    match gameplay.sim.recruit_faction_group(&self.data, &id) {
                        Ok(()) => self.notifications.success("A new people has come aboard."),
                        Err(err) => self.notifications.warning(err),
                    }
                }
                None
            }

            _ => None,
        }
    }

    pub(super) fn handle_repair_subsystem(&mut self, action: UiAction) -> Option<StateTransition> {
        match action {
            UiAction::RepairSubsystem(id) => {
                if matches!(&self.state, GameState::Gameplay(gameplay) if gameplay.sim.contract.is_some())
                {
                    return self.apply_project_action(UiAction::QueueProject {
                        project_id: "service_subsystem".to_owned(),
                        target_id: Some(id),
                    });
                }
                self.subsystem_verb(subsystems::repair_subsystem, &id, "Subsystem mended.");
                None
            }

            _ => None,
        }
    }

    pub(super) fn handle_upgrade_subsystem(&mut self, action: UiAction) -> Option<StateTransition> {
        match action {
            UiAction::UpgradeSubsystem(id) => {
                self.subsystem_verb(
                    subsystems::upgrade_subsystem,
                    &id,
                    "Subsystem rebuilt stronger.",
                );
                None
            }

            _ => None,
        }
    }

    pub(super) fn handle_install_fitting(&mut self, action: UiAction) -> Option<StateTransition> {
        match action {
            UiAction::InstallFitting(id) => {
                self.subsystem_verb(
                    subsystems::install_fitting,
                    &id,
                    "A recovered design goes in.",
                );
                None
            }

            _ => None,
        }
    }

    pub(super) fn handle_train_subsystem_knowledge(
        &mut self,
        action: UiAction,
    ) -> Option<StateTransition> {
        match action {
            UiAction::TrainSubsystemKnowledge(id) => {
                if matches!(&self.state, GameState::Gameplay(gameplay) if gameplay.sim.contract.is_some())
                {
                    return self.apply_project_action(UiAction::QueueProject {
                        project_id: "train_replacement_cohort".to_owned(),
                        target_id: Some(id),
                    });
                }
                self.subsystem_verb(
                    subsystems::train_subsystem_knowledge,
                    &id,
                    "A new cohort takes up the craft.",
                );
                None
            }

            _ => None,
        }
    }

    pub(super) fn handle_establish_school(&mut self, action: UiAction) -> Option<StateTransition> {
        match action {
            UiAction::EstablishSchool(id) => {
                if let GameState::Gameplay(gameplay) = &mut self.state {
                    match institutions::establish_or_support_school(
                        &mut gameplay.sim,
                        &self.data,
                        &id,
                    ) {
                        Ok(name) => self.notifications.success(format!("{name} school funded.")),
                        Err(err) => self.notifications.warning(err),
                    }
                }
                None
            }

            _ => None,
        }
    }

    pub(super) fn handle_compile_procedure_archive(
        &mut self,
        action: UiAction,
    ) -> Option<StateTransition> {
        match action {
            UiAction::CompileProcedureArchive(id) => {
                if let GameState::Gameplay(gameplay) = &mut self.state {
                    match institutions::compile_archive(&mut gameplay.sim, &self.data, &id) {
                        Ok(name) => self
                            .notifications
                            .success(format!("{name} archive compiled.")),
                        Err(err) => self.notifications.warning(err),
                    }
                }
                None
            }

            _ => None,
        }
    }

    pub(super) fn handle_begin_discipline_custody(
        &mut self,
        action: UiAction,
    ) -> Option<StateTransition> {
        match action {
            UiAction::BeginDisciplineCustody(id) => {
                self.custody_picker = Some(id);
                None
            }

            _ => None,
        }
    }

    pub(super) fn handle_cancel_discipline_custody(
        &mut self,
        action: UiAction,
    ) -> Option<StateTransition> {
        match action {
            UiAction::CancelDisciplineCustody => {
                self.custody_picker = None;
                None
            }

            _ => None,
        }
    }

    pub(super) fn handle_grant_discipline_custody(
        &mut self,
        action: UiAction,
    ) -> Option<StateTransition> {
        match action {
            UiAction::GrantDisciplineCustody {
                subsystem_id,
                faction_id,
            } => {
                let mut granted = false;
                if let GameState::Gameplay(gameplay) = &mut self.state {
                    match institutions::grant_custodianship(
                        &mut gameplay.sim,
                        &self.data,
                        &subsystem_id,
                        &faction_id,
                    ) {
                        Ok(name) => {
                            granted = true;
                            self.notifications
                                .success(format!("{name} granted custody."));
                        }
                        Err(err) => self.notifications.warning(err),
                    }
                }
                if granted {
                    self.custody_picker = None;
                }
                None
            }

            _ => None,
        }
    }

    pub(super) fn handle_open_obligation_history(
        &mut self,
        action: UiAction,
    ) -> Option<StateTransition> {
        match action {
            UiAction::OpenObligationHistory(id) => {
                self.obligation_detail = Some(id);
                self.obligation_history_scroll
                    .set(macroquad_toolkit::ui::ScrollArea::new());
                None
            }

            _ => None,
        }
    }

    pub(super) fn handle_close_obligation_history(
        &mut self,
        action: UiAction,
    ) -> Option<StateTransition> {
        match action {
            UiAction::CloseObligationHistory => {
                self.obligation_detail = None;
                None
            }

            _ => None,
        }
    }
}
