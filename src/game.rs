//! Game struct: owns the state machine, drives the tick, and composes the
//! per-frame draw. UiAction dispatch lives in [`actions`]; screenshot scene
//! seeding in [`capture_scenes`].

mod actions;
mod build_mode;
mod capture_scenes;
mod realtime;
mod render;
mod tutorial;

use crate::audio::{AudioManager, Cue};
use crate::boot::BootScreen;
use crate::chronicle::ChronicleStore;
use crate::data::GameData;
use crate::save;
use crate::settings::DisplaySettings;
use crate::simulation::legacy;
use crate::state::{GameState, GameplayState, MenuState, SimState, StateTransition};
use crate::ui::{self, UiAction};
use macroquad::prelude::*;
use macroquad_toolkit::achievements::Achievements;
use macroquad_toolkit::assets::AssetManager;
use macroquad_toolkit::events::EventBus;
use macroquad_toolkit::fx::{CrtOverlay, CrtStyle};
use macroquad_toolkit::notifications::{
    NotificationAnchor, NotificationManager, NotificationRenderConfig,
};
use macroquad_toolkit::prelude::end_virtual_ui_frame;
use macroquad_toolkit::ui::{end_frame_neighbours, Pointer, ScrollArea};
use std::cell::Cell;

/// True on the frame the number key for a 0-based list index (1..=9) is pressed.
fn digit_pressed(index: usize) -> bool {
    let key = match index {
        0 => KeyCode::Key1,
        1 => KeyCode::Key2,
        2 => KeyCode::Key3,
        3 => KeyCode::Key4,
        4 => KeyCode::Key5,
        5 => KeyCode::Key6,
        6 => KeyCode::Key7,
        7 => KeyCode::Key8,
        8 => KeyCode::Key9,
        _ => return false,
    };
    is_key_pressed(key)
}

/// A stable key identifying the blocking council decision currently up, if any
/// (`E:{template}` for an event, `D:{dilemma}` for a legacy dilemma). Drives both
/// the typewriter reveal clock and the auto-resolve countdown (real-time loop §2).
fn current_decision_key(sim: &SimState) -> Option<String> {
    if let Some(review) = &sim.authority.pending_review {
        Some(format!("A:{}", review.proposed.label()))
    } else if let Some(p) = &sim.pending_event {
        Some(format!("E:{}:{}", p.template_id, p.rolled_month_clock))
    } else {
        sim.pending_dilemma
            .as_ref()
            .map(|p| format!("D:{}:{}", p.dilemma_id, p.rolled_month_clock))
    }
}

