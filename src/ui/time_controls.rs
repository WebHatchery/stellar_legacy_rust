//! Persistent top-right time controls, drawn above gameplay overlays.
//!
//! Pause/Resume and the speed bank form one group, apart from the cog menu.
//! When no voyage clock exists (in port, Homecoming review) the group becomes
//! a plain state plate, so choosing 3x never looks like it moves a docked
//! ship. When the clock exists but is held, a status line names why.
use crate::state::sim::{GameSpeed, SimState};
use crate::ui::{logical_width, term, term_button, UiAction};
use macroquad::prelude::*;
use macroquad_toolkit::prelude::{draw_text_centered_in_box_ex, TextStyle};
use macroquad_toolkit::ui::{draw_ui_text_ex, measure_text_size, occlude, Pointer};

const PANEL_W: f32 = 360.0;

pub(crate) fn fallback_label(speed: GameSpeed, remaining: f32) -> String {
    let seconds = remaining.ceil().max(0.0) as u32;
    if speed == GameSpeed::Paused {
        format!("Fallback paused · {seconds}s")
    } else {
        format!("Fallback in {seconds}s")
    }
}

/// Why the voyage clock is or is not running this frame.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClockState {
    /// Docked with no charter under way: time is frozen by design.
    InPort,
    /// A sealed voyage report awaits review; nothing advances until filed.
    ReportReview,
    /// A blocking decision holds the voyage; its fallback countdown runs
    /// unless paused, in port as well as underway.
    Decision,
    /// The guide holds the clock until its current lesson is done.
    TutorialHold,
    /// The player paused.
    Paused,
    /// Time runs but the dry tank stops travel progress.
    FuelStall,
    /// Time runs normally.
    Running,
}

impl ClockState {
    /// `tutorial_hold` and `fuel_stalled` come from the game shell, which owns
    /// the guide and the shared travel fuel check.
    pub fn of(sim: &SimState, tutorial_hold: bool, fuel_stalled: bool) -> Self {
        // A decision's fallback countdown runs even in port, so its Pause
        // control outranks the docked plate.
        if sim.has_pending_decision() {
            Self::Decision
        } else if sim.debrief.is_some() {
            Self::ReportReview
        } else if sim.contract.is_none() {
            Self::InPort
        } else if tutorial_hold {
            Self::TutorialHold
        } else if sim.speed == GameSpeed::Paused {
            Self::Paused
        } else if fuel_stalled {
            Self::FuelStall
        } else {
            Self::Running
        }
    }

    /// The status line beside the controls; `None` while time simply runs.
    fn status(self, sim: &SimState) -> Option<(&'static str, Color)> {
        match self {
            Self::InPort | Self::ReportReview | Self::Running => None,
            Self::Decision if sim.speed == GameSpeed::Paused => {
                Some(("Decision · fallback paused", term::dim()))
            }
            Self::Decision => Some(("Decision · fallback running", term::alert())),
            Self::TutorialHold => Some(("Held by the guide", term::dim())),
            Self::Paused => Some(("Paused", term::dim())),
            Self::FuelStall => Some(("! Fuel stall · no travel progress", term::alert())),
        }
    }
}

pub fn draw(sim: &SimState, clock: ClockState, pointer: Pointer, actions: &mut Vec<UiAction>) {
    let x = logical_width() - 370.0;
    draw_at(sim, clock, pointer, actions, x, 19.0);
}

pub fn draw_at(
    sim: &SimState,
    clock: ClockState,
    pointer: Pointer,
    actions: &mut Vec<UiAction>,
    x: f32,
    y: f32,
) {
    let panel = Rect::new(x - 6.0, y - 7.0, PANEL_W, 58.0);
    draw_rectangle(panel.x, panel.y, panel.w, panel.h, term::panel());
    occlude(panel);
    let plate = match clock {
        ClockState::InPort => Some("In port · time stopped"),
        ClockState::ReportReview => Some("Report review · time stopped"),
        _ => None,
    };
    if let Some(plate) = plate {
        let rect = Rect::new(x, y, PANEL_W - 12.0, 44.0);
        draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 1.0, term::faint());
        draw_text_centered_in_box_ex(
            plate,
            rect.x,
            rect.y,
            rect.w,
            rect.h,
            TextStyle::new(17.0, term::dim()),
        );
        return;
    }
    let paused = sim.speed == GameSpeed::Paused;
    if term_button(
        Rect::new(x, y, 114.0, 44.0),
        if paused { "RESUME" } else { "PAUSE" },
        true,
        pointer,
    ) {
        actions.push(UiAction::TogglePause);
    }
    for (i, speed) in GameSpeed::ALL.into_iter().skip(1).enumerate() {
        let label = if sim.speed == speed {
            format!("[{}]", speed.label())
        } else {
            speed.label().to_owned()
        };
        if term_button(
            Rect::new(x + 120.0 + i as f32 * 78.0, y, 72.0, 44.0),
            &label,
            true,
            pointer,
        ) {
            actions.push(UiAction::SetSpeed(speed));
        }
    }
    if let Some((status, color)) = clock.status(sim) {
        let style = TextStyle::new(17.0, color);
        let w = measure_text_size(status, style).width;
        draw_ui_text_ex(status, panel.x - 16.0 - w, y + 28.0, style.params());
    }
}
