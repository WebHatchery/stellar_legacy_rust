//! Vertical game document: full prose and explicit intents, backed by toolkit scrolling.
use super::*;
use macroquad_toolkit::ui::{wrap_text, ScrollArea};

enum Item<A> {
    Text(String, bool),
    Action(String, bool, A),
}
pub(crate) struct Form<A = UiAction> {
    items: Vec<Item<A>>,
}
impl<A> Form<A> {
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }
    pub fn text(&mut self, text: &str) {
        self.items.push(Item::Text(text.to_owned(), false));
    }
    pub fn heading(&mut self, text: &str) {
        self.items.push(Item::Text(text.to_owned(), true));
    }
    pub fn action(&mut self, label: &str, enabled: bool, action: A) {
        self.items
            .push(Item::Action(label.to_owned(), enabled, action));
    }
    pub fn draw(
        self,
        view: Rect,
        state: &presentation::Presentation,
        pointer: Pointer,
        key: &str,
        actions: &mut Vec<A>,
    ) {
        let overlay = matches!(key, "settings" | "help" | "welcome");
        let key_cell = if overlay {
            &state.overlay_key
        } else {
            &state.form_key
        };
        let scroll_cell = if overlay {
            &state.overlay_scroll
        } else {
            &state.form_scroll
        };
        self.draw_scrolled(view, state, pointer, key, actions, scroll_cell, key_cell);
    }

    #[allow(clippy::too_many_arguments)]
    pub fn draw_scrolled(
        self,
        view: Rect,
        state: &presentation::Presentation,
        pointer: Pointer,
        key: &str,
        actions: &mut Vec<A>,
        scroll_cell: &std::cell::Cell<ScrollArea>,
        key_cell: &std::cell::RefCell<String>,
    ) {
        let overlay = matches!(key, "settings" | "help" | "welcome");
        if key_cell.borrow().as_str() != key {
            *key_cell.borrow_mut() = key.to_owned();
            scroll_cell.set(ScrollArea::new());
        }
        let width = view.w - 16.0;
        let text_scale = macroquad_toolkit::ui::ui_text_scale();
        let sizes = item_sizes(&self.items, width, text_scale);
        let total = sizes.iter().sum();
        // Keep the reading hint inside its own footer instead of painting it
        // against the last line of the document or outside the owning panel.
        let outer = view;
        let view = reading_view(outer, total, text_scale);
        let mut scroll = scroll_cell.get();
        if overlay || !state.overlay_active.get() {
            scroll.update_at(view, total, pointer.position);
        }
        if let Some((for_overlay, offset)) = state.capture_form_offset.get() {
            if for_overlay == overlay {
                state.capture_form_offset.set(None);
                scroll.set_offset(offset);
            }
        }
        let tap = if scroll.absorbs_press() || !view.contains(pointer.position) {
            pointer.suppressed()
        } else {
            pointer
        };
        let items = self.items;
        let offset = scroll.offset();
        canvas().with_clip(view, || {
            draw_scrolled_items(
                items,
                sizes,
                ScrolledLayout {
                    view,
                    width,
                    text_scale,
                    offset,
                },
                tap,
                actions,
            );
        });
        scroll.draw_scrollbar_with(
            view,
            total,
            term::surface_inset(),
            term::dim(),
            term::primary(),
        );
        scroll_cell.set(scroll);
        if total > view.h {
            draw_text_centered_in_box_ex(
                if view.w < 240.0 && scroll.offset() + view.h >= total - 1.0 {
                    "End · Drag back"
                } else if view.w < 240.0 {
                    "Drag for more"
                } else if scroll.offset() + view.h >= total - 1.0 {
                    "End of section · Drag to go back"
                } else {
                    "More below · Drag to read"
                },
                view.x,
                view.bottom() + 4.0,
                view.w,
                outer.bottom() - view.bottom() - 4.0,
                TextStyle::new(14.0, term::dim()),
            );
        }
    }
}

fn item_sizes<A>(items: &[Item<A>], width: f32, text_scale: f32) -> Vec<f32> {
    items
        .iter()
        .map(|item| match item {
            Item::Text(text, title) => {
                wrap_text(text, width, if *title { 24.0 } else { 18.0 }).len() as f32
                    * (if *title { 32.0 } else { 26.0 } * text_scale)
                    + 12.0
            }
            Item::Action(text, ..) => {
                (wrap_text(text, width - 24.0, 18.0).len() as f32 * 26.0 * text_scale + 20.0)
                    .max(48.0)
                    + 12.0
            }
        })
        .collect()
}

struct ScrolledLayout {
    view: Rect,
    width: f32,
    text_scale: f32,
    offset: f32,
}

fn draw_scrolled_items<A>(
    items: Vec<Item<A>>,
    sizes: Vec<f32>,
    layout: ScrolledLayout,
    pointer: Pointer,
    actions: &mut Vec<A>,
) {
    let ScrolledLayout {
        view,
        width,
        text_scale,
        offset,
    } = layout;
    let mut top = view.y - offset;
    for (item, height) in items.into_iter().zip(sizes) {
        let rect = Rect::new(view.x, top, width, height - 12.0);
        top += height;
        if !rect.overlaps(&view) {
            continue;
        }
        match item {
            Item::Text(text, title) => draw_text_item(&text, title, rect, view, text_scale),
            Item::Action(label, enabled, action) => {
                // A partly scrolled-out button is not pressable.
                if rect.y >= view.y
                    && rect.bottom() <= view.bottom()
                    && term_button(rect, &label, enabled, pointer)
                {
                    actions.push(action);
                }
            }
        }
    }
}

fn draw_text_item(text: &str, title: bool, rect: Rect, view: Rect, text_scale: f32) {
    let size = if title { 24.0 } else { 18.0 };
    let stride = if title { 32.0 } else { 26.0 } * text_scale;
    for (index, line) in wrap_text(text, rect.w, size).iter().enumerate() {
        let y = rect.y + index as f32 * stride;
        if y >= view.y && y + stride <= view.bottom() {
            draw_ui_text_ex(
                line,
                rect.x,
                y + size * text_scale,
                TextStyle::new(size, if title { term::primary() } else { term::dim() }).params(),
            );
        }
    }
}

/// Reserve a footer for the reading hint only when the document overflows.
fn reading_view(area: Rect, content_height: f32, text_scale: f32) -> Rect {
    if content_height <= area.h {
        area
    } else {
        Rect::new(
            area.x,
            area.y,
            area.w,
            (area.h - 28.0 * text_scale).max(1.0),
        )
    }
}
