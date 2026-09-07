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
            f.heading(&format!(
                "Choose {} founding peoples",
                ctx.data.config.factions.starting_count
            ));
            for id in GameData::sorted_ids(&ctx.data.factions) {
                let d = ctx.data.factions.get(&id).expect("sorted registry id");
                f.action(
                    &format!(
                        "{}{}",
                        if ctx.menu.selected_factions.contains(&id) {
                            "Selected · "
                        } else {
                            "Choose · "
                        },
                        d.name
                    ),
                    true,
                    UiAction::ToggleFaction(id),
                );
            }
            f.action(
                "Begin the voyage",
                ctx.menu.selected_factions.len()
                    == ctx.data.config.factions.starting_count as usize,
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
