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
const PROVISION_STRIDE: f32 = 44.0;

fn launch_commit_label(conflicts: usize, shortfalls: usize) -> String {
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

    // Swap column: the charter list, so a different charter can be selected.
    // Cards start below the panel's header band so they never overlap its title.
    term_panel(right, Some("CHOOSE / SWAP CHARTER"));
    let inner = right.inset(18.0);
    let cards = Rect::new(inner.x, inner.y + 28.0, inner.w, inner.h - 28.0);
    crate::ui::contract_systems::draw_charter_cards(ctx, cards, pointer, actions);
}

/// One `LABEL — have / need` provisioning line, reddened when short.
fn provision_line(x: f32, y: f32, label: &str, have: i64, need: i64, note: &str) {
    let color = if have < need {
        term::alert()
    } else {
        term::accent()
    };
    let tail = if note.is_empty() {
        String::new()
    } else {
        format!("   ·   {note}")
    };
    draw_ui_text_ex(
        &format!("{label} — have {have} / need {need}{tail}"),
        x,
        y,
        TextStyle::new(13.0, color).params(),
    );
}

fn draw_prep(ctx: &GameplayCtx<'_>, rect: Rect, pointer: Pointer, actions: &mut Vec<UiAction>) {
    let sim = ctx.sim;
    let Some(id) = sim.selected_charter.as_deref() else {
        return;
    };
    let Some(template) = ctx.data.contracts.get(id) else {
        return;
    };
    let config = &ctx.data.config;
    let forecast = crate::simulation::contract::forecast::for_departure(sim, ctx.data, template);

    term_panel(rect, Some("PREP // DEPARTURE"));
    let content = rect.inset(18.0);
    let mut y = content.y + 38.0;

    draw_ui_text_ex(
        &template.name,
        content.x,
        y,
        TextStyle::new(19.0, term::accent()).params(),
    );
    y += 24.0;
    draw_ui_text_ex(
        &format!(
            "{} · {} YEARS · reward {} cr",
            template.objective.label().to_uppercase(),
            template.target_duration_years,
            template.reward.credits
        ),
        content.x,
        y,
        TextStyle::new(13.0, term::dim()).params(),
    );
    y += 28.0;

    let conflicts = crate::simulation::contract::obligation_conflicts(sim, template);
    let conflict_count = conflicts.len();
    if !conflicts.is_empty() {
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
        y += 17.0;
        y += 6.0;
    }

    // --- Phase plan (authored segments, proportional) ---
    draw_ui_text_ex(
        "PHASE PLAN",
        content.x,
        y,
        TextStyle::new(14.0, term::primary()).params(),
    );
    y += 12.0;
    let total_years = template.target_duration_years.max(1) as f32;
    let bar = Rect::new(content.x, y, content.w, 22.0);
    let mut bx = bar.x;
    for seg in &template.phases {
        let w = bar.w * (seg.years as f32 / total_years);
        let seg_rect = Rect::new(bx, bar.y, (w - 3.0).max(1.0), bar.h);
        draw_surface(
            seg_rect,
            &SurfaceStyle::new(term::surface_inset()).with_border(1.0, term::faint()),
        );
        draw_ui_text_ex(
            &format!("{} {}y", seg.kind.label().to_uppercase(), seg.years),
            seg_rect.x + 5.0,
            seg_rect.y + 15.0,
            TextStyle::new(10.0, term::dim()).params(),
        );
        bx += w;
    }
    y += 36.0;

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
    y += 20.0;

    // --- Provisioning readout ---
    draw_ui_text_ex(
        "BASELINE PROVISIONING FORECAST",
        content.x,
        y,
        TextStyle::new(14.0, term::primary()).params(),
    );
    y += 22.0;
    // Each provisioning row carries its own stock-up button so filling the
    // stores never means leaving the PREP screen.
    // 36 tall on a PROVISION_STRIDE row: the 8px it leaves is what the touch
    // expansion grows into, 4px each way, so the target reaches the full 44 the
    // standard asks for. This screen had the room; the CREW posts column did not.
    let stock_btn = |y: f32| Rect::new(content.right() - 200.0, y - 22.0, 194.0, 36.0);

    // Food: a current-state projection that includes crew skill, agriculture
    // tier/condition, consumption, and a standing route toll. This is a useful
    // reserve rather than gross centuries of consumption that onboard farms
    // will replace. Events and future deterioration remain explicitly outside
    // the baseline.
    let food_need = forecast.recommended_food_store;
    provision_line(
        content.x,
        y,
        "FOOD ",
        sim.resources.food,
        food_need,
        &format!(
            "end {} · net {:+}/yr ({} made / {} eaten)",
            forecast.projected_food_end,
            forecast.annual_food_net,
            forecast.annual_food_output,
            forecast.annual_food_use
        ),
    );
    let food_short = (food_need - sim.resources.food).max(0);
    let unit_quote = crate::simulation::market::buy_quote(sim, TradeResource::Food, 1);
    let food_afford = if unit_quote.effective_unit_price > 0.0 {
        (sim.resources.credits as f32 / unit_quote.effective_unit_price).floor() as i64
    } else {
        0
    };
    let food_buy = food_short.min(food_afford);
    let food_cost =
        crate::simulation::market::buy_quote(sim, TradeResource::Food, food_buy).total_credits;
    let food_label = if food_short == 0 {
        "FOOD STOCKED".to_owned()
    } else if food_buy <= 0 {
        "NO CREDITS FOR FOOD".to_owned()
    } else {
        format!("+{food_buy} FOOD · {food_cost} CR")
    };
    if term_button(stock_btn(y), &food_label, food_buy > 0, pointer) {
        actions.push(UiAction::Buy(TradeResource::Food, food_buy));
    }
    y += PROVISION_STRIDE;

    // Spare parts: yearly upkeep across the voyage vs stores. The button stocks
    // the shortfall at the drydock part price, capped by the treasury.
    let parts_need = forecast.parts_upkeep;
    provision_line(
        content.x,
        y,
        "PARTS",
        sim.ship.spare_parts,
        parts_need,
        "or restock via a full refit",
    );
    let parts_short = (parts_need - sim.ship.spare_parts).max(0);
    let part_price = config.provisioning.part_cost_credits;
    let parts_afford = if part_price > 0 {
        sim.resources.credits / part_price
    } else {
        0
    };
    let parts_buy = parts_short.min(parts_afford);
    let parts_label = if parts_short == 0 {
        "PARTS STOCKED".to_owned()
    } else if parts_buy <= 0 {
        "NO CREDITS FOR PARTS".to_owned()
    } else {
        format!("+{parts_buy} PARTS · {} CR", parts_buy * part_price)
    };
    if term_button(stock_btn(y), &parts_label, parts_buy > 0, pointer) {
        actions.push(UiAction::BuyParts(parts_buy));
    }
    y += PROVISION_STRIDE;

    // Fuel: burned only across Travel months; the tank caps at 1.0 and the
    // engine regen tops it up underway, so need can exceed a single tank.
    let fuel_color = if sim.ship.fuel < 1.0 {
        term::alert()
    } else {
        term::accent()
    };
    draw_ui_text_ex(
        &format!(
            "FUEL  — tank {:.0}%  ·  burn {:.2} over {} travel yrs · regen up to {:.2}/yr",
            sim.ship.fuel * 100.0,
            forecast.fuel_burn,
            forecast.travel_years,
            forecast.fuel_regen_per_year
        ),
        content.x,
        y,
        TextStyle::new(13.0, fuel_color).params(),
    );
    y += 8.0;

    // --- First-voyage checklist (tutorial) ---
    // Shown until the Chronicle records a mission or the player dismisses it;
    // every label and tip is authored in game_config, per the data rule.
    if !sim.tutorial_dismissed && ctx.chronicle.entries.is_empty() {
        let step_done = |id: &str| match id {
            "choose_charter" => true, // being on PREP means one is selected
            "stock_food" => food_short == 0,
            "stock_parts" => parts_short == 0,
            "fuel_tanks" => sim.ship.fuel >= 0.999,
            // "launch" (and anything unknown) completes only by doing it.
            _ => false,
        };
        let steps = &ctx.data.config.tutorial.steps;
        let boxed = Rect::new(
            content.x,
            y + 20.0,
            content.w,
            92.0 + steps.len() as f32 * 22.0,
        );
        draw_surface(
            boxed,
            &SurfaceStyle::new(term::surface_inset()).with_border(1.0, term::faint()),
        );
        draw_ui_text_ex(
            "FIRST VOYAGE // PRE-LAUNCH CHECKLIST",
            boxed.x + 12.0,
            boxed.y + 22.0,
            TextStyle::new(14.0, term::primary()).params(),
        );
        if term_button(
            Rect::new(boxed.right() - 96.0, boxed.y + 6.0, 88.0, 28.0),
            "DISMISS",
            true,
            pointer,
        ) {
            actions.push(UiAction::DismissTutorial);
        }

        let active = steps.iter().position(|s| !step_done(&s.id));
        let mut sy = boxed.y + 46.0;
        for (i, step) in steps.iter().enumerate() {
            let done = step_done(&step.id);
            let (mark, color) = if done {
                ("[x]", term::accent())
            } else if active == Some(i) {
                ("[>]", term::primary())
            } else {
                ("[ ]", term::dim())
            };
            draw_ui_text_ex(
                &format!("{mark} {}", step.label),
                boxed.x + 12.0,
                sy,
                TextStyle::new(13.0, color).params(),
            );
            sy += 22.0;
        }
        // The tip for whatever the voyage needs next.
        if let Some(step) = active.and_then(|i| steps.get(i)) {
            draw_text_block(
                &step.tip,
                boxed.x + 12.0,
                sy + 4.0,
                boxed.w - 24.0,
                40.0,
                12.0,
                3.0,
                term::dim(),
            );
        }
    }

    // --- Commit / refuel ---
    let refuel_missing = 1.0 - sim.ship.fuel;
    let refuel_cost =
        (config.provisioning.fuel_cost_credits_per_point as f32 * refuel_missing * 100.0).ceil()
            as i64;
    let by = content.bottom() - 44.0;
    let bw = (content.w - 12.0) / 2.0;
    // Under-provisioning remains an allowed strategic risk, but the commit
    // button must name that risk at the instant the player takes it. Promise
    // conflicts remain independently counted because they create defaults.
    let shortfall_count = usize::from(food_short > 0)
        + usize::from(parts_short > 0)
        + usize::from(refuel_missing > 0.001);
    let launch_label = launch_commit_label(conflict_count, shortfall_count);
    if term_button(
        Rect::new(content.x, by, bw, 44.0),
        &launch_label,
        true,
        pointer,
    ) {
        actions.push(UiAction::Launch);
    }
    let refuel_label = if refuel_missing > 0.0 && sim.resources.credits < refuel_cost {
        format!("NEED {refuel_cost} CR")
    } else if refuel_missing > 0.0 {
        format!("REFUEL ({refuel_cost} CR)")
    } else {
        "TANKS FULL".to_owned()
    };
    if term_button(
        Rect::new(content.x + bw + 12.0, by, bw, 44.0),
        &refuel_label,
        refuel_missing > 0.0 && sim.resources.credits >= refuel_cost,
        pointer,
    ) {
        actions.push(UiAction::Refuel);
    }
}

#[cfg(test)]
mod tests;
