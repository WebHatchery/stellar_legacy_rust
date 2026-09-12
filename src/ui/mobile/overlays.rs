//! Mobile modal overlays, occlusion, and dismissal controls.

use super::*;
fn draw<A>(f: Form<A>, state: &presentation::Presentation, pointer: Pointer, key: &str) -> Vec<A> {
    let (w, h) = size();
    macroquad_toolkit::ui::occlude(Rect::new(0.0, 0.0, w, h));
    draw_rectangle(0.0, 0.0, w, h, term::bg());
    let mut actions = Vec::new();
    f.draw(
        Rect::new(16.0, 16.0, w - 32.0, h - 44.0),
        state,
        pointer,
        key,
        &mut actions,
    );
    actions
}
pub fn welcome(
    config: &crate::data::WelcomeConfig,
    state: &presentation::Presentation,
    pointer: Pointer,
) -> bool {
    let mut f = Form::new();
    f.heading(&config.title);
    f.text(&config.intro);
    for section in &config.sections {
        f.heading(&section.heading);
        f.text(&section.body);
    }
    f.action(&config.dismiss_label, true, ());
    !draw(f, state, pointer, "welcome").is_empty()
}
pub fn help(
    state: &presentation::Presentation,
    pointer: Pointer,
) -> Option<crate::ui::help::HelpAction> {
    use crate::ui::help::HelpAction;
    let mut f = Form::new();
    f.heading("The Custodian");
    f.text("You are the persistent intelligence aboard a generation ship. Captains age and councils change; you carry their promises across centuries.");
    f.heading("Touch controls");
    f.text("Tap Bridge, Ship, People, Voyage or History in the bottom bar. Tap Pause or a speed in the top bar. Drag the content to read. Tap Utilities to save, change display settings, or return to the menu.");
    f.heading("Council decisions");
    f.text("Read the situation and each choice's known consequences. Tap its Commit button to act. Officer advice is optional. The captain's fallback countdown remains active while you read; use Pause when you need more time.");
    f.heading("Authority");
    f.text("The Custodian handles routine operations under a standing mandate. The captain can object to a proposed command posture. Keep the current policy or accept the legal compromise shown in that review.");
    f.action("Open save folder", true, HelpAction::OpenSaveFolder);
    f.action("Close help", true, HelpAction::Close);
    draw(f, state, pointer, "help").into_iter().next()
}
