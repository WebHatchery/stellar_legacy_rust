//! Narrow screens use a vertical reading view with persistent time and navigation.
use super::*;
mod decisions;
mod form;
mod history;
mod menu;
mod overlays;
pub use overlays::{help, settings, welcome};
mod people;
mod ship;
mod voyage;
use form::Form;
pub use menu::draw_menu;

pub fn active() -> bool {
    screen_width() < 1100.0
        || screen_height() < 640.0
        || (macroquad_toolkit::ui::ui_scale() - 1.0).abs() > 0.001
}
pub fn size() -> (f32, f32) {
    let viewport = macroquad_toolkit::ui::VirtualUi::scaled(320.0, 480.0);
    (viewport.logical_width, viewport.logical_height)
}

pub fn draw(ctx: &GameplayCtx<'_>) -> Vec<UiAction> {
    let (width, height) = size();
    let mut actions = Vec::new();
    let pointer = ctx.pointer;
    draw_rectangle(0.0, 0.0, width, height, term::bg());
    draw_ui_text_ex(
        "STELLAR LEGACY",
        12.0,
        25.0,
        TextStyle::new(22.0, term::primary()).params(),
    );
    draw_ui_text_ex(
        &if ctx.sim.has_pending_decision() && ctx.sim.debrief.is_none() {
            format!("Captain fallback: {:.0}s", ctx.decision_remaining.ceil())
        } else {
            format!(
                "Year {} · Generation {}",
                ctx.sim.year(),
                ctx.sim.dynasty.generation
            )
        },
        12.0,
        49.0,
        TextStyle::new(16.0, term::dim()).params(),
    );
    if term_button(
        Rect::new(width - 114.0, 8.0, 102.0, 48.0),
        if ctx.sim.speed == GameSpeed::Paused {
            "Resume"
        } else {
            "Pause"
        },
        true,
        pointer,
    ) {
        actions.push(UiAction::TogglePause);
    }
    for (index, speed) in GameSpeed::ALL.into_iter().skip(1).enumerate() {
        if term_button(
            Rect::new(12.0 + index as f32 * 64.0, 64.0, 56.0, 44.0),
            &format!("{}×", index + 1),
            true,
            pointer,
        ) {
            actions.push(UiAction::SetSpeed(speed));
        }
    }
    if term_button(
        Rect::new(width - 150.0, 64.0, 138.0, 44.0),
        "Utilities",
        !ctx.sim.has_pending_decision()
            && !ctx.sim.survival.warning_active
            && ctx.sim.debrief.is_none(),
        pointer,
    ) {
        ctx.presentation
            .utilities
            .set(!ctx.presentation.utilities.get());
    }
    let section = ctx.presentation.mobile_section.borrow().clone();
    let mut form = Form::new();
    let key = if ctx.presentation.utilities.get() {
        form.heading("Utilities");
        for (label, action) in [
            ("Save game", UiAction::SaveGame),
            ("Help", UiAction::OpenHelp),
            ("Display & sound", UiAction::OpenSettings),
            ("Return to menu", UiAction::ToMenu),
        ] {
            form.action(label, true, action);
        }
        form.close_utilities();
        "utilities".to_owned()
    } else if let Some(key) = decisions::build(ctx, &mut form) {
        key
    } else {
        if ctx.tutorial_enabled && ctx.tutorial_open && !ctx.sim.tutorial_dismissed {
            if let Some(step) = ctx
                .data
                .config
                .tutorial
                .guided_steps
                .get(ctx.sim.tutorial_step)
            {
                form.heading(&format!("Tutorial · {}", step.label));
                form.text(&step.tip);
                form.action("Skip tutorial", true, UiAction::SkipTutorial);
            }
        }
        match ctx.screen {
            Screen::Dashboard => bridge(ctx, &mut form),
            Screen::ShipBuilder | Screen::Subsystems | Screen::Agenda => {
                ship::build(ctx, &mut form, &section)
            }
            Screen::CrewDynasty => people::build(ctx, &mut form, &section),
            Screen::Drydock | Screen::Contract | Screen::Market => {
                voyage::build(ctx, &mut form, &section)
            }
            Screen::Chronicle => history::build(ctx, &mut form, &section),
        }
        format!(
            "{:?}:{section}:{:?}:{:?}:{:?}:{:?}",
            ctx.screen,
            ctx.project_cancel_confirm.get(),
            ctx.custody_picker,
            ctx.sim.selected_charter,
            ctx.obligation_detail
        )
    };
    form.draw(
        Rect::new(12.0, 120.0, width - 24.0, height - 208.0),
        ctx.presentation,
        pointer,
        &key,
        &mut actions,
    );
    let blocked = ctx.sim.has_pending_decision()
        || ctx.sim.survival.warning_active
        || ctx.sim.debrief.is_some();
    if ctx.sim.debrief.is_some()
        && nav_button(
            Rect::new(12.0, height - 62.0, width - 24.0, 54.0),
            "File the report",
            pointer,
        )
    {
        actions.push(UiAction::FileReport);
    }
    if !blocked {
        for (index, destination) in navigation::Destination::ALL.into_iter().enumerate() {
            let rect = Rect::new(
                4.0 + index as f32 * (width / 5.0),
                height - 62.0,
                width / 5.0 - 8.0,
                54.0,
            );
            if nav_button(rect, destination.label(), pointer) {
                ctx.presentation.mobile_section.borrow_mut().clear();
                ctx.presentation.utilities.set(false);
                actions.push(UiAction::SelectScreen(
                    destination.home(ctx.sim.contract.is_none()),
                ));
            }
        }
    }
    actions
}

