//! Original amber/yellow and green phosphor-terminal palettes (GDD §0).
//! Text, panels, and controls follow the selected tube; alerts remain warm red.
use crate::settings::Phosphor;
use macroquad::prelude::Color;
use std::cell::Cell;

thread_local! {
    static PHOSPHOR: Cell<Phosphor> = const { Cell::new(Phosphor::Amber) };
}

/// Switch the active phosphor tube for all subsequent draws.
pub fn set_phosphor(phosphor: Phosphor) {
    PHOSPHOR.with(|cell| cell.set(phosphor));
}

fn tube(amber: Color, green: Color) -> Color {
    match PHOSPHOR.with(Cell::get) {
        Phosphor::Amber => amber,
        Phosphor::Green => green,
    }
}

pub fn bg() -> Color {
    tube(
        Color::new(0.015, 0.012, 0.004, 1.0),
        Color::new(0.003, 0.016, 0.006, 1.0),
    )
}

pub fn panel() -> Color {
    tube(
        Color::new(0.06, 0.047, 0.012, 0.98),
        Color::new(0.015, 0.052, 0.023, 0.98),
    )
}

pub fn panel_header() -> Color {
    tube(
        Color::new(0.13, 0.095, 0.02, 1.0),
        Color::new(0.03, 0.12, 0.045, 1.0),
    )
}

pub fn primary() -> Color {
    tube(
        Color::new(1.0, 0.75, 0.14, 1.0),
        Color::new(0.42, 1.0, 0.56, 1.0),
    )
}

pub fn dim() -> Color {
    tube(
        Color::new(0.74, 0.54, 0.11, 1.0),
        Color::new(0.26, 0.74, 0.38, 1.0),
    )
}

pub fn faint() -> Color {
    tube(
        Color::new(0.44, 0.32, 0.08, 1.0),
        Color::new(0.14, 0.42, 0.2, 1.0),
    )
}

/// Success / value accent — a brighter tint of the tube hue.
pub fn accent() -> Color {
    tube(
        Color::new(0.2, 1.0, 0.5, 1.0),
        Color::new(0.62, 1.0, 0.72, 1.0),
    )
}

/// Alert red — warm on both tubes so danger still reads on a green screen.
pub fn alert() -> Color {
    Color::new(1.0, 0.32, 0.24, 1.0)
}

pub fn border() -> Color {
    tube(
        Color::new(0.82, 0.6, 0.14, 0.95),
        Color::new(0.3, 0.8, 0.42, 0.95),
    )
}

// Dark interactive surface fills (buttons, tabs, selectable rows), tinted to
// the tube so nothing reads warm on the green screen.
pub fn surface() -> Color {
    tube(
        Color::new(0.12, 0.092, 0.017, 1.0),
        Color::new(0.022, 0.075, 0.034, 1.0),
    )
}

pub fn surface_hover() -> Color {
    tube(
        Color::new(0.26, 0.19, 0.025, 1.0),
        Color::new(0.05, 0.16, 0.075, 1.0),
    )
}

pub fn surface_active() -> Color {
    tube(
        Color::new(0.3, 0.22, 0.03, 1.0),
        Color::new(0.06, 0.18, 0.085, 1.0),
    )
}

pub fn surface_disabled() -> Color {
    tube(
        Color::new(0.05, 0.04, 0.02, 1.0),
        Color::new(0.01, 0.035, 0.016, 1.0),
    )
}

pub fn surface_inset() -> Color {
    tube(
        Color::new(0.07, 0.055, 0.012, 1.0),
        Color::new(0.014, 0.05, 0.024, 1.0),
    )
}
