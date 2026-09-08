//! Frame composition and responsive viewport dispatch.
use super::*;

impl Game {
    pub fn draw(&mut self) {
        clear_background(ui::term::bg());
        macroquad_toolkit::ui::set_ui_scale(self.display.ui_scale);

        let modal_reveal = self.modal_reveal();
        let log_reveal = self.log_reveal();

        let show_boot = !self.boot.is_done() && matches!(self.state, GameState::Menu(_));

        let (width, height) = if ui::mobile::active() {
            ui::mobile::size()
        } else {
            (ui::LOGICAL_WIDTH, ui::LOGICAL_HEIGHT)
        };
        let screen_pointer = Pointer::read(|p| p);
        let canvas = if ui::mobile::active() {
            None
        } else {
            let mut pan = self
                .presentation
                .desktop_pan
                .get()
                .unwrap_or(vec2(0.5, 0.5));
            let content = vec2(width, height);
            let canvas = macroquad_toolkit::ui::UiCanvas::new(content, self.display.ui_scale, pan);
            canvas.navigate(screen_pointer, &mut pan);
            self.presentation.desktop_pan.set(Some(pan));
            Some(macroquad_toolkit::ui::UiCanvas::new(
                content,
                self.display.ui_scale,
                pan,
            ))
        };
        // One pointer for the whole frame, in logical coordinates — a mouse or a
        // finger, read the same way. Built here rather than per screen so every
        // control agrees about where it is and whether it just let go.
        let pointer = if let Some(canvas) = canvas {
            canvas.begin();
            canvas.pointer(screen_pointer)
        } else {
            let virtual_ui = begin_virtual_ui_frame(width, height);
            Pointer::read(|p| virtual_ui.screen_to_ui(p))
        };
        self.presentation
            .overlay_active
            .set(self.settings_open || self.help_open || self.welcome_open);
        let base_pointer = if self.presentation.overlay_active.get() {
            pointer.suppressed()
        } else {
            pointer
        };
        let actions = if show_boot {
            if ui::mobile::active() {
                macroquad_toolkit::ui::draw_ui_text(
                    "STELLAR LEGACY",
                    16.0,
                    48.0,
                    24.0,
                    ui::term::primary(),
                );
                macroquad_toolkit::ui::draw_ui_text(
                    "Preparing ship systems…",
                    16.0,
                    90.0,
                    18.0,
                    ui::term::dim(),
                );
            } else {
                self.boot.draw();
            }
            Vec::new()
        } else {
            match &self.state {
                GameState::Menu(menu) => ui::draw_menu(ui::MenuCtx {
                    presentation: &self.presentation,
                    data: &self.data,
                    menu,
                    legacy_ids: &self.legacy_ids,
                    chronicle: &self.chronicle,
                    pointer: base_pointer,
                    title_art: self.assets.get_texture("title"),
                }),
                GameState::Gameplay(gameplay) => ui::draw_gameplay(ui::GameplayCtx {
                    presentation: &self.presentation,
                    data: &self.data,
                    sim: &gameplay.sim,
                    screen: gameplay.screen,
                    chronicle: &self.chronicle,
                    achievements: &self.achievements,
                    pointer: base_pointer,
                    modal_reveal,
                    log_reveal,
                    run_clock: self.run_clock_for(&gameplay.sim),
                    decision_remaining: self.decision_remaining(&gameplay.sim),
                    custody_picker: self.custody_picker.as_deref(),
                    obligation_detail: self.obligation_detail.as_deref(),
                    charter_scroll: &self.charter_scroll,
                    description_scroll: &self.description_scroll,
                    abort_confirm: &self.abort_confirm,
                    ship_scroll: &self.ship_scroll,
                    roster_scroll: &self.roster_scroll,
                    chronicle_scroll: &self.chronicle_scroll,
                    chronicle_records_tab: &self.chronicle_records_tab,
                    obligations_scroll: &self.obligations_scroll,
                    obligation_resolved_tab: &self.obligation_resolved_tab,
                    obligation_history_scroll: &self.obligation_history_scroll,
                    debrief_commanders_scroll: &self.debrief_commanders_scroll,
                    debrief_log_scroll: &self.debrief_log_scroll,
                    agenda_scroll: &self.agenda_scroll,
                    agenda_readiness_scroll: &self.agenda_readiness_scroll,
                    project_cancel_confirm: &self.project_cancel_confirm,
                    ship_modules_tab: &self.ship_modules_tab,
                    ship_preview: &self.ship_preview,
                    tutorial_enabled: self.display.tutorial_enabled,
                    tutorial_open: self.tutorial_open,
                }),
            }
        };

        // The F1/F2 panels float above everything and capture their own input.
        let display_actions = if self.settings_open {
            if ui::mobile::active() {
                ui::mobile::settings(
                    &self.display,
                    &self.delegation_defaults,
                    &self.presentation,
                    pointer,
                )
            } else {
                ui::settings::draw(&self.display, &self.delegation_defaults, pointer)
            }
        } else {
            Vec::new()
        };
        let help_action = self
            .help_open
            .then(|| {
                if ui::mobile::active() {
                    ui::mobile::help(&self.presentation, pointer)
                } else {
                    ui::help::draw(pointer, &self.data.config.version)
                }
            })
            .flatten();
        let mut time_actions = Vec::new();
        if let GameState::Gameplay(gameplay) = &self.state {
            if !ui::mobile::active() {
                ui::time_controls::draw(&gameplay.sim, pointer, &mut time_actions);
            }
        }
        for action in time_actions {
            self.events.push(action);
        }
        // First-run welcome overlay, above the menu only; its button dismisses it.
        let welcome_dismiss = self.welcome_open
            && matches!(self.state, GameState::Menu(_))
            && if ui::mobile::active() {
                ui::mobile::welcome(&self.data.config.welcome, &self.presentation, pointer)
            } else {
                ui::welcome::draw(&self.data.config.welcome, pointer)
            };
        // Roll this frame's controls into next frame's hit-area growth limits.
        // Every button grows toward the 44px touch standard only as far as its
        // neighbours allow, and the neighbours are whatever drew last frame — so
        // this has to happen once, after everything has drawn.
        end_frame_neighbours();
        end_virtual_ui_frame();
        if let Some(canvas) = canvas {
            canvas.draw_navigation(
                self.presentation
                    .desktop_pan
                    .get()
                    .unwrap_or(vec2(0.5, 0.5)),
                ui::term::surface_inset(),
                ui::term::primary(),
            );
        }

        // While a panel or the welcome overlay is open, swallow the underlying
        // screen's intents.
        if !self.settings_open && !self.help_open && !self.welcome_open {
            for action in actions {
                self.events.push(action);
            }
        }
        for action in display_actions {
            self.apply_display_action(action);
        }
        match help_action {
            Some(ui::help::HelpAction::Close) => self.help_open = false,
            Some(ui::help::HelpAction::OpenSaveFolder) => {
                match crate::support::reveal_save_folder(&self.data.config.game_name) {
                    Ok(()) => self.notifications.info("Save folder opened."),
                    Err(error) => self.notifications.warning(error),
                }
            }
            None => {}
        }
        if welcome_dismiss {
            self.dismiss_welcome();
        }

        self.notifications.draw_with_config_and_offset(
            &NotificationRenderConfig {
                anchor: NotificationAnchor::BottomRight,
                width: if ui::mobile::active() {
                    width - 32.0
                } else {
                    360.0
                },
                ..Default::default()
            },
            vec2(0.0, if ui::mobile::active() { -76.0 } else { 0.0 }),
        );

        // Phosphor-monitor overlay sits on top of everything else.
        if self.display.crt_enabled
            && matches!(self.state, GameState::Menu(_))
            && !self.settings_open
            && !self.help_open
            && !self.welcome_open
        {
            self.crt.draw(get_time() as f32, &self.crt_style);
        }
    }
}
