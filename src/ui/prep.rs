//! PREP screen (W4): the pre-launch beat. Shows the selected charter's phase
//! plan and a provisioning readout (food / parts / fuel need vs stores) with
//! stock-up buttons per store, and commits the voyage with the explicit
//! [ LAUNCH ] button. Pure view — it emits `SelectCharter` / `Launch` /
//! `Refuel` / `Buy` / `BuyParts` only.

use crate::state::sim::TradeResource;
use crate::ui::{term, term_button, term_panel, GameplayCtx, UiAction};
use macroquad::prelude::*;
use macroquad_toolkit::prelude::*;
use macroquad_toolkit::ui::{draw_ui_text_ex, RectExt};

/// Vertical stride of one PROVISIONING row, and so of its stock-up target.
///
/// 44 is the touch standard (WCAG 2.5.5, Apple, roughly Google's 48dp), and it
/// is the *stride* that decides it: a hit area grows only halfway toward its
/// neighbour, so rows packed at 30 cap their targets at 30 however tall the
/// button is drawn.
const PROVISION_STRIDE: f32 = 64.0;

pub(crate) fn launch_commit_label(conflicts: usize, shortfalls: usize) -> String {
    match (conflicts, shortfalls) {
        (0, 0) => "[ LAUNCH ]".to_owned(),
        (0, shortfalls) => format!("LAUNCH UNDERSTOCKED · {shortfalls}"),
        (conflicts, 0) => format!("LAUNCH & DEFAULT {conflicts}"),
        (conflicts, shortfalls) => {
            format!("LAUNCH · {shortfalls} SHORT · DEFAULT {conflicts}")
        }
    }
}

pub fn draw(ctx: &GameplayCtx<'_>, area: Rect, pointer: Pointer, actions: &mut Vec<UiAction>) {
    let left = Rect::new(area.x, area.y, area.w * 0.55, area.h);
    let right = Rect::new(left.right() + 12.0, area.y, area.w - left.w - 12.0, area.h);

    draw_prep(ctx, left, pointer, actions);

    crate::ui::mission::draw_selected(ctx, right, pointer, actions);
}

/// One `LABEL — have / need` provisioning line, reddened when short.
fn provision_line(x: f32, y: f32, label: &str, have: i64, need: i64, note: &str) {
    let color = if have < need {
        term::alert()
    } else {
        term::accent()
    };
    draw_ui_text_ex(
        &format!("{label} - stored {have} / recommended {need}"),
        x,
        y,
        TextStyle::new(13.0, color).params(),
    );
    if !note.is_empty() {
        draw_ui_text_ex(
            note,
            x,
            y + 17.0,
            TextStyle::new(11.0, term::dim()).params(),
        );
    }
}

fn draw_prep(ctx: &GameplayCtx<'_>, rect: Rect, pointer: Pointer, actions: &mut Vec<UiAction>) {
    let sim = ctx.sim;
    let Some(id) = sim.selected_charter.as_deref() else {
        return;
    };
    let Some(template) = ctx.data.contracts.get(id) else {
        return;
    };
    let forecast = crate::simulation::contract::forecast::for_departure(sim, ctx.data, template);

    term_panel(rect, Some("PREP // DEPARTURE"));
    let content = rect.inset(18.0);
    let y = draw_prep_header(template, content);
    let (conflict_count, y) = draw_conflicts(sim, template, content, y);
    let y = draw_phase_plan(template, content, y);
    let y = draw_route_load(template, &forecast, content, y);
    let provisioning = draw_provisioning(ctx, content, &forecast, y, pointer, actions);
    draw_commit(ctx, content, conflict_count, provisioning, pointer, actions);
}

fn draw_prep_header(template: &crate::data::contracts::ContractTemplate, content: Rect) -> f32 {
    let y = content.y + 38.0;
    draw_ui_text_ex(
        &template.name,
        content.x,
        y,
        TextStyle::new(19.0, term::accent()).params(),
    );
    draw_ui_text_ex(
        &format!(
            "{} · {} YEARS · reward {} cr",
            template.objective.label().to_uppercase(),
            template.target_duration_years,
            template.reward.credits
        ),
        content.x,
        y + 24.0,
        TextStyle::new(13.0, term::dim()).params(),
    );
    y + 52.0
}

