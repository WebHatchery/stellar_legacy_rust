//! Frame composition and responsive viewport dispatch.
use super::*;

impl Game {
    pub fn draw(&mut self) {
        clear_background(ui::term::bg());
        macroquad_toolkit::ui::set_ui_scale(self.display.ui_scale);
        macroquad_toolkit::ui::set_ui_text_scale(self.display.text_scale);

        let modal_reveal = self.modal_reveal();
        let log_reveal = self.log_reveal();

        let show_boot = !self.boot.is_done() && matches!(self.state, GameState::Menu(_));

        let virtual_ui = macroquad_toolkit::ui::VirtualUi::responsive();
        let (width, height) = (virtual_ui.logical_width, virtual_ui.logical_height);
        virtual_ui.begin();
        let pointer = Pointer::read(|p| virtual_ui.screen_to_ui(p));
        self.presentation
            .overlay_active
            .set(self.settings_open || self.help_open || self.welcome_open);
        let base_pointer = if self.presentation.overlay_active.get() {
            pointer.suppressed()
        } else {
            pointer
        };
        let actions = self.draw_scene(show_boot, base_pointer, modal_reveal, log_reveal);

        // The F1/F2 panels float above everything and capture their own input.
        let (display_actions, help_action, time_actions, welcome_dismiss) =
            self.draw_overlays(pointer);
        for action in time_actions {
            self.events.push(action);
        }
        // Roll this frame's controls into next frame's hit-area growth limits.
        // Every button grows toward the 44px touch standard only as far as its
        // neighbours allow, and the neighbours are whatever drew last frame — so
        // this has to happen once, after everything has drawn.
        end_frame_neighbours();
        end_virtual_ui_frame();
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

        self.draw_notifications(width, height);
        self.draw_crt_overlay();
    }

    fn draw_scene(
        &self,
        show_boot: bool,
        pointer: Pointer,
        modal_reveal: f32,
        log_reveal: f32,
    ) -> Vec<UiAction> {
        if show_boot {
            self.draw_boot_screen();
            return Vec::new();
        }
        match &self.state {
            GameState::Menu(menu) => ui::draw_menu(ui::MenuCtx {
                presentation: &self.presentation,
                data: &self.data,
                menu,
                legacy_ids: &self.legacy_ids,
                chronicle: &self.chronicle,
                pointer,
                title_art: self.assets.get_texture("title"),
            }),
            GameState::Gameplay(gameplay) => ui::draw_gameplay(ui::GameplayCtx {
                presentation: &self.presentation,
                data: &self.data,
                sim: &gameplay.sim,
                screen: gameplay.screen,
                chronicle: &self.chronicle,
                achievements: &self.achievements,
                pointer,
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
    }

    fn draw_boot_screen(&self) {
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
    }

    fn draw_overlays(
        &mut self,
        pointer: Pointer,
    ) -> (
        Vec<ui::settings::DisplayAction>,
        Option<ui::help::HelpAction>,
        Vec<UiAction>,
        bool,
    ) {
        let display_actions = if self.settings_open {
            ui::settings::draw(
                &self.display,
                &self.delegation_defaults,
                &self.presentation,
                pointer,
            )
        } else {
            Vec::new()
        };
        let help_action = self.help_open.then(|| self.draw_help(pointer)).flatten();
        let mut time_actions = Vec::new();
        if let GameState::Gameplay(gameplay) = &self.state {
            if !ui::mobile::active()
                && !ui::compact()
                && !self.settings_open
                && !self.help_open
                && !self.welcome_open
            {
                ui::time_controls::draw(&gameplay.sim, pointer, &mut time_actions);
            }
        }
        let welcome_dismiss = self.welcome_open
            && matches!(self.state, GameState::Menu(_))
            && if ui::mobile::active() || ui::compact() {
                ui::mobile::welcome(&self.data.config.welcome, &self.presentation, pointer)
            } else {
                ui::welcome::draw(&self.data.config.welcome, pointer)
            };
        (display_actions, help_action, time_actions, welcome_dismiss)
    }

    fn draw_help(&self, pointer: Pointer) -> Option<ui::help::HelpAction> {
        if ui::mobile::active() || ui::compact() {
            ui::mobile::help(&self.presentation, pointer)
        } else {
            ui::help::draw(pointer, &self.data.config.version)
        }
    }

    fn draw_notifications(&self, width: f32, height: f32) {
        let virtual_ui = macroquad_toolkit::ui::VirtualUi::responsive();
        virtual_ui.begin();
        macroquad_toolkit::notifications::draw_notifications_in_viewport(
            self.notifications.get_notifications(),
            &NotificationRenderConfig {
                anchor: NotificationAnchor::BottomRight,
                width: 360.0_f32.min(width - 32.0),
                ..Default::default()
            },
            vec2(width, height),
            Vec2::ZERO,
        );
        end_virtual_ui_frame();
    }

    fn draw_crt_overlay(&mut self) {
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
