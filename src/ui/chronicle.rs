//! Chronicle: completed contracts across playthroughs, plus the achievement
//! roster (GDD §7, §10).

pub(crate) mod archive;
pub(crate) mod milestones;
use crate::ui::{term, term_button, term_panel, GameplayCtx, UiAction};
use macroquad::prelude::*;
use macroquad_toolkit::prelude::*;
use macroquad_toolkit::ui::{
    draw_ui_text_ex, is_fully_visible, note_neighbour, note_target, occlude, touch_area, RectExt,
    Region,
};
use std::cmp::Reverse;

/// Reserved at the panel's right edge for the scrollbar.
const GUTTER: f32 = 12.0;
const TAB_H: f32 = 44.0;

pub fn draw(ctx: &GameplayCtx<'_>, area: Rect, pointer: Pointer, actions: &mut Vec<UiAction>) {
    let base_pointer = if ctx.obligation_detail.is_some() {
        pointer.suppressed()
    } else {
        pointer
    };
    match ctx.presentation.history_page.get() {
        1 => draw_obligations(ctx, area, base_pointer, actions),
        2 => milestones::draw(ctx, area, base_pointer),
        _ => draw_archive(ctx, area, base_pointer),
    }
    if let Some(obligation_id) = ctx.obligation_detail {
        draw_obligation_history(ctx, area, obligation_id, pointer, actions);
    }
}

fn draw_archive(ctx: &GameplayCtx<'_>, area: Rect, pointer: Pointer) {
    let records = ctx.chronicle_records_tab.get();
    let gap = 8.0;
    let tab_w = (area.w - gap) * 0.5;
    for (index, label) in ["VOYAGE RECORD", "MISSION ARCHIVE"].iter().enumerate() {
        let rect = Rect::new(area.x + index as f32 * (tab_w + gap), area.y, tab_w, TAB_H);
        let hit = touch_area(rect);
        let active = (index == 0) == records;
        note_neighbour(rect);
        note_target(label, rect);
        draw_surface(
            rect,
            &SurfaceStyle::new(if active || pointer.pressing(hit) {
                term::surface_active()
            } else if pointer.hovering_over(hit) {
                term::surface_hover()
            } else {
                term::surface()
            })
            .with_border(
                1.0,
                if active {
                    term::accent()
                } else {
                    term::faint()
                },
            ),
        );
        draw_text_centered_in_box_ex(
            label,
            rect.x,
            rect.y,
            rect.w,
            rect.h,
            TextStyle::new(13.0, if active { term::accent() } else { term::dim() }),
        );
        if pointer.released_on(hit) && !active {
            ctx.chronicle_records_tab.set(index == 0);
            ctx.chronicle_scroll.set(ScrollArea::new());
        }
    }

    let body = Rect::new(area.x, area.y + TAB_H + 8.0, area.w, area.h - TAB_H - 8.0);
    if records {
        draw_decision_records(ctx, body, pointer);
    } else {
        draw_mission_archive(ctx, body, pointer);
    }
}

mod timeline;
use timeline::draw_decision_records;

fn draw_obligations(
    ctx: &GameplayCtx<'_>,
    area: Rect,
    pointer: Pointer,
    actions: &mut Vec<UiAction>,
) {
    term_panel(area, Some("OBLIGATIONS LEDGER"));
    let content = area.inset(18.0);
    let show_resolved = ctx.obligation_resolved_tab.get();
    let active_count = ctx.sim.active_obligations().count();
    let resolved_count = ctx.sim.obligations.len() - active_count;
    draw_obligation_tabs(
        ctx,
        content,
        show_resolved,
        active_count,
        resolved_count,
        pointer,
    );
    let mut obligations: Vec<_> = ctx
        .sim
        .obligations
        .iter()
        .filter(|obligation| obligation.status.is_active() != show_resolved)
        .collect();
    if obligations.is_empty() {
        draw_empty_obligations(content, show_resolved);
        return;
    }
    let overdue_ids: Vec<String> = ctx
        .sim
        .due_obligations()
        .into_iter()
        .map(|obligation| obligation.id.clone())
        .collect();
    sort_obligations(&mut obligations, show_resolved, &overdue_ids);
    draw_obligation_summary(
        &obligations,
        show_resolved,
        ctx.sim.year(),
        &overdue_ids,
        content,
    );
    draw_obligation_rows(
        ctx,
        content,
        show_resolved,
        obligations,
        overdue_ids,
        pointer,
        actions,
    );
}

