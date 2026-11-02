//! The operational view: voyage first, the living ship, then actionable attention.
use super::*;
use crate::simulation::readiness::{self, ReadinessBand};

mod watch;

pub fn draw(ctx: &GameplayCtx<'_>, area: Rect, pointer: Pointer, actions: &mut Vec<UiAction>) {
    if ctx.presentation.instruments.get() {
        dashboard::draw(ctx, area, pointer, actions);
        return;
    }
    // 1080p composition: the ship dominates at full height and the current
    // concern stands beside it. Developments are a brief change line inside
    // the ship panel; the full record lives in History.
    const GAP: f32 = 16.0;
    let subject = Rect::new(area.x, area.y, (area.w * 0.68).floor(), area.h);
    let attention = Rect::new(
        subject.right() + GAP,
        area.y,
        area.w - subject.w - GAP,
        area.h,
    );
    draw_voyage_subject(ctx, subject, pointer, actions);
    draw_attention(ctx, attention, pointer, actions);
}

fn draw_voyage_subject(
    ctx: &GameplayCtx<'_>,
    subject: Rect,
    pointer: Pointer,
    actions: &mut Vec<UiAction>,
) {
    let sim = ctx.sim;
    term_panel(subject, None);
    // Identity only: the charter's name, or plainly "In port". What to do in
    // port is the attention panel's single call to action.
    let title = sim.contract.as_ref().map_or("In port", |c| c.name.as_str());
    draw_ui_text_ex(
        title,
        subject.x + 28.0,
        subject.y + 46.0,
        TextStyle::new(30.0, term::primary()).params(),
    );
    draw_condition_gauges(
        sim,
        Rect::new(subject.x + 28.0, subject.y + 64.0, subject.w - 56.0, 26.0),
    );
    if let Some(c) = sim.contract.as_ref() {
        let phase = format!(
            "{} · Voyage {:.0}% · Objective {:.0}%",
            c.phase.label(),
            c.progress() * 100.0,
            c.objective_fraction() * 100.0
        );
        draw_ui_text_ex(
            &phase,
            subject.x + 28.0,
            subject.y + 116.0,
            TextStyle::new(20.0, term::dim()).params(),
        );
    }
    let diagram = Rect::new(
        subject.x + 28.0,
        subject.y + 136.0,
        subject.w - 56.0,
        subject.h - 236.0,
    );
    let ship = ship_schematic::build(sim, ctx.data, diagram);
    ship_schematic::draw(diagram, &ship);
    crate::ui::subsystems::select_compartments(ctx, &ship, pointer, actions, true);
    draw_footer(ctx, subject, pointer, actions);
}

/// The quiet bottom row of the ship panel: the latest development while it
/// is fresh, then the History and inspection routes at utility weight.
fn draw_footer(
    ctx: &GameplayCtx<'_>,
    subject: Rect,
    pointer: Pointer,
    actions: &mut Vec<UiAction>,
) {
    const BTN_W: f32 = 200.0;
    const BTN_H: f32 = 44.0;
    let y = subject.bottom() - 68.0;
    let inspect = Rect::new(subject.right() - 28.0 - BTN_W, y, BTN_W, BTN_H);
    let history = Rect::new(inspect.x - 12.0 - BTN_W, y, BTN_W, BTN_H);
    if let Some(entry) = fresh_development(ctx.sim) {
        draw_text_block(
            &format!("Year {} · {}", entry.year, entry.text),
            subject.x + 28.0,
            y + 2.0,
            history.x - subject.x - 52.0,
            BTN_H,
            18.0,
            4.0,
            term::dim(),
        );
    }
    if term_button_sized(history, "History", true, pointer, 15.0) {
        ctx.presentation.history_page.set(0);
        ctx.chronicle_records_tab.set(2);
        ctx.chronicle_scroll
            .set(macroquad_toolkit::ui::ScrollArea::new());
        actions.push(UiAction::SelectScreen(Screen::Chronicle));
    }
    if term_button_sized(inspect, "Inspect ship", true, pointer, 15.0) {
        actions.push(UiAction::SelectScreen(Screen::ShipBuilder));
    }
}

/// How many voyage months a development stays on the Bridge before it is
/// left to History.
const FRESH_MONTHS: u32 = 6;

/// The newest log entry, if it is recent enough to count as a change rather
/// than history.
fn fresh_development(sim: &crate::state::SimState) -> Option<&crate::state::sim::LogEntry> {
    let entry = sim.log.last()?;
    let now = sim.year() * 12 + sim.month();
    let then = entry.year * 12 + entry.month;
    (now.saturating_sub(then) <= FRESH_MONTHS).then_some(entry)
}

/// Hull, air and fuel as three gauges with the head count beside them: the
/// living ship's essentials, read at a glance instead of as a sentence.
fn draw_condition_gauges(sim: &crate::state::SimState, row: Rect) {
    const GAUGE_W: f32 = 220.0;
    const GAP: f32 = 16.0;
    let gauges = [
        ("HULL", sim.ship.hull_integrity),
        ("AIR", sim.ship.life_support),
        ("FUEL", sim.ship.fuel),
    ];
    for (i, (label, value)) in gauges.into_iter().enumerate() {
        let x = row.x + i as f32 * (GAUGE_W + GAP);
        term_meter(
            Rect::new(x, row.y, GAUGE_W, row.h),
            value,
            1.0,
            label,
            &format!("{:.0}%", value * 100.0),
        );
    }
    draw_ui_text_ex(
        &format!("{} aboard", sim.population.count),
        row.x + 3.0 * (GAUGE_W + GAP) + 8.0,
        row.y + row.h * 0.5 + 7.0,
        TextStyle::new(20.0, term::accent()).params(),
    );
}

