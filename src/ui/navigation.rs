//! Stable destinations; port restrictions apply to sections, never tab positions.
use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Destination {
    Bridge,
    Ship,
    People,
    Voyage,
    History,
}
impl Destination {
    pub const ALL: [Self; 5] = [
        Self::Bridge,
        Self::Ship,
        Self::People,
        Self::Voyage,
        Self::History,
    ];
    pub fn of(screen: Screen) -> Self {
        match screen {
            Screen::Dashboard => Self::Bridge,
            Screen::ShipBuilder | Screen::Subsystems | Screen::Agenda => Self::Ship,
            Screen::CrewDynasty => Self::People,
            Screen::Drydock | Screen::Contract | Screen::Market => Self::Voyage,
            Screen::Chronicle => Self::History,
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            Self::Bridge => "Bridge",
            Self::Ship => "Ship",
            Self::People => "People",
            Self::Voyage => "Voyage",
            Self::History => "History",
        }
    }
    pub fn home(self, in_port: bool) -> Screen {
        match self {
            Self::Bridge => Screen::Dashboard,
            Self::Ship => Screen::ShipBuilder,
            Self::People => Screen::CrewDynasty,
            Self::Voyage => {
                if in_port {
                    Screen::Drydock
                } else {
                    Screen::Contract
                }
            }
            Self::History => Screen::Chronicle,
        }
    }
    pub fn sections(self, in_port: bool) -> Vec<(&'static str, Screen)> {
        match self {
            Self::Ship => vec![
                ("Loadout", Screen::ShipBuilder),
                ("Systems", Screen::Subsystems),
                ("Agenda", Screen::Agenda),
            ],
            Self::Voyage if in_port => {
                vec![("Drydock", Screen::Drydock), ("Market", Screen::Market)]
            }
            Self::Voyage => vec![("Contract", Screen::Contract)],
            _ => vec![],
        }
    }
}

pub fn draw(ctx: &GameplayCtx<'_>, pointer: Pointer, actions: &mut Vec<UiAction>) {
    let current = Destination::of(ctx.screen);
    draw_destinations(ctx, current, pointer, actions);
    let sections = current.sections(ctx.sim.contract.is_none());
    draw_screen_sections(ctx, &sections, pointer, actions);
    draw_context_subtabs(ctx, current, pointer);
    draw_utilities_toggle(ctx, pointer);
}

fn draw_destinations(
    ctx: &GameplayCtx<'_>,
    current: Destination,
    pointer: Pointer,
    actions: &mut Vec<UiAction>,
) {
    for (index, destination) in Destination::ALL.into_iter().enumerate() {
        let rect = Rect::new(16.0 + index as f32 * 116.0, 78.0, 108.0, 44.0);
        let label = if destination == Destination::History
            && ctx
                .sim
                .next_timed_obligation()
                .is_some_and(|d| d.due_year.is_some_and(|year| year <= ctx.sim.year()))
        {
            "History !"
        } else {
            destination.label()
        };
        if term_button(rect, label, true, pointer) {
            ctx.presentation.instruments.set(false);
            actions.push(UiAction::SelectScreen(
                destination.home(ctx.sim.contract.is_none()),
            ));
        }
        if destination == current {
            draw_rectangle(
                rect.x + 8.0,
                rect.bottom() - 3.0,
                rect.w - 16.0,
                3.0,
                term::primary(),
            );
        }
    }
}

fn draw_screen_sections(
    ctx: &GameplayCtx<'_>,
    sections: &[(&str, Screen)],
    pointer: Pointer,
    actions: &mut Vec<UiAction>,
) {
    for (index, (label, screen)) in sections.iter().enumerate() {
        let rect = Rect::new(614.0 + index as f32 * 158.0, 78.0, 150.0, 44.0);
        let label = if *screen == Screen::Agenda {
            format!(
                "Agenda ({})",
                ctx.sim.projects.active_count() + ctx.sim.projects.waiting_count()
            )
        } else {
            (*label).to_owned()
        };
        if term_button(rect, &label, true, pointer) {
            actions.push(UiAction::SelectScreen(*screen));
        }
        if *screen == ctx.screen {
            draw_rectangle(
                rect.x + 8.0,
                rect.bottom() - 3.0,
                rect.w - 16.0,
                3.0,
                term::accent(),
            );
        }
    }
}

fn draw_context_subtabs(ctx: &GameplayCtx<'_>, current: Destination, pointer: Pointer) {
    if current == Destination::History {
        for (index, label) in ["Timeline", "Obligations", "Milestones"]
            .into_iter()
            .enumerate()
        {
            let rect = Rect::new(614.0 + index as f32 * 158.0, 78.0, 150.0, 44.0);
            if term_button(rect, label, true, pointer) {
                ctx.presentation.history_page.set(index);
            }
            if ctx.presentation.history_page.get() == index {
                draw_rectangle(rect.x, rect.bottom() - 3.0, rect.w, 3.0, term::accent());
            }
        }
    }
    if current == Destination::People {
        for (index, label) in ["Family", "Officers", "Factions", "Council"]
            .into_iter()
            .enumerate()
        {
            let rect = Rect::new(614.0 + index as f32 * 122.0, 78.0, 114.0, 44.0);
            if term_button(rect, label, true, pointer) {
                ctx.presentation.people_page.set(index);
            }
            if ctx.presentation.people_page.get() == index {
                draw_rectangle(rect.x, rect.bottom() - 3.0, rect.w, 3.0, term::accent());
            }
        }
    }
    if current == Destination::Bridge {
        for (index, label) in ["Overview", "Instruments"].into_iter().enumerate() {
            if term_button(
                Rect::new(614.0 + index as f32 * 158.0, 78.0, 150.0, 44.0),
                label,
                true,
                pointer,
            ) {
                ctx.presentation.instruments.set(index == 1);
            }
        }
    }
}

fn draw_utilities_toggle(ctx: &GameplayCtx<'_>, pointer: Pointer) {
    if term_button(
        Rect::new(logical_width() - 146.0, 78.0, 130.0, 44.0),
        "Utilities",
        true,
        pointer,
    ) {
        ctx.presentation
            .utilities
            .set(!ctx.presentation.utilities.get());
    }
}

pub fn draw_utilities(ctx: &GameplayCtx<'_>, pointer: Pointer, actions: &mut Vec<UiAction>) {
    if !ctx.presentation.utilities.get() {
        return;
    }
    actions.clear();
    let rect = Rect::new(928.0, 128.0, 336.0, 320.0);
    macroquad_toolkit::ui::occlude(rect);
    term_panel(rect, Some("Utilities"));
    for (i, (label, action)) in [
        ("Save game", UiAction::SaveGame),
        ("Help", UiAction::OpenHelp),
        ("Display & sound", UiAction::OpenSettings),
        ("Return to menu", UiAction::ToMenu),
    ]
    .into_iter()
    .enumerate()
    {
        if term_button(
            Rect::new(
                rect.x + 16.0,
                rect.y + 48.0 + i as f32 * 52.0,
                rect.w - 32.0,
                44.0,
            ),
            label,
            true,
            pointer,
        ) {
            ctx.presentation.utilities.set(false);
            actions.push(action);
        }
    }
    if term_button(
        Rect::new(rect.x + 16.0, rect.bottom() - 52.0, rect.w - 32.0, 44.0),
        "Close utilities",
        true,
        pointer,
    ) {
        ctx.presentation.utilities.set(false);
    }
}
#[allow(dead_code, unused_imports)]
mod tests {
    include!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/unit/ui/navigation/tests.rs"
    ));
}