fn draw_obligation_tabs(
    ctx: &GameplayCtx<'_>,
    content: Rect,
    show_resolved: bool,
    active_count: usize,
    resolved_count: usize,
    pointer: Pointer,
) {
    let tab_gap = 8.0;
    let tab_w = (content.w - tab_gap) * 0.5;
    for (index, (label, count)) in [("ACTIVE", active_count), ("RESOLVED", resolved_count)]
        .into_iter()
        .enumerate()
    {
        let selected = (index == 1) == show_resolved;
        let tab = Rect::new(
            content.x + index as f32 * (tab_w + tab_gap),
            content.y + 18.0,
            tab_w,
            44.0,
        );
        if term_button(
            tab,
            &if selected {
                format!("[ {label} {count} ]")
            } else {
                format!("{label} {count}")
            },
            true,
            pointer,
        ) && !selected
        {
            ctx.obligation_resolved_tab.set(index == 1);
            ctx.obligations_scroll
                .set(macroquad_toolkit::ui::ScrollArea::new());
        }
    }
}

fn draw_empty_obligations(content: Rect, show_resolved: bool) {
    draw_text_block(
        if show_resolved {
            "No resolved duties yet. Fulfilled, defaulted, and voided promises will remain here with their complete histories."
        } else {
            "No active duties. Promises created by councils and charters will remain here until honoured, revised, defaulted, or voided."
        },
        content.x,
        content.y + 82.0,
        content.w,
        90.0,
        13.0,
        4.0,
        term::dim(),
    );
}

fn sort_obligations(
    obligations: &mut Vec<&crate::state::sim::Obligation>,
    show_resolved: bool,
    overdue_ids: &[String],
) {
    let is_overdue = |obligation: &&crate::state::sim::Obligation| {
        overdue_ids.iter().any(|id| id == &obligation.id)
    };
    if show_resolved {
        obligations.sort_by_key(|obligation| {
            Reverse((
                obligation
                    .history
                    .last()
                    .map_or(obligation.created_year, |entry| entry.year),
                obligation.created_year,
                obligation.id.as_str(),
            ))
        });
    } else {
        obligations.sort_by_key(|obligation| {
            (
                !is_overdue(obligation),
                obligation.due_year.unwrap_or(u32::MAX),
                obligation.created_year,
                obligation.id.as_str(),
            )
        });
    }
}

fn draw_obligation_summary(
    obligations: &[&crate::state::sim::Obligation],
    show_resolved: bool,
    year: u32,
    overdue_ids: &[String],
    content: Rect,
) {
    let overdue_count = obligations
        .iter()
        .filter(|obligation| overdue_ids.iter().any(|id| id == &obligation.id))
        .count();
    let (status_summary, summary_warning) = if show_resolved {
        let fulfilled = obligations
            .iter()
            .filter(|obligation| {
                obligation.status == crate::state::sim::ObligationStatus::Fulfilled
            })
            .count();
        let defaulted = obligations
            .iter()
            .filter(|obligation| {
                obligation.status == crate::state::sim::ObligationStatus::Defaulted
            })
            .count();
        (
            format!(
                "{} RESOLVED · {fulfilled} KEPT · {defaulted} DEFAULTED",
                obligations.len()
            ),
            defaulted > 0,
        )
    } else {
        (
            format!(
                "{} ACTIVE · {} DUE NOW · YEAR {year:03}",
                obligations.len(),
                overdue_count
            ),
            overdue_count > 0,
        )
    };
    draw_ui_text_ex(
        &status_summary,
        content.x,
        content.y + 82.0,
        TextStyle::new(
            12.0,
            if summary_warning {
                term::alert()
            } else {
                term::accent()
            },
        )
        .params(),
    );
}

