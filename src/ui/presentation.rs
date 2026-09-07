//! Session-local presentation choices. These never enter the simulation save.
use macroquad_toolkit::ui::ScrollArea;
use std::cell::Cell;

#[derive(Default)]
pub struct Presentation {
    pub capture_mobile_offset: Cell<Option<(bool, f32)>>,
    pub overlay_active: Cell<bool>,
    pub overlay_key: std::cell::RefCell<String>,
    pub overlay_scroll: Cell<ScrollArea>,
    pub mobile_section: std::cell::RefCell<String>,
    pub mobile_key: std::cell::RefCell<String>,
    pub mobile_scroll: Cell<ScrollArea>,
    pub history_page: Cell<usize>,
    pub selected_record: Cell<usize>,
    pub record_scroll: Cell<ScrollArea>,
    pub report_page: Cell<usize>,
    pub report_scroll: Cell<ScrollArea>,
    pub people_page: Cell<usize>,
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
