//! Market: truthful buy/sell quotes at two visible lot sizes (GDD §5.1).

use crate::simulation::market::{buy_quote, sell_quote, TradeQuote};
use crate::simulation::ship::loadout_stats;
use crate::state::sim::TradeResource;
use crate::ui::{term, term_button, term_panel, GameplayCtx, UiAction};
use macroquad::prelude::*;
use macroquad_toolkit::prelude::*;
use macroquad_toolkit::ui::{draw_ui_text_ex, RectExt};

fn held_amount(ctx: &GameplayCtx<'_>, resource: TradeResource) -> i64 {
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

fn terms_line(buy: TradeQuote, sell: TradeQuote) -> String {
    let mut buy_terms = Vec::new();
    if let Some(label) = factor_label("name", buy.reputation_factor) {
        buy_terms.push(label);
    }
    if let Some(label) = factor_label("need", buy.pressure_factor) {
        buy_terms.push(label);
    }
    let mut sell_terms = Vec::new();
    if let Some(label) = factor_label("name", sell.reputation_factor) {
        sell_terms.push(label);
    }
    if let Some(label) = factor_label("distress", sell.pressure_factor) {
        sell_terms.push(label);
    }
    let buy_suffix = if buy_terms.is_empty() {
        "standard".to_owned()
    } else {
        buy_terms.join(", ")
    };
    let sell_suffix = if sell_terms.is_empty() {
        "standard".to_owned()
    } else {
        sell_terms.join(", ")
    };
    format!(
        "QUOTED TERMS  ·  buy {:.2}/u ({buy_suffix})  ·  sell {:.2}/u ({sell_suffix})",
        buy.effective_unit_price, sell.effective_unit_price
    )
}

fn trade_lot_sizes(cargo: i64) -> (i64, i64) {
    let cargo_lot = cargo.max(50);
    ((cargo_lot / 4).max(10).min(cargo_lot), cargo_lot)
}

pub fn draw(ctx: &GameplayCtx<'_>, area: Rect, pointer: Pointer, actions: &mut Vec<UiAction>) {
    term_panel(area, Some("COMMODITY EXCHANGE // QUOTES LOCK ON TAP"));
    let content = area.inset(24.0);
    let mut y = content.y + 44.0;

    let (small_lot, cargo_lot) = trade_lot_sizes(loadout_stats(ctx.sim, ctx.data).cargo as i64);

    draw_ui_text_ex(
        &format!(
            "TREASURY {}cr   ·   SMALL LOT {}   ·   CARGO LOT {}",
            ctx.sim.resources.credits, small_lot, cargo_lot
        ),
        content.x,
        y,
        TextStyle::new(17.0, term::accent()).params(),
    );
    y += 28.0;

    let col_held = 220.0;
    let col_price = 380.0;
    let col_trend = 540.0;
    for (label, offset) in [
        ("COMMODITY", 0.0),
        ("HELD", col_held),
        ("MARKET", col_price),
        ("TREND", col_trend),
    ] {
        draw_ui_text_ex(
            label,
            content.x + 14.0 + offset,
            y,
            TextStyle::new(12.0, term::faint()).params(),
        );
    }
    y += 10.0;

    const ROW_H: f32 = 100.0;
    const ROW_GAP: f32 = 8.0;
    for entry in &ctx.sim.market.entries {
        let row = Rect::new(content.x, y, content.w, ROW_H);
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
            ("—", term::dim())
        };
        draw_ui_text_ex(
            entry.resource.label(),
            cx,
            row.y + 18.0,
            TextStyle::new(15.0, term::primary()).params(),
        );
        draw_ui_text_ex(
            &held.to_string(),
            cx + col_held,
            row.y + 18.0,
            TextStyle::new(15.0, term::accent()).params(),
        );
        draw_ui_text_ex(
            &format!("{:.2} cr/u", entry.price),
            cx + col_price,
            row.y + 18.0,
            TextStyle::new(15.0, term::primary()).params(),
        );
        draw_ui_text_ex(
            &format!("{arrow} {:+.2}", entry.trend),
            cx + col_trend,
            row.y + 18.0,
            TextStyle::new(15.0, trend_color).params(),
        );

        let buy_small = buy_quote(ctx.sim, entry.resource, small_lot);
        let buy_full = buy_quote(ctx.sim, entry.resource, cargo_lot);
        let sell_small = sell_quote(ctx.sim, entry.resource, small_lot);
        let sell_full = sell_quote(ctx.sim, entry.resource, cargo_lot);
        draw_ui_text_ex(
            &terms_line(buy_small, sell_small),
            cx,
            row.y + 38.0,
            TextStyle::new(11.0, term::dim()).params(),
        );

        let gap = 8.0;
        let button_w = (row.w - 28.0 - gap * 3.0) / 4.0;
        let button_y = row.y + 48.0;
        for (index, (buying, quote)) in [
            (true, buy_small),
            (true, buy_full),
            (false, sell_small),
            (false, sell_full),
        ]
        .into_iter()
        .enumerate()
        {
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
                    UiAction::Buy(entry.resource, quote.amount)
                } else {
                    UiAction::Sell(entry.resource, quote.amount)
                });
            }
        }
        y += ROW_H + ROW_GAP;
    }

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
            y + 2.0,
            TextStyle::new(12.0, term::accent()).params(),
        );
    }
    draw_text_block(
        "The market line is raw. Quoted terms include your name and any desperation or distress penalty before you commit.",
        content.x,
        y + 19.0,
        content.w,
        34.0,
        12.0,
        2.0,
        term::dim(),
    );
}

#[cfg(test)]
mod tests;
