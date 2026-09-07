//! Reading surfaces are neutral; phosphor remains an identity/selection accent.
use crate::settings::Phosphor;
use macroquad::prelude::Color;
use std::cell::Cell;

thread_local! {
    static PHOSPHOR: Cell<Phosphor> = const { Cell::new(Phosphor::Amber) };
}
pub fn set_phosphor(value: Phosphor) {
    PHOSPHOR.with(|p| p.set(value));
}
pub fn bg() -> Color {
    Color::new(0.018, 0.027, 0.045, 1.0)
}
pub fn panel() -> Color {
    Color::new(0.040, 0.059, 0.086, 1.0)
}
pub fn panel_header() -> Color {
    Color::new(0.072, 0.094, 0.125, 1.0)
}
pub fn primary() -> Color {
    match PHOSPHOR.with(Cell::get) {
        Phosphor::Amber => Color::new(0.95, 0.77, 0.40, 1.0),
        Phosphor::Green => Color::new(0.56, 0.88, 0.68, 1.0),
    }
}
/// Warm off-white prose (separate from gold headings).
pub fn dim() -> Color {
    Color::new(0.90, 0.89, 0.85, 1.0)
}
/// Secondary copy still clears normal-text contrast on reading surfaces.
pub fn faint() -> Color {
    Color::new(0.68, 0.71, 0.73, 1.0)
}
pub fn accent() -> Color {
    Color::new(0.48, 0.81, 0.65, 1.0)
}
pub fn alert() -> Color {
    Color::new(1.0, 0.61, 0.50, 1.0)
}
pub fn border() -> Color {
    Color::new(0.44, 0.47, 0.51, 1.0)
}
pub fn surface() -> Color {
    Color::new(0.085, 0.11, 0.15, 1.0)
}
pub fn surface_hover() -> Color {
    Color::new(0.14, 0.18, 0.23, 1.0)
}
pub fn surface_active() -> Color {
    Color::new(0.18, 0.22, 0.27, 1.0)
}
pub fn surface_disabled() -> Color {
    Color::new(0.065, 0.080, 0.10, 1.0)
}
pub fn surface_inset() -> Color {
    Color::new(0.028, 0.043, 0.066, 1.0)
}
