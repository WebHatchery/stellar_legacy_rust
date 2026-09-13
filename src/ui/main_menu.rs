//! The main menu and the new-campaign screen: the only UI the player sees
//! before a ship exists.

use super::*;

/// Left edge of the menu column over the title art, aligned under the wordmark
/// and clear of the ship's engine glow to its right.
const TITLE_COLUMN_X: f32 = 98.0;

pub struct MenuCtx<'a> {
    pub presentation: &'a presentation::Presentation,
    pub data: &'a GameData,
    pub menu: &'a MenuState,
    pub legacy_ids: &'a [String],
    pub chronicle: &'a ChronicleStore,
    /// This frame's pointer, in logical coordinates — a mouse or a finger,
    /// asked the same way. Built once in `game.rs` so every control on the
    /// screen agrees about where it is and whether it just let go.
    pub pointer: Pointer,
    /// The title plate drawn behind the main menu, when it loaded. `None` falls
    /// back to the drawn wordmark, so a missing asset costs art, never a menu.
    pub title_art: Option<&'a Texture2D>,
}

pub fn draw_menu(ctx: MenuCtx<'_>) -> Vec<UiAction> {
    if mobile::active() {
        return mobile::draw_menu(&ctx, ctx.presentation);
    }
    match ctx.menu.phase {
        crate::state::MenuPhase::Main => draw_main_menu(&ctx),
        crate::state::MenuPhase::NewGame if compact() => mobile::draw_menu(&ctx, ctx.presentation),
        crate::state::MenuPhase::NewGame => draw_new_game(&ctx),
    }
}

/// The title / main-menu screen (real-time loop follow-up): the title plate over
/// four options — CONTINUE, NEW GAME, SETTINGS, EXIT. The new-game picker is one
/// step in from here (NEW GAME). This is the screen *after* the boot log, which
/// still plays its own power-on sequence untouched.
///
/// With the title art loaded the plate fills the frame and the options sit in a
/// column under its wordmark, clear of the ship on the right. Without it the
/// menu falls back to the drawn wordmark and a centered column, so a missing or
/// failed texture costs the art and nothing else.
fn draw_main_menu(ctx: &MenuCtx<'_>) -> Vec<UiAction> {
    // A dynasty inheriting a storied Chronicle begins with a head start (§7).
    let heritage = crate::heritage::derive(ctx.chronicle, &ctx.data.config.heritage);

    let bw = if ctx.title_art.is_some() {
        260.0
    } else {
        300.0
    };
    let (bx, by) = draw_menu_backdrop(ctx, &heritage, bw);
    draw_menu_options(ctx, bx, by, bw)
}

fn draw_menu_backdrop(
    ctx: &MenuCtx<'_>,
    heritage: &crate::heritage::Heritage,
    bw: f32,
) -> (f32, f32) {
    let (bx, by) = match ctx.title_art {
        Some(art) => {
            draw_texture_ex(
                art,
                0.0,
                0.0,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(vec2(logical_width(), logical_height())),
                    ..Default::default()
                },
            );
            // Left column, aligned under the plate's wordmark. The heritage note
            // wraps to two short lines so it stays inside the column instead of
            // running across the ship.
            if heritage.has_bonus() {
                draw_ui_text_ex(
                    &format!(
                        "HERITAGE: {} · renown {}",
                        heritage.tier_name, heritage.renown
                    ),
                    TITLE_COLUMN_X,
                    logical_height() - 320.0,
                    TextStyle::new(14.0, term::accent()).params(),
                );
                draw_ui_text_ex(
                    &format!(
                        "+{} cr / +{} inf / +{} tradition",
                        heritage.credits, heritage.influence, heritage.tradition
                    ),
                    TITLE_COLUMN_X,
                    logical_height() - 301.0,
                    TextStyle::new(14.0, term::accent()).params(),
                );
            }
            (
                TITLE_COLUMN_X.min(logical_width() * 0.08),
                logical_height() - 276.0,
            )
        }
        None => {
            draw_text_glow(
                "STELLAR LEGACY",
                logical_width() / 2.0 - 230.0,
                250.0,
                TextStyle::new(58.0, term::primary()),
                0.1,
                3.0,
            );
            draw_text_centered(
                "// CUSTODIAN // GENERATIONAL STARSHIP INTELLIGENCE //",
                logical_width() / 2.0,
                295.0,
                TextStyle::new(18.0, term::dim()),
            );
            if heritage.has_bonus() {
                draw_text_centered(
                    &format!(
                        "HERITAGE: {} · renown {} · +{} cr / +{} inf / +{} tradition",
                        heritage.tier_name,
                        heritage.renown,
                        heritage.credits,
                        heritage.influence,
                        heritage.tradition
                    ),
                    logical_width() / 2.0,
                    325.0,
                    TextStyle::new(14.0, term::accent()),
                );
            }
            (logical_width() / 2.0 - bw / 2.0, logical_height() - 250.0)
        }
    };

    if crate::data::contracts::is_demo_build() {
        draw_ui_text_ex(
            "ITCH.IO DEMO · ONE 150-YEAR TUTORIAL VOYAGE",
            bx,
            by - 18.0,
            TextStyle::new(12.0, term::accent()).params(),
        );
    }

    (bx, by)
}

