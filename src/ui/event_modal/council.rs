//! A council decision separates the situation from the selected commitment.
use super::*;
use crate::ui::reading::read_lines;
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
    let left = Rect::new(frame.x + 20.0, frame.y + 52.0, 418.0, frame.h - 72.0);
    let right = Rect::new(
        left.right() + 26.0,
        left.y,
        frame.right() - left.right() - 46.0,
        left.h,
    );
    crate::ui::identity::portrait(Rect::new(left.x, left.y, 54.0, 64.0), captain);
    draw_text_block(
        &event.title,
        left.x + 72.0,
        left.y,
        left.w - 72.0,
        66.0,
        26.0,
        5.0,
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
    let situation = Rect::new(left.x, left.y + 80.0, left.w, left.h - 148.0);
    read_lines(
        &text,
        situation,
        &state.situation_scroll,
        pointer,
        term::dim(),
    );
    if term_button(
        Rect::new(left.x, left.bottom() - 48.0, left.w, 44.0),
        if advice_open {
            "Hide officer advice"
        } else {
            "Read officer advice"
        },
        true,
        pointer,
    ) {
        state.event_advice.set(!advice_open);
        state.situation_scroll.set(ScrollArea::new());
    }
}

/// One option's known terms and its uncertain population band, if any.
type OptionEffects = (Vec<super::effects::EffectTerm>, Option<(i64, i64)>);

/// Width of the comparison grid's row-label column; the option buttons and
/// their effect columns share the rest of the panel.
const GRID_LABEL_W: f32 = 300.0;
const GRID_ROW_H: f32 = 30.0;

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
    let gap = 10.0;
    let cols_x = right.x + GRID_LABEL_W;
    let col_w =
        (right.right() - cols_x - gap * (available.len() - 1) as f32) / available.len() as f32;
    let column = |index: usize| cols_x + index as f32 * (col_w + gap);
    for (index, &outcome_index) in available.iter().enumerate() {
        let option = &event.outcomes[outcome_index];
        let rect = Rect::new(column(index), right.y, col_w, 68.0);
        if term_button(rect, &option.label, true, pointer) {
            state.event_choice.set(index);
            state.event_scroll.set(ScrollArea::new());
        }
        if selected == index {
            draw_rectangle(rect.x, rect.bottom() + 3.0, rect.w, 3.0, term::primary());
        }
    }
    let grid_top = right.y + 92.0;
    let grid_bottom = draw_effect_grid(ctx, event, available, selected, grid_top, &column, col_w);

    let index = available[selected];
    let outcome = &event.outcomes[index];
    let affordable = crate::simulation::event_resolver::outcome_affordable(ctx.sim, outcome);
    let briefing = if affordable {
        outcome.description.clone()
    } else {
        format!(
            "{}\n\nUnavailable: insufficient stores for this choice.",
            outcome.description
        )
    };
    let detail_top = grid_bottom + 20.0;
    let detail = Rect::new(
        right.x,
        detail_top,
        right.w,
        (right.bottom() - 66.0 - detail_top).max(60.0),
    );
    let read_to_end = read_lines(&briefing, detail, &state.event_scroll, pointer, term::dim());
    let label = if !affordable {
        "Commit choice · insufficient stores".to_owned()
    } else if !read_to_end {
        "Scroll through consequences to commit".to_owned()
    } else {
        format!("Commit: {}", outcome.label)
    };
    if term_button(
        Rect::new(right.x, right.bottom() - 48.0, right.w, 48.0),
        &label,
        affordable && read_to_end,
        pointer,
    ) {
        actions.push(UiAction::ResolveEvent(index));
    }
}

/// Every option's known changes side by side: one row per change any option
/// makes, one column per option, the selected column lit. A dash marks an
/// option that leaves that row alone. Returns the grid's bottom edge.
fn draw_effect_grid(
    ctx: &GameplayCtx<'_>,
    event: &crate::data::events::EventTemplate,
    available: &[usize],
    selected: usize,
    top: f32,
    column: &dyn Fn(usize) -> f32,
    col_w: f32,
) -> f32 {
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
    let has_souls = per_option.iter().any(|(_, souls)| souls.is_some());
    let row_count = rows.len() + usize::from(has_souls);
    let height = GRID_ROW_H * row_count.max(1) as f32;
    draw_rectangle(
        column(selected) - 4.0,
        top,
        col_w + 8.0,
        height,
        term::surface_inset(),
    );
    if row_count == 0 {
        draw_ui_text_ex(
            "No option makes an immediate material change.",
            column(0) - GRID_LABEL_W,
            top + 21.0,
            TextStyle::new(16.0, term::dim()).params(),
        );
        return top + height;
    }
    let label_x = column(0) - GRID_LABEL_W;
    for (row, label) in rows.iter().enumerate() {
        let y = top + row as f32 * GRID_ROW_H + 21.0;
        draw_ui_text_ex(
            label,
            label_x,
            y,
            TextStyle::new(16.0, term::dim()).params(),
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
                TextStyle::new(16.0, color).params(),
            );
        }
    }
    if has_souls {
        let y = top + rows.len() as f32 * GRID_ROW_H + 21.0;
        draw_ui_text_ex(
            "souls (uncertain)",
            label_x,
            y,
            TextStyle::new(16.0, term::dim()).params(),
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
                TextStyle::new(16.0, color).params(),
            );
        }
    }
    top + height
}
