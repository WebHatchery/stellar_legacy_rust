//! A council decision separates the situation from the selected commitment.
use super::*;
use crate::ui::reading::{flow_text, read_lines_sized};
use crate::ui::term_button_sized;
use macroquad_toolkit::ui::ScrollArea;

pub fn draw(ctx: &GameplayCtx<'_>, pointer: Pointer, actions: &mut Vec<UiAction>) {
    let Some(pending) = &ctx.sim.pending_event else {
        return;
    };
    let Some(event) = ctx.data.events.get(&pending.template_id) else {
        return;
    };
    let _overlay_region = Region::on(
        Rect::new(0.0, 72.0, logical_width(), logical_height() - 72.0),
        Color::new(0.0, 0.0, 0.0, 0.94),
    );
    let state = ctx.presentation;
    let key = format!("{}:{}", pending.template_id, pending.rolled_month_clock);
    if *state.event_key.borrow() != key {
        *state.event_key.borrow_mut() = key;
        state.event_choice.set(0);
        state.event_scroll.set(ScrollArea::new());
        state.situation_scroll.set(ScrollArea::new());
        state.event_advice.set(false);
    }
    let available = crate::simulation::event_resolver::available_outcome_indices(ctx.sim, event);
    if available.is_empty() {
        return;
    }
    let selected = state.event_choice.get().min(available.len() - 1);
    let frame = Rect::new(
        24.0,
        130.0,
        logical_width() - 48.0,
        logical_height() - 146.0,
    );
    draw_council_panel(ctx, event, &available, selected, frame, pointer, actions);
}

fn draw_council_panel(
    ctx: &GameplayCtx<'_>,
    event: &crate::data::events::EventTemplate,
    available: &[usize],
    selected: usize,
    frame: Rect,
    pointer: Pointer,
    actions: &mut Vec<UiAction>,
) {
    draw_rectangle(
        0.0,
        72.0,
        logical_width(),
        logical_height() - 72.0,
        Color::new(0.0, 0.0, 0.0, 0.94),
    );
    crate::ui::term_panel(frame, Some("Council decision"));
    let _modal_region = Region::on(frame, term::panel());
    let captain = ctx
        .sim
        .dynasty
        .leader()
        .map_or("Captain", |p| p.name.as_str());
    draw_text_right(
        &format!(
            "{captain} · {}",
            crate::ui::time_controls::fallback_label(ctx.sim.speed, ctx.decision_remaining)
        ),
        frame.right() - 18.0,
        frame.y + 25.0,
        TextStyle::new(16.0, term::alert()),
    );
    let left = Rect::new(frame.x + 24.0, frame.y + 56.0, 560.0, frame.h - 76.0);
    let right = Rect::new(
        left.right() + 26.0,
        left.y,
        frame.right() - left.right() - 46.0,
        left.h,
    );
    crate::ui::identity::portrait(Rect::new(left.x, left.y, 68.0, 80.0), captain);
    draw_text_block(
        &event.title,
        left.x + 88.0,
        left.y + 4.0,
        left.w - 88.0,
        80.0,
        34.0,
        6.0,
        term::primary(),
    );
    draw_council_situation(ctx, event, left, pointer);
    draw_council_choices(ctx, event, available, selected, right, pointer, actions);
}

fn draw_council_situation(
    ctx: &GameplayCtx<'_>,
    event: &crate::data::events::EventTemplate,
    left: Rect,
    pointer: Pointer,
) {
    let state = ctx.presentation;
    let advice_open = state.event_advice.get();
    let mut text = crate::simulation::event_resolver::shown_description(ctx.sim, event);
    if advice_open {
        for counsel in crate::simulation::advice::for_event(ctx.sim, ctx.data, event) {
            let name = counsel
                .officer_name
                .as_deref()
                .unwrap_or(&counsel.post_name);
            text.push_str(&format!(
                "\n\n{name} · {}\n{}",
                counsel.post_name, counsel.text
            ));
        }
    }
    let situation = Rect::new(left.x, left.y + 104.0, left.w, left.h - 180.0);
    read_lines_sized(
        &text,
        situation,
        &state.situation_scroll,
        pointer,
        term::dim(),
        22.0,
    );
    if term_button_sized(
        Rect::new(left.x, left.bottom() - COMMIT_H, left.w, COMMIT_H),
        if advice_open {
            "Hide officer advice"
        } else {
            "Read officer advice"
        },
        true,
        pointer,
        20.0,
    ) {
        state.event_advice.set(!advice_open);
        state.situation_scroll.set(ScrollArea::new());
    }
}

