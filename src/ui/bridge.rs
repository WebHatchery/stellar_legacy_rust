//! The operational view: voyage first, the living ship, then actionable attention.
use super::*;
use crate::simulation::readiness::{self, ReadinessBand};

pub fn draw(ctx: &GameplayCtx<'_>, area: Rect, pointer: Pointer, actions: &mut Vec<UiAction>) {
    if ctx.presentation.instruments.get() {
        dashboard::draw(ctx, area, pointer, actions);
        return;
    }
    let subject = Rect::new(area.x, area.y, area.w * 0.64, 400.0);
    let attention = Rect::new(
        subject.right() + 16.0,
        area.y,
        area.w - subject.w - 16.0,
        400.0,
    );
    draw_voyage_subject(ctx, subject, pointer, actions);
    draw_attention(ctx, attention, pointer, actions);
    let bottom = Rect::new(area.x, subject.bottom() + 14.0, area.w, area.h - 414.0);
    draw_operational_summary(ctx, bottom, pointer, actions);
}

fn draw_voyage_subject(
    ctx: &GameplayCtx<'_>,
    subject: Rect,
    pointer: Pointer,
    actions: &mut Vec<UiAction>,
) {
    let sim = ctx.sim;
    term_panel(subject, None);
    let title = sim
        .contract
        .as_ref()
        .map_or("Choose the next voyage", |c| c.name.as_str());
    draw_text_block(
        title,
        subject.x + 20.0,
        subject.y + 16.0,
        subject.w - 40.0,
        36.0,
        26.0,
        4.0,
        term::primary(),
    );
    let phase = sim.contract.as_ref().map_or_else(
        || "In port · Review a charter, then prepare provisions".to_owned(),
        |c| {
            format!(
                "{} · Voyage {:.0}% · Objective {:.0}%",
                c.phase.label(),
                c.progress() * 100.0,
                c.objective_fraction() * 100.0
            )
        },
    );
    draw_text_block(
        &phase,
        subject.x + 20.0,
        subject.y + 56.0,
        subject.w - 40.0,
        44.0,
        17.0,
        4.0,
        term::dim(),
    );
    let diagram = Rect::new(subject.x + 24.0, subject.y + 82.0, subject.w - 48.0, 250.0);
    let ship = ship_schematic::build(sim, ctx.data, diagram);
    ship_schematic::draw(diagram, &ship);
    crate::ui::subsystems::select_compartments(ctx, &ship, pointer, actions, true);
    if term_button(
        Rect::new(subject.x + 20.0, subject.bottom() - 58.0, 250.0, 44.0),
        "Inspect ship & compartments",
        true,
        pointer,
    ) {
        actions.push(UiAction::SelectScreen(Screen::ShipBuilder));
    }
}

fn draw_operational_summary(
    ctx: &GameplayCtx<'_>,
    bottom: Rect,
    pointer: Pointer,
    actions: &mut Vec<UiAction>,
) {
    let sim = ctx.sim;
    let condition = Rect::new(bottom.x, bottom.y, 340.0, bottom.h);
    term_panel(condition, Some("Ship & people"));
    let rows = [
        format!(
            "Hull {:.0}% · Air {:.0}% · Fuel {:.0}%",
            sim.ship.hull_integrity * 100.0,
            sim.ship.life_support * 100.0,
            sim.ship.fuel * 100.0
        ),
        format!(
            "{} people · Generation {}",
            sim.population.count, sim.dynasty.generation
        ),
        format!(
            "{} active projects · {} waiting",
            sim.projects.active_count(),
            sim.projects.waiting_count()
        ),
    ];
    for (i, line) in rows.iter().take(2).enumerate() {
        draw_text_block(
            line,
            condition.x + 16.0,
            condition.y + 44.0 + i as f32 * 26.0,
            condition.w - 32.0,
            24.0,
            16.0,
            4.0,
            term::dim(),
        );
    }
    if term_button(
        Rect::new(
            condition.x + 16.0,
            condition.bottom() - 54.0,
            condition.w - 32.0,
            44.0,
        ),
        "All instruments & maintenance",
        true,
        pointer,
    ) {
        ctx.presentation.instruments.set(true);
    }
    let recent = Rect::new(
        condition.right() + 14.0,
        bottom.y,
        bottom.w - condition.w - 14.0,
        bottom.h,
    );
    term_panel(recent, Some("Recent developments"));
    let mut y = recent.y + 44.0;
    for entry in sim
        .log
        .iter()
        .rev()
        .take(((recent.h - 44.0) / 44.0).floor().max(0.0) as usize)
    {
        let text = format!("Year {} · {}", entry.year, entry.text);
        draw_text_block(
            &text,
            recent.x + 16.0,
            y,
            recent.w - 232.0,
            40.0,
            16.0,
            3.0,
            term::dim(),
        );
        y += 44.0;
    }
    if term_button(
        Rect::new(recent.right() - 204.0, recent.y + 48.0, 188.0, 44.0),
        "Open History",
        true,
        pointer,
    ) {
        actions.push(UiAction::SelectScreen(Screen::Chronicle));
    }
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
                "{}\n{}\nForecast holds current conditions steady.",
                row.evidence, row.trend
            ),
            "Review recovery projects",
            Screen::Agenda,
        )
    } else if let Some(duty) = sim
        .next_timed_obligation()
        .filter(|d| d.due_year.is_some_and(|year| year <= sim.year() + 1))
    {
        (
            "! A promise needs attention".to_owned(),
            duty.title.clone(),
            "Review obligations",
            Screen::Chronicle,
        )
    } else if sim.contract.is_none() {
        (
            "Ready for your next charter".to_owned(),
            "Choose a destination and review its commitments before provisioning the ship."
                .to_owned(),
            "Choose a charter",
            Screen::Drydock,
        )
    } else {
        ("No urgent reserve warning".to_owned(), "The current readiness forecast has no vulnerable or critical concern. Follow the voyage or review your queued work.".to_owned(), "Follow voyage", Screen::Contract)
    };
    draw_text_block(
        &heading,
        rect.x + 20.0,
        rect.y + 52.0,
        rect.w - 40.0,
        60.0,
        24.0,
        4.0,
        if heading.starts_with('!') {
            term::alert()
        } else {
            term::primary()
        },
    );
    draw_text_block(
        &detail,
        rect.x + 20.0,
        rect.y + 120.0,
        rect.w - 40.0,
        112.0,
        18.0,
        6.0,
        term::dim(),
    );
    if term_button(
        Rect::new(rect.x + 20.0, rect.bottom() - 116.0, rect.w - 40.0, 48.0),
        label,
        true,
        pointer,
    ) {
        if target == Screen::Chronicle {
            ctx.presentation.history_page.set(1);
        }
        actions.push(UiAction::SelectScreen(target));
    }
    if term_button(
        Rect::new(rect.x + 20.0, rect.bottom() - 58.0, rect.w - 40.0, 44.0),
        "Open project queue",
        true,
        pointer,
    ) {
        actions.push(UiAction::SelectScreen(Screen::Agenda));
    }
}
