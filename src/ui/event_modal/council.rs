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
    macroquad_toolkit::ui::occlude(Rect::new(
        0.0,
        72.0,
        logical_width(),
        logical_height() - 72.0,
    ));
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
    draw_rectangle(
        0.0,
        72.0,
        logical_width(),
        logical_height() - 72.0,
        Color::new(0.0, 0.0, 0.0, 0.94),
    );
    crate::ui::term_panel(frame, Some("Council decision"));
    let captain = ctx
        .sim
        .dynasty
        .leader()
        .map_or("Captain", |p| p.name.as_str());
    draw_text_right(
        &format!(
            "{captain} · fallback in {}s",
            countdown_secs(ctx.decision_remaining)
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
    let gap = 10.0;
    let width = (right.w - gap * (available.len() - 1) as f32) / available.len() as f32;
    for (index, &outcome_index) in available.iter().enumerate() {
        let option = &event.outcomes[outcome_index];
        let rect = Rect::new(right.x + index as f32 * (width + gap), right.y, width, 68.0);
        if term_button(rect, &option.label, true, pointer) {
            state.event_choice.set(index);
            state.event_scroll.set(ScrollArea::new());
        }
        if selected == index {
            draw_rectangle(rect.x, rect.bottom() + 3.0, rect.w, 3.0, term::primary());
        }
    }
    let index = available[selected];
    let outcome = &event.outcomes[index];
    let population = crate::simulation::event_resolver::outcome_pop_impact_range(
        ctx.sim, ctx.data, event, index,
    );
    let fuel =
        crate::simulation::event_resolver::outcome_fuel_preview(ctx.sim, ctx.data, event, index);
    let (effects, _) = known_effects(outcome, population, fuel);
    let affordable = crate::simulation::event_resolver::outcome_affordable(ctx.sim, outcome);
    let briefing = format!(
        "{}\n\n{}{}",
        outcome.description,
        effects,
        if affordable {
            ""
        } else {
            "\n\nUnavailable: insufficient stores for this choice."
        }
    );
    let detail = Rect::new(right.x, right.y + 92.0, right.w, right.h - 158.0);
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