/// One option's known terms and its uncertain population band, if any.
type OptionEffects = (Vec<super::effects::EffectTerm>, Option<(i64, i64)>);

/// Width of the comparison grid's row-label column; the option buttons and
/// their effect columns share the rest of the panel.
const GRID_LABEL_W: f32 = 310.0;
const GRID_ROW_H: f32 = 40.0;
const SOULS_LABEL: &str = "souls (uncertain)";
const GRID_TEXT: f32 = 20.0;
const OPTION_H: f32 = 76.0;
const COMMIT_H: f32 = 56.0;

fn draw_council_choices(
    ctx: &GameplayCtx<'_>,
    event: &crate::data::events::EventTemplate,
    available: &[usize],
    selected: usize,
    right: Rect,
    pointer: Pointer,
    actions: &mut Vec<UiAction>,
) {
    let state = ctx.presentation;
    let gap = 14.0;
    let grid = option_effects(ctx, event, available);
    let label_w = grid_label_width(&grid, right.w);
    let cols_x = right.x + label_w;
    let col_w =
        (right.right() - cols_x - gap * (available.len() - 1) as f32) / available.len() as f32;
    let column = |index: usize| cols_x + index as f32 * (col_w + gap);
    let card_bottom = right.bottom() - COMMIT_H - 18.0;
    // The chosen option is lit as a whole card, top to bottom, so the
    // comparison reads down a column rather than across a gap.
    draw_rectangle(
        column(selected) - 6.0,
        right.y - 6.0,
        col_w + 12.0,
        card_bottom - right.y + 12.0,
        term::surface_inset(),
    );
    for (index, &outcome_index) in available.iter().enumerate() {
        let option = &event.outcomes[outcome_index];
        let rect = Rect::new(column(index), right.y, col_w, OPTION_H);
        if term_button_sized(rect, &option.label, true, pointer, 20.0) {
            state.event_choice.set(index);
            state.event_scroll.set(ScrollArea::new());
        }
        if selected == index {
            draw_rectangle(rect.x, rect.bottom() + 4.0, rect.w, 4.0, term::primary());
        }
    }
    let grid_top = right.y + OPTION_H + 24.0;
    let grid_bottom = draw_effect_grid(&grid, label_w, grid_top, &column);

    // Each option's consequence sits under its own column, so every choice
    // is legible at once; only the selected one gates the commit on reading.
    let detail_top = grid_bottom + 28.0;
    let mut read_to_end = false;
    for (index, &outcome_index) in available.iter().enumerate() {
        let outcome = &event.outcomes[outcome_index];
        let affordable = crate::simulation::event_resolver::outcome_affordable(ctx.sim, outcome);
        let briefing = if affordable {
            outcome.description.clone()
        } else {
            format!(
                "{}

Unavailable: insufficient stores for this choice.",
                outcome.description
            )
        };
        let x = column(index) + 8.0;
        if index == selected {
            let view = Rect::new(
                x,
                detail_top,
                col_w - 8.0,
                (card_bottom - detail_top).max(60.0),
            );
            read_to_end = read_lines_sized(
                &briefing,
                view,
                &state.event_scroll,
                pointer,
                term::primary(),
                20.0,
            );
        } else {
            flow_text(&briefing, x, detail_top, col_w - 24.0, 20.0, term::dim());
        }
    }
    draw_ui_text_ex(
        "If chosen",
        right.x,
        detail_top + 21.0,
        TextStyle::new(GRID_TEXT, term::dim()).params(),
    );
    draw_commit(
        ctx,
        event,
        available[selected],
        read_to_end,
        right,
        pointer,
        actions,
    );
}

/// The label column fits its longest row at the current text size, so a
/// value never lands on its label; it never takes more than 40% of `width`.
fn grid_label_width(grid: &(Vec<OptionEffects>, Vec<String>), width: f32) -> f32 {
    grid.1
        .iter()
        .map(String::as_str)
        .chain(
            grid.0
                .iter()
                .any(|(_, souls)| souls.is_some())
                .then_some(SOULS_LABEL),
        )
        .map(|label| measure_text_size(label, TextStyle::new(GRID_TEXT, term::dim())).width + 28.0)
        .fold(GRID_LABEL_W, f32::max)
        .min(width * 0.4)
}

