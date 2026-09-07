//! Session-local presentation choices. These never enter the simulation save.
use macroquad_toolkit::ui::ScrollArea;
use std::cell::Cell;

#[derive(Default)]
pub struct Presentation {
    pub selected_system: Cell<usize>,
    pub selected_agenda: Cell<usize>,
    pub utilities: Cell<bool>,
    pub instruments: Cell<bool>,
    pub event_key: std::cell::RefCell<String>,
    pub event_choice: Cell<usize>,
    pub situation_scroll: Cell<ScrollArea>,
    pub event_advice: Cell<bool>,
    pub event_scroll: Cell<ScrollArea>,
}
