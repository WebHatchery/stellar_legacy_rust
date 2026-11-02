//! The main menu and the new-campaign screen: the only UI the player sees
//! before a ship exists.

use super::*;

mod new_game;
use new_game::draw_new_game;

/// Left edge of the menu column over the title art, aligned under the wordmark
/// and clear of the ship's engine glow to its right.
const TITLE_COLUMN_X: f32 = 98.0;

pub struct MenuCtx<'a> {
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
    match ctx.menu.phase {
        crate::state::MenuPhase::Main => draw_main_menu(&ctx),
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
