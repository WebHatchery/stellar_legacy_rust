//! Peoples aboard as instruments: a population-share bar, one approval gauge
//! per people with the restless and devoted lines marked, and a ties diagram
//! with rivals and allies drawn as coloured edges. Replaces the old two-line
//! sentence per people (docs/UI_TEXT_REVIEW.md, item 3).

use crate::state::sim::factions::{approval_band_label, mood_band_for, FactionState};
use crate::ui::{chart, term, GameplayCtx};
use macroquad::prelude::*;
use macroquad_toolkit::prelude::*;
use macroquad_toolkit::ui::draw_ui_text_ex;

/// The approval lines where a people turns restless or devoted.
const RESTLESS_LINE: f32 = 0.3;
const DEVOTED_LINE: f32 = 0.7;
const CARD_H: f32 = 116.0;

fn mood_color(approval: f32) -> Color {
    match mood_band_for(approval) {
        -1 => term::alert(),
        1 => term::accent(),
        _ => term::primary(),
    }
}

fn full_name(ctx: &GameplayCtx<'_>, id: &str) -> String {
    ctx.data
        .factions
        .get(id)
        .map(|definition| definition.name.clone())
        .unwrap_or_else(|| id.to_owned())
}

fn short_name(ctx: &GameplayCtx<'_>, id: &str) -> String {
    let name = full_name(ctx, id);
    name.strip_prefix("The ")
        .or_else(|| name.strip_prefix("Keepers of the "))
        .unwrap_or(&name)
        .to_uppercase()
}

/// Draws the aboard peoples and returns the y below the cards, where the
/// caller lists any peoples the ship has lost.
pub(super) fn draw_aboard(ctx: &GameplayCtx<'_>, content: Rect) -> f32 {
    let aboard: Vec<&FactionState> = ctx.sim.factions.iter().filter(|f| f.is_aboard()).collect();
    let bar = Rect::new(content.x, content.y + 34.0, content.w, 40.0);
    draw_share_bar(ctx, &aboard, bar);

    let top = bar.bottom() + 28.0;
    let cards_w = content.w * 0.56;
    let mut y = top;
    for faction in &aboard {
        draw_card(
            ctx,
            faction,
            Rect::new(content.x, y, cards_w, CARD_H - 12.0),
        );
        y += CARD_H;
    }
    let ties = Rect::new(
        content.x + cards_w + 32.0,
        top,
        content.right() - (content.x + cards_w + 32.0),
        content.bottom() - top,
    );
    draw_ties(ctx, &aboard, ties);
    y + 8.0
}

/// One bar split by head count, each span in its people's mood colour and
/// named where it is wide enough to hold the name.
fn draw_share_bar(ctx: &GameplayCtx<'_>, aboard: &[&FactionState], bar: Rect) {
    let total: u32 = aboard.iter().map(|f| f.members).sum();
    if total == 0 {
        return;
    }
    let mut x = bar.x;
    for faction in aboard {
        let w = bar.w * faction.members as f32 / total as f32;
        let span = Rect::new(x + 1.0, bar.y, (w - 2.0).max(1.0), bar.h);
        draw_rectangle(span.x, span.y, span.w, span.h, mood_color(faction.approval));
        let label = format!(
            "{} {}",
            short_name(ctx, &faction.faction_id),
            faction.members
        );
        let style = TextStyle::new(16.0, term::bg());
        if measure_text_size(&label, style).width + 20.0 < span.w {
            draw_ui_text_ex(&label, span.x + 10.0, span.y + 26.0, style.params());
        }
        x += w;
    }
}

