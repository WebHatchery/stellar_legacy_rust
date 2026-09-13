//! Live drydock silhouette from the same model as the underway blueprint.
use super::*;
use crate::ui::ship_schematic::ModuleKind;
use macroquad_toolkit::ui::{draw_text_block, draw_text_centered_in_box_ex};
use std::hash::{Hash, Hasher};

pub(super) fn draw(ctx: &GameplayCtx<'_>, area: Rect) {
    term_panel(area, Some("YOUR SHIP // LIVE REFIT VIEW"));
    let source = Rect::new(0.0, 0.0, 1900.0, 330.0);
    let schematic = ship_schematic::build(ctx.sim, ctx.data, source);
    let mut hash = std::collections::hash_map::DefaultHasher::new();
    schematic.hull_id.hash(&mut hash);
    for module in &schematic.modules {
        module.id.hash(&mut hash);
        module.tier.hash(&mut hash);
    }
    let signature = hash.finish();
    let (previous, changed_at) = ctx.ship_preview.get();
    let changed_at = if previous != signature && previous != 0 {
        get_time()
    } else {
        changed_at
    };
    ctx.ship_preview.set((signature, changed_at));
    let changed = get_time() - changed_at < 1.6;
    let frame = Rect::new(area.x + 12.0, area.y + 36.0, area.w - 354.0, area.h - 44.0);
    let scale = (frame.w / source.w).min(frame.h / source.h);
    let origin = vec2(
        frame.x + (frame.w - source.w * scale) / 2.0,
        frame.y + (frame.h - source.h * scale) / 2.0,
    );
    let tone = if changed {
        term::primary()
    } else {
        term::accent()
    };
    draw_outline(&schematic, origin, scale, tone, changed);
    draw_modules(&schematic, origin, scale, tone);
    draw_summary(ctx, area, &schematic, tone, changed);
}

fn draw_outline(
    schematic: &ship_schematic::ShipSchematic,
    origin: Vec2,
    scale: f32,
    tone: Color,
    changed: bool,
) {
    let point = |p: Vec2| origin + p * scale;
    for i in 0..schematic.outline.len() {
        let a = point(schematic.outline[i]);
        let b = point(schematic.outline[(i + 1) % schematic.outline.len()]);
        draw_line(a.x, a.y, b.x, b.y, if changed { 3.0 } else { 1.5 }, tone);
    }
    if let Some((center, radius)) = schematic.ring {
        let center = point(center);
        draw_circle_lines(center.x, center.y, radius * scale, 1.5, tone);
    }
    let a = point(schematic.corridor.0);
    let b = point(schematic.corridor.1);
    draw_line(a.x, a.y, b.x, b.y, 1.0, term::dim());
}

fn draw_modules(schematic: &ship_schematic::ShipSchematic, origin: Vec2, scale: f32, tone: Color) {
    let point = |p: Vec2| origin + p * scale;
    for module in &schematic.modules {
        let p = point(vec2(module.rect.x, module.rect.y));
        let r = Rect::new(p.x, p.y, module.rect.w * scale, module.rect.h * scale);
        let color = if module.condition < 0.35 {
            term::alert()
        } else {
            tone
        };
        draw_rectangle(r.x, r.y, r.w, r.h, term::surface_active());
        draw_rectangle_lines(r.x, r.y, r.w, r.h, 1.0, color);
        // Codes sit below the compartments so even the smallest fitting is legible.
        let label = if module.kind == ModuleKind::Subsystem {
            format!("{} {}", module.code, module.tier)
        } else {
            module.code.clone()
        };
        draw_text_centered_in_box_ex(
            &label,
            r.x - 18.0,
            r.bottom() + 2.0,
            r.w + 36.0,
            14.0,
            TextStyle::new(10.0, color),
        );
    }
}

fn draw_summary(
    ctx: &GameplayCtx<'_>,
    area: Rect,
    schematic: &ship_schematic::ShipSchematic,
    tone: Color,
    changed: bool,
) {
    let engine = ctx
        .data
        .ship_components
        .find(ComponentKind::Engine, &ctx.sim.ship.engine)
        .map(|c| c.name.as_str())
        .unwrap_or("Unknown drive");
    let weapon = ctx
        .sim
        .ship
        .weapon
        .as_ref()
        .and_then(|id| ctx.data.ship_components.find(ComponentKind::Weapon, id))
        .map(|c| c.name.as_str())
        .unwrap_or("Unarmed");
    let summary = format!(
        "{}\n{}\n{}\n{}\n{}",
        schematic.hull_name,
        engine,
        weapon,
        stats_line(&schematic.stats),
        if changed {
            "REFIT APPLIED"
        } else {
            "Module numbers show installed tiers"
        }
    );
    draw_text_block(
        &summary,
        area.right() - 328.0,
        area.y + 46.0,
        312.0,
        area.h - 52.0,
        13.0,
        5.0,
        tone,
    );
}