pub struct Game {
    data: GameData,
    state: GameState,
    chronicle: ChronicleStore,
    /// Cross-playthrough achievements (GDD §10), persisted separately.
    achievements: Achievements,
    notifications: NotificationManager,
    /// Procedural ambience and redundant state-change cues.
    audio: AudioManager,
    events: EventBus<UiAction>,
    /// The one piece of bitmap art the game ships: the title plate behind the
    /// main menu. Everything else is drawn (GDD §0), so the manifest stays tiny.
    assets: AssetManager,
    /// Legacy ids in stable sorted order for the menu.
    legacy_ids: Vec<String>,
    /// Screen-space phosphor-monitor overlay (scanlines, vignette, flicker),
    /// drawn on top of every frame. Toggle with F10; tune via the F1 panel.
    crt: CrtOverlay,
    /// Cached overlay style derived from `display`.
    crt_style: CrtStyle,
    /// Persisted CRT display preferences.
    display: DisplaySettings,
    /// Persisted default council delegation applied to each new voyage (§5.4).
    delegation_defaults: crate::state::sim::DelegationSettings,
    /// Whether the F1 display-settings overlay is open.
    settings_open: bool,
    /// Whether the F2 help/controls overlay is open.
    help_open: bool,
    /// One-time onboarding flags (persisted separately from the campaign save).
    onboarding: crate::settings::Onboarding,
    /// Whether the first-run welcome overlay is showing (once per install).
    welcome_open: bool,
    /// Whether the guided tutorial overlay is currently open this session.
    tutorial_open: bool,
    /// Terminal typewriter reveal for blocking modals: which modal is showing
    /// and when it appeared, so its body text streams in. Purely cosmetic —
    /// never touches the deterministic sim.
    modal_key: Option<String>,
    modal_started: f64,
    /// Wall-clock `get_time()` when the current mission's charter was accepted,
    /// for the cosmetic run timer (PLAN M4.7). Session-local; never touches the
    /// deterministic sim.
    mission_started: Option<f64>,
    /// Real seconds the last completed mission took, shown in the drydock
    /// Homecoming until the next charter is accepted.
    last_mission_real_secs: Option<f32>,
    /// Capture-only override so the run timer is deterministic in screenshots.
    capture_run_secs: Option<f32>,
    /// Ship's-log stream clock: last-seen entry count and when it last grew,
    /// so the newest line types in. Cosmetic; never touches the sim.
    log_len: usize,
    log_started: f64,
    /// Reveal text instantly (screenshot capture) instead of typing it out.
    instant_reveal: bool,
    /// Capture-only: freeze the ship's-log stream at this elapsed time.
    capture_log_reveal: Option<f32>,
    /// One-shot power-on boot log shown before the menu on launch.
    boot: BootScreen,
    /// Real-time auto-advance accumulator (real-time loop §1): real seconds banked
    /// toward the next month tick while under way. Reset whenever time is not
    /// advancing (docked, paused, blocked).
    month_accumulator: f32,
    /// The blocking decision the auto-resolve countdown is currently timing
    /// (`E:{id}` / `D:{id}`), and the wall-clock it started at (real-time loop §2).
    /// Reset when the pending decision changes.
    decision_key: Option<String>,
    decision_elapsed: f32,
    /// Session-local institution picker. The resulting custody is persistent;
    /// merely opening this overlay is not simulation state.
    custody_picker: Option<String>,
    /// Session-local obligation whose full ledger history is open.
    obligation_detail: Option<String>,
    /// Smooth-scroll state for the drydock charter board / PREP swap column,
    /// which now outgrows its panel (grouped by objective, all tiers listed).
    /// A `Cell` so the pure-view draw path can update it through `&GameplayCtx`.
    charter_scroll: Cell<ScrollArea>,
    description_scroll: Cell<ScrollArea>,
    abort_confirm: Cell<bool>,
    /// Smooth-scroll state for the SHIP builder's three catalog columns
    /// (Hull/Engine/Weapon), which overflow once a mission-reward part joins a
    /// full column. One `ScrollArea` per column.
    ship_scroll: Cell<[ScrollArea; 3]>,
    /// Smooth-scroll state for the CREW dynasty roster. The list used to be
    /// truncated to whatever fitted the panel, so a dynasty of five showed two
    /// and told you about the rest.
    roster_scroll: Cell<ScrollArea>,
    /// Smooth-scroll state for the CHRONICLE log, which grows across
    /// playthroughs and long ago outgrew the nine entries it used to show.
    chronicle_scroll: Cell<ScrollArea>,
    /// CHRONICLE sub-tab: current voyage records or completed mission archive.
    chronicle_records_tab: Cell<bool>,
    /// Smooth-scroll state for active duties. Obligations accumulate independently
    /// of the record/archive sub-tab and must remain reachable on long voyages.
    obligations_scroll: Cell<ScrollArea>,
    /// Obligation ledger sub-tab: active duties or resolved archive.
    obligation_resolved_tab: Cell<bool>,
    /// Scroll state inside the selected obligation's history overlay.
    obligation_history_scroll: Cell<ScrollArea>,
    /// Smooth-scroll state for the homecoming debrief's chain-of-command list:
    /// a century-long charter can pass through more captains than the panel
    /// holds, and the point of the list is that none of them is dropped.
    debrief_commanders_scroll: Cell<ScrollArea>,
    /// Smooth-scroll state for the homecoming debrief's voyage log, which holds
    /// up to `voyage_highlight_limit` remembered beats.
    debrief_log_scroll: Cell<ScrollArea>,
    /// Smooth-scroll state for the Custodian Agenda catalogue and project log.
    agenda_scroll: Cell<ScrollArea>,
    agenda_readiness_scroll: Cell<ScrollArea>,
    /// Session-local cancellation preview for one Agenda job.
    project_cancel_confirm: Cell<Option<u64>>,
    /// SHIP builder sub-tab: `false` = LOADOUT (hull/engine/weapon catalog),
    /// `true` = MODULES (the six subsystems' named version ladders). Pure view
    /// state, flipped by the on-screen toggle.
    ship_modules_tab: Cell<bool>,
    ship_preview: Cell<(u64, f64)>,
    presentation: ui::presentation::Presentation,
}