/// The full-width commit bar: live only once the selected option is both
/// affordable and read to its end, and saying which of the two is missing.
fn draw_commit(
    ctx: &GameplayCtx<'_>,
    event: &crate::data::events::EventTemplate,
    index: usize,
    read_to_end: bool,
    right: Rect,
    pointer: Pointer,
    actions: &mut Vec<UiAction>,
) {
    let outcome = &event.outcomes[index];
    let affordable = crate::simulation::event_resolver::outcome_affordable(ctx.sim, outcome);
    let label = if !affordable {
        "Commit choice · insufficient stores".to_owned()
    } else if !read_to_end {
        "Scroll through consequences to commit".to_owned()
    } else {
        format!("Commit: {}", outcome.label)
    };
    if term_button_sized(
        Rect::new(right.x, right.bottom() - COMMIT_H, right.w, COMMIT_H),
        &label,
        affordable && read_to_end,
        pointer,
        20.0,
    ) {
        actions.push(UiAction::ResolveEvent(index));
    }
}

/// Every option's known changes side by side: one row per change any option
/// makes, one column per option, the selected column lit. A dash marks an
/// option that leaves that row alone. Returns the grid's bottom edge.
fn draw_effect_grid(
    grid: &(Vec<OptionEffects>, Vec<String>),
    label_w: f32,
    top: f32,
    column: &dyn Fn(usize) -> f32,
) -> f32 {
    let (per_option, rows) = grid;
    let has_souls = per_option.iter().any(|(_, souls)| souls.is_some());
    let row_count = rows.len() + usize::from(has_souls);
    let height = GRID_ROW_H * row_count.max(1) as f32;
    if row_count == 0 {
        draw_ui_text_ex(
            "No option makes an immediate material change.",
            column(0) - label_w,
            top + 27.0,
            TextStyle::new(GRID_TEXT, term::dim()).params(),
        );
        return top + height;
    }
    let label_x = column(0) - label_w;
    for (row, label) in rows.iter().enumerate() {
        let y = top + row as f32 * GRID_ROW_H + 27.0;
        draw_ui_text_ex(
            label,
            label_x,
            y,
            TextStyle::new(GRID_TEXT, term::dim()).params(),
        );
        for (index, (terms, _)) in per_option.iter().enumerate() {
            let (text, color) = match terms.iter().find(|term| &term.label == label) {
                Some(term) if term.value.is_empty() => ("●".to_owned(), term.color()),
                Some(term) => (term.value.clone(), term.color()),
                None => ("—".to_owned(), term::dim()),
            };
            draw_ui_text_ex(
                &text,
                column(index) + 8.0,
                y,
                TextStyle::new(GRID_TEXT, color).params(),
            );
        }
    }
    if has_souls {
        let y = top + rows.len() as f32 * GRID_ROW_H + 27.0;
        draw_ui_text_ex(
            SOULS_LABEL,
            label_x,
            y,
            TextStyle::new(GRID_TEXT, term::dim()).params(),
        );
        for (index, (_, souls)) in per_option.iter().enumerate() {
            let (text, color) = match souls {
                Some((lo, hi)) => super::effects::impact_label(*lo, *hi),
                None => ("—".to_owned(), term::dim()),
            };
            draw_ui_text_ex(
                &text,
                column(index) + 8.0,
                y,
                TextStyle::new(GRID_TEXT, color).params(),
            );
        }
    }
    top + height
}

/// Each available option's known terms and population band, plus the union
/// of row labels in first-seen order.
fn option_effects(
    ctx: &GameplayCtx<'_>,
    event: &crate::data::events::EventTemplate,
    available: &[usize],
) -> (Vec<OptionEffects>, Vec<String>) {
    let per_option: Vec<OptionEffects> = available
        .iter()
        .map(|&index| {
            let fuel = crate::simulation::event_resolver::outcome_fuel_preview(
                ctx.sim, ctx.data, event, index,
            );
            let souls = crate::simulation::event_resolver::outcome_pop_impact_range(
                ctx.sim, ctx.data, event, index,
            );
            (
                super::effects::effect_terms(&event.outcomes[index], fuel),
                souls,
            )
        })
        .collect();
    let mut rows: Vec<String> = Vec::new();
    for (terms, _) in &per_option {
        for term in terms {
            if !rows.contains(&term.label) {
                rows.push(term.label.clone());
            }
        }
    }
    (per_option, rows)
}
