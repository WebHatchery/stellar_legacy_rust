//! Market: truthful buy/sell quotes at two visible lot sizes (GDD §5.1).

use crate::simulation::market::{buy_quote, sell_quote, TradeQuote};
use crate::simulation::ship::loadout_stats;
use crate::state::sim::TradeResource;
use crate::ui::{term, term_button, term_panel, GameplayCtx, UiAction};
use macroquad::prelude::*;
use macroquad_toolkit::prelude::*;
use macroquad_toolkit::ui::{draw_ui_text_ex, RectExt};

pub(crate) fn held_amount(ctx: &GameplayCtx<'_>, resource: TradeResource) -> i64 {
    match resource {
        TradeResource::Energy => ctx.sim.resources.energy,
        TradeResource::Minerals => ctx.sim.resources.minerals,
        TradeResource::Food => ctx.sim.resources.food,
        TradeResource::Influence => ctx.sim.resources.influence,
    }
}

fn factor_label(name: &str, factor: f32) -> Option<String> {
    let percent = ((factor - 1.0) * 100.0).round() as i32;
    (percent != 0).then(|| format!("{name} {percent:+}%"))
}

/// Unit buy and sell prices for one commodity, with a tag only when the
/// ship's need or distress is bending this good's terms right now. The ship's
/// name bends every good alike, so it is stated once in the header instead.
fn terms_line(buy: TradeQuote, sell: TradeQuote) -> String {
    let mut line = format!(
        "BUY {:.2}/U  ·  SELL {:.2}/U",
        buy.effective_unit_price, sell.effective_unit_price
    );
    for tag in [
        factor_label("NEED", buy.pressure_factor),
        factor_label("DISTRESS", sell.pressure_factor),
    ]
    .into_iter()
    .flatten()
    {
        line.push_str(&format!("  ·  {tag}"));
    }
    line
}

/// The ship's reputation terms, shared by every commodity; empty when the
/// name moves no price.
fn name_terms(buy: TradeQuote, sell: TradeQuote) -> String {
    let terms: Vec<String> = [
        factor_label("BUY", buy.reputation_factor),
        factor_label("SELL", sell.reputation_factor),
    ]
    .into_iter()
    .flatten()
    .collect();
    if terms.is_empty() {
        String::new()
    } else {
        format!("YOUR NAME  {}", terms.join(" · "))
    }
}

pub(crate) fn trade_lot_sizes(cargo: i64) -> (i64, i64) {
    let cargo_lot = cargo.max(50);
    ((cargo_lot / 4).max(10).min(cargo_lot), cargo_lot)
}

const COL_HELD: f32 = 220.0;
const COL_HISTORY: f32 = 380.0;
const HISTORY_W: f32 = 420.0;
const COL_PRICE: f32 = COL_HISTORY + HISTORY_W + 24.0;
const COL_TERMS: f32 = COL_PRICE + 200.0;

pub fn draw(ctx: &GameplayCtx<'_>, area: Rect, pointer: Pointer, actions: &mut Vec<UiAction>) {
    term_panel(area, Some("COMMODITY EXCHANGE"));
    let content = area.inset(24.0);
    let (small_lot, cargo_lot) = trade_lot_sizes(loadout_stats(ctx.sim, ctx.data).cargo as i64);
    let y = draw_market_header(ctx, content, small_lot, cargo_lot);
    let y = draw_market_rows(ctx, content, pointer, actions, small_lot, cargo_lot, y);
    draw_market_footer(ctx, content, y);
}

fn draw_market_header(ctx: &GameplayCtx<'_>, content: Rect, small_lot: i64, cargo_lot: i64) -> f32 {
    let mut y = content.y + 44.0;
    draw_ui_text_ex(
        &format!(
            "TREASURY {}cr   ·   SMALL LOT {}   ·   CARGO LOT {}",
            ctx.sim.resources.credits, small_lot, cargo_lot
        ),
        content.x,
        y,
        TextStyle::new(17.0, term::accent()).params(),
    );
    // Reputation bends every good alike; probe one good for the shared terms.
    let name = name_terms(
        buy_quote(ctx.sim, TradeResource::Energy, small_lot),
        sell_quote(ctx.sim, TradeResource::Energy, small_lot),
    );
    if !name.is_empty() {
        let style = TextStyle::new(16.0, term::dim());
        let w = measure_text_size(&name, style).width;
        draw_ui_text_ex(&name, content.right() - w, y, style.params());
    }
    y += 30.0;
    for (label, offset) in [
        ("COMMODITY", 0.0),
        ("HELD", COL_HELD),
        ("PRICE HISTORY", COL_HISTORY),
        ("MARKET", COL_PRICE),
        ("QUOTED", COL_TERMS),
    ] {
        draw_ui_text_ex(
            label,
            content.x + 14.0 + offset,
            y,
            TextStyle::new(16.0, term::dim()).params(),
        );
    }
    y + 12.0
}

fn draw_market_rows(
    ctx: &GameplayCtx<'_>,
    content: Rect,
    pointer: Pointer,
    actions: &mut Vec<UiAction>,
    small_lot: i64,
    cargo_lot: i64,
    mut y: f32,
) -> f32 {
    const ROW_H: f32 = 118.0;
    const ROW_GAP: f32 = 10.0;
    for entry in &ctx.sim.market.entries {
        draw_market_row(
            ctx,
            entry,
            Rect::new(content.x, y, content.w, ROW_H),
            pointer,
            actions,
            small_lot,
            cargo_lot,
        );
        y += ROW_H + ROW_GAP;
    }
    y
}