fn bridge(ctx: &GameplayCtx<'_>, form: &mut Form) {
    let sim = ctx.sim;
    form.heading(
        sim.contract
            .as_ref()
            .map_or("Choose the next voyage", |c| c.name.as_str()),
    );
    form.text(&sim.contract.as_ref().map_or(
        "In port · choose a charter, then review provisions.".to_owned(),
        |c| {
            format!(
                "{} · Voyage {:.0}% · Objective {:.0}%",
                c.phase.label(),
                c.progress() * 100.0,
                c.objective_fraction() * 100.0
            )
        },
    ));
    form.vessel(ctx);
    if sim.resources.energy < ctx.data.config.low_energy_threshold {
        form.heading("! Energy shortage");
        form.text(&format!(
            "{} energy remains. Trade in port or inspect engineering and demand underway.",
            sim.resources.energy
        ));
        form.action(
            "Inspect energy recovery",
            true,
            UiAction::SelectScreen(if sim.contract.is_none() {
                Screen::Market
            } else {
                Screen::Subsystems
            }),
        );
    } else {
        let forecast = crate::simulation::readiness::forecast(sim, ctx.data);
        if let Some(row) = forecast
            .rows
            .iter()
            .filter(|r| {
                matches!(
                    r.band,
                    crate::simulation::readiness::ReadinessBand::Critical
                        | crate::simulation::readiness::ReadinessBand::Vulnerable
                )
            })
            .min_by(|a, b| a.score.total_cmp(&b.score))
        {
            form.heading(&format!("! {} · {}", row.concern, row.band.label()));
            form.text(&format!("{}\n{}", row.evidence, row.trend));
            form.action_section(
                "Review recovery projects",
                UiAction::SelectScreen(Screen::Agenda),
                "readiness",
            );
        } else {
            form.text("No urgent reserve warning in the current readiness forecast.");
        }
        form.action(
            if sim.contract.is_none() {
                "Choose a charter"
            } else {
                "Follow voyage"
            },
            true,
            UiAction::SelectScreen(if sim.contract.is_none() {
                Screen::Drydock
            } else {
                Screen::Contract
            }),
        );
    }
    if let Some(duty) = sim
        .next_timed_obligation()
        .filter(|d| d.due_year.is_some_and(|year| year <= sim.year() + 1))
    {
        form.heading("! A promise needs attention");
        form.text(&duty.title);
        form.action_section(
            "Review obligations",
            UiAction::SelectScreen(Screen::Chronicle),
            "obligations",
        );
    }
    form.text(&format!(
        "Hull {:.0}% · Air {:.0}% · Fuel {:.0}%\n{} people · {} active projects",
        sim.ship.hull_integrity * 100.0,
        sim.ship.life_support * 100.0,
        sim.ship.fuel * 100.0,
        sim.population.count,
        sim.projects.active_count()
    ));
    form.action(
        "Open project queue",
        true,
        UiAction::SelectScreen(Screen::Agenda),
    );
    form.heading("Stores");
    form.text(&format!(
        "Credits {} · Energy {}\nMinerals {} · Food {}\nInfluence {}",
        sim.resources.credits,
        sim.resources.energy,
        sim.resources.minerals,
        sim.resources.food,
        sim.resources.influence
    ));
    form.heading("Recent developments");
    for entry in sim.log.iter().rev().take(3) {
        form.text(&format!("Year {} · {}", entry.year, entry.text));
    }
    form.action(
        "Open History",
        true,
        UiAction::SelectScreen(Screen::Chronicle),
    );
}

fn nav_button(rect: Rect, label: &str, pointer: Pointer) -> bool {
    macroquad_toolkit::ui::note_neighbour(rect);
    macroquad_toolkit::ui::note_target(label, rect);
    draw_rectangle(rect.x, rect.y, rect.w, rect.h, term::surface());
    draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 1.0, term::faint());
    draw_text_centered_in_box_ex(
        label,
        rect.x,
        rect.y,
        rect.w,
        rect.h,
        TextStyle::new(16.0, term::primary()),
    );
    pointer.released_on(rect)
}
