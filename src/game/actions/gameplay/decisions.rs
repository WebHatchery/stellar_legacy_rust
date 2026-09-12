//! Decisions gameplay action handlers.

use super::Game;
use crate::simulation::{crew, event_resolver, institutions, legacy};
use crate::state::{GameState, StateTransition};
use crate::ui::UiAction;

impl Game {
    pub(super) fn handle_resolve_event(&mut self, action: UiAction) -> Option<StateTransition> {
        match action {
            UiAction::ResolveEvent(index) => {
                if let GameState::Gameplay(gameplay) = &mut self.state {
                    let sim = &mut gameplay.sim;
                    let template = sim
                        .pending_event
                        .as_ref()
                        .and_then(|p| self.data.events.get(&p.template_id))
                        .cloned();
                    if let Some(template) = template {
                        event_resolver::apply_outcome(sim, &self.data, &template, index);
                    } else {
                        sim.pending_event = None;
                    }
                }
                self.check_terminal_after_action();
                None
            }

            _ => None,
        }
    }

    pub(super) fn handle_resolve_dilemma(&mut self, action: UiAction) -> Option<StateTransition> {
        match action {
            UiAction::ResolveDilemma(index) => {
                if let GameState::Gameplay(gameplay) = &mut self.state {
                    legacy::resolve_dilemma(&mut gameplay.sim, &self.data, index);
                }
                self.check_terminal_after_action();
                None
            }

            _ => None,
        }
    }

    pub(super) fn handle_recruit_crew(&mut self, action: UiAction) -> Option<StateTransition> {
        match action {
            UiAction::RecruitCrew(archetype_id) => {
                if let GameState::Gameplay(gameplay) = &mut self.state {
                    match crew::recruit(&mut gameplay.sim, &self.data, &archetype_id) {
                        Ok(name) => self.notifications.success(format!("{name} signed on.")),
                        Err(err) => self.notifications.warning(err),
                    }
                }
                None
            }

            _ => None,
        }
    }

    pub(super) fn handle_train_crew(&mut self, action: UiAction) -> Option<StateTransition> {
        match action {
            UiAction::TrainCrew(archetype_id) => {
                if let GameState::Gameplay(gameplay) = &mut self.state {
                    match crew::train(&mut gameplay.sim, &self.data, &archetype_id) {
                        Ok(name) => self
                            .notifications
                            .success(format!("{name} completed training.")),
                        Err(err) => self.notifications.warning(err),
                    }
                }
                None
            }

            _ => None,
        }
    }

    pub(super) fn handle_designate_apprentice(
        &mut self,
        action: UiAction,
    ) -> Option<StateTransition> {
        match action {
            UiAction::DesignateApprentice(archetype_id) => {
                if let GameState::Gameplay(gameplay) = &mut self.state {
                    match institutions::designate_apprentice(
                        &mut gameplay.sim,
                        &self.data,
                        &archetype_id,
                    ) {
                        Ok(name) => self.notifications.success(format!("{name} designated.")),
                        Err(err) => self.notifications.warning(err),
                    }
                }
                None
            }

            _ => None,
        }
    }

    pub(super) fn handle_select_heir(&mut self, action: UiAction) -> Option<StateTransition> {
        match action {
            UiAction::SelectHeir(member_id) => {
                if let GameState::Gameplay(gameplay) = &mut self.state {
                    let sim = &mut gameplay.sim;
                    let name = sim
                        .dynasty
                        .members
                        .iter()
                        .find(|m| m.id == member_id && !m.is_leader)
                        .map(|m| m.name.clone());
                    if let Some(name) = name {
                        sim.dynasty.designated_heir = Some(member_id);
                        sim.push_log(format!("The council named {name} heir designate."));
                        self.notifications.success(format!("{name} named heir."));
                    }
                }
                None
            }

            _ => None,
        }
    }
}