fn draw_card(ctx: &GameplayCtx<'_>, faction: &FactionState, card: Rect) {
    let tone = mood_color(faction.approval);
    crate::ui::identity::emblem(
        Rect::new(card.x, card.y + 8.0, 64.0, 64.0),
        &faction.faction_id,
    );
    let x = card.x + 86.0;
    draw_ui_text_ex(
        &full_name(ctx, &faction.faction_id),
        x,
        card.y + 22.0,
        TextStyle::new(18.0, tone).params(),
    );
    let mood = format!(
        "{} {:.0}%",
        approval_band_label(faction.approval),
        faction.approval * 100.0
    );
    let mood_w = measure_text_size(&mood, TextStyle::new(16.0, tone)).width;
    draw_ui_text_ex(
        &mood,
        card.right() - mood_w,
        card.y + 22.0,
        TextStyle::new(16.0, tone).params(),
    );
    let gauge = Rect::new(x, card.y + 38.0, card.right() - x, 20.0);
    chart::gauge(gauge, faction.approval, None, tone);
    // The band lines a people crosses into restlessness or devotion.
    for line in [RESTLESS_LINE, DEVOTED_LINE] {
        let lx = gauge.x + 1.0 + (gauge.w - 2.0) * line;
        draw_rectangle(lx - 1.0, gauge.y - 3.0, 2.0, gauge.h + 6.0, term::dim());
    }
    if let Some(tended) = ctx
        .data
        .factions
        .get(&faction.faction_id)
        .and_then(|definition| ctx.data.subsystems.get(&definition.tended_subsystem))
    {
        draw_ui_text_ex(
            &format!("CARES FOR {}", tended.name.to_uppercase()),
            x,
            card.y + 84.0,
            TextStyle::new(16.0, term::dim()).params(),
        );
    }
}

/// Whether two peoples are rivals (-1), allies (+1) or neither (0). Either
/// side naming the other is enough.
fn tie(ctx: &GameplayCtx<'_>, a: &str, b: &str) -> i8 {
    let names = |from: &str, to: &str, rivals: bool| {
        ctx.data.factions.get(from).is_some_and(|definition| {
            let list = if rivals {
                &definition.rivals
            } else {
                &definition.allies
            };
            list.iter().any(|id| id == to)
        })
    };
    if names(a, b, true) || names(b, a, true) {
        -1
    } else if names(a, b, false) || names(b, a, false) {
        1
    } else {
        0
    }
}

/// Peoples on a ring, joined by red rival and green ally edges.
fn draw_ties(ctx: &GameplayCtx<'_>, aboard: &[&FactionState], area: Rect) {
    draw_ui_text_ex(
        "TIES",
        area.x,
        area.y + 18.0,
        TextStyle::new(16.0, term::primary()).params(),
    );
    let legend_x = area.right() - 220.0;
    draw_rectangle(legend_x, area.y + 10.0, 24.0, 3.0, term::alert());
    draw_ui_text_ex(
        "RIVAL",
        legend_x + 32.0,
        area.y + 18.0,
        TextStyle::new(16.0, term::dim()).params(),
    );
    draw_rectangle(legend_x + 116.0, area.y + 10.0, 24.0, 3.0, term::accent());
    draw_ui_text_ex(
        "ALLY",
        legend_x + 148.0,
        area.y + 18.0,
        TextStyle::new(16.0, term::dim()).params(),
    );
    if aboard.is_empty() {
        return;
    }
    let center = vec2(
        area.x + area.w * 0.5,
        area.y + 40.0 + (area.h - 40.0) * 0.46,
    );
    let radius = (area.w.min(area.h - 40.0) * 0.32).max(60.0);
    let nodes: Vec<Vec2> = (0..aboard.len())
        .map(|index| {
            let angle = -std::f32::consts::FRAC_PI_2
                + std::f32::consts::TAU * index as f32 / aboard.len() as f32;
            center + vec2(angle.cos(), angle.sin()) * radius
        })
        .collect();
    for i in 0..aboard.len() {
        for j in (i + 1)..aboard.len() {
            let (color, thickness) = match tie(ctx, &aboard[i].faction_id, &aboard[j].faction_id) {
                -1 => (term::alert(), 3.0),
                1 => (term::accent(), 3.0),
                _ => (term::faint(), 1.0),
            };
            draw_line(
                nodes[i].x, nodes[i].y, nodes[j].x, nodes[j].y, thickness, color,
            );
        }
    }
    for (faction, node) in aboard.iter().zip(&nodes) {
        draw_circle(node.x, node.y, 40.0, term::bg());
        draw_circle_lines(node.x, node.y, 40.0, 3.0, mood_color(faction.approval));
        crate::ui::identity::emblem(
            Rect::new(node.x - 28.0, node.y - 28.0, 56.0, 56.0),
            &faction.faction_id,
        );
        let label = short_name(ctx, &faction.faction_id);
        let style = TextStyle::new(16.0, term::dim());
        let w = measure_text_size(&label, style).width;
        // A plate under the name keeps edges from striking through it.
        draw_rectangle(
            node.x - w * 0.5 - 8.0,
            node.y + 46.0,
            w + 16.0,
            26.0,
            term::bg(),
        );
        draw_ui_text_ex(&label, node.x - w * 0.5, node.y + 64.0, style.params());
    }
}