fn draw_menu_options(ctx: &MenuCtx<'_>, bx: f32, mut by: f32, bw: f32) -> Vec<UiAction> {
    let mut actions = Vec::new();
    let pointer = ctx.pointer;
    let bh = 46.0;
    let gap = 12.0;
    // A single column of options, most-common action first.
    if term_button(
        Rect::new(bx, by, bw, bh),
        "CONTINUE",
        ctx.menu.save_exists,
        pointer,
    ) {
        actions.push(UiAction::ContinueGame);
    }
    by += bh + gap;
    if term_button(Rect::new(bx, by, bw, bh), "NEW GAME", true, pointer) {
        actions.push(UiAction::GoToNewGame);
    }
    by += bh + gap;
    if term_button(Rect::new(bx, by, bw, bh), "SETTINGS", true, pointer) {
        actions.push(UiAction::OpenSettings);
    }
    by += bh + gap;
    if term_button(Rect::new(bx, by, bw, bh), "EXIT GAME", true, pointer) {
        actions.push(UiAction::ExitGame);
    }

    actions
}

/// The new-game screen (W7): legacy + founding-faction selection, reached from
/// the main menu via NEW GAME. BEGIN VOYAGE launches; BACK returns to the menu.
fn draw_new_game(ctx: &MenuCtx<'_>) -> Vec<UiAction> {
    let mut actions = Vec::new();
    let pointer = ctx.pointer;

    draw_text_glow(
        "STELLAR LEGACY",
        logical_width() / 2.0 - 190.0,
        130.0,
        TextStyle::new(48.0, term::primary()),
        0.1,
        3.0,
    );
    draw_ui_text_ex(
        "// generational starship command //",
        logical_width() / 2.0 - 165.0,
        165.0,
        TextStyle::new(17.0, term::dim()).params(),
    );

    // A dynasty inheriting a storied Chronicle begins with a head start (§7).
    let heritage = crate::heritage::derive(ctx.chronicle, &ctx.data.config.heritage);
    if heritage.has_bonus() {
        draw_text_centered(
            &format!(
                "HERITAGE: {} · renown {} · +{} cr / +{} inf / +{} tradition",
                heritage.tier_name,
                heritage.renown,
                heritage.credits,
                heritage.influence,
                heritage.tradition
            ),
            logical_width() / 2.0,
            193.0,
            TextStyle::new(14.0, term::accent()),
        );
    }

    let panel = Rect::new(logical_width() / 2.0 - 430.0, 198.0, 860.0, 508.0);
    term_panel(panel, Some("FOUNDING CHARTER"));
    let content = panel.inset(24.0);
    let col_gap = 24.0;
    let col_w = (content.w - col_gap) / 2.0;
    let left_x = content.x;
    let right_x = content.x + col_w + col_gap;
    let list_top = content.y + 106.0;
    draw_legacy_column(ctx, content, left_x, col_w, list_top, pointer, &mut actions);
    draw_faction_column(
        ctx,
        content,
        right_x,
        col_w,
        list_top,
        pointer,
        &mut actions,
    );
    draw_new_game_buttons(ctx, content, pointer, &mut actions);

    actions
}

fn draw_legacy_column(
    ctx: &MenuCtx<'_>,
    content: Rect,
    left_x: f32,
    col_w: f32,
    list_top: f32,
    pointer: Pointer,
    actions: &mut Vec<UiAction>,
) {
    let tut = &ctx.data.config.tutorial;
    draw_text_block(
        &tut.legacy_intro,
        left_x,
        content.y + 28.0,
        col_w,
        58.0,
        14.0,
        3.0,
        term::dim(),
    );
    let mut y = list_top;
    for (i, legacy_id) in ctx.legacy_ids.iter().enumerate() {
        let Some(legacy) = ctx.data.legacies.get(legacy_id) else {
            continue;
        };
        let rect = Rect::new(left_x, y + 8.0, col_w, 62.0);
        let selected = i == ctx.menu.selected_legacy;
        draw_surface(
            rect,
            &SurfaceStyle::new(if selected {
                term::surface_active()
            } else {
                term::surface_inset()
            })
            .with_border(
                1.0,
                if selected {
                    term::primary()
                } else {
                    term::faint()
                },
            ),
        );
        draw_ui_text_ex(
            &format!("{} {}", i + 1, legacy.name),
            rect.x + 14.0,
            rect.y + 24.0,
            TextStyle::new(
                18.0,
                if selected {
                    term::accent()
                } else {
                    term::primary()
                },
            )
            .params(),
        );
        draw_text_block(
            &legacy.description,
            rect.x + 14.0,
            rect.y + 32.0,
            rect.w - 28.0,
            26.0,
            13.0,
            2.0,
            term::dim(),
        );
        if pointer.released_on(rect) {
            actions.push(UiAction::SelectLegacy(i));
        }
        y += 70.0;
    }
    if let Some(legacy) = ctx
        .legacy_ids
        .get(ctx.menu.selected_legacy)
        .and_then(|id| ctx.data.legacies.get(id))
    {
        let callout = Rect::new(left_x, y + 6.0, col_w, content.bottom() - (y + 6.0) - 56.0);
        draw_surface(
            callout,
            &SurfaceStyle::new(term::surface_inset()).with_border(1.0, term::faint()),
        );
        draw_ui_text_ex(
            "WHAT THIS CHANGES",
            callout.x + 14.0,
            callout.y + 22.0,
            TextStyle::new(13.0, term::accent()).params(),
        );
        draw_text_block(
            &legacy.effects,
            callout.x + 14.0,
            callout.y + 30.0,
            callout.w - 28.0,
            callout.h - 40.0,
            13.0,
            3.0,
            term::primary(),
        );
    }
}

