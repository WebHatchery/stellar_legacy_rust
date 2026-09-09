//! Vertical game document: full prose and explicit intents, backed by toolkit scrolling.
use super::*;
use macroquad_toolkit::ui::{wrap_text, ScrollArea};

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
        let sizes: Vec<f32> = self
            .items
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
                Item::Sections(items) => items.len().div_ceil(2) as f32 * 60.0,
                Item::Actions(..) => 60.0,
                Item::Portrait(_) => 100.0,
                Item::Ship | Item::Vessel(_) => 180.0_f32.min(view.h),
                Item::Art(_) => ((width * 9.0 / 16.0).min(260.0) + 12.0).min(view.h),
                Item::CloseUtilities => 60.0,
            })
            .collect();
        let total = sizes.iter().sum();
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
        macroquad_toolkit::ui::VirtualUi::responsive().with_clip(view, || {
            let mut top = view.y - scroll.offset();
            for (item, h) in self.items.into_iter().zip(sizes) {
                let rect = Rect::new(view.x, top, width, h - 12.0);
                top += h;
                match item {
                    Item::Text(text, title) => {
                        let size = if title { 24.0 } else { 18.0 };
                        let stride = if title { 32.0 } else { 26.0 } * text_scale;
                        for (i, line) in wrap_text(&text, width, size).iter().enumerate() {
                            let y = rect.y + i as f32 * stride;
                            if y + stride > view.y && y < view.bottom() {
                                draw_ui_text_ex(
                                    line,
                                    rect.x,
                                    y + size * text_scale,
                                    TextStyle::new(
                                        size,
                                        if title { term::primary() } else { term::dim() },
                                    )
                                    .params(),
                                );
                            }
                        }
                    }
                    _ if !rect.overlaps(&view) => {}
                    Item::Action(label, enabled, action, section) => {
                        if term_button(rect, &label, enabled, tap) {
                            if let Some(section) = section {
                                *state.mobile_section.borrow_mut() = section;
                            }
                            actions.push(action);
                        }
                    }
                    Item::Section(label, section) => {
                        if term_button(rect, &label, true, tap) {
                            *state.mobile_section.borrow_mut() = section;
                        }
                    }
                    Item::CloseUtilities => {
                        if term_button(rect, "Close utilities", true, tap) {
                            state.utilities.set(false);
                        }
                    }
                    Item::Sections(items) => {
                        for (i, (label, section)) in items.into_iter().enumerate() {
                            let r = Rect::new(
                                rect.x + (i % 2) as f32 * (rect.w + 12.0) / 2.0,
                                rect.y + (i / 2) as f32 * 60.0,
                                (rect.w - 12.0) / 2.0,
                                48.0,
                            );
                            if nav_button(r, &label, tap) {
                                *state.mobile_section.borrow_mut() = section;
                            } else if state.mobile_section.borrow().as_str() == section {
                                selection_marker(r);
                            }
                        }
                    }
                    Item::Actions(items, selected) => {
                        let count = items.len() as f32;
                        let width = (rect.w - 12.0 * (count - 1.0)) / count;
                        for (i, (label, action)) in items.into_iter().enumerate() {
                            let r =
                                Rect::new(rect.x + i as f32 * (width + 12.0), rect.y, width, 48.0);
                            if nav_button(r, &label, tap) {
                                actions.push(action);
                            }
                            if i == selected {
                                selection_marker(r);
                            }
                        }
                    }
                    Item::Portrait(name) => {
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
                    Item::Vessel(ship) => draw_vessel(rect, &ship),
                    Item::Art(texture) => {
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
                    Item::Ship => {
                        let x = rect.x;
                        let y = rect.y;
                        let w = rect.w;
                        draw_line(x + 10.0, y + 80.0, x + 65.0, y + 24.0, 2.0, term::faint());
                        draw_line(x + 10.0, y + 80.0, x + 65.0, y + 136.0, 2.0, term::faint());
                        draw_rectangle_lines(
                            x + 65.0,
                            y + 24.0,
                            w - 80.0,
                            112.0,
                            2.0,
                            term::faint(),
                        );
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
                }
            }
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
            draw_ui_text_ex(
                "Drag to read",
                view.x,
                view.bottom() + 14.0,
                TextStyle::new(14.0, term::faint()).params(),
            );
        }
    }
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
