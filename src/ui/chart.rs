//! Chart primitives for the monitor: a gauge with a target tick, phase bands,
//! and a line series. Pure drawing; callers supply the numbers. These replace
//! sentences such as "stored 45 / recommended 360" with a shape the player
//! reads at a glance (docs/UI_TEXT_REVIEW.md).

use crate::ui::term;
use macroquad::prelude::*;
use macroquad_toolkit::prelude::*;
use macroquad_toolkit::ui::draw_ui_text_ex;

/// A horizontal gauge filled to `frac`, with an optional bright tick at
/// `target` (both 0-1 of the track). The tick stands proud of the track so it
/// still reads where the fill covers it.
pub fn gauge(rect: Rect, frac: f32, target: Option<f32>, fill: Color) {
    draw_surface(
        rect,
        &SurfaceStyle::new(term::surface_inset()).with_border(1.0, term::faint()),
    );
    let fill_w = (rect.w - 2.0) * frac.clamp(0.0, 1.0);
    if fill_w > 0.5 {
        draw_rectangle(rect.x + 1.0, rect.y + 1.0, fill_w, rect.h - 2.0, fill);
    }
    if let Some(target) = target {
        let x = rect.x + 1.0 + (rect.w - 2.0) * target.clamp(0.0, 1.0);
        draw_rectangle(x - 1.5, rect.y - 4.0, 3.0, rect.h + 8.0, term::primary());
    }
}

/// One labelled span of a horizontal timeline, sized by its share of the whole.
pub struct Band<'a> {
    pub label: &'a str,
    pub span: f32,
}

/// Alternating shaded columns across `rect`, labelled at the top-left of each.
/// Spans are relative weights, so phase years can be passed straight in.
pub fn bands(rect: Rect, bands: &[Band<'_>]) {
    let total: f32 = bands.iter().map(|band| band.span.max(0.0)).sum();
    if total <= 0.0 {
        return;
    }
    let mut x = rect.x;
    for (index, band) in bands.iter().enumerate() {
        let w = rect.w * band.span.max(0.0) / total;
        if index % 2 == 0 {
            draw_rectangle(x, rect.y, w, rect.h, term::surface_inset());
        }
        if index > 0 {
            draw_line(x, rect.y, x, rect.bottom(), 1.0, term::faint());
        }
        draw_ui_text_ex(
            band.label,
            x + 6.0,
            rect.y + 18.0,
            TextStyle::new(14.0, term::faint()).params(),
        );
        x += w;
    }
    draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 1.0, term::faint());
}

/// Map evenly spaced `values` across `rect` between `lo` (bottom) and `hi`
/// (top). Returns the screen points so callers can mark events on the line.
pub fn line_points(rect: Rect, values: &[f32], lo: f32, hi: f32) -> Vec<Vec2> {
    let span = (hi - lo).max(f32::EPSILON);
    let steps = values.len().saturating_sub(1).max(1) as f32;
    values
        .iter()
        .enumerate()
        .map(|(index, value)| {
            let t = ((value - lo) / span).clamp(0.0, 1.0);
            vec2(
                rect.x + rect.w * index as f32 / steps,
                rect.bottom() - rect.h * t,
            )
        })
        .collect()
}

/// Draw a polyline through `points`.
pub fn polyline(points: &[Vec2], thickness: f32, color: Color) {
    for pair in points.windows(2) {
        draw_line(pair[0].x, pair[0].y, pair[1].x, pair[1].y, thickness, color);
    }
}

/// A small filled diamond, used to pin an event (a dry tank, a milestone) on
/// a line or timeline.
pub fn marker(at: Vec2, size: f32, color: Color) {
    let (x, y, s) = (at.x, at.y, size);
    draw_triangle(vec2(x - s, y), vec2(x, y - s), vec2(x + s, y), color);
    draw_triangle(vec2(x - s, y), vec2(x, y + s), vec2(x + s, y), color);
}