impl Game {
    pub async fn new() -> Self {
        // Seed macroquad's shared generator from the wall clock so `random_u64`
        // (the per-campaign seed source) actually varies launch to launch — left
        // unseeded it starts from a fixed default, so every new game came out the
        // same. A `fixed_seed` in game_config still overrides this for testing.
        macroquad_toolkit::rng::srand((macroquad::miniquad::date::now() * 1000.0) as u64);
        if let Err(error) = macroquad_toolkit::ui::set_default_ui_font_from_bytes(include_bytes!(
            "../assets/fonts/DejaVuSans.ttf"
        )) {
            eprintln!("Body font unavailable; using bundled display fallback: {error}");
        }
        let data = build_mode::load_data();
        let chronicle = ChronicleStore::load(
            &data.config.game_name,
            &data.config.chronicle_slot,
            &data.config.version,
        );
        // Menu display order: Preservers first — the founders' path reads as
        // the intuitive default — then the rest in stable sorted-id order.
        // Purely cosmetic: legacy choice is the player's, never RNG-driven.
        let mut legacy_ids = GameData::sorted_ids(&data.legacies);
        legacy_ids.sort_by_key(|id| match id.as_str() {
            "preservers" => 0,
            "adaptors" => 1,
            "wanderers" => 2,
            _ => 3,
        });
        let save_exists = save::save_exists(&data.config);
        let display = DisplaySettings::load(&data.config.game_name);
        let crt_style = display.crt_style();
        ui::term::set_phosphor(display.phosphor);
        macroquad_toolkit::ui::set_ui_scale(display.ui_scale);
        let delegation_defaults = crate::settings::load_delegation(&data.config.game_name);
        let audio = AudioManager::new().await;

        let mut assets = AssetManager::new();
        let _ = assets.load_asset_pack("assets.zip").await;
        let _ = assets.load_texture_configs(&data.texture_manifest).await;

        let achievements = crate::achievements::load(&data.config.game_name);
        // The first-run welcome overlay is triggered by the first NEW GAME (see
        // GoToNewGame), not on boot — the main menu shows clean. The flag decides
        // whether that first NEW GAME opens it.
        let onboarding = crate::settings::Onboarding::load(&data.config.game_name);

        Self {
            data,
            state: GameState::Menu(MenuState::new(save_exists)),
            chronicle,
            achievements,
            notifications: NotificationManager::new(),
            audio,
            events: EventBus::new(),
            assets,
            legacy_ids,
            crt: CrtOverlay::new(),
            crt_style,
            display,
            delegation_defaults,
            settings_open: false,
            help_open: false,
            onboarding,
            welcome_open: false,
            tutorial_open: false,
            modal_key: None,
            modal_started: 0.0,
            mission_started: None,
            last_mission_real_secs: None,
            capture_run_secs: None,
            log_len: 0,
            log_started: 0.0,
            instant_reveal: false,
            capture_log_reveal: None,
            boot: BootScreen::new(),
            month_accumulator: 0.0,
            decision_key: None,
            decision_elapsed: 0.0,
            custody_picker: None,
            obligation_detail: None,
            charter_scroll: Cell::new(ScrollArea::new()),
            description_scroll: Cell::new(ScrollArea::new()),
            abort_confirm: Cell::new(false),
            ship_scroll: Cell::new([ScrollArea::new(); 3]),
            roster_scroll: Cell::new(ScrollArea::new()),
            chronicle_scroll: Cell::new(ScrollArea::new()),
            chronicle_records_tab: Cell::new(true),
            obligations_scroll: Cell::new(ScrollArea::new()),
            obligation_resolved_tab: Cell::new(false),
            obligation_history_scroll: Cell::new(ScrollArea::new()),
            debrief_commanders_scroll: Cell::new(ScrollArea::new()),
            debrief_log_scroll: Cell::new(ScrollArea::new()),
            agenda_scroll: Cell::new(ScrollArea::new()),
            agenda_readiness_scroll: Cell::new(ScrollArea::new()),
            project_cancel_confirm: Cell::new(None),
            ship_modules_tab: Cell::new(false),
            ship_preview: Cell::new((0, -100.0)),
            presentation: Default::default(),
        }
    }

