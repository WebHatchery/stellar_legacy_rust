//! The ship's log as History keeps it: every development, newest first. The
//! Bridge shows only the latest change while it is fresh; this is where the
//! older lines remain retrievable.

use super::*;

pub(super) fn draw(ctx: &GameplayCtx<'_>, area: Rect, pointer: Pointer) {
    term_panel(area, Some("Ship's log · newest first"));
    let view = Rect::new(area.x + 20.0, area.y + 48.0, area.w - 40.0, area.h - 62.0);
    let log = &ctx.sim.log;
    let text = if log.is_empty() {
        "Nothing has been logged yet. Developments appear here as the voyage runs.".to_owned()
    } else {
        log.iter()
            .rev()
            .map(|entry| format!("Y{}·M{:02}  {}", entry.year, entry.month, entry.text))
            .collect::<Vec<_>>()
            .join("\n")
    };
    crate::ui::reading::read_lines(&text, view, ctx.chronicle_scroll, pointer, term::dim());
}