fn draw_attention(
    ctx: &GameplayCtx<'_>,
    rect: Rect,
    pointer: Pointer,
    actions: &mut Vec<UiAction>,
) {
    term_panel(rect, Some("Attention & next action"));
    let sim = ctx.sim;
    let forecast = readiness::forecast(sim, ctx.data);
    let concern = forecast
        .rows
        .iter()
        .filter(|r| matches!(r.band, ReadinessBand::Critical | ReadinessBand::Vulnerable))
        .min_by(|a, b| a.score.total_cmp(&b.score));
    let energy_low = sim.resources.energy < ctx.data.config.low_energy_threshold;
    let (heading, detail, label, target) = if energy_low {
        ("! Energy shortage".to_owned(), format!("{} energy in stores; low-energy threshold {}. Trading is available in port; underway, inspect engineering and demand.", sim.resources.energy, ctx.data.config.low_energy_threshold), if sim.contract.is_none() { "Trade energy" } else { "Inspect engineering" }, if sim.contract.is_none() { Screen::Market } else { Screen::Subsystems })
    } else if let Some(row) = concern {
        (
            format!("! {} · {}", row.concern, row.band.label()),
            format!(
                "{}\nForecast holds current conditions steady.",
                row.evidence_line("\n")
            ),
            "Review recovery projects",
            Screen::Agenda,
        )
    } else if sim.contract.is_none() {
        (
            String::new(),
            String::new(),
            "Choose a charter",
            Screen::Drydock,
        )
    } else {
        (
            String::new(),
            String::new(),
            "Follow voyage",
            Screen::Contract,
        )
    };
    let resource_top = draw_due_commitment(ctx, rect, pointer, actions);
    if heading.is_empty() {
        // Nothing wrong: a lit lamp says so; the button below is the next step.
        draw_circle(rect.x + 34.0, resource_top + 14.0, 7.0, term::accent());
        draw_ui_text_ex(
            "All clear",
            rect.x + 52.0,
            resource_top + 21.0,
            TextStyle::new(22.0, term::dim()).params(),
        );
        watch::draw(ctx, rect, resource_top + 48.0, rect.bottom() - 88.0);
        attention_action(
            ctx,
            Rect::new(rect.x + 24.0, rect.bottom() - 72.0, rect.w - 48.0, 52.0),
            label,
            target,
            pointer,
            actions,
        );
        return;
    }
    draw_text_block(
        &heading,
        rect.x + 24.0,
        resource_top,
        rect.w - 48.0,
        76.0,
        26.0,
        6.0,
        if heading.starts_with('!') {
            term::alert()
        } else {
            term::primary()
        },
    );
    draw_text_block(
        &detail,
        rect.x + 24.0,
        resource_top + 84.0,
        rect.w - 48.0,
        rect.bottom() - resource_top - 172.0,
        20.0,
        6.0,
        term::dim(),
    );
    attention_action(
        ctx,
        Rect::new(rect.x + 24.0, rect.bottom() - 72.0, rect.w - 48.0, 52.0),
        label,
        target,
        pointer,
        actions,
    );
}

fn draw_due_commitment(
    ctx: &GameplayCtx<'_>,
    rect: Rect,
    pointer: Pointer,
    actions: &mut Vec<UiAction>,
) -> f32 {
    let sim = ctx.sim;
    let duty = sim
        .next_timed_obligation()
        .filter(|d| d.due_year.is_some_and(|year| year <= sim.year() + 1));
    let top = rect.y + 56.0;
    if let Some(duty) = duty {
        draw_text_block(
            &format!(
                "! Due year {} · {}",
                duty.due_year.unwrap_or_default(),
                duty.title
            ),
            rect.x + 24.0,
            top,
            rect.w - 48.0,
            60.0,
            20.0,
            5.0,
            term::alert(),
        );
        if term_button(
            Rect::new(rect.x + 24.0, top + 68.0, rect.w - 48.0, 52.0),
            "Review obligations",
            true,
            pointer,
        ) {
            go_to(ctx, Screen::Chronicle, actions);
        }
        top + 140.0
    } else {
        top
    }
}

fn attention_action(
    ctx: &GameplayCtx<'_>,
    rect: Rect,
    label: &str,
    target: Screen,
    pointer: Pointer,
    actions: &mut Vec<UiAction>,
) {
    if term_button_primary(rect, label, true, pointer) {
        go_to(ctx, target, actions);
    }
}

/// Obligation routes open History on the obligations ledger.
fn go_to(ctx: &GameplayCtx<'_>, target: Screen, actions: &mut Vec<UiAction>) {
    if target == Screen::Chronicle {
        ctx.presentation.history_page.set(1);
    }
    actions.push(UiAction::SelectScreen(target));
}