/// The price line: earlier settled prices then the current one, scaled to
/// their own range so small moves still read, toned by the latest change.
fn draw_sparkline(entry: &crate::state::sim::MarketEntry, rect: Rect, tone: Color) {
    draw_rectangle(rect.x, rect.y, rect.w, rect.h, term::panel());
    let mut values = entry.history.clone();
    values.push(entry.price);
    if values.len() < 2 {
        // No settled history yet: a flat line at mid-height.
        let mid = rect.y + rect.h * 0.5;
        draw_line(rect.x, mid, rect.right(), mid, 1.5, term::dim());
        return;
    }
    let lo = values.iter().copied().fold(f32::INFINITY, f32::min);
    let hi = values.iter().copied().fold(f32::NEG_INFINITY, f32::max);
    let pad = ((hi - lo) * 0.15).max(entry.price * 0.02);
    let points = crate::ui::chart::line_points(rect, &values, lo - pad, hi + pad);
    crate::ui::chart::polyline(&points, 2.0, tone);
    if let Some(last) = points.last() {
        draw_circle(last.x, last.y, 4.0, tone);
    }
}

fn draw_market_row(
    ctx: &GameplayCtx<'_>,
    entry: &crate::state::sim::MarketEntry,
    row: Rect,
    pointer: Pointer,
    actions: &mut Vec<UiAction>,
    small_lot: i64,
    cargo_lot: i64,
) {
    draw_surface(
        row,
        &SurfaceStyle::new(term::surface_inset()).with_border(1.0, term::faint()),
    );
    let cx = row.x + 14.0;
    let held = held_amount(ctx, entry.resource);
    let (arrow, trend_color) = if entry.trend > 0.005 {
        ("▲", term::accent())
    } else if entry.trend < -0.005 {
        ("▼", term::alert())
    } else {
        ("", term::primary())
    };
    let line_y = row.y + 34.0;
    draw_ui_text_ex(
        entry.resource.label(),
        cx,
        line_y,
        TextStyle::new(18.0, term::primary()).params(),
    );
    draw_ui_text_ex(
        &held.to_string(),
        cx + COL_HELD,
        line_y,
        TextStyle::new(18.0, term::accent()).params(),
    );
    draw_sparkline(
        entry,
        Rect::new(cx + COL_HISTORY, row.y + 10.0, HISTORY_W, 36.0),
        trend_color,
    );
    draw_ui_text_ex(
        &format!("{:.2} {arrow}", entry.price),
        cx + COL_PRICE,
        line_y,
        TextStyle::new(18.0, trend_color).params(),
    );

    let buy_small = buy_quote(ctx.sim, entry.resource, small_lot);
    let buy_full = buy_quote(ctx.sim, entry.resource, cargo_lot);
    let sell_small = sell_quote(ctx.sim, entry.resource, small_lot);
    let sell_full = sell_quote(ctx.sim, entry.resource, cargo_lot);
    draw_ui_text_ex(
        &terms_line(buy_small, sell_small),
        cx + COL_TERMS,
        line_y,
        TextStyle::new(16.0, term::dim()).params(),
    );

    draw_trade_buttons(
        ctx,
        entry.resource,
        held,
        [buy_small, buy_full, sell_small, sell_full],
        row,
        pointer,
        actions,
    );
}

fn draw_trade_buttons(
    ctx: &GameplayCtx<'_>,
    resource: TradeResource,
    held: i64,
    quotes: [TradeQuote; 4],
    row: Rect,
    pointer: Pointer,
    actions: &mut Vec<UiAction>,
) {
    let gap = 8.0;
    let button_w = (row.w - 28.0 - gap * 3.0) / 4.0;
    let button_y = row.y + 62.0;
    for (index, quote) in quotes.into_iter().enumerate() {
        let buying = index < 2;
        let rect = Rect::new(
            row.x + 14.0 + index as f32 * (button_w + gap),
            button_y,
            button_w,
            44.0,
        );
        let enabled = if buying {
            ctx.sim.resources.credits >= quote.total_credits
        } else {
            held >= quote.amount
        };
        let label = if buying {
            format!("BUY {} · {}cr", quote.amount, quote.total_credits)
        } else {
            format!("SELL {} · +{}cr", quote.amount, quote.total_credits)
        };
        if term_button(rect, &label, enabled, pointer) {
            actions.push(if buying {
                UiAction::Buy(resource, quote.amount)
            } else {
                UiAction::Sell(resource, quote.amount)
            });
        }
    }
}

fn draw_market_footer(ctx: &GameplayCtx<'_>, content: Rect, y: f32) {
    if let Some(receipt) = ctx.sim.market.last_trade {
        draw_ui_text_ex(
            &format!(
                "LAST TICKET · {} {} {} · {}CR · MARKET {:.2} → {:.2}/U",
                if receipt.buying { "BOUGHT" } else { "SOLD" },
                receipt.amount,
                receipt.resource.label().to_uppercase(),
                receipt.total_credits,
                receipt.market_price_before,
                receipt.market_price_after
            ),
            content.x,
            y + 20.0,
            TextStyle::new(12.0, term::accent()).params(),
        );
    }
}
