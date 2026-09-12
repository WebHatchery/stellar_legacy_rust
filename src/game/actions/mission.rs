//! Return-home review lifetime belongs to action handling, never rendering.
use crate::{simulation::contract, state::SimState, ui::UiAction};

pub(super) fn review_after_action(open: bool, action: &UiAction, sim: &SimState) -> bool {
    match action {
        UiAction::ReviewReturnHome => !sim.has_pending_decision() && contract::can_return_home(sim),
        UiAction::DismissReturnHome
        | UiAction::AbortMission
        | UiAction::SelectScreen(_)
        | UiAction::ToMenu => false,
        _ => open,
    }
}
#[allow(dead_code, unused_imports)]
mod tests {
    include!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/unit/game/actions/mission/tests.rs"
    ));
}
