//! Scrollable achievement goals shared across display sizes.
use super::*;
use crate::ui::mobile::form::Form;

pub(super) fn draw(ctx: &GameplayCtx<'_>, area: Rect, pointer: Pointer) {
    term_panel(area, Some("MILESTONES"));
    let mut form = Form::new();
    build(ctx, &mut form);
    let mut view = area.inset(24.0);
    view.y += 24.0;
    view.h -= 24.0;
    form.draw(
        view,
        ctx.presentation,
        pointer,
        "milestones",
        &mut Vec::new(),
    );
}

pub(crate) fn build(ctx: &GameplayCtx<'_>, form: &mut Form) {
    let (unlocked, total) = ctx.achievements.progress();
    form.heading(&format!("Milestones · {unlocked} / {total} reached"));
    form.text("Milestones celebrate your history and stay unlocked across campaigns. They grant no gameplay bonuses. Generation and year progress below describe the current dynasty.");
    for achievement in ctx.achievements.iter() {
        form.heading(&format!(
            "{} · {}",
            if achievement.unlocked {
                "Reached"
            } else {
                "Not reached"
            },
            achievement.name
        ));
        form.text(&achievement.description);
        if achievement.unlocked {
            continue;
        }
        let progress = match achievement.id.as_str() {
            "first_charter" => format!("{} / 1 voyages recorded", ctx.chronicle.entries.len()),
            "flawless" => format!("{} / 1 Complete outcomes", ctx.chronicle.stats().completed),
            "full_registry" => format!("{} / 5 voyages recorded", ctx.chronicle.entries.len()),
            "long_line" => format!(
                "Current dynasty: generation {} / 5",
                ctx.sim.dynasty.generation
            ),
            "against_the_void" => format!("Current dynasty: Year {} / 100", ctx.sim.year()),
            "storied_house" => format!("{} / 250 renown", crate::heritage::renown(ctx.chronicle)),
            _ => continue,
        };
        form.text(&progress);
    }
}
