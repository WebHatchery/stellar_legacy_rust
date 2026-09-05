//! Persistent top-right controls, drawn above gameplay overlays.
use crate::state::sim::{GameSpeed, SimState};
use crate::ui::{term, term_button, UiAction, LOGICAL_WIDTH};
use macroquad::prelude::*;
use macroquad_toolkit::ui::{occlude, Pointer};

pub fn draw(sim: &SimState, pointer: Pointer, actions: &mut Vec<UiAction>) {
    let x = LOGICAL_WIDTH - 370.0;
    let panel = Rect::new(x - 6.0, 12.0, 360.0, 58.0);
    draw_rectangle(panel.x, panel.y, panel.w, panel.h, term::panel());
    occlude(panel);
    let paused = sim.speed == GameSpeed::Paused;
    if term_button(
        Rect::new(x, 19.0, 114.0, 44.0),
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
            Rect::new(x + 120.0 + i as f32 * 78.0, 19.0, 72.0, 44.0),
            &label,
            true,
            pointer,
        ) {
            actions.push(UiAction::SetSpeed(speed));
        }
    }
}
