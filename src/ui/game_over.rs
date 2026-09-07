//! Full-screen "voyage terminated" takeover for every authored loss condition.

use crate::ui::{
    spec_line, term, term_button, term_panel, GameplayCtx, UiAction, LOGICAL_HEIGHT, LOGICAL_WIDTH,
};
use macroquad::prelude::*;
use macroquad_toolkit::prelude::*;
use macroquad_toolkit::ui::{draw_ui_text_ex, RectExt};

pub fn draw(ctx: &GameplayCtx<'_>, pointer: Pointer, actions: &mut Vec<UiAction>) {
    let sim = ctx.sim;
    draw_rectangle(0.0, 0.0, LOGICAL_WIDTH, LOGICAL_HEIGHT, term::bg());

    let legacy = ctx
        .data
        .legacies
        .get(&sim.legacy.legacy_id)
        .map(|l| l.name.clone())
        .unwrap_or_default();

    // Halted-terminal banner.
    let reason = sim
        .terminal
        .as_ref()
        .map(|terminal| terminal.reason.label())
        .unwrap_or("DYNASTY EXTINCTION");
    draw_text_glow(
        reason,
        LOGICAL_WIDTH / 2.0 - reason.len() as f32 * 13.0,
        140.0,
        TextStyle::new(46.0, term::alert()),
        0.14,
        3.0,
    );
    draw_ui_text_ex(
        &format!(
            "// {} — the founding commission has ended //",
            legacy.to_uppercase()
        ),
        LOGICAL_WIDTH / 2.0 - 250.0,
        175.0,
        TextStyle::new(16.0, term::dim()).params(),
    );

    let panel = Rect::new(LOGICAL_WIDTH / 2.0 - 330.0, 220.0, 660.0, 384.0);
    term_panel(panel, Some("FINAL LOG // DYNASTY REGISTRY SEALED"));
    let content = panel.inset(28.0);

    let contracts = ctx
        .chronicle
        .entries
        .iter()
        .filter(|e| e.legacy_id == sim.legacy.legacy_id)
        .count();
    let leader = sim
        .dynasty
        .leader()
        .map(|l| l.name.clone())
        .unwrap_or_else(|| "an empty chair".to_owned());

    let rows: [(&str, String); 6] = [
        ("YEARS ELAPSED", sim.year().to_string()),
        ("GENERATIONS", sim.dynasty.generation.to_string()),
        ("FINAL POPULATION", sim.population.count.to_string()),
        ("TRADITION EARNED", sim.legacy.tradition_points.to_string()),
        ("CONTRACTS LOGGED", contracts.to_string()),
        ("LAST COMMANDER", leader),
    ];
    let mut y = content.y + 30.0;
    for (label, value) in rows {
        spec_line(content.x, y, content.w, label, &value, term::accent());
        y += 30.0;
    }

    y += 14.0;
    let evidence = sim
        .terminal
        .as_ref()
        .map(|terminal| terminal.evidence.as_str())
        .unwrap_or("The captaincy has no eligible heir.");
    draw_text_block(
        &format!("{evidence} The Custodian is archived with the vessel's records; the Chronicle preserves what the generations carried."),
        content.x,
        y,
        content.w,
        44.0,
        14.0,
        4.0,
        term::dim(),
    );

    // The terminal always leaves the player an explicit route to the Chronicle,
    // a fresh campaign, or the menu. No keyboard is needed to recover.
    let caret = if blink(get_time() as f32, 2.5) {
        ">"
    } else {
        " "
    };
    let gap = 8.0;
    let btn_w = (content.w - gap * 2.0) / 3.0;
    let y = content.bottom() - 48.0;
    let chronicle = Rect::new(content.x, y, btn_w, 44.0);
    let new_game = Rect::new(content.x + btn_w + gap, y, btn_w, 44.0);
    let menu = Rect::new(content.right() - btn_w, y, btn_w, 44.0);
    if term_button(chronicle, "CHRONICLE", true, pointer) {
        actions.push(UiAction::SelectScreen(crate::state::Screen::Chronicle));
    }
    if term_button(new_game, &format!("{caret} NEW GAME"), true, pointer) {
        actions.push(UiAction::RetireVoyage);
    }
    if term_button(menu, "MENU", true, pointer) {
        actions.push(UiAction::ToMenu);
    }
}
