//! PREP provisioning: one gauge row per store (food, parts, fuel) with its
//! stock-up action, then a year-by-year forecast of food and fuel drawn over
//! the charter's phase bands. The shapes replace the old "stored X /
//! recommended Y" sentences and the scoop/burn explanations.

use crate::data::contracts::ContractTemplate;
use crate::simulation::contract::forecast::{DepartureForecast, StockCurve};
use crate::state::sim::TradeResource;
use crate::ui::chart::{self, Band};
use crate::ui::{term, term_button, GameplayCtx, UiAction};
use macroquad::prelude::*;
use macroquad_toolkit::prelude::*;
use macroquad_toolkit::ui::draw_ui_text_ex;

/// Row stride: a 44px touch target plus breathing room.
const ROW_STRIDE: f32 = 56.0;
const LABEL_W: f32 = 96.0;
const VALUE_W: f32 = 132.0;
const BUTTON_W: f32 = 194.0;

pub(super) struct ProvisioningState {
    pub food_short: i64,
    pub parts_short: i64,
    pub refuel_missing: f32,
}

/// One store: label, gauge with the recommended level as a tick, and value.
/// The gauge's scale leaves headroom so both the fill and the tick fit.
fn store_row(content: Rect, y: f32, label: &str, frac: f32, target: f32, value: &str, short: bool) {
    let color = if short { term::alert() } else { term::accent() };
    draw_ui_text_ex(
        label,
        content.x,
        y + 28.0,
        TextStyle::new(16.0, color).params(),
    );
    let gauge_w = content.w - LABEL_W - VALUE_W - BUTTON_W - 12.0;
    chart::gauge(
        Rect::new(content.x + LABEL_W, y + 13.0, gauge_w, 18.0),
        frac,
        Some(target),
        color,
    );
    draw_ui_text_ex(
        value,
        content.x + LABEL_W + gauge_w + 12.0,
        y + 28.0,
        TextStyle::new(16.0, term::dim()).params(),
    );
}

fn row_button(content: Rect, y: f32) -> Rect {
    Rect::new(content.right() - BUTTON_W, y, BUTTON_W, 44.0)
}

/// Fill and tick positions for a stock against its recommendation.
fn stock_scale(have: i64, need: i64) -> (f32, f32) {
    let scale = (have.max(need).max(1) as f32) * 1.15;
    (have.max(0) as f32 / scale, need.max(0) as f32 / scale)
}

pub(super) fn draw(
    ctx: &GameplayCtx<'_>,
    template: &ContractTemplate,
    content: Rect,
    forecast: &DepartureForecast,
    mut y: f32,
    pointer: Pointer,
    actions: &mut Vec<UiAction>,
) -> ProvisioningState {
    let sim = ctx.sim;
    let food_short = draw_food_row(ctx, content, forecast, y, pointer, actions);
    y += ROW_STRIDE;
    let parts_short = draw_parts_row(ctx, content, forecast, y, pointer, actions);
    y += ROW_STRIDE;
    let refuel_missing = draw_fuel_row(ctx, content, y, pointer, actions);
    y += ROW_STRIDE;
    if ctx.tutorial_enabled
        && ctx.tutorial_open
        && !sim.tutorial_dismissed
        && sim.tutorial_step == 2
    {
        let review = Rect::new(content.x, y, content.w, 44.0);
        if term_button(review, "PROVISIONS REVIEWED", true, pointer) {
            actions.push(UiAction::ReviewProvisions);
        }
        y += ROW_STRIDE;
    }
    let curve = forecast.stock_curve(sim.resources.food, sim.ship.fuel, template);
    let chart_area = Rect::new(
        content.x,
        y + 8.0,
        content.w,
        content.bottom() - 60.0 - y - 8.0,
    );
    draw_forecast(template, forecast, &curve, chart_area);
    ProvisioningState {
        food_short,
        parts_short,
        refuel_missing,
    }
}

fn draw_food_row(
    ctx: &GameplayCtx<'_>,
    content: Rect,
    forecast: &DepartureForecast,
    y: f32,
    pointer: Pointer,
    actions: &mut Vec<UiAction>,
) -> i64 {
    let sim = ctx.sim;
    let (have, need) = (sim.resources.food, forecast.recommended_food_store);
    let short = (need - have).max(0);
    let (frac, target) = stock_scale(have, need);
    store_row(
        content,
        y,
        "FOOD",
        frac,
        target,
        &format!("{have} / {need}"),
        short > 0,
    );
    let quote = crate::simulation::market::buy_quote(sim, TradeResource::Food, 1);
    let afford = if quote.effective_unit_price > 0.0 {
        (sim.resources.credits as f32 / quote.effective_unit_price).floor() as i64
    } else {
        0
    };
    let buy = short.min(afford);
    let cost = crate::simulation::market::buy_quote(sim, TradeResource::Food, buy).total_credits;
    let label = if short == 0 {
        "FOOD STOCKED".to_owned()
    } else if buy <= 0 {
        "NO CREDITS".to_owned()
    } else {
        format!("+{buy} · {cost} CR")
    };
    if term_button(row_button(content, y), &label, buy > 0, pointer) {
        actions.push(UiAction::Buy(TradeResource::Food, buy));
    }
    short
}

