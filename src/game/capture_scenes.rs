//! Deterministic scene seeding for the headless screenshot harness.
//!
//! The root owns capture-wide reset and post-processing; scene-specific state
//! lives in focused route helpers so each capture remains reviewable.

use super::Game;
use crate::ui;
use macroquad_toolkit::achievements::Achievements;

mod agenda;
pub(crate) mod blueprint;
mod display;
mod homecoming;
mod routes;
mod stranded;

fn is_agenda_scene(scene: &str) -> bool {
    matches!(
        scene,
        "agenda"
            | "agenda_review"
            | "agenda_cancel"
            | "agenda_review_blocked"
            | "agenda_queue"
            | "agenda_catalogue"
            | "agenda_recovery"
    )
}

impl Game {
    /// Seed a deterministic state for the headless screenshot harness.
    pub fn begin_capture_scene(&mut self, scene: &str) {
        let (scene, form_offset) = scene
            .split_once('@')
            .map_or((scene, None), |(name, offset)| {
                (name, offset.parse::<f32>().ok())
            });
        // Screenshots want the final composed frame, not a mid-type one, and
        // never the boot log. Force canonical amber display so captures are
        // deterministic regardless of any persisted preference.
        self.presentation = Default::default();
        self.presentation.capture_form_offset.set(
            form_offset.map(|offset| (matches!(scene, "settings" | "help" | "welcome"), offset)),
        );
        self.chronicle = Default::default();
        self.achievements = Achievements::from_definitions(crate::achievements::definitions());
        self.instant_reveal = true;
        self.capture_run_secs = None;
        self.custody_picker = None;
        self.obligation_detail = None;
        self.settings_open = false;
        self.help_open = false;
        self.obligation_resolved_tab.set(false);
        self.boot.finish();
        // The first-run welcome overlay would otherwise sit over every menu
        // scene; scenes opt into it explicitly (the "welcome" scene below).
        self.welcome_open = false;
        self.tutorial_open = false;
        self.abort_confirm.set(false);
        self.display = crate::settings::DisplaySettings::default();
        let scene = display::prepare(scene, &mut self.display);
        let scene = if scene == "settings_gameplay" {
            self.settings_open = true;
            "gameplay"
        } else {
            scene
        };
        self.crt_style = self.display.crt_style();
        ui::term::set_phosphor(self.display.phosphor);
        self.delegation_defaults = crate::state::sim::DelegationSettings::default();
        if is_agenda_scene(scene) {
            self.capture_agenda(scene);
        } else {
            routes::capture(
                self,
                if scene == "bridge_commitments" {
                    "obligation_watch"
                } else {
                    scene
                },
            );
        }

        self.presentation
            .history_page
            .set(if scene.starts_with("obligation_") {
                1
            } else {
                0
            });
        self.capture_warning_state(scene);
        self.capture_presentation(scene);
    }

    fn capture_presentation(&mut self, scene: &str) {
        self.presentation.report_page.set(match scene {
            "debrief_captains" => 1,
            "debrief_moments" => 2,
            _ => 0,
        });
        if scene == "debrief_accounting" {
            let mut scroll = macroquad_toolkit::ui::ScrollArea::new();
            scroll.set_offset(450.0);
            self.presentation.report_scroll.set(scroll);
        }
        self.presentation.people_page.set(match scene {
            "people_officers" => 1,
            "people_factions" => 2,
            "people_council" => 3,
            _ => 0,
        });
        if scene == "milestones" {
            self.presentation.history_page.set(2);
        }
        self.presentation.utilities.set(scene == "utilities");
        self.presentation.instruments.set(scene == "instruments");
    }
    fn capture_warning_state(&mut self, scene: &str) {
        if scene == "bridge_commitments" {
            if let crate::state::GameState::Gameplay(g) = &mut self.state {
                g.sim.resources.energy = 0;
                for duty in &mut g.sim.obligations {
                    if duty.due_year.is_some() {
                        duty.due_year = Some(1);
                    }
                }
            }
        }
        if scene == "recovery_warning" {
            if let crate::state::GameState::Gameplay(g) = &mut self.state {
                g.sim.ship.life_support = 0.02;
                g.sim.survival.warning_active = true;
                g.sim.speed = crate::state::sim::GameSpeed::Paused;
            }
        }
    }
}
