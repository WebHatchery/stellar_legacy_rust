use super::*;
use crate::state::Screen;

#[test]
fn navigation_releases_project_review_and_cancellation_holds() {
    for exit in [
        UiAction::SelectScreen(Screen::Dashboard),
        UiAction::SelectScreen(Screen::Agenda),
        UiAction::DismissCancelProject,
        UiAction::ReviewRecovery,
        UiAction::ReviewReturnHome,
        UiAction::FileReport,
        UiAction::RetireVoyage,
        UiAction::ToMenu,
        UiAction::CancelProject(7),
    ] {
        assert_eq!(review_after_action(Some(7), None, &exit), (None, None));
        assert_eq!(review_after_action(Some(7), Some(7), &exit), (None, None));
    }
}

#[test]
fn project_management_keeps_review_open_and_scopes_confirmation_to_its_job() {
    let mut state = review_after_action(None, None, &UiAction::PreviewCancelProject(7));
    for action in [UiAction::PauseProject(7), UiAction::ResumeProject(7)] {
        state = review_after_action(state.0, state.1, &action);
        assert_eq!(state, (Some(7), None));
    }
    state = review_after_action(state.0, state.1, &UiAction::ReviewCancelProject(8));
    assert_eq!(state, (Some(7), None));
    state = review_after_action(state.0, state.1, &UiAction::ReviewCancelProject(7));
    assert_eq!(state, (Some(7), Some(7)));
    state = review_after_action(state.0, state.1, &UiAction::PreviewCancelProject(8));
    assert_eq!(state, (Some(8), None));
}
