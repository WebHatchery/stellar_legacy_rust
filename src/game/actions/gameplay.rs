//! Gameplay actions: ship, crew, council, and voyage decisions.

use super::Game;
use crate::state::StateTransition;
use crate::ui::UiAction;

mod charter;
mod core;
mod decisions;
mod logistics;
mod systems;

impl Game {
    pub(super) fn apply_gameplay_action(&mut self, action: UiAction) -> Option<StateTransition> {
        match action {
            action @ UiAction::TogglePause => self.handle_toggle_pause(action),
            action @ UiAction::SetSpeed(..) => self.handle_set_speed(action),
            action @ UiAction::SetPosture(..) => self.handle_set_posture(action),
            action @ UiAction::ResolveAuthority(..) => self.handle_resolve_authority(action),
            action @ UiAction::ReviewReturnHome => self.handle_review_return_home(action),
            action @ UiAction::DismissReturnHome => self.handle_dismiss_return_home(action),
            action @ UiAction::AbortMission => self.handle_abort_mission(action),
            action @ UiAction::RecruitFactionGroup(..) => self.handle_recruit_faction_group(action),
            action @ UiAction::RepairSubsystem(..) => self.handle_repair_subsystem(action),
            action @ UiAction::UpgradeSubsystem(..) => self.handle_upgrade_subsystem(action),
            action @ UiAction::InstallFitting(..) => self.handle_install_fitting(action),
            action @ UiAction::TrainSubsystemKnowledge(..) => {
                self.handle_train_subsystem_knowledge(action)
            }
            action @ UiAction::EstablishSchool(..) => self.handle_establish_school(action),
            action @ UiAction::CompileProcedureArchive(..) => {
                self.handle_compile_procedure_archive(action)
            }
            action @ UiAction::BeginDisciplineCustody(..) => {
                self.handle_begin_discipline_custody(action)
            }
            action @ UiAction::CancelDisciplineCustody => {
                self.handle_cancel_discipline_custody(action)
            }
            action @ UiAction::GrantDisciplineCustody { .. } => {
                self.handle_grant_discipline_custody(action)
            }
            action @ UiAction::OpenObligationHistory(..) => {
                self.handle_open_obligation_history(action)
            }
            action @ UiAction::CloseObligationHistory => {
                self.handle_close_obligation_history(action)
            }
            action @ UiAction::ResolveEvent(..) => self.handle_resolve_event(action),
            action @ UiAction::ResolveDilemma(..) => self.handle_resolve_dilemma(action),
            action @ UiAction::RecruitCrew(..) => self.handle_recruit_crew(action),
            action @ UiAction::TrainCrew(..) => self.handle_train_crew(action),
            action @ UiAction::DesignateApprentice(..) => self.handle_designate_apprentice(action),
            action @ UiAction::SelectHeir(..) => self.handle_select_heir(action),
            action @ UiAction::CancelSelection => self.handle_cancel_selection(action),
            action @ UiAction::SelectCharter(..) => self.handle_select_charter(action),
            action @ UiAction::SetCharterApproach(..) => self.handle_set_charter_approach(action),
            action @ UiAction::Launch => self.handle_launch(action),
            action @ UiAction::Refuel => self.handle_refuel(action),
            action @ UiAction::ReviewProvisions => self.handle_review_provisions(action),
            action @ UiAction::NextTutorial => self.handle_next_tutorial(action),
            action @ UiAction::SkipTutorial => self.handle_skip_tutorial(action),
            action @ UiAction::BuyParts(..) => self.handle_buy_parts(action),
            action @ UiAction::PurchaseComponent(..) => self.handle_purchase_component(action),
            action @ UiAction::FieldRepair(..) => self.handle_field_repair(action),
            action @ UiAction::FullRepair => self.handle_full_repair(action),
            action @ UiAction::InstallSalvage(..) => self.handle_install_salvage(action),
            action @ UiAction::CommissionShip(..) => self.handle_commission_ship(action),
            action @ UiAction::Buy(..) => self.handle_buy(action),
            action @ UiAction::Sell(..) => self.handle_sell(action),
            action @ UiAction::ToggleDelegation(..) => self.handle_toggle_delegation(action),
            _ => None,
        }
    }
}
