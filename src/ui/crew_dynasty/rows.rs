//! Person rows as instruments: a lifespan bar with the heir window shaded,
//! leadership and skill bars, and a ghost segment for what training would
//! add. Replaces "— 45 · Sanitation · LD 43 [LEADER] / trait:" and "Age 32 ·
//! 37 years left · Skill 50" (docs/UI_TEXT_REVIEW.md, item 5).

use crate::ui::{chart, term};
use macroquad::prelude::*;
use macroquad_toolkit::prelude::*;
use macroquad_toolkit::ui::draw_ui_text_ex;

/// Column offsets shared by the header and every row of a table.
pub(super) struct Columns {
    pub name: f32,
    pub first_bar: f32,
    pub second_bar: f32,
    pub tail: f32,
    pub bar_w: f32,
}

impl Columns {
    pub fn for_width(x: f32, width: f32) -> Self {
        let bar_w = (width * 0.2).clamp(220.0, 400.0);
        Self {
            name: x,
            first_bar: x + 360.0,
            second_bar: x + 360.0 + bar_w + 40.0,
            tail: x + 360.0 + 2.0 * (bar_w + 40.0),
            bar_w,
        }
    }
}

pub(super) fn header(columns: &Columns, y: f32, titles: [&str; 4]) {
    let style = TextStyle::new(16.0, term::faint());
    for (x, title) in [
        columns.name,
        columns.first_bar,
        columns.second_bar,
        columns.tail,
    ]
    .into_iter()
    .zip(titles)
    {
        draw_ui_text_ex(title, x, y, style.params());
    }
}

/// A slim bar with a caption beneath it. `window` shades a span of the track
/// (the heir age window), `ghost` extends the fill faintly (a training gain),
/// and `cap` ticks a ceiling short of the track's end (a post's skill cap).
pub(super) struct Meter<'a> {
    pub frac: f32,
    pub color: Color,
    pub caption: &'a str,
    pub window: Option<(f32, f32)>,
    pub ghost: Option<f32>,
    pub cap: Option<f32>,
}

pub(super) fn meter(x: f32, y: f32, w: f32, m: &Meter<'_>) {
    let bar = Rect::new(x, y + 8.0, w, 16.0);
    chart::gauge(bar, 0.0, None, m.color);
    let inner = |frac: f32| bar.x + 1.0 + (bar.w - 2.0) * frac.clamp(0.0, 1.0);
    if let Some((from, to)) = m.window {
        let shade = term::primary();
        draw_rectangle(
            inner(from),
            bar.y + 1.0,
            inner(to) - inner(from),
            bar.h - 2.0,
            Color::new(shade.r, shade.g, shade.b, 0.22),
        );
    }
    if let Some(ghost) = m.ghost {
        let tone = m.color;
        draw_rectangle(
            inner(m.frac),
            bar.y + 1.0,
            inner(ghost) - inner(m.frac),
            bar.h - 2.0,
            Color::new(tone.r, tone.g, tone.b, 0.35),
        );
    }
    let fill = inner(m.frac) - inner(0.0);
    if fill > 0.5 {
        draw_rectangle(inner(0.0), bar.y + 1.0, fill, bar.h - 2.0, m.color);
    }
    if let Some(cap) = m.cap.filter(|cap| *cap < 0.999) {
        draw_rectangle(inner(cap) - 1.0, bar.y - 3.0, 2.0, bar.h + 6.0, term::dim());
    }
    draw_ui_text_ex(
        m.caption,
        x,
        y + 44.0,
        TextStyle::new(16.0, term::dim()).params(),
    );
}

/// An empty officer seat: a faint outline where the portrait would be and a
/// red lamp at its corner.
pub(super) fn vacant_seat(rect: Rect) {
    draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 1.0, term::faint());
    let head = vec2(rect.center().x, rect.y + rect.h * 0.32);
    draw_circle_lines(head.x, head.y, rect.w * 0.18, 1.0, term::faint());
    draw_circle(rect.right() - 6.0, rect.y + 6.0, 5.0, term::alert());
}