fn draw_obligation_rows(
    ctx: &GameplayCtx<'_>,
    content: Rect,
    show_resolved: bool,
    obligations: Vec<&crate::state::sim::Obligation>,
    overdue_ids: Vec<String>,
    pointer: Pointer,
    actions: &mut Vec<UiAction>,
) {
    let year = ctx.sim.year();
    let is_overdue = |obligation: &crate::state::sim::Obligation| {
        overdue_ids.iter().any(|id| id == &obligation.id)
    };
    const ROW_H: f32 = 194.0;
    const ROW_GAP: f32 = 8.0;
    let view = Rect::new(
        content.x,
        content.y + 96.0,
        content.w,
        content.bottom() - content.y - 96.0,
    );
    let content_h = obligations.len() as f32 * (ROW_H + ROW_GAP) - ROW_GAP;
    let mut scroll = ctx.obligations_scroll.get();
    scroll.update_at(view, content_h, pointer.position);
    let mut y = view.y - scroll.offset();
    for obligation in obligations {
        let overdue = is_overdue(obligation);
        let row = Rect::new(view.x, y, view.w - GUTTER, ROW_H);
        y += ROW_H + ROW_GAP;
        if !is_fully_visible(row, view) {
            continue;
        }
        draw_obligation_row(
            obligation,
            row,
            overdue,
            show_resolved,
            year,
            pointer,
            actions,
        );
    }
    scroll.draw_scrollbar_with(
        view,
        content_h,
        term::surface_inset(),
        term::dim(),
        term::primary(),
    );
    ctx.obligations_scroll.set(scroll);
}

fn draw_obligation_row(
    obligation: &crate::state::sim::Obligation,
    row: Rect,
    overdue: bool,
    show_resolved: bool,
    year: u32,
    pointer: Pointer,
    actions: &mut Vec<UiAction>,
) {
    draw_rectangle(row.x, row.y, row.w, row.h, term::surface_inset());
    draw_rectangle_lines(
        row.x,
        row.y,
        row.w,
        row.h,
        1.0,
        if overdue {
            term::alert()
        } else {
            term::faint()
        },
    );
    let timing = if show_resolved {
        let closed = obligation
            .history
            .last()
            .map_or(obligation.created_year, |entry| entry.year);
        format!("CLOSED Y{closed:03}")
    } else {
        obligation
            .due_year
            .map(|due_year| {
                if due_year <= year {
                    format!("DUE Y{due_year:03} · {}Y OVERDUE", year - due_year)
                } else {
                    format!("DUE Y{due_year:03} · IN {}Y", due_year - year)
                }
            })
            .unwrap_or_else(|| "DUE OPEN".to_owned())
    };
    let inherited = if obligation.successions_crossed > 0 {
        " · INHERITED"
    } else {
        ""
    };
    draw_text_block(
        &format!(
            "{} [{}{}]",
            obligation.title,
            if overdue {
                "DUE"
            } else {
                obligation.status.label()
            },
            inherited
        ),
        row.x + 10.0,
        row.y + 7.0,
        row.w - 132.0,
        38.0,
        18.0,
        4.0,
        if overdue {
            term::alert()
        } else {
            term::primary()
        },
    );
    if term_button(
        Rect::new(row.right() - 112.0, row.y + 4.0, 104.0, 44.0),
        "HISTORY",
        true,
        pointer,
    ) {
        actions.push(UiAction::OpenObligationHistory(obligation.id.clone()));
    }
    let mut line_y = row.y + 58.0;
    for line in [
        format!("TO: {}", obligation.beneficiary),
        format!("OWNER: {} · {timing}", obligation.responsible),
        format!("MATERIAL: {}", obligation.stakes.material),
        format!(
            "NAME: {} · {}",
            obligation.stakes.reputation,
            obligation.visibility.label()
        ),
    ] {
        draw_text_block(
            &line,
            row.x + 10.0,
            line_y,
            row.w - 20.0,
            28.0,
            16.0,
            4.0,
            term::dim(),
        );
        line_y += 30.0;
    }
}