    pub fn update(&mut self, dt: f32) {
        self.notifications.update(dt);

        // First-run welcome overlay sits above the menu once the boot log ends.
        // Any key dismisses it (its button is handled in draw); until then, no
        // other input runs and any menu intents queued beneath it are dropped.
        if self.boot.is_done() && self.welcome_open {
            if get_last_key_pressed().is_some() {
                self.dismiss_welcome();
            }
            let _ = self.events.drain();
            return;
        }

        // F10 toggles the CRT effect outright; F1 opens the display panel.
        if is_key_pressed(KeyCode::F10) {
            self.display.crt_enabled = !self.display.crt_enabled;
            self.persist_display();
        }
        if self.boot.is_done() && is_key_pressed(KeyCode::F1) {
            self.settings_open = !self.settings_open;
            self.presentation.overlay_scroll.set(ScrollArea::new());
            self.help_open = false;
        }
        if self.boot.is_done() && is_key_pressed(KeyCode::F2) {
            self.help_open = !self.help_open;
            self.settings_open = false;
        }
        // Esc closes whichever panel is open (help first, then settings).
        if is_key_pressed(KeyCode::Escape) {
            if self.help_open {
                self.help_open = false;
            } else if self.settings_open {
                self.settings_open = false;
            }
        }

        // Boot log plays once before the menu; any input skips it. Capture mode
        // freezes it at a seeked frame (instant_reveal), so don't advance then.
        if !self.boot.is_done() && matches!(self.state, GameState::Menu(_)) {
            if !self.instant_reveal {
                self.boot.update(dt);
                if is_mouse_button_pressed(MouseButton::Left) || get_last_key_pressed().is_some() {
                    self.boot.finish();
                }
            }
            return;
        }

        let mut actions: Vec<UiAction> = self.events.drain().collect();
        self.gather_keyboard_actions(&mut actions);
        let mut transition = None;
        for action in actions {
            self.audio.cue(Cue::Button, self.display.audio_volume);
            if matches!(
                action,
                UiAction::ResolveEvent(_) | UiAction::ResolveDilemma(_)
            ) {
                self.audio.cue(Cue::Resolution, self.display.audio_volume);
            }
            let tutorial_action = action.clone();
            if let Some(t) = self.apply_action(action) {
                transition = Some(t);
            }
            self.track_tutorial(&tutorial_action);
        }
        if let Some(transition) = transition {
            self.transition(transition);
        }

        // Drive real time last, once input has been applied (real-time loop §1/§2):
        // auto-advance the month clock under way, or run the decision countdown.
        self.update_realtime(dt);
        let ambience = self.display.ambience
            && matches!(&self.state, GameState::Gameplay(gameplay) if gameplay.sim.contract.is_some() && gameplay.sim.debrief.is_none() && !gameplay.sim.dynasty.extinct && gameplay.sim.terminal.is_none());
        self.audio
            .update_ambience(ambience, self.display.audio_volume);
    }

    /// Terminal-style keyboard navigation. On the menu, number keys pick a
    /// legacy, arrows move the selection, Enter begins the voyage. In gameplay,
    /// a blocking council modal takes the number keys for its options, otherwise
    /// the number keys switch screen tabs. Time advances on its own (real-time
    /// loop §1), so there is no manual step key. Suppressed while the settings or
    /// help panel is up.
    fn gather_keyboard_actions(&mut self, actions: &mut Vec<UiAction>) {
        if matches!(self.state, GameState::Gameplay(_)) && is_key_pressed(KeyCode::Space) {
            actions.push(UiAction::TogglePause);
        }
        if self.settings_open || self.help_open {
            return;
        }

        // Menu: keyboard is only wired on the new-game picker (terminals are
        // keyboard-first); the main-menu screen is mouse-driven.
        if let GameState::Menu(menu) = &self.state {
            if menu.phase != crate::state::MenuPhase::NewGame {
                return;
            }
            let selected = menu.selected_legacy;
            let count = self.legacy_ids.len();
            for i in 0..count {
                if digit_pressed(i) {
                    actions.push(UiAction::SelectLegacy(i));
                }
            }
            if is_key_pressed(KeyCode::Up) {
                actions.push(UiAction::SelectLegacy(selected.saturating_sub(1)));
            }
            if is_key_pressed(KeyCode::Down) {
                actions.push(UiAction::SelectLegacy(
                    (selected + 1).min(count.saturating_sub(1)),
                ));
            }
            if is_key_pressed(KeyCode::Enter) {
                actions.push(UiAction::StartNewGame);
            }
            if is_key_pressed(KeyCode::Escape) {
                actions.push(UiAction::BackToMainMenu);
            }
            return;
        }

        let GameState::Gameplay(gameplay) = &self.state else {
            return;
        };
        let sim = &gameplay.sim;

        // A pending council decision claims the number keys for its choices.
        if let Some(pending) = &sim.pending_event {
            if let Some(template) = self.data.events.get(&pending.template_id) {
                for i in 0..template.outcomes.len() {
                    if digit_pressed(i) {
                        actions.push(UiAction::ResolveEvent(i));
                    }
                }
            }
            return;
        }
        if sim.pending_dilemma.is_some() {
            if let Some(dilemma) = legacy::pending_dilemma_def(sim, &self.data) {
                for i in 0..dilemma.options.len() {
                    if digit_pressed(i) {
                        actions.push(UiAction::ResolveDilemma(i));
                    }
                }
            }
            return;
        }

        // Number keys switch tabs within the current voyage state's set (real-time
        // loop §5). Time advances on its own now — there is no manual step key.
        let in_port = sim.contract.is_none();
        for (i, screen) in crate::state::Screen::tabs(in_port).iter().enumerate() {
            if digit_pressed(i) {
                actions.push(UiAction::SelectScreen(*screen));
            }
        }
    }

