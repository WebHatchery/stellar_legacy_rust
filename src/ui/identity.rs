//! Original geometric officer silhouettes and faction seals; no external assets.
use super::*;

fn signature(name: &str) -> u64 {
    name.bytes().fold(14695981039346656037u64, |hash, byte| {
        (hash ^ byte as u64).wrapping_mul(1099511628211)
    })
}

pub fn portrait(rect: Rect, name: &str) {
    let key = signature(name);
    let tone = [
        term::primary(),
        term::accent(),
        Color::new(0.56, 0.72, 0.91, 1.0),
        Color::new(0.81, 0.64, 0.79, 1.0),
    ][key as usize % 4];
    draw_rectangle(rect.x, rect.y, rect.w, rect.h, term::surface_inset());
    let cx = rect.x + rect.w * 0.5;
    let cy = rect.y + rect.h * 0.36;
    let radius = rect.w * (0.16 + (key % 3) as f32 * 0.018);
    draw_circle(cx, cy, radius, tone);
    draw_ellipse(
        cx,
        rect.y + rect.h * 0.80,
        rect.w * 0.31,
        rect.h * 0.20,
        0.0,
        tone,
    );
    if key & 4 != 0 {
        draw_rectangle(
            cx - radius,
            cy - radius,
            radius * 2.0,
            radius * 0.55,
            term::dim(),
        );
    }
    if key & 8 != 0 {
        draw_line(
            cx - radius * 0.8,
            cy,
            cx + radius * 0.8,
            cy,
            2.0,
            term::bg(),
        );
    }
    draw_rectangle(rect.x + 4.0, rect.bottom() - 5.0, rect.w - 8.0, 2.0, tone);
}

pub fn emblem(rect: Rect, id: &str) {
    let key = signature(id);
    let center = rect.center();
    let sides = 3 + (key % 5) as u8;
    draw_poly_lines(
        center.x,
        center.y,
        sides,
        rect.w * 0.42,
        (key % 90) as f32,
        2.0,
        term::primary(),
    );
    draw_circle_lines(center.x, center.y, rect.w * 0.20, 2.0, term::accent());
    for i in 0..(key % 3 + 1) {
        draw_circle(
            rect.x + 12.0 + i as f32 * 9.0,
            rect.bottom() - 7.0,
            2.0,
            term::dim(),
        );
    }
}