fn draw_conflicts(
    sim: &crate::state::sim::SimState,
    template: &crate::data::contracts::ContractTemplate,
    content: Rect,
    mut y: f32,
) -> (usize, f32) {
    let conflicts = crate::simulation::contract::obligation_conflicts(sim, template);
    let count = conflicts.len();
    if conflicts.is_empty() {
        return (count, y);
    }
    draw_ui_text_ex(
        "! OBLIGATION CONFLICT — LAUNCH WOULD CONTRADICT:",
        content.x,
        y,
        TextStyle::new(13.0, term::alert()).params(),
    );
    y += 18.0;
    for obligation in conflicts {
        draw_ui_text_ex(
            &format!(
                "  {} — owed to {}",
                obligation.title, obligation.beneficiary
            ),
            content.x,
            y,
            TextStyle::new(12.0, term::alert()).params(),
        );
        y += 17.0;
    }
    draw_ui_text_ex(
        "  LAUNCH & DEFAULT records each promise broken.",
        content.x,
        y,
        TextStyle::new(12.0, term::alert()).params(),
    );
    (count, y + 23.0)
}

fn draw_phase_plan(
    template: &crate::data::contracts::ContractTemplate,
    content: Rect,
    y: f32,
) -> f32 {
    draw_ui_text_ex(
        "PHASE PLAN",
        content.x,
        y,
        TextStyle::new(14.0, term::primary()).params(),
    );
    let bar_y = y + 12.0;
    let total_years = template.target_duration_years.max(1) as f32;
    let bar = Rect::new(content.x, bar_y, content.w, 22.0);
    let mut bx = bar.x;
    for segment in &template.phases {
        let width = bar.w * (segment.years as f32 / total_years);
        let rect = Rect::new(bx, bar.y, (width - 3.0).max(1.0), bar.h);
        draw_surface(
            rect,
            &SurfaceStyle::new(term::surface_inset()).with_border(1.0, term::faint()),
        );
        draw_ui_text_ex(
            &format!("{} {}y", segment.kind.label().to_uppercase(), segment.years),
            rect.x + 5.0,
            rect.y + 15.0,
            TextStyle::new(10.0, term::dim()).params(),
        );
        bx += width;
    }
    bar_y + 36.0
}

fn draw_route_load(
    template: &crate::data::contracts::ContractTemplate,
    forecast: &crate::simulation::contract::forecast::DepartureForecast,
    content: Rect,
    y: f32,
) -> f32 {
    draw_ui_text_ex(
        &format!(
            "ROUTE LOAD · crisis weight +{:.2} · hull {:+.0}% · life support {:+.0}% over charter",
            template.hazard,
            forecast.route_hull_change * 100.0,
            forecast.route_life_support_change * 100.0
        ),
        content.x,
        y,
        TextStyle::new(
            12.0,
            if template.hazard > 0.0 || !template.annual_toll.is_none() {
                term::alert()
            } else {
                term::dim()
            },
        )
        .params(),
    );
    y + 20.0
}

struct ProvisioningState {
    food_short: i64,
    parts_short: i64,
    refuel_missing: f32,
    refuel_cost: i64,
}

fn draw_provisioning(
    ctx: &GameplayCtx<'_>,
    content: Rect,
    forecast: &crate::simulation::contract::forecast::DepartureForecast,
    mut y: f32,
    pointer: Pointer,
    actions: &mut Vec<UiAction>,
) -> ProvisioningState {
    let sim = ctx.sim;
    draw_ui_text_ex(
        "BASELINE PROVISIONING FORECAST",
        content.x,
        y,
        TextStyle::new(14.0, term::primary()).params(),
    );
    y += 20.0;
    draw_ui_text_ex(
        "Food is a reserve; farms supply the crossing. Events can change this forecast.",
        content.x,
        y,
        TextStyle::new(11.0, term::dim()).params(),
    );
    y += 22.0;
    let food_short = draw_food_provision(sim, content, forecast, y, pointer, actions);
    y += PROVISION_STRIDE;
    let parts_short = draw_parts_provision(ctx, content, forecast, y, pointer, actions);
    y += PROVISION_STRIDE;
    draw_fuel_provision(sim, content, forecast, y);
    let refuel_missing = 1.0 - sim.ship.fuel;
    let refuel_cost =
        (ctx.data.config.provisioning.fuel_cost_credits_per_point as f32 * refuel_missing * 100.0)
            .ceil() as i64;
    let review_y = y + 26.0;
    if ctx.tutorial_enabled
        && ctx.tutorial_open
        && !sim.tutorial_dismissed
        && sim.tutorial_step == 2
    {
        let review = Rect::new(content.x, review_y + 24.0, content.w, 44.0);
        if term_button(review, "PROVISIONS REVIEWED", true, pointer) {
            actions.push(UiAction::ReviewProvisions);
        }
    }
    ProvisioningState {
        food_short,
        parts_short,
        refuel_missing,
        refuel_cost,
    }
}