    /// Re-derive the cached CRT style from the current settings and save them.
    fn persist_display(&mut self) {
        self.crt_style = self.display.crt_style();
        ui::term::set_phosphor(self.display.phosphor);
        macroquad_toolkit::ui::set_ui_scale(self.display.ui_scale);
        if let Err(err) = self.display.save(&self.data.config.game_name) {
            self.notifications
                .warning(format!("Display settings not saved: {err}"));
        }
    }

    /// Close the first-run welcome overlay and remember it was seen, so it never
    /// shows again on this install.
    fn dismiss_welcome(&mut self) {
        if !self.welcome_open {
            return;
        }
        self.welcome_open = false;
        self.onboarding.welcome_seen = true;
        if let Err(err) = self.onboarding.save(&self.data.config.game_name) {
            self.notifications
                .warning(format!("Onboarding flag not saved: {err}"));
        }
    }

    /// Apply an intent from the display-settings overlay.
    fn apply_display_action(&mut self, action: crate::ui::settings::DisplayAction) {
        use crate::ui::settings::DisplayAction;
        match action {
            DisplayAction::ToggleCrt => self.display.crt_enabled = !self.display.crt_enabled,
            DisplayAction::ToggleScanlines => self.display.scanlines = !self.display.scanlines,
            DisplayAction::ToggleFlicker => self.display.flicker = !self.display.flicker,
            DisplayAction::SetPhosphor(p) => self.display.phosphor = p,
            DisplayAction::SetUiScale(scale) => {
                self.display.ui_scale = macroquad_toolkit::ui::sanitize_ui_scale(scale);
                self.presentation
                    .overlay_scroll
                    .set(macroquad_toolkit::ui::ScrollArea::new());
                self.presentation
                    .mobile_scroll
                    .set(macroquad_toolkit::ui::ScrollArea::new());
            }
            DisplayAction::SetAudio(value) => self.display.audio_volume = value.clamp(0.0, 1.0),
            DisplayAction::SetTextScale(value) => {
                self.display.text_scale = value.clamp(0.75, 1.5);
                self.presentation.overlay_scroll.set(ScrollArea::new());
            }
            DisplayAction::ToggleAmbience => self.display.ambience = !self.display.ambience,
            DisplayAction::ToggleTutorial => {
                self.display.tutorial_enabled = !self.display.tutorial_enabled;
                if self.display.tutorial_enabled {
                    if let GameState::Gameplay(gameplay) = &self.state {
                        self.tutorial_open = !gameplay.sim.tutorial_dismissed;
                    }
                } else {
                    self.tutorial_open = false;
                }
            }
            DisplayAction::ToggleDelegationDefault(category) => {
                self.delegation_defaults.toggle(category);
                if let Err(err) = crate::settings::save_delegation(
                    &self.delegation_defaults,
                    &self.data.config.game_name,
                ) {
                    self.notifications
                        .warning(format!("Delegation defaults not saved: {err}"));
                }
                return; // no CRT re-derive needed
            }
            DisplayAction::Close => self.settings_open = false,
        }
        self.persist_display();
    }

    /// Seconds since the current blocking modal appeared, resetting the clock
    /// whenever a different modal takes over. Returns a large value (instant
    /// reveal) in capture mode; the value is unused when no modal is showing.
    fn modal_reveal(&mut self) -> f32 {
        if self.instant_reveal {
            return f32::MAX;
        }
        let key = match &self.state {
            GameState::Gameplay(g) => current_decision_key(&g.sim),
            _ => None,
        };
        if key != self.modal_key {
            self.modal_key = key;
            self.modal_started = get_time();
        }
        (get_time() - self.modal_started) as f32
    }

