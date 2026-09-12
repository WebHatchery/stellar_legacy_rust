//! Session-local presentation choices. These never enter the simulation save.
use macroquad_toolkit::ui::ScrollArea;
use std::cell::Cell;

#[derive(Default)]
pub struct Presentation {
    pub project_cancellation: Cell<Option<u64>>,
    pub navigation_open: Cell<bool>,
    pub navigation_scroll: Cell<ScrollArea>,
    pub navigation_key: std::cell::RefCell<String>,
    pub bridge_subject_scroll: Cell<ScrollArea>,
    pub bridge_subject_key: std::cell::RefCell<String>,
    pub bridge_details_scroll: Cell<ScrollArea>,
    pub bridge_details_key: std::cell::RefCell<String>,
    pub capture_mobile_offset: Cell<Option<(bool, f32)>>,
    pub overlay_active: Cell<bool>,
    pub overlay_key: std::cell::RefCell<String>,
    pub overlay_scroll: Cell<ScrollArea>,
    pub mobile_section: std::cell::RefCell<String>,
    pub mobile_key: std::cell::RefCell<String>,
    pub mobile_scroll: Cell<ScrollArea>,
    pub history_page: Cell<usize>,
    pub selected_record: Cell<Option<usize>>,
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

impl Presentation {
    /// Decision records are append-only; store their original index, not their
    /// position in the newest-first list, so incoming deeds cannot move a reader.
    pub fn record_index(&self, count: usize) -> Option<usize> {
        let latest = count.checked_sub(1)?;
        let selected = self.selected_record.get().unwrap_or(latest).min(latest);
        self.selected_record.set(Some(selected));
        Some(selected)
    }
}
#[allow(dead_code, unused_imports)]
mod tests {
    include!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/unit/ui/presentation/tests.rs"
    ));
}
