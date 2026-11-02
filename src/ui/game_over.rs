//! Full-screen "vessel lost" takeover. The ship is lost only when it is wrecked,
//! emptied, or stranded beyond repair; a lost captain is never an ending. The
//! screen names the loss, shows the ship as it was left, and draws the
//! voyage's population curve beside the record.

use crate::state::sim::TerminalReason;
use crate::ui::{
    chart, logical_width, spec_line, term, term_button, term_meter, term_panel, GameplayCtx,
    UiAction,
};
use macroquad::prelude::*;
use macroquad_toolkit::prelude::*;
use macroquad_toolkit::ui::{draw_ui_text_ex, RectExt};

const PANEL_TOP: f32 = 236.0;
const PANEL_H: f32 = 540.0;
const LEFT_W: f32 = 640.0;
const RIGHT_W: f32 = 700.0;
const GAP: f32 = 36.0;

pub fn draw(ctx: &GameplayCtx<'_>, pointer: Pointer, actions: &mut Vec<UiAction>) {
    draw_rectangle(
        0.0,
        0.0,
        logical_width(),
        crate::ui::logical_height(),
        term::bg(),
    );
    draw_title(ctx);
    let left_x = (logical_width() - LEFT_W - GAP - RIGHT_W) * 0.5;
    draw_loss(ctx, Rect::new(left_x, PANEL_TOP, LEFT_W, PANEL_H));
    draw_voyage(
        ctx,
        Rect::new(left_x + LEFT_W + GAP, PANEL_TOP, RIGHT_W, PANEL_H),
    );
    draw_closing(pointer, actions);
}

/// The loss in two words, with the vessel's name for it above and the
/// campaign's place in time below.
fn headline(reason: Option<TerminalReason>) -> &'static str {
    match reason {
        Some(TerminalReason::Stranded) => "STRANDED",
        Some(TerminalReason::HullLoss) => "HULL FAILURE",
        Some(TerminalReason::PopulationLoss) => "NO ONE ABOARD",
        Some(TerminalReason::LifeSupportFailure) => "LIFE SUPPORT FAILURE",
        Some(TerminalReason::DynastyExtinction) | None => "VOYAGE ENDED",
    }
}

fn draw_title(ctx: &GameplayCtx<'_>) {
    let sim = ctx.sim;
    let cx = logical_width() * 0.5;
    draw_text_centered("VESSEL LOST", cx, 92.0, TextStyle::new(20.0, term::dim()));
    let title = headline(sim.terminal.as_ref().map(|t| t.reason));
    let style = TextStyle::new(56.0, term::alert());
    let w = measure_text_size(title, style).width;
    draw_text_glow(title, cx - w * 0.5, 156.0, style, 0.14, 3.0);
    let legacy = ctx
        .data
        .legacies
        .get(&sim.legacy.legacy_id)
        .map_or("", |l| l.name.as_str());
    draw_text_centered(
        &format!(
            "{legacy} · Year {} · Generation {}",
            sim.year(),
            sim.dynasty.generation
        ),
        cx,
        198.0,
        TextStyle::new(20.0, term::dim()),
    );
}

/// What happened, then the ship as it was left: hull, air and fuel gauges and
/// the people and parts still aboard.
fn draw_loss(ctx: &GameplayCtx<'_>, rect: Rect) {
    let sim = ctx.sim;
    term_panel(rect, Some("The loss"));
    let content = rect.inset(28.0);
    let evidence = sim
        .terminal
        .as_ref()
        .map_or("The voyage has ended.", |t| t.evidence.as_str());
    draw_text_block(
        evidence,
        content.x,
        content.y + 30.0,
        content.w,
        150.0,
        22.0,
        8.0,
        term::primary(),
    );

    let mut y = content.y + 210.0;
    caption("THE SHIP AS IT WAS LEFT", content.x, y);
    y += 16.0;
    let gauges = [
        ("HULL", sim.ship.hull_integrity),
        ("AIR", sim.ship.life_support),
        ("FUEL", sim.ship.fuel),
    ];
    for (label, value) in gauges {
        term_meter(
            Rect::new(content.x, y, content.w, 28.0),
            value,
            1.0,
            label,
            &format!("{:.0}%", value * 100.0),
        );
        y += 42.0;
    }
    y += 22.0;
    spec_line(
        content.x,
        y,
        content.w,
        "PEOPLE ABOARD",
        &sim.population.count.to_string(),
        term::accent(),
    );
    spec_line(
        content.x,
        y + 32.0,
        content.w,
        "SPARE PARTS",
        &sim.ship.spare_parts.to_string(),
        term::accent(),
    );
}