    /// Seconds since the newest ship's-log line appeared, resetting whenever the
    /// log grows so the latest entry streams in. Large (instant) in capture or
    /// outside gameplay.
    fn log_reveal(&mut self) -> f32 {
        if let Some(frozen) = self.capture_log_reveal {
            return frozen;
        }
        if self.instant_reveal {
            return f32::MAX;
        }
        let GameState::Gameplay(gameplay) = &self.state else {
            return f32::MAX;
        };
        let len = gameplay.sim.log.len();
        if len != self.log_len {
            self.log_len = len;
            self.log_started = get_time();
        }
        (get_time() - self.log_started) as f32
    }

    fn run_clock_for(&self, sim: &SimState) -> Option<f32> {
        if let Some(secs) = self.capture_run_secs {
            return Some(secs);
        }
        if sim.contract.is_some() {
            self.mission_started.map(|t| (get_time() - t) as f32)
        } else {
            self.last_mission_real_secs
        }
    }

    fn transition(&mut self, transition: StateTransition) {
        self.project_cancel_confirm.set(None);
        self.presentation.project_cancellation.set(None);
        // Any state change clears the session-local run timer (PLAN M4.7).
        self.mission_started = None;
        self.last_mission_real_secs = None;
        match transition {
            StateTransition::NewCampaign {
                legacy_id,
                seed,
                faction_ids,
            } => {
                let mut sim = SimState::new_campaign(&self.data, &legacy_id, seed, &faction_ids);
                // A new dynasty inherits a head start from the Chronicle (§7)
                // and the player's default council delegation (§5.4).
                sim.delegation = self.delegation_defaults;
                let heritage = crate::heritage::derive(&self.chronicle, &self.data.config.heritage);
                crate::heritage::apply(&mut sim, &heritage);
                self.state = GameState::Gameplay(Box::new(GameplayState::new(sim)));
                self.tutorial_open = self.display.tutorial_enabled;
                if heritage.has_bonus() {
                    self.notifications.success(format!(
                        "The {} heritage steadies the founding oath.",
                        heritage.tier_name
                    ));
                } else {
                    self.notifications
                        .success("The founding generation takes its oath.");
                }
            }
            StateTransition::LoadCampaign => match save::load_campaign(&self.data.config) {
                Ok(mut sim) => {
                    if sim.terminal.is_none() && sim.dynasty.extinct {
                        crate::simulation::survival::check_and_record(&mut sim, &self.data);
                    }
                    // Older slideshow saves may resume after their launch steps.
                    if sim.has_pending_decision() && sim.tutorial_step < 10 {
                        sim.tutorial_step = 10;
                    } else if sim.contract.is_some() && sim.tutorial_step < 6 {
                        sim.tutorial_step = 6;
                    } else if sim.contract.is_none()
                        && sim.tutorial_step >= 6
                        && !sim.tutorial_dismissed
                    {
                        sim.tutorial_step = 0;
                    }
                    self.state = GameState::Gameplay(Box::new(GameplayState::new(sim)));
                    self.tutorial_open = self.display.tutorial_enabled;
                    self.notifications.success("Voyage resumed.");
                }
                Err(err) => self.notifications.danger(format!("Load failed: {err}")),
            },
            StateTransition::ToMenu => {
                if let GameState::Gameplay(gameplay) = &self.state {
                    match save::save_campaign(&self.data.config, &gameplay.sim) {
                        Ok(()) => self.notifications.info("Voyage autosaved."),
                        Err(err) => self.notifications.danger(format!("Autosave failed: {err}")),
                    }
                }
                self.state = GameState::Menu(MenuState::new(save::save_exists(&self.data.config)));
            }
        }
        self.check_achievements();
    }

    /// Unlock any achievements the current state satisfies, notifying once each
    /// and persisting on change. Cheap to call on any state change.
    fn check_achievements(&mut self) {
        let ids: Vec<&'static str> = match &self.state {
            GameState::Gameplay(gameplay) => {
                crate::achievements::evaluate(&gameplay.sim, &self.chronicle)
            }
            GameState::Menu(_) => return,
        };
        let mut changed = false;
        for id in ids {
            if self.achievements.unlock(id) {
                changed = true;
                if let Some(achievement) = self.achievements.get(id) {
                    self.notifications
                        .success(format!("Achievement unlocked: {}", achievement.name));
                }
            }
        }
        if changed && !self.instant_reveal {
            let _ = crate::achievements::save(&self.achievements, &self.data.config.game_name);
        }
    }
}
