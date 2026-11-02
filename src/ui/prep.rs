//! PREP screen (W4): the pre-launch beat. Shows the selected charter's phase
//! plan and a provisioning readout (food / parts / fuel need vs stores) with
//! stock-up buttons per store, and commits the voyage with the explicit
//! [ LAUNCH ] button. Pure view — it emits `SelectCharter` / `Launch` /
//! `Refuel` / `Buy` / `BuyParts` only.

use crate::ui::{term, term_button, term_panel, GameplayCtx, UiAction};
use macroquad::prelude::*;
use macroquad_toolkit::prelude::*;
use macroquad_toolkit::ui::{draw_ui_text_ex, RectExt};

mod provisioning;

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
    let provisioning =
        provisioning::draw(ctx, template, content, &forecast, y + 8.0, pointer, actions);
    draw_commit(content, conflict_count, provisioning, pointer, actions);
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
    // Only the loads this route actually carries; a benign route draws nothing.
    let hull = forecast.route_hull_change * 100.0;
    let life = forecast.route_life_support_change * 100.0;
    let terms: Vec<String> = [
        (template.hazard >= 0.005).then(|| format!("crisis weight +{:.2}", template.hazard)),
        (hull.round() != 0.0).then(|| format!("hull {hull:+.0}%")),
        (life.round() != 0.0).then(|| format!("life support {life:+.0}%")),
    ]
    .into_iter()
    .flatten()
    .collect();
    if terms.is_empty() {
        return y;
    }
    draw_ui_text_ex(
        &format!("ROUTE LOAD · {} over charter", terms.join(" · ")),
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

fn draw_commit(
    content: Rect,
    conflict_count: usize,
    provisioning: provisioning::ProvisioningState,
    pointer: Pointer,
    actions: &mut Vec<UiAction>,
) {
    let shortfalls = usize::from(provisioning.food_short > 0)
        + usize::from(provisioning.parts_short > 0)
        + usize::from(provisioning.refuel_missing > 0.001);
    let label = launch_commit_label(conflict_count, shortfalls);
    let launch = Rect::new(content.x, content.bottom() - 44.0, content.w, 44.0);
    if term_button(launch, &label, true, pointer) {
        actions.push(UiAction::Launch);
    }
}
#[allow(dead_code, unused_imports)]
mod tests {
    include!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/unit/ui/prep/tests.rs"
    ));
}
