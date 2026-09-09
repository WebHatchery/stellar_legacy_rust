//! Narrow screens use a vertical reading view with persistent time and navigation.
use super::*;
mod decisions;
pub(crate) mod form;
mod history;
mod layout;
mod menu;
mod overlays;
pub use overlays::{help, welcome};
mod people;
mod ship;
mod voyage;
use form::Form;
pub use menu::draw_menu;

pub fn active() -> bool {
    screen_width() < 1100.0 || screen_height() < 640.0
}
pub fn size() -> (f32, f32) {
    (logical_width(), logical_height())
}

fn shell_layout() -> layout::Layout {
    let (width, height) = size();
    let widest = navigation::Destination::ALL
        .iter()
        .map(|d| measure_text_size(d.label(), TextStyle::new(16.0, term::primary())).width)
        .fold(0.0, f32::max);
    layout::Layout::new(
        width,
        height,
        macroquad_toolkit::ui::ui_text_scale(),
        widest,
    )
}

pub(crate) fn navigation_is_compact() -> bool {
    active() && shell_layout().compact_navigation
}

pub fn draw(ctx: &GameplayCtx<'_>) -> Vec<UiAction> {
    let (width, height) = size();
    let mut actions = Vec::new();
    let pointer = ctx.pointer;
    let layout = shell_layout();
    if !layout.compact_navigation {
        ctx.presentation.navigation_open.set(false);
    }
    draw_rectangle(0.0, 0.0, width, height, term::bg());
    draw_text_centered_in_box_ex(
        "STELLAR LEGACY",
        layout.title.x,
        layout.title.y,
        layout.title.w,
        layout.title.h,
        TextStyle::new(22.0, term::primary()),
    );
    draw_text_centered_in_box_ex(
        &if ctx.sim.has_pending_decision() && ctx.sim.debrief.is_none() {
            time_controls::fallback_label(ctx.sim.speed, ctx.decision_remaining)
        } else {
            format!(
                "Year {} · Generation {}",
                ctx.sim.year(),
                ctx.sim.dynasty.generation
            )
        },
        layout.status.x,
        layout.status.y,
        layout.status.w,
        layout.status.h,
        TextStyle::new(16.0, term::dim()),
    );
    let control_font = if width < 300.0 { 12.0 } else { 16.0 };
    if term_button_sized(
        layout.pause,
        if ctx.sim.speed == GameSpeed::Paused {
            "Resume"
        } else {
            "Pause"
        },
        true,
        pointer,
        control_font,
    ) {
        actions.push(UiAction::TogglePause);
    }
    for (index, speed) in GameSpeed::ALL.into_iter().skip(1).enumerate() {
        let rect = layout.speeds[index];
        if term_button(rect, &format!("{}×", index + 1), true, pointer) {
            actions.push(UiAction::SetSpeed(speed));
        }
        if ctx.sim.speed == speed {
            selection_marker(rect);
        }
    }
    if term_button_sized(
        layout.utilities,
        "Utilities",
        !ctx.sim.has_pending_decision()
            && !ctx.sim.survival.warning_active
            && ctx.sim.debrief.is_none(),
        pointer,
        control_font,
    ) {
        ctx.presentation.navigation_open.set(false);
        actions.push(UiAction::DismissReturnHome);
        actions.push(UiAction::DismissCancelProject);
        ctx.presentation
            .utilities
            .set(!ctx.presentation.utilities.get());
    }
    let blocked = ctx.sim.has_pending_decision()
        || ctx.sim.survival.warning_active
        || ctx.sim.debrief.is_some();
    if ctx.presentation.navigation_open.get() && !blocked {
        let mut form = Form::new();
        for destination in navigation::Destination::ALL {
            form.action(
                destination.label(),
                true,
                UiAction::SelectScreen(destination.home(ctx.sim.contract.is_none())),
            );
        }
        // Opening navigation must not replace the current report's reading position.
        form.draw_scrolled(
            layout.content,
            ctx.presentation,
            pointer,
            "navigation",
            &mut actions,
            &ctx.presentation.navigation_scroll,
            &ctx.presentation.navigation_key,
        );
        if actions
            .iter()
            .any(|action| matches!(action, UiAction::SelectScreen(_)))
        {
            ctx.presentation.navigation_open.set(false);
            ctx.presentation.mobile_section.borrow_mut().clear();
        }
    } else {
        actions.extend(draw_content(ctx, layout.content));
    }
    if ctx.sim.debrief.is_some() && nav_button(layout.navigation, "File the report", pointer) {
        actions.push(UiAction::FileReport);
    }
    if !blocked {
        if layout.compact_navigation {
            let label = if ctx.presentation.navigation_open.get() {
                "Back".to_owned()
            } else {
                "Navigate".to_owned()
            };
            if nav_button(layout.navigation, &label, pointer) {
                ctx.presentation.utilities.set(false);
                ctx.presentation
                    .navigation_open
                    .set(!ctx.presentation.navigation_open.get());
            }
            return actions;
        }
        for (index, destination) in navigation::Destination::ALL.into_iter().enumerate() {
            let rect = layout.destination(index);
            if nav_button(rect, destination.label(), pointer) {
                ctx.presentation.mobile_section.borrow_mut().clear();
                ctx.presentation.utilities.set(false);
                actions.push(UiAction::SelectScreen(
                    destination.home(ctx.sim.contract.is_none()),
                ));
            }
            if destination == navigation::Destination::of(ctx.screen)
                && !ctx.presentation.utilities.get()
            {
                selection_marker(rect);
            }
        }
    }
    actions
}

/// Shared flow layout for a panel whose available space cannot hold the wide layout.
pub fn draw_content(ctx: &GameplayCtx<'_>, view: Rect) -> Vec<UiAction> {
    let pointer = ctx.pointer;
    let mut actions = Vec::new();
    let section = ctx.presentation.mobile_section.borrow().clone();
    let mut form = Form::new();
    let key = if ctx.abort_confirm.get() && !ctx.sim.has_pending_decision() {
        mission::build_abort(ctx, &mut form);
        "return-home-review".to_owned()
    } else if ctx.presentation.utilities.get() {
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
            tutorial::form(ctx, &mut form);
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
            "{:?}:{section}:{:?}:{:?}:{:?}:{:?}:{:?}:{:?}",
            ctx.screen,
            ctx.project_cancel_confirm.get(),
            ctx.custody_picker,
            ctx.sim.selected_charter,
            ctx.obligation_detail,
            ctx.presentation.project_cancellation.get(),
            tutorial::reading_step(ctx)
        )
    };
    if ctx.sim.debrief.is_some() {
        form.action("File the report", true, UiAction::FileReport);
    }
    form.draw(view, ctx.presentation, pointer, &key, &mut actions);
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
    bridge_details(ctx, form);
}

pub(crate) fn bridge_details(ctx: &GameplayCtx<'_>, form: &mut Form) {
    let sim = ctx.sim;
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
    let fill = if pointer.pressing(rect) {
        term::surface_active()
    } else if pointer.hovering_over(rect) {
        term::surface_hover()
    } else {
        term::surface()
    };
    draw_rectangle(rect.x, rect.y, rect.w, rect.h, fill);
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