fn draw_parts_row(
    ctx: &GameplayCtx<'_>,
    content: Rect,
    forecast: &DepartureForecast,
    y: f32,
    pointer: Pointer,
    actions: &mut Vec<UiAction>,
) -> i64 {
    let sim = ctx.sim;
    let (have, need) = (sim.ship.spare_parts, forecast.parts_upkeep);
    let short = (need - have).max(0);
    let (frac, target) = stock_scale(have, need);
    store_row(
        content,
        y,
        "PARTS",
        frac,
        target,
        &format!("{have} / {need}"),
        short > 0,
    );
    let price = ctx.data.config.provisioning.part_cost_credits;
    let afford = if price > 0 {
        sim.resources.credits / price
    } else {
        0
    };
    let buy = short.min(afford);
    let label = if short == 0 {
        "PARTS STOCKED".to_owned()
    } else if buy <= 0 {
        "NO CREDITS".to_owned()
    } else {
        format!("+{buy} · {} CR", buy * price)
    };
    if term_button(row_button(content, y), &label, buy > 0, pointer) {
        actions.push(UiAction::BuyParts(buy));
    }
    short
}

/// The tank against a full tank; the refuel action lives on this row so every
/// store is topped up where it is read.
fn draw_fuel_row(
    ctx: &GameplayCtx<'_>,
    content: Rect,
    y: f32,
    pointer: Pointer,
    actions: &mut Vec<UiAction>,
) -> f32 {
    let sim = ctx.sim;
    let missing = (1.0 - sim.ship.fuel).max(0.0);
    let short = missing > 0.001;
    // A full tank sits at the track's end, so the tick marks the target there.
    store_row(
        content,
        y,
        "FUEL",
        sim.ship.fuel,
        1.0,
        &format!("{:.0}%", sim.ship.fuel * 100.0),
        short,
    );
    let cost = (ctx.data.config.provisioning.fuel_cost_credits_per_point as f32 * missing * 100.0)
        .ceil() as i64;
    let affordable = sim.resources.credits >= cost;
    let label = if !short {
        "TANKS FULL".to_owned()
    } else if affordable {
        format!("REFUEL · {cost} CR")
    } else {
        format!("NEED {cost} CR")
    };
    if term_button(row_button(content, y), &label, short && affordable, pointer) {
        actions.push(UiAction::Refuel);
    }
    missing
}

/// Two stacked charts sharing the phase bands: food stores and the fuel tank,
/// each with its end state or the year it runs dry in the header.
fn draw_forecast(
    template: &ContractTemplate,
    forecast: &DepartureForecast,
    curve: &StockCurve,
    area: Rect,
) {
    let labels: Vec<String> = template
        .phases
        .iter()
        .map(|phase| phase.kind.label().to_uppercase())
        .collect();
    let bands: Vec<Band<'_>> = template
        .phases
        .iter()
        .zip(&labels)
        .map(|(phase, label)| Band {
            label,
            span: phase.years as f32,
        })
        .collect();
    let header_h = 28.0;
    let gap = 14.0;
    let chart_h = ((area.h - 2.0 * header_h - gap) / 2.0).max(40.0);

    let food: Vec<f32> = curve.food.iter().map(|value| *value as f32).collect();
    let food_hi = food
        .iter()
        .copied()
        .fold(forecast.recommended_food_store as f32, f32::max)
        .max(1.0)
        * 1.1;
    let food_status = match curve.food_dry_year {
        Some(year) => (format!("RUNS OUT Y{year}"), true),
        None => (
            format!("END {}", curve.food.last().copied().unwrap_or(0)),
            false,
        ),
    };
    let food_rect = Rect::new(area.x, area.y + header_h, area.w, chart_h);
    draw_chart(
        "FOOD STORES",
        food_status,
        food_rect,
        &bands,
        &food,
        food_hi,
        curve.food_dry_year,
    );

    let fuel_status = match curve.fuel_dry_year {
        Some(year) => (format!("TANK DRY Y{year} · CLOCK STALLS"), true),
        None => (
            format!(
                "END {:.0}%",
                curve.fuel.last().copied().unwrap_or(0.0) * 100.0
            ),
            false,
        ),
    };
    let fuel_rect = Rect::new(area.x, food_rect.bottom() + gap + header_h, area.w, chart_h);
    draw_chart(
        "FUEL TANK",
        fuel_status,
        fuel_rect,
        &bands,
        &curve.fuel,
        1.0,
        curve.fuel_dry_year,
    );
}

fn draw_chart(
    title: &str,
    (status, dry): (String, bool),
    rect: Rect,
    bands: &[Band<'_>],
    values: &[f32],
    hi: f32,
    dry_year: Option<u32>,
) {
    let tone = if dry { term::alert() } else { term::accent() };
    draw_ui_text_ex(
        title,
        rect.x,
        rect.y - 8.0,
        TextStyle::new(16.0, term::primary()).params(),
    );
    let status_w = measure_text_size(&status, TextStyle::new(16.0, tone)).width;
    draw_ui_text_ex(
        &status,
        rect.right() - status_w,
        rect.y - 8.0,
        TextStyle::new(16.0, tone).params(),
    );
    chart::bands(rect, bands);
    // Inset the plot so the line never rides on the frame or under the labels.
    let plot = Rect::new(rect.x + 4.0, rect.y + 26.0, rect.w - 8.0, rect.h - 30.0);
    let points = chart::line_points(plot, values, 0.0, hi);
    chart::polyline(&points, 2.5, tone);
    if let Some(point) = dry_year.and_then(|year| points.get(year as usize)) {
        chart::marker(*point, 7.0, term::alert());
    }
}