fn draw_faction_column(
    ctx: &MenuCtx<'_>,
    content: Rect,
    right_x: f32,
    col_w: f32,
    list_top: f32,
    pointer: Pointer,
    actions: &mut Vec<UiAction>,
) {
    let starting = ctx.data.config.factions.starting_count as usize;
    let chosen = ctx.menu.selected_factions.len();
    let tut = &ctx.data.config.tutorial;
    draw_text_block(
        &tut.factions_intro,
        right_x,
        content.y + 28.0,
        col_w,
        50.0,
        14.0,
        3.0,
        term::dim(),
    );
    draw_ui_text_ex(
        &format!("Choose {starting} founding peoples  ({chosen}/{starting}):"),
        right_x,
        content.y + 100.0,
        TextStyle::new(
            14.0,
            if chosen == starting {
                term::accent()
            } else {
                term::primary()
            },
        )
        .params(),
    );
    let mut y = list_top;
    for id in GameData::sorted_ids(&ctx.data.factions) {
        let Some(faction) = ctx.data.factions.get(&id) else {
            continue;
        };
        let selected = ctx.menu.selected_factions.iter().any(|f| f == &id);
        let rect = Rect::new(right_x, y + 4.0, col_w, 44.0);
        draw_surface(
            rect,
            &SurfaceStyle::new(if selected {
                term::surface_active()
            } else {
                term::surface_inset()
            })
            .with_border(
                1.0,
                if selected {
                    term::primary()
                } else {
                    term::faint()
                },
            ),
        );
        draw_ui_text_ex(
            &format!("{} {}", if selected { "[x]" } else { "[ ]" }, faction.name),
            rect.x + 10.0,
            rect.y + 18.0,
            TextStyle::new(
                14.0,
                if selected {
                    term::accent()
                } else {
                    term::primary()
                },
            )
            .params(),
        );
        draw_ui_text_ex(
            faction_ideology_label(faction.ideology),
            rect.x + 10.0,
            rect.y + 36.0,
            TextStyle::new(11.0, term::dim()).params(),
        );
        if pointer.released_on(rect) {
            actions.push(UiAction::ToggleFaction(id.clone()));
        }
        y += 50.0;
    }
}

fn draw_new_game_buttons(
    ctx: &MenuCtx<'_>,
    content: Rect,
    pointer: Pointer,
    actions: &mut Vec<UiAction>,
) {
    let starting = ctx.data.config.factions.starting_count as usize;
    let chosen = ctx.menu.selected_factions.len();
    let by = content.bottom() - 44.0;
    let btn_w = (content.w - 20.0) / 3.0;
    if term_button(
        Rect::new(content.x, by, btn_w, 44.0),
        "BEGIN VOYAGE [ENTER]",
        chosen == starting,
        pointer,
    ) {
        actions.push(UiAction::StartNewGame);
    }
    if term_button(
        Rect::new(content.x + btn_w + 10.0, by, btn_w, 44.0),
        "BACK [ESC]",
        true,
        pointer,
    ) {
        actions.push(UiAction::BackToMainMenu);
    }
    if term_button(
        Rect::new(content.x + (btn_w + 10.0) * 2.0, by, btn_w, 44.0),
        "DELETE SAVE",
        ctx.menu.save_exists,
        pointer,
    ) {
        actions.push(UiAction::DeleteSave);
    }
}

/// A short tech-spectrum tag for a faction's ideology (W7 picker flavor).
fn faction_ideology_label(ideology: f32) -> &'static str {
    if ideology > 0.66 {
        "tech-embracing · radical"
    } else if ideology > 0.2 {
        "tech-embracing"
    } else if ideology >= -0.2 {
        "pragmatic middle"
    } else if ideology >= -0.66 {
        "tech-averse"
    } else {
        "tech-averse · traditional"
    }
}