fn draw_obligation_history(
    ctx: &GameplayCtx<'_>,
    area: Rect,
    obligation_id: &str,
    pointer: Pointer,
    actions: &mut Vec<UiAction>,
) {
    let Some(obligation) = ctx
        .sim
        .obligations
        .iter()
        .find(|obligation| obligation.id == obligation_id)
    else {
        actions.push(UiAction::CloseObligationHistory);
        return;
    };

    draw_rectangle(
        area.x,
        area.y,
        area.w,
        area.h,
        Color::new(0.0, 0.0, 0.0, 0.84),
    );
    occlude(area);
    let modal = Rect::new(area.x + 190.0, area.y + 20.0, area.w - 380.0, area.h - 40.0);
    draw_surface(
        modal,
        &SurfaceStyle::new(term::panel())
            .with_border(2.0, term::accent())
            .with_header(50.0, term::panel_header())
            .with_header_divider(1.0, term::accent()),
    );
    let _modal_region = Region::on(modal, term::panel());
    draw_text_centered_in_box_ex(
        "OBLIGATION HISTORY",
        modal.x,
        modal.y,
        modal.w,
        50.0,
        TextStyle::new(15.0, term::accent()),
    );
    if term_button(
        Rect::new(modal.right() - 110.0, modal.y + 3.0, 102.0, 44.0),
        "CLOSE",
        true,
        pointer,
    ) {
        actions.push(UiAction::CloseObligationHistory);
    }

    draw_history_content(ctx, obligation, modal, pointer);
}

fn draw_history_content(
    ctx: &GameplayCtx<'_>,
    obligation: &crate::state::sim::Obligation,
    modal: Rect,
    pointer: Pointer,
) {
    let content = modal.inset(22.0);
    draw_ui_text_ex(
        &obligation.title,
        content.x,
        content.y + 48.0,
        TextStyle::new(18.0, term::primary()).params(),
    );
    let due = obligation
        .due_year
        .map(|year| format!("Y{year:03}"))
        .unwrap_or_else(|| "OPEN".to_owned());
    draw_ui_text_ex(
        &format!(
            "{} · CREATED Y{:03} · DUE {due} · {} · {} SUCCESSIONS",
            obligation.status.label().to_uppercase(),
            obligation.created_year,
            obligation.visibility.label().to_uppercase(),
            obligation.successions_crossed
        ),
        content.x,
        content.y + 72.0,
        TextStyle::new(12.0, term::accent()).params(),
    );
    draw_text_block(
        &format!(
            "TO: {}\nMATERIAL: {}\nNAME: {}",
            obligation.beneficiary, obligation.stakes.material, obligation.stakes.reputation
        ),
        content.x,
        content.y + 78.0,
        content.w,
        70.0,
        12.0,
        3.0,
        term::dim(),
    );

    let view = Rect::new(
        content.x,
        content.y + 158.0,
        content.w,
        content.bottom() - content.y - 158.0,
    );
    const HISTORY_H: f32 = 76.0;
    const HISTORY_GAP: f32 = 8.0;
    let content_h = obligation.history.len() as f32 * (HISTORY_H + HISTORY_GAP) - HISTORY_GAP;
    let mut scroll = ctx.obligation_history_scroll.get();
    scroll.update_at(view, content_h, pointer.position);
    let mut y = view.y - scroll.offset();
    for entry in obligation.history.iter().rev() {
        let row = Rect::new(view.x, y, view.w - GUTTER, HISTORY_H);
        y += HISTORY_H + HISTORY_GAP;
        if !is_fully_visible(row, view) {
            continue;
        }
        draw_rectangle(row.x, row.y, row.w, row.h, term::surface_inset());
        draw_rectangle_lines(row.x, row.y, row.w, row.h, 1.0, term::faint());
        draw_ui_text_ex(
            &format!(
                "Y{:03} · {} · {}",
                entry.year,
                entry.status.label().to_uppercase(),
                entry.captain
            ),
            row.x + 12.0,
            row.y + 20.0,
            TextStyle::new(13.0, term::primary()).params(),
        );
        draw_text_block(
            &entry.note,
            row.x + 12.0,
            row.y + 25.0,
            row.w - 24.0,
            42.0,
            12.0,
            3.0,
            term::dim(),
        );
    }
    scroll.draw_scrollbar_with(
        view,
        content_h,
        term::surface_inset(),
        term::dim(),
        term::primary(),
    );
    ctx.obligation_history_scroll.set(scroll);
}

use archive::draw as draw_mission_archive;
