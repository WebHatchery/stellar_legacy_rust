use super::*;
pub fn draw_menu(ctx: &MenuCtx<'_>, state: &presentation::Presentation) -> Vec<UiAction> {
    let mut f = Form::new();
    let mut actions = Vec::new();
    let (w, h) = size();
    f.heading("STELLAR LEGACY");
    f.text("Custodian of a living generation ship");
    match ctx.menu.phase {
        crate::state::MenuPhase::Main => {
            if let Some(art) = ctx.title_art {
                f.art(art);
            } else {
                f.ship();
            }
            f.action(
                "Continue game",
                ctx.menu.save_exists,
                UiAction::ContinueGame,
            );
            f.action("New game", true, UiAction::GoToNewGame);
            f.action("Display & sound", true, UiAction::OpenSettings);
            f.action("Help", true, UiAction::OpenHelp);
        }
        crate::state::MenuPhase::NewGame => {
            f.heading("Choose the founding legacy");
            for (index, id) in ctx.legacy_ids.iter().enumerate() {
                if let Some(d) = ctx.data.legacies.get(id) {
                    f.action(
                        &format!(
                            "{}{}",
                            if ctx.menu.selected_legacy == index {
                                "Selected · "
                            } else {
                                "Choose · "
                            },
                            d.name
                        ),
                        true,
                        UiAction::SelectLegacy(index),
                    );
                    if ctx.menu.selected_legacy == index {
                        f.text(&d.description);
                        f.text(&d.effects);
                    }
                }
            }
            let required = ctx.data.config.factions.starting_count as usize;
            let selected_count = ctx.menu.selected_factions.len();
            f.heading(&format!(
                "Founding peoples · {selected_count} / {required} selected"
            ));
            f.text(if selected_count >= required {
                "Your founding group is full. Tap Remove on a selected people before choosing another, or scroll to Begin the voyage."
            } else {
                "Read each people's outlook, then tap Choose. These peoples will share the ship across generations."
            });
            for id in GameData::sorted_ids(&ctx.data.factions) {
                let d = ctx.data.factions.get(&id).expect("sorted registry id");
                let selected = ctx.menu.selected_factions.contains(&id);
                f.heading(&format!(
                    "{}{}",
                    d.name,
                    if selected { " · Selected" } else { "" }
                ));
                f.text(&d.description);
                if let Some(system) = ctx.data.subsystems.get(&d.tended_subsystem) {
                    f.text(&format!(
                        "Cares for {}. Neglecting it damages their approval.",
                        system.name
                    ));
                }
                f.action(
                    &format!(
                        "{}{}",
                        if selected { "Remove · " } else { "Choose · " },
                        d.name
                    ),
                    selected || selected_count < required,
                    UiAction::ToggleFaction(id),
                );
            }
            f.action(
                &if selected_count == required {
                    "Begin the voyage".to_owned()
                } else {
                    format!(
                        "Choose {} more to begin",
                        required.saturating_sub(selected_count)
                    )
                },
                selected_count == required,
                UiAction::StartNewGame,
            );
            f.action("Back to main menu", true, UiAction::BackToMainMenu);
        }
    }
    f.draw(
        Rect::new(16.0, 16.0, w - 32.0, h - 44.0),
        state,
        ctx.pointer,
        &format!("menu:{:?}", ctx.menu.phase),
        &mut actions,
    );
    actions
}
