//! Select a dated deed, then read every account without truncation.
use super::*;

pub(super) fn draw_decision_records(ctx: &GameplayCtx<'_>, area: Rect, pointer: Pointer) {
    term_panel(
        area,
        Some("Voyage timeline · select a deed to read its accounts"),
    );
    let view = Rect::new(area.x + 18.0, area.y + 48.0, 370.0, area.h - 66.0);
    let records = &ctx.sim.decision_records;
    if records.is_empty() {
        draw_text_block(
            "No consequential decisions recorded yet.",
            view.x,
            view.y,
            area.w - 36.0,
            80.0,
            18.0,
            6.0,
            term::dim(),
        );
        return;
    }
    let selected = ctx
        .presentation
        .selected_record
        .get()
        .min(records.len() - 1);
    let height = records.len() as f32 * 80.0;
    let mut scroll = ctx.chronicle_scroll.get();
    scroll.update_at(view, height, pointer.position);
    let list_pointer = if scroll.absorbs_press() {
        pointer.suppressed()
    } else {
        pointer
    };
    for (index, record) in records.iter().rev().enumerate() {
        let row = Rect::new(
            view.x + 14.0,
            view.y + index as f32 * 80.0 - scroll.offset(),
            view.w - 34.0,
            70.0,
        );
        if !is_fully_visible(row, view) {
            continue;
        }
        draw_circle(view.x + 4.0, row.y + 35.0, 4.0, term::primary());
        if term_button(
            row,
            &format!("Y{}.{} · {}", record.year, record.month, record.event_title),
            true,
            list_pointer,
        ) {
            ctx.presentation.selected_record.set(index);
            ctx.presentation.record_scroll.set(ScrollArea::new());
        }
        if selected == index {
            draw_rectangle(row.x, row.bottom() - 3.0, row.w, 3.0, term::primary());
        }
    }
    scroll.draw_scrollbar_with(
        view,
        height,
        term::surface_inset(),
        term::dim(),
        term::primary(),
    );
    ctx.chronicle_scroll.set(scroll);
    let record = &records[records.len() - 1 - selected];
    let mut text = format!(
        "{}\nDecision: {}\n\nFACT\n{}\n\nCOMMAND LOG\n{}\n\n{}'S HOUSE\n{}",
        record.event_title,
        record.outcome_label,
        record.fact,
        record.official_account,
        record.captain,
        record.dynasty_account
    );
    for account in &record.affected_accounts {
        text.push_str(&format!("\n\n{}\n{}", account.people, account.account));
    }
    crate::ui::reading::read_lines(
        &text,
        Rect::new(
            view.right() + 26.0,
            view.y,
            area.right() - view.right() - 44.0,
            view.h,
        ),
        &ctx.presentation.record_scroll,
        pointer,
        term::dim(),
    );
}
