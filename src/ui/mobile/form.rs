//! Vertical game document: full prose and explicit intents, backed by toolkit scrolling.
use super::*;
use macroquad_toolkit::ui::{wrap_text, ScrollArea};
mod layout;
use layout::GroupLayout;

enum Item<A> {
    Text(String, bool),
    Action(String, bool, A, Option<String>),
    Section(String, String),
    Portrait(String),
    Sections(Vec<(String, String)>),
    Actions(Vec<(String, A)>, usize),
    Ship,
    Art(Texture2D),
    Vessel(ship_schematic::ShipSchematic),
    CloseUtilities,
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
            .push(Item::Action(label.to_owned(), enabled, action, None));
    }
    pub fn action_section(&mut self, label: &str, action: A, section: &str) {
        self.items.push(Item::Action(
            label.to_owned(),
            true,
            action,
            Some(section.to_owned()),
        ));
    }
    pub fn section(&mut self, label: &str, section: &str) {
        self.items
            .push(Item::Section(label.to_owned(), section.to_owned()));
    }
    pub fn sections(&mut self, items: &[(&str, &str)]) {
        self.items.push(Item::Sections(
            items
                .iter()
                .map(|(a, b)| (a.to_string(), b.to_string()))
                .collect(),
        ));
    }
    pub fn actions(&mut self, items: Vec<(&str, A)>, selected: usize) {
        self.items.push(Item::Actions(
            items
                .into_iter()
                .map(|(label, a)| (label.to_owned(), a))
                .collect(),
            selected,
        ));
    }
    pub fn portrait(&mut self, name: &str) {
        self.items.push(Item::Portrait(name.to_owned()));
    }
    pub fn art(&mut self, texture: &Texture2D) {
        self.items.push(Item::Art(texture.clone()));
    }
    pub fn ship(&mut self) {
        self.items.push(Item::Ship);
    }
    pub fn vessel(&mut self, ctx: &GameplayCtx<'_>) {
        self.items.push(Item::Vessel(ship_schematic::build(
            ctx.sim,
            ctx.data,
            Rect::new(0.0, 0.0, size().0 - 40.0, 168.0),
        )));
    }
    pub fn close_utilities(&mut self) {
        self.items.push(Item::CloseUtilities);
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
            &state.mobile_key
        };
        let scroll_cell = if overlay {
            &state.overlay_scroll
        } else {
            &state.mobile_scroll
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
        let sizes = item_sizes(&self.items, width, text_scale, view.h);
        let total = sizes.iter().sum();
        // Keep the reading hint inside its own footer instead of painting it
        // against the last line of the document or outside the owning panel.
        let outer = view;
        let view = layout::reading_view(outer, total, text_scale);
        let mut scroll = scroll_cell.get();
        if overlay || !state.overlay_active.get() {
            scroll.update_at(view, total, pointer.position);
        }
        if let Some((for_overlay, offset)) = state.capture_mobile_offset.get() {
            if for_overlay == overlay {
                state.capture_mobile_offset.set(None);
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
        macroquad_toolkit::ui::VirtualUi::responsive().with_clip(view, || {
            draw_scrolled_items(
                items, sizes, view, width, text_scale, offset, tap, state, actions,
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

fn item_sizes<A>(items: &[Item<A>], width: f32, text_scale: f32, view_h: f32) -> Vec<f32> {
    items
        .iter()
        .map(|item| match item {
            Item::Text(text, title) => {
                wrap_text(text, width, if *title { 24.0 } else { 18.0 }).len() as f32
                    * (if *title { 32.0 } else { 26.0 } * text_scale)
                    + 12.0
            }
            Item::Action(text, ..) | Item::Section(text, _) => {
                (wrap_text(text, width - 24.0, 18.0).len() as f32 * 26.0 * text_scale + 20.0)
                    .max(48.0)
                    + 12.0
            }
            Item::Sections(items) => {
                group_layout(items.iter().map(|(label, _)| label.as_str()), width, 2).height()
            }
            Item::Actions(items, _) => {
                group_layout(items.iter().map(|(label, _)| label.as_str()), width, 3).height()
            }
            Item::Portrait(_) => 100.0,
            Item::Ship | Item::Vessel(_) => 180.0_f32.min(view_h),
            Item::Art(_) => ((width * 9.0 / 16.0).min(260.0) + 12.0).min(view_h),
            Item::CloseUtilities => 60.0,
        })
        .collect()
}

fn draw_scrolled_items<A>(
    items: Vec<Item<A>>,
    sizes: Vec<f32>,
    view: Rect,
    width: f32,
    text_scale: f32,
    offset: f32,
    pointer: Pointer,
    state: &presentation::Presentation,
    actions: &mut Vec<A>,
) {
    let mut top = view.y - offset;
    for (item, height) in items.into_iter().zip(sizes) {
        let rect = Rect::new(view.x, top, width, height - 12.0);
        top += height;
        draw_item(item, rect, view, text_scale, pointer, state, actions);
    }
}

fn draw_item<A>(
    item: Item<A>,
    rect: Rect,
    view: Rect,
    text_scale: f32,
    pointer: Pointer,
    state: &presentation::Presentation,
    actions: &mut Vec<A>,
) {
    if !rect.overlaps(&view) {
        return;
    }
    let interactive = matches!(
        &item,
        Item::Action(..)
            | Item::Section(..)
            | Item::Actions(..)
            | Item::Sections(..)
            | Item::CloseUtilities
    );
    if interactive && (rect.y < view.y || rect.bottom() > view.bottom()) {
        return;
    }
    match item {
        Item::Text(text, title) => draw_text_item(&text, title, rect, view, text_scale),
        Item::Action(label, enabled, action, section) => draw_action_item(
            label, enabled, action, section, rect, pointer, state, actions,
        ),
        Item::Section(label, section) => draw_section_item(label, section, rect, pointer, state),
        Item::CloseUtilities => draw_close_item(rect, pointer, state),
        Item::Sections(items) => draw_sections_item(items, rect, pointer, state),
        Item::Actions(items, selected) => {
            draw_actions_item(items, selected, rect, pointer, actions)
        }
        Item::Portrait(name) => draw_portrait_item(name, rect),
        Item::Vessel(ship) => draw_vessel(rect, &ship),
        Item::Art(texture) => draw_art_item(texture, rect),
        Item::Ship => draw_ship_item(rect),
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

fn draw_action_item<A>(
    label: String,
    enabled: bool,
    action: A,
    section: Option<String>,
    rect: Rect,
    pointer: Pointer,
    state: &presentation::Presentation,
    actions: &mut Vec<A>,
) {
    if term_button(rect, &label, enabled, pointer) {
        if let Some(section) = section {
            *state.mobile_section.borrow_mut() = section;
        }
        actions.push(action);
    }
}

fn draw_section_item(
    label: String,
    section: String,
    rect: Rect,
    pointer: Pointer,
    state: &presentation::Presentation,
) {
    if term_button(rect, &label, true, pointer) {
        *state.mobile_section.borrow_mut() = section;
    }
}

fn draw_close_item(rect: Rect, pointer: Pointer, state: &presentation::Presentation) {
    if term_button(rect, "Close utilities", true, pointer) {
        state.utilities.set(false);
    }
}

fn draw_sections_item(
    items: Vec<(String, String)>,
    rect: Rect,
    pointer: Pointer,
    state: &presentation::Presentation,
) {
    let grid = group_layout(items.iter().map(|(label, _)| label.as_str()), rect.w, 2);
    for (index, (label, section)) in items.into_iter().enumerate() {
        let cell = grid.cell(rect, index);
        if nav_button(cell, &label, pointer) {
            *state.mobile_section.borrow_mut() = section;
        } else if state.mobile_section.borrow().as_str() == section {
            selection_marker(cell);
        }
    }
}

fn draw_actions_item<A>(
    items: Vec<(String, A)>,
    selected: usize,
    rect: Rect,
    pointer: Pointer,
    actions: &mut Vec<A>,
) {
    let grid = group_layout(items.iter().map(|(label, _)| label.as_str()), rect.w, 3);
    for (index, (label, action)) in items.into_iter().enumerate() {
        let cell = grid.cell(rect, index);
        if nav_button(cell, &label, pointer) {
            actions.push(action);
        }
        if index == selected {
            selection_marker(cell);
        }
    }
}

fn draw_portrait_item(name: String, rect: Rect) {
    identity::portrait(Rect::new(rect.x, rect.y, 68.0, 80.0), &name);
    draw_text_block(
        &name,
        rect.x + 84.0,
        rect.y + 12.0,
        rect.w - 84.0,
        70.0,
        22.0,
        6.0,
        term::primary(),
    );
}

fn draw_art_item(texture: Texture2D, rect: Rect) {
    let crop_h = (texture.width() * rect.h / rect.w).min(texture.height());
    let source = Rect::new(
        0.0,
        (texture.height() - crop_h) * 0.5,
        texture.width(),
        crop_h,
    );
    draw_texture_ex(
        &texture,
        rect.x,
        rect.y,
        WHITE,
        DrawTextureParams {
            source: Some(source),
            dest_size: Some(vec2(rect.w, rect.h)),
            ..Default::default()
        },
    );
}

fn draw_ship_item(rect: Rect) {
    let x = rect.x;
    let y = rect.y;
    let w = rect.w;
    draw_line(x + 10.0, y + 80.0, x + 65.0, y + 24.0, 2.0, term::faint());
    draw_line(x + 10.0, y + 80.0, x + 65.0, y + 136.0, 2.0, term::faint());
    draw_rectangle_lines(x + 65.0, y + 24.0, w - 80.0, 112.0, 2.0, term::faint());
    for i in 0..3 {
        for j in 0..2 {
            draw_rectangle_lines(
                x + 82.0 + i as f32 * (w - 115.0) / 3.0,
                y + 42.0 + j as f32 * 48.0,
                (w - 130.0) / 3.0,
                30.0,
                2.0,
                term::accent(),
            );
        }
    }
}

fn group_layout<'a>(
    labels: impl Iterator<Item = &'a str>,
    width: f32,
    max_columns: usize,
) -> GroupLayout {
    let labels: Vec<_> = labels.collect();
    let widest = labels
        .iter()
        .map(|label| measure_text_size(label, TextStyle::new(16.0, term::primary())).width)
        .fold(0.0, f32::max);
    let mut grid = GroupLayout::new(width, labels.len(), max_columns, widest);
    let lines = labels
        .iter()
        .map(|label| wrap_text(label, grid.cell_width - 24.0, 16.0).len())
        .max()
        .unwrap_or(1);
    grid.row_height =
        (lines as f32 * 26.0 * macroquad_toolkit::ui::ui_text_scale() + 20.0).max(48.0);
    grid
}

fn draw_vessel(rect: Rect, ship: &ship_schematic::ShipSchematic) {
    let mut bounds = Rect::new(f32::MAX, f32::MAX, 0.0, 0.0);
    let mut far = vec2(f32::MIN, f32::MIN);
    for p in &ship.outline {
        bounds.x = bounds.x.min(p.x);
        bounds.y = bounds.y.min(p.y);
        far.x = far.x.max(p.x);
        far.y = far.y.max(p.y);
    }
    if let Some((c, r)) = ship.ring {
        bounds.x = bounds.x.min(c.x - r);
        bounds.y = bounds.y.min(c.y - r);
        far.x = far.x.max(c.x + r);
        far.y = far.y.max(c.y + r);
    }
    bounds.w = far.x - bounds.x;
    bounds.h = far.y - bounds.y;
    let scale = ((rect.w - 16.0) / bounds.w).min((rect.h - 16.0) / bounds.h);
    let offset = vec2(
        rect.x + (rect.w - bounds.w * scale) * 0.5 - bounds.x * scale,
        rect.y + (rect.h - bounds.h * scale) * 0.5 - bounds.y * scale,
    );
    let point = |p: Vec2| p * scale + offset;
    for i in 0..ship.outline.len() {
        let a = point(ship.outline[i]);
        let b = point(ship.outline[(i + 1) % ship.outline.len()]);
        draw_line(a.x, a.y, b.x, b.y, 2.0, term::faint());
    }
    if let Some((c, r)) = ship.ring {
        let c = point(c);
        draw_circle_lines(c.x, c.y, r * scale, 2.0, term::dim());
    }
    let a = point(ship.corridor.0);
    let b = point(ship.corridor.1);
    draw_line(a.x, a.y, b.x, b.y, 1.0, term::dim());
    for module in &ship.modules {
        let r = module.rect;
        let p = point(vec2(r.x, r.y));
        let tone = if module.condition < 0.35 {
            term::alert()
        } else {
            term::accent()
        };
        draw_rectangle_lines(p.x, p.y, r.w * scale, r.h * scale, 2.0, tone);
        if module.manned {
            draw_circle(p.x + r.w * scale - 4.0, p.y + 4.0, 2.0, tone);
        }
    }
}
