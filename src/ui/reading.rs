//! Game reading surfaces composed from toolkit wrapping and scrolling.
use super::*;
use macroquad_toolkit::ui::{wrap_text, ScrollArea};
/// Uses the toolkit's wrapping and scroll ownership. Individual lines are culled
/// at the viewport; no font shrinking or fixed-height paragraph truncation.
pub fn read_lines(
    text: &str,
    view: Rect,
    cell: &std::cell::Cell<ScrollArea>,
    pointer: Pointer,
    color: Color,
) -> bool {
    read_lines_sized(text, view, cell, pointer, color, 18.0)
}

/// [`read_lines`] at a chosen type size, for surfaces whose prose is the
/// point of the screen and earns a larger reading measure.
pub fn read_lines_sized(
    text: &str,
    view: Rect,
    cell: &std::cell::Cell<ScrollArea>,
    pointer: Pointer,
    color: Color,
    size: f32,
) -> bool {
    let stride = (size * 1.45).ceil();
    let lines = wrap_text(text, view.w - 20.0, size);
    let height = lines.len() as f32 * stride;
    let mut scroll = cell.get();
    scroll.update_at(view, height, pointer.position);
    for (index, line) in lines.iter().enumerate() {
        let y = view.y + index as f32 * stride - scroll.offset();
        if y >= view.y && y + stride <= view.bottom() {
            draw_ui_text_ex(
                line,
                view.x,
                y + (size * 1.05).round(),
                TextStyle::new(size, color).params(),
            );
        }
    }
    scroll.draw_scrollbar_with(
        view,
        height,
        term::surface_inset(),
        term::dim(),
        term::primary(),
    );
    let read_to_end = height <= view.h + scroll.offset() + 1.0;
    cell.set(scroll);
    read_to_end
}

/// Draw a wrapped paragraph from its top edge and return the y just below it,
/// so stacked sections flow from their measured height instead of fixed rows.
pub fn flow_text(text: &str, x: f32, top: f32, width: f32, size: f32, color: Color) -> f32 {
    let drawn = size.max(MIN_FONT_SIZE) * macroquad_toolkit::ui::ui_text_scale();
    let stride = (drawn * 1.4).ceil();
    let lines = wrap_text(text, width, size);
    for (index, line) in lines.iter().enumerate() {
        let baseline = top + index as f32 * stride + drawn;
        draw_ui_text_ex(line, x, baseline, TextStyle::new(size, color).params());
    }
    top + lines.len() as f32 * stride
}
