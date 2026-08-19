//! Chronicle: completed contracts across playthroughs, plus the achievement
//! roster (GDD §7, §10).

use crate::ui::{term, term_panel, GameplayCtx, UiAction};
use macroquad::prelude::*;
use macroquad_toolkit::prelude::*;
use macroquad_toolkit::ui::{
    draw_ui_text_ex, is_fully_visible, note_neighbour, note_target, touch_area, RectExt,
};

/// Vertical stride of one Chronicle entry, and the height of the entry itself.
const ENTRY_STRIDE: f32 = 46.0;
const ENTRY_H: f32 = 40.0;
/// Reserved at the panel's right edge for the scrollbar.
const GUTTER: f32 = 12.0;
const TAB_H: f32 = 44.0;

pub fn draw(ctx: &GameplayCtx<'_>, area: Rect, pointer: Pointer, _actions: &mut Vec<UiAction>) {
    let left = Rect::new(area.x, area.y, area.w * 0.42, area.h);
    let ledger = Rect::new(left.right() + 12.0, area.y, area.w * 0.34, area.h);
    let right = Rect::new(
        ledger.right() + 12.0,
        area.y,
        area.right() - ledger.right() - 12.0,
        area.h,
    );
    draw_archive(ctx, left, pointer);
    draw_obligations(ctx, ledger, pointer);
    draw_milestones(ctx, right);
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

fn record_height(record: &crate::state::sim::DecisionRecord) -> f32 {
    154.0 + record.affected_accounts.len() as f32 * 42.0
}

fn draw_decision_records(ctx: &GameplayCtx<'_>, area: Rect, pointer: Pointer) {
    term_panel(area, Some("FACT & INTERPRETATION"));
    let content = area.inset(20.0);
    let view = Rect::new(content.x, content.y + 32.0, content.w, content.h - 32.0);
    if ctx.sim.decision_records.is_empty() {
        draw_text_block(
            "No interpreted deeds yet. Consequential council choices will enter one authoritative fact here, followed by the command log, the captain's house, and affected peoples remembering it in their own words.",
            view.x,
            view.y + 10.0,
            view.w,
            120.0,
            14.0,
            4.0,
            term::dim(),
        );
        return;
    }

    let content_h: f32 = ctx
        .sim
        .decision_records
        .iter()
        .map(|record| record_height(record) + 8.0)
        .sum();
    let mut scroll = ctx.chronicle_scroll.get();
    scroll.update_at(view, content_h, pointer.position);
    let mut top = view.y - scroll.offset();

    for record in ctx.sim.decision_records.iter().rev() {
        let height = record_height(record);
        let row = Rect::new(view.x, top, view.w - GUTTER, height);
        top += height + 8.0;
        if !is_fully_visible(row, view) {
            continue;
        }
        draw_rectangle(row.x, row.y, row.w, row.h, term::surface_inset());
        draw_rectangle_lines(row.x, row.y, row.w, row.h, 1.0, term::faint());
        draw_ui_text_ex(
            &format!(
                "Y{:03}.{:02}  {} — {}",
                record.year, record.month, record.event_title, record.outcome_label
            ),
            row.x + 10.0,
            row.y + 18.0,
            TextStyle::new(14.0, term::primary()).params(),
        );
        draw_text_block(
            &format!("FACT: {}", record.fact),
            row.x + 10.0,
            row.y + 26.0,
            row.w - 20.0,
            42.0,
            11.0,
            2.0,
            term::accent(),
        );
        draw_text_block(
            &format!("COMMAND LOG: {}", record.official_account),
            row.x + 10.0,
            row.y + 70.0,
            row.w - 20.0,
            34.0,
            11.0,
            2.0,
            term::dim(),
        );
        draw_text_block(
            &format!("{}'S HOUSE: {}", record.captain, record.dynasty_account),
            row.x + 10.0,
            row.y + 108.0,
            row.w - 20.0,
            34.0,
            11.0,
            2.0,
            term::dim(),
        );
        let mut account_y = row.y + 146.0;
        for account in &record.affected_accounts {
            draw_text_block(
                &format!("{}: {}", account.people.to_uppercase(), account.account),
                row.x + 10.0,
                account_y,
                row.w - 20.0,
                36.0,
                11.0,
                2.0,
                term::faint(),
            );
            account_y += 42.0;
        }
    }

    scroll.draw_scrollbar_with(
        view,
        content_h,
        term::surface_inset(),
        term::dim(),
        term::primary(),
    );
    ctx.chronicle_scroll.set(scroll);
}

fn draw_obligations(ctx: &GameplayCtx<'_>, area: Rect, pointer: Pointer) {
    term_panel(area, Some("OBLIGATIONS LEDGER"));
    let content = area.inset(18.0);
    let mut obligations: Vec<_> = ctx.sim.active_obligations().collect();
    if obligations.is_empty() {
        draw_text_block(
            "No active duties. Promises created by councils and charters will remain here until honoured, revised, defaulted, or voided.",
            content.x, content.y + 38.0, content.w, 90.0, 13.0, 4.0, term::dim(),
        );
        return;
    }

    let year = ctx.sim.year();
    let overdue_ids: Vec<_> = ctx
        .sim
        .due_obligations()
        .into_iter()
        .map(|obligation| obligation.id.as_str())
        .collect();
    let is_overdue =
        |obligation: &crate::state::sim::Obligation| overdue_ids.contains(&obligation.id.as_str());
    obligations.sort_by_key(|obligation| {
        (
            !is_overdue(obligation),
            obligation.due_year.unwrap_or(u32::MAX),
            obligation.created_year,
            obligation.id.as_str(),
        )
    });
    let overdue_count = obligations
        .iter()
        .filter(|obligation| is_overdue(obligation))
        .count();
    draw_ui_text_ex(
        &format!(
            "{} ACTIVE · {} DUE NOW · YEAR {year:03}",
            obligations.len(),
            overdue_count
        ),
        content.x,
        content.y + 38.0,
        TextStyle::new(
            12.0,
            if overdue_count > 0 {
                term::alert()
            } else {
                term::accent()
            },
        )
        .params(),
    );

    const ROW_H: f32 = 126.0;
    const ROW_GAP: f32 = 8.0;
    let view = Rect::new(
        content.x,
        content.y + 52.0,
        content.w,
        content.bottom() - content.y - 52.0,
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
        let due = obligation
            .due_year
            .map(|due_year| {
                if due_year <= year {
                    format!("Y{due_year:03} · {}Y OVERDUE", year - due_year)
                } else {
                    format!("Y{due_year:03} · IN {}Y", due_year - year)
                }
            })
            .unwrap_or_else(|| "OPEN".to_owned());
        let inherited = if obligation.successions_crossed > 0 {
            " · INHERITED"
        } else {
            ""
        };
        draw_ui_text_ex(
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
            row.y + 18.0,
            TextStyle::new(
                14.0,
                if overdue {
                    term::alert()
                } else {
                    term::primary()
                },
            )
            .params(),
        );
        let mut line_y = row.y + 36.0;
        for line in [
            format!("TO: {}", obligation.beneficiary),
            format!("OWNER: {} · DUE {due}", obligation.responsible),
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
                11.0,
                2.0,
                term::dim(),
            );
            line_y += 22.0;
        }
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

fn draw_mission_archive(ctx: &GameplayCtx<'_>, area: Rect, pointer: Pointer) {
    term_panel(area, Some("COMPLETED CHARTERS"));
    let content = area.inset(24.0);
    let y = content.y + 46.0;

    if ctx.chronicle.entries.is_empty() {
        draw_text_block(
            "No voyages recorded yet.\n\nEvery completed contract is written here, and the Chronicle outlives any single save. Renown automatically grants a stronger Heritage tier to new dynasties.",
            content.x,
            y,
            content.w,
            120.0,
            15.0,
            5.0,
            term::dim(),
        );
        return;
    }

    // The Chronicle outlives any single save, so it only ever grows — and it
    // used to show its newest nine and drop the rest without saying so, which
    // for a record whose whole purpose is to be the long memory is the one
    // thing it must not do. It scrolls now, oldest still reachable.
    let view = Rect::new(
        content.x,
        y - 14.0,
        content.w,
        content.bottom() - (y - 14.0),
    );
    let content_h = ctx.chronicle.entries.len() as f32 * ENTRY_STRIDE;
    let mut scroll = ctx.chronicle_scroll.get();
    scroll.update_at(view, content_h, pointer.position);

    let mut row_top = view.y - scroll.offset();
    for entry in ctx.chronicle.entries.iter().rev() {
        let row = Rect::new(view.x, row_top, view.w - GUTTER, ENTRY_H);
        row_top += ENTRY_STRIDE;
        // macroquad has no scissor rect, so cull the partly-scrolled entries
        // rather than letting them spill past the panel.
        if !is_fully_visible(row, view) {
            continue;
        }
        draw_ui_text_ex(
            &format!(
                "Y{:03} — {} [{}]",
                entry.completed_year, entry.contract_name, entry.outcome
            ),
            row.x,
            row.y + 14.0,
            TextStyle::new(16.0, term::primary()).params(),
        );
        draw_ui_text_ex(
            &format!(
                "   {} charter · {} yr · gen {} · under {} · score {:.2}",
                entry.objective,
                entry.duration_years,
                entry.generation,
                entry.leader_name,
                entry.score
            ),
            row.x,
            row.y + 32.0,
            TextStyle::new(13.0, term::dim()).params(),
        );
    }

    scroll.draw_scrollbar_with(
        view,
        content_h,
        term::surface_inset(),
        term::dim(),
        term::primary(),
    );
    ctx.chronicle_scroll.set(scroll);
}

fn draw_milestones(ctx: &GameplayCtx<'_>, area: Rect) {
    let (unlocked, total) = ctx.achievements.progress();
    term_panel(area, Some("MILESTONES"));
    let content = area.inset(20.0);
    let mut y = content.y + 42.0;

    draw_ui_text_ex(
        &format!("UNLOCKED {unlocked} / {total}"),
        content.x,
        y,
        TextStyle::new(14.0, term::accent()).params(),
    );
    y += 28.0;

    for achievement in ctx.achievements.iter() {
        let (mark, name_color) = if achievement.unlocked {
            ("[x]", term::accent())
        } else {
            ("[ ]", term::dim())
        };
        draw_ui_text_ex(
            &format!("{mark} {}", achievement.name),
            content.x,
            y,
            TextStyle::new(15.0, name_color).params(),
        );
        draw_text_block(
            &achievement.description,
            content.x + 22.0,
            y + 6.0,
            content.w - 22.0,
            30.0,
            12.0,
            2.0,
            term::faint(),
        );
        y += 46.0;
    }
}