/// The charter the ship was on, its population year by year, and the
/// campaign's record.
fn draw_voyage(ctx: &GameplayCtx<'_>, rect: Rect) {
    let sim = ctx.sim;
    term_panel(rect, Some("The voyage"));
    let content = rect.inset(28.0);
    let mut y = content.y + 30.0;
    if let Some(contract) = sim.contract.as_ref() {
        draw_ui_text_ex(
            &contract.name,
            content.x,
            y,
            TextStyle::new(22.0, term::primary()).params(),
        );
        let share = format!("{:.0}% of the way", contract.progress() * 100.0);
        draw_text_right(
            &share,
            content.right(),
            y,
            TextStyle::new(18.0, term::dim()),
        );
        let mut track: Vec<f32> = contract
            .population_track
            .iter()
            .map(|v| *v as f32)
            .collect();
        if track.is_empty() {
            track.push(contract.starting_population as f32);
        }
        track.push(sim.population.count as f32);
        let plot = Rect::new(content.x, y + 18.0, content.w, 190.0);
        draw_population(plot, &track, contract.starting_population as f32);
        y = plot.bottom() + 26.0;
        let ends = TextStyle::new(17.0, term::dim());
        draw_ui_text_ex(
            &format!("Launch {}", contract.starting_population),
            content.x,
            y,
            ends.params(),
        );
        draw_text_right(
            &format!("Now {}", sim.population.count),
            content.right(),
            y,
            ends,
        );
        y += 46.0;
    }

    let charters = ctx
        .chronicle
        .entries
        .iter()
        .filter(|e| e.legacy_id == sim.legacy.legacy_id)
        .count();
    let captain = sim
        .dynasty
        .leader()
        .map_or_else(|| "none".to_owned(), |l| l.name.clone());
    let rows = [
        ("YEARS UNDER WAY", sim.year().to_string()),
        ("CAPTAINS", sim.dynasty.reigns.len().max(1).to_string()),
        ("LAST CAPTAIN", captain),
        ("CHARTERS COMPLETED", charters.to_string()),
        ("TRADITION EARNED", sim.legacy.tradition_points.to_string()),
    ];
    for (label, value) in rows {
        spec_line(content.x, y, content.w, label, &value, term::accent());
        y += 32.0;
    }
}

/// Population over the charter against a faint launch line, ending in a dot
/// where the ship was lost.
fn draw_population(plot: Rect, values: &[f32], launch: f32) {
    draw_rectangle(plot.x, plot.y, plot.w, plot.h, term::surface_inset());
    draw_rectangle_lines(plot.x, plot.y, plot.w, plot.h, 1.0, term::faint());
    let hi = values.iter().copied().fold(launch.max(1.0), f32::max) * 1.1;
    let inner = plot.inset(8.0);
    if let Some(start) = chart::line_points(inner, &[launch], 0.0, hi).first() {
        draw_line(inner.x, start.y, inner.right(), start.y, 1.0, term::dim());
    }
    let points = chart::line_points(inner, values, 0.0, hi);
    chart::polyline(&points, 2.5, term::accent());
    if let Some(last) = points.last() {
        draw_circle(last.x, last.y, 6.0, term::alert());
    }
}

fn caption(text: &str, x: f32, y: f32) {
    draw_ui_text_ex(text, x, y, TextStyle::new(16.0, term::dim()).params());
}

/// One closing line, then the way on: the Chronicle, a new campaign, or the
/// menu. No keyboard is needed to leave.
fn draw_closing(pointer: Pointer, actions: &mut Vec<UiAction>) {
    let cx = logical_width() * 0.5;
    let y = PANEL_TOP + PANEL_H + 52.0;
    draw_text_centered(
        "The Custodian is archived with the ship's records. The Chronicle keeps what the generations carried.",
        cx,
        y,
        TextStyle::new(20.0, term::dim()),
    );
    let caret = if blink(get_time() as f32, 2.5) {
        ">"
    } else {
        " "
    };
    let (btn_w, gap) = (300.0, 20.0);
    let x = cx - (btn_w * 3.0 + gap * 2.0) * 0.5;
    let by = y + 34.0;
    if term_button(Rect::new(x, by, btn_w, 52.0), "Chronicle", true, pointer) {
        actions.push(UiAction::SelectScreen(crate::state::Screen::Chronicle));
    }
    if term_button(
        Rect::new(x + btn_w + gap, by, btn_w, 52.0),
        &format!("{caret} New game"),
        true,
        pointer,
    ) {
        actions.push(UiAction::RetireVoyage);
    }
    if term_button(
        Rect::new(x + (btn_w + gap) * 2.0, by, btn_w, 52.0),
        "Menu",
        true,
        pointer,
    ) {
        actions.push(UiAction::ToMenu);
    }
}
