//! The all-clear watch board: when nothing needs the Custodian, the attention
//! panel still shows what is coming (the next timed commitment) and how each
//! readiness concern stands, as lamps rather than sentences.

use super::*;
use crate::simulation::readiness::ReadinessRow;

const ROW_H: f32 = 34.0;
const LAMP_R: f32 = 6.0;

/// Draw the board from `top` down inside `rect`, stopping above `bottom`.
pub(super) fn draw(ctx: &GameplayCtx<'_>, rect: Rect, top: f32, bottom: f32) {
    let x = rect.x + 24.0;
    let w = rect.w - 48.0;
    let mut y = top;
    if let Some((title, years)) = next_commitment(ctx) {
        y = section_caption("Next commitment", x, y);
        lamp(x + 10.0, y + ROW_H * 0.5, LampState::Lit(term::dim()));
        draw_text_block(
            title,
            x + 28.0,
            y + 4.0,
            w - 150.0,
            ROW_H,
            18.0,
            2.0,
            term::primary(),
        );
        draw_text_right(
            &format!("in {years} y"),
            x + w,
            y + ROW_H * 0.5 + 7.0,
            TextStyle::new(18.0, term::dim()),
        );
        y += ROW_H + 18.0;
    }
    y = section_caption("Readiness", x, y);
    let forecast = readiness::forecast(ctx.sim, ctx.data);
    let mut rows: Vec<&ReadinessRow> = forecast.rows.iter().collect();
    rows.sort_by(|a, b| a.score.total_cmp(&b.score));
    for row in rows {
        if y + ROW_H > bottom {
            break;
        }
        readiness_row(row, x, y, w);
        y += ROW_H;
    }
}

/// The next timed duty that is not already raised as due by the alert above.
fn next_commitment<'a>(ctx: &'a GameplayCtx<'_>) -> Option<(&'a str, u32)> {
    let sim = ctx.sim;
    let now = sim.year();
    let duty = sim.next_timed_obligation()?;
    let due = duty.due_year?;
    (due > now + 1).then(|| (duty.title.as_str(), due - now))
}

fn section_caption(text: &str, x: f32, y: f32) -> f32 {
    draw_ui_text_ex(
        text,
        x,
        y + 16.0,
        TextStyle::new(16.0, term::faint()).params(),
    );
    y + 26.0
}

fn readiness_row(row: &ReadinessRow, x: f32, y: f32, w: f32) {
    let (state, fill) = match row.band {
        ReadinessBand::Strong => (LampState::Lit(term::accent()), term::accent()),
        ReadinessBand::Stable => (LampState::Ring(term::accent()), term::accent()),
        ReadinessBand::Vulnerable => (LampState::Lit(term::dim()), term::dim()),
        ReadinessBand::Critical => (LampState::Lit(term::alert()), term::alert()),
    };
    let cy = y + ROW_H * 0.5;
    lamp(x + 10.0, cy, state);
    draw_ui_text_ex(
        &row.concern,
        x + 28.0,
        cy + 6.0,
        TextStyle::new(17.0, term::primary()).params(),
    );
    // A slim unlabeled bar: the lamp gives the band, the bar the margin.
    let bar_w = (w * 0.32).min(180.0);
    let bar = Rect::new(x + w - bar_w, cy - 5.0, bar_w, 10.0);
    draw_surface(
        bar,
        &SurfaceStyle::new(term::surface_inset()).with_border(1.0, term::faint()),
    );
    let fill_w = (bar.w - 2.0) * row.score.clamp(0.0, 1.0);
    if fill_w > 0.5 {
        draw_rectangle(bar.x + 1.0, bar.y + 1.0, fill_w, bar.h - 2.0, fill);
    }
}

#[derive(Clone, Copy)]
enum LampState {
    Lit(Color),
    Ring(Color),
}

fn lamp(cx: f32, cy: f32, state: LampState) {
    match state {
        LampState::Lit(c) => draw_circle(cx, cy, LAMP_R, c),
        LampState::Ring(c) => draw_circle_lines(cx, cy, LAMP_R, 1.5, c),
    }
}
