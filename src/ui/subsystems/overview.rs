//! A compartment overview and a selected system's full operating controls.
use super::*;
use crate::ui::ship_schematic;

pub fn draw(ctx: &GameplayCtx<'_>, area: Rect, pointer: Pointer, actions: &mut Vec<UiAction>) {
    let ids = GameData::sorted_ids(&ctx.data.subsystems);
    if ids.is_empty() {
        return;
    }
    let selected = ctx.presentation.selected_system.get().min(ids.len() - 1);
    let rail = Rect::new(area.x, area.y, 292.0, area.h);
    term_panel(rail, Some("Compartments"));
    let base_pointer = if ctx.custody_picker.is_some() {
        pointer.suppressed()
    } else {
        pointer
    };
    for (index, id) in ids.iter().enumerate() {
        let Some(state) = ctx.sim.subsystems.get(id) else {
            continue;
        };
        let Some(definition) = ctx.data.subsystems.get(id) else {
            continue;
        };
        let name = &definition.name;
        let label = format!(
            "{}\n{:.0}% condition · Tier {}",
            name,
            state.condition * 100.0,
            state.tier
        );
        let rect = Rect::new(
            rail.x + 12.0,
            rail.y + 48.0 + index as f32 * 76.0,
            rail.w - 24.0,
            64.0,
        );
        if term_button(rect, &label, true, base_pointer) {
            ctx.presentation.selected_system.set(index);
        }
        if selected == index {
            draw_rectangle(rect.x, rect.y, 4.0, rect.h, term::primary());
        }
        if state.condition < 0.35 {
            draw_circle(rect.right() - 8.0, rect.y + 8.0, 4.0, term::alert());
        }
    }
    let detail = Rect::new(rail.right() + 16.0, area.y, area.w - rail.w - 16.0, 320.0);
    draw_card(ctx, detail, &ids[selected], base_pointer, actions);
    let diagram = Rect::new(
        detail.x + 16.0,
        detail.bottom() + 30.0,
        detail.w - 32.0,
        area.h - detail.h - 52.0,
    );
    let ship = ship_schematic::build(ctx.sim, ctx.data, diagram);
    ship_schematic::draw(diagram, &ship);
    select_compartments(ctx, &ship, base_pointer, actions, false);
    if let Some(glyph) = ship.modules.iter().find(|glyph| glyph.id == ids[selected]) {
        draw_ui_text_ex(
            if glyph.manned {
                "Officer assigned"
            } else {
                "! Officer post vacant"
            },
            rail.x + 16.0,
            rail.bottom() - 42.0,
            TextStyle::new(16.0, term::dim()).params(),
        );
        draw_ui_text_ex(
            "Tap a compartment to inspect",
            rail.x + 16.0,
            rail.bottom() - 18.0,
            TextStyle::new(14.0, term::dim()).params(),
        );
    }
    if let Some(id) = ctx.custody_picker {
        draw_custody_picker(ctx, area, id, pointer, actions);
    }
}

pub(crate) fn select_compartments(
    ctx: &GameplayCtx<'_>,
    ship: &ship_schematic::ShipSchematic,
    pointer: Pointer,
    actions: &mut Vec<UiAction>,
    navigate: bool,
) {
    let ids = GameData::sorted_ids(&ctx.data.subsystems);
    for glyph in &ship.modules {
        let Some(index) = ids.iter().position(|id| id == &glyph.id) else {
            continue;
        };
        let rect = Rect::new(
            glyph.rect.x,
            glyph.rect.center().y - 22.0,
            glyph.rect.w.max(44.0),
            44.0,
        );
        macroquad_toolkit::ui::note_neighbour(rect);
        macroquad_toolkit::ui::note_target(&glyph.label, rect);
        if pointer.released_on(rect) {
            ctx.presentation.selected_system.set(index);
            if navigate {
                actions.push(UiAction::SelectScreen(crate::state::Screen::Subsystems));
            }
        }
        if ctx.sim.projects.jobs.iter().any(|job| {
            job.status == crate::state::sim::ProjectStatus::Running
                && job.target_id.as_deref() == Some(glyph.id.as_str())
        }) {
            draw_ui_text_ex(
                "WORK",
                rect.right() + 6.0,
                rect.center().y + 5.0,
                TextStyle::new(12.0, term::primary()).params(),
            );
        }
        if ctx.presentation.selected_system.get() == index {
            draw_rectangle_lines(
                rect.x - 3.0,
                rect.y - 3.0,
                rect.w + 6.0,
                rect.h + 6.0,
                2.0,
                term::primary(),
            );
        }
    }
}