fn draw_food_provision(
    sim: &crate::state::sim::SimState,
    content: Rect,
    forecast: &crate::simulation::contract::forecast::DepartureForecast,
    y: f32,
    pointer: Pointer,
    actions: &mut Vec<UiAction>,
) -> i64 {
    let food_need = forecast.recommended_food_store;
    provision_line(
        content.x,
        y,
        "FOOD",
        sim.resources.food,
        food_need,
        &format!(
            "Projected end {} · net {:+}/yr ({} made / {} eaten)",
            forecast.projected_food_end,
            forecast.annual_food_net,
            forecast.annual_food_output,
            forecast.annual_food_use
        ),
    );
    let short = (food_need - sim.resources.food).max(0);
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
        "NO CREDITS FOR FOOD".to_owned()
    } else {
        format!("+{buy} FOOD · {cost} CR")
    };
    if term_button(stock_button(content, y), &label, buy > 0, pointer) {
        actions.push(UiAction::Buy(TradeResource::Food, buy));
    }
    short
}

fn draw_parts_provision(
    ctx: &GameplayCtx<'_>,
    content: Rect,
    forecast: &crate::simulation::contract::forecast::DepartureForecast,
    y: f32,
    pointer: Pointer,
    actions: &mut Vec<UiAction>,
) -> i64 {
    let sim = ctx.sim;
    let need = forecast.parts_upkeep;
    provision_line(
        content.x,
        y,
        "PARTS",
        sim.ship.spare_parts,
        need,
        "or restock via a full refit",
    );
    let short = (need - sim.ship.spare_parts).max(0);
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
        "NO CREDITS FOR PARTS".to_owned()
    } else {
        format!("+{buy} PARTS · {} CR", buy * price)
    };
    if term_button(stock_button(content, y), &label, buy > 0, pointer) {
        actions.push(UiAction::BuyParts(buy));
    }
    short
}

fn stock_button(content: Rect, y: f32) -> Rect {
    Rect::new(content.right() - 200.0, y - 24.0, 194.0, 44.0)
}

fn draw_fuel_provision(
    sim: &crate::state::sim::SimState,
    content: Rect,
    forecast: &crate::simulation::contract::forecast::DepartureForecast,
    y: f32,
) {
    let color = if sim.ship.fuel < 1.0 {
        term::alert()
    } else {
        term::accent()
    };
    draw_ui_text_ex(
        &format!(
            "FUEL  — tank {:.0}%  ·  {} travel years · total burn {:.2} full tanks",
            sim.ship.fuel * 100.0,
            forecast.travel_years,
            forecast.fuel_burn
        ),
        content.x,
        y,
        TextStyle::new(13.0, color).params(),
    );
    draw_ui_text_ex(
        &format!(
            "Scoops replenish up to {:.1}% of a tank per year; capacity is one full tank.",
            forecast.fuel_regen_per_year * 100.0
        ),
        content.x,
        y + 18.0,
        TextStyle::new(12.0, term::dim()).params(),
    );
}

fn draw_commit(
    ctx: &GameplayCtx<'_>,
    content: Rect,
    conflict_count: usize,
    provisioning: ProvisioningState,
    pointer: Pointer,
    actions: &mut Vec<UiAction>,
) {
    let sim = ctx.sim;
    let by = content.bottom() - 44.0;
    let bw = (content.w - 12.0) / 2.0;
    let shortfalls = usize::from(provisioning.food_short > 0)
        + usize::from(provisioning.parts_short > 0)
        + usize::from(provisioning.refuel_missing > 0.001);
    let label = launch_commit_label(conflict_count, shortfalls);
    if term_button(Rect::new(content.x, by, bw, 44.0), &label, true, pointer) {
        actions.push(UiAction::Launch);
    }
    let refuel_label =
        if provisioning.refuel_missing > 0.0 && sim.resources.credits < provisioning.refuel_cost {
            format!("NEED {} CR", provisioning.refuel_cost)
        } else if provisioning.refuel_missing > 0.0 {
            format!("REFUEL ({} CR)", provisioning.refuel_cost)
        } else {
            "TANKS FULL".to_owned()
        };
    if term_button(
        Rect::new(content.x + bw + 12.0, by, bw, 44.0),
        &refuel_label,
        provisioning.refuel_missing > 0.0 && sim.resources.credits >= provisioning.refuel_cost,
        pointer,
    ) {
        actions.push(UiAction::Refuel);
    }
}
#[allow(dead_code, unused_imports)]
mod tests {
    include!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/unit/ui/prep/tests.rs"
    ));
}
