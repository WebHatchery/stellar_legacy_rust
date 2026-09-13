//! The gameplay shell: the header, the tab strip, and the per-frame
//! dispatch into whichever screen module is showing.

use super::*;

pub struct GameplayCtx<'a> {
    pub presentation: &'a presentation::Presentation,
    pub data: &'a GameData,
    pub sim: &'a SimState,
    pub screen: Screen,
    pub chronicle: &'a ChronicleStore,
    pub achievements: &'a Achievements,
    /// This frame's pointer, in logical coordinates — a mouse or a finger,
    /// asked the same way. Built once in `game.rs` so every control on the
    /// screen agrees about where it is and whether it just let go.
    pub pointer: Pointer,
    /// Seconds since the current blocking modal appeared, for the terminal
    /// typewriter reveal. Large/instant when the effect is disabled.
    pub modal_reveal: f32,
    /// Seconds since the newest ship's-log entry appeared, so it streams in
    /// like live console output. Large/instant in capture.
    pub log_reveal: f32,
    /// Cosmetic wall-clock run timer (PLAN M4.7): elapsed real seconds for the
    /// current mission (live), or the last mission's while in port. `None`
    /// before the first charter. Never feeds the deterministic sim.
    pub run_clock: Option<f32>,
    /// Real seconds left before a blocking decision uses its authored human
    /// fallback (real-time loop §2). Only meaningful while a decision is
    /// pending; the modal renders it as a countdown.
    pub decision_remaining: f32,
    /// Discipline whose custodianship picker is open, if any. Session-local UI
    /// state; the completed grant alone enters the deterministic simulation.
    pub custody_picker: Option<&'a str>,
    /// Obligation whose full persistent history is open, if any.
    pub obligation_detail: Option<&'a str>,
    /// Smooth-scroll state for the charter board / PREP swap column (the list
    /// outgrows its panel). A `Cell` so this pure-view path can update the offset
    /// through the shared `&GameplayCtx` without threading `&mut` everywhere.
    pub description_scroll: &'a std::cell::Cell<macroquad_toolkit::ui::ScrollArea>,
    pub abort_confirm: &'a std::cell::Cell<bool>,
    pub charter_scroll: &'a std::cell::Cell<macroquad_toolkit::ui::ScrollArea>,
    /// Smooth-scroll state for the SHIP builder's three catalog columns, so a
    /// column that overflows (e.g. a mission-reward part added to a full one)
    /// stays reachable. Indexed Hull / Engine / Weapon.
    pub ship_scroll: &'a std::cell::Cell<[macroquad_toolkit::ui::ScrollArea; 3]>,
    /// Smooth-scroll state for the CREW dynasty roster, so a dynasty larger than
    /// the panel can be read rather than counted.
    pub roster_scroll: &'a std::cell::Cell<macroquad_toolkit::ui::ScrollArea>,
    /// Smooth-scroll state for the CHRONICLE log, which accumulates across
    /// playthroughs and outlives any single save.
    pub chronicle_scroll: &'a std::cell::Cell<macroquad_toolkit::ui::ScrollArea>,
    /// Chronicle sub-tab: `true` shows the current voyage's competing decision
    /// records; `false` shows the cross-campaign mission archive.
    pub chronicle_records_tab: &'a std::cell::Cell<bool>,
    /// Smooth-scroll state for the active obligations ledger.
    pub obligations_scroll: &'a std::cell::Cell<macroquad_toolkit::ui::ScrollArea>,
    /// Obligation ledger sub-tab: `false` active, `true` resolved.
    pub obligation_resolved_tab: &'a std::cell::Cell<bool>,
    /// Smooth-scroll state for the selected obligation's history.
    pub obligation_history_scroll: &'a std::cell::Cell<macroquad_toolkit::ui::ScrollArea>,
    /// Smooth-scroll state for the homecoming debrief's chain-of-command list —
    /// a long charter passes through more captains than the panel holds.
    pub debrief_commanders_scroll: &'a std::cell::Cell<macroquad_toolkit::ui::ScrollArea>,
    /// Smooth-scroll state for the homecoming debrief's voyage log.
    pub debrief_log_scroll: &'a std::cell::Cell<macroquad_toolkit::ui::ScrollArea>,
    /// Smooth-scroll state for the Custodian Agenda catalogue and project log.
    pub agenda_scroll: &'a std::cell::Cell<macroquad_toolkit::ui::ScrollArea>,
    pub agenda_readiness_scroll: &'a std::cell::Cell<macroquad_toolkit::ui::ScrollArea>,
    /// The one-frame cancellation preview selected by the Agenda.
    pub project_cancel_confirm: &'a std::cell::Cell<Option<u64>>,
    /// SHIP builder sub-tab: `false` = LOADOUT catalog, `true` = MODULES (named
    /// subsystem version ladders). Pure view state, flipped by the on-screen toggle.
    pub ship_preview: &'a std::cell::Cell<(u64, f64)>,
    pub ship_modules_tab: &'a std::cell::Cell<bool>,
    pub tutorial_enabled: bool,
    pub tutorial_open: bool,
}

pub fn draw_gameplay(ctx: GameplayCtx<'_>) -> Vec<UiAction> {
    if mobile::active() {
        return mobile::draw(&ctx);
    }
    if compact() {
        return responsive::draw(&ctx);
    }
    let mut actions = Vec::new();
    let pointer = ctx.pointer;

    // Extinction halts the voyage: a full-screen terminal takeover replaces the
    // normal screens (GDD §7).
    if ctx.sim.terminal.is_some() || ctx.sim.dynasty.extinct {
        draw_terminal_gameplay(&ctx, pointer, &mut actions);
        return actions;
    }

    // A charter that just concluded takes the screen the way extinction does —
    // the homecoming is the run's climax, and it used to pass by as two lines in
    // a log that scrolls. Reading it is the only thing on offer until the player
    // files the report.
    if ctx.sim.debrief.is_some() {
        debrief::draw(&ctx, pointer, &mut actions);
        return actions;
    }

    draw_header(&ctx);
    draw_tabs(&ctx, pointer, &mut actions);
    draw_screen(&ctx, pointer, &mut actions);
    draw_overlays(&ctx, pointer, &mut actions);
    actions
}

fn draw_terminal_gameplay(ctx: &GameplayCtx<'_>, pointer: Pointer, actions: &mut Vec<UiAction>) {
    if ctx.screen == Screen::Chronicle {
        draw_header(ctx);
        draw_tabs(ctx, pointer, actions);
        chronicle::draw(
            ctx,
            Rect::new(
                16.0,
                128.0,
                logical_width() - 32.0,
                logical_height() - 144.0,
            ),
            pointer,
            actions,
        );
    } else {
        game_over::draw(ctx, pointer, actions);
    }
}

fn draw_screen(ctx: &GameplayCtx<'_>, pointer: Pointer, actions: &mut Vec<UiAction>) {
    // Fall back to the dashboard if the open tab is not in the current voyage
    // state's set (real-time loop §5) — e.g. an old save resuming on CONTRACT
    // while docked, before the launch/dock clamps take effect.
    let in_port = ctx.sim.contract.is_none();
    let screen = if Screen::tabs(in_port).contains(&ctx.screen) {
        ctx.screen
    } else {
        Screen::Dashboard
    };

    let content = Rect::new(
        16.0,
        128.0,
        logical_width() - 32.0,
        logical_height() - 144.0,
    );
    let content_pointer = if ctx.presentation.utilities.get() {
        pointer.suppressed()
    } else {
        pointer
    };
    let _content_bounds = Region::new(content);
    match screen {
        Screen::Dashboard => bridge::draw(ctx, content, content_pointer, actions),
        Screen::Agenda => agenda::draw(ctx, content, content_pointer, actions),
        Screen::Drydock => contract_systems::draw_drydock(ctx, content, content_pointer, actions),
        Screen::ShipBuilder => ship_builder::draw(ctx, content, content_pointer, actions),
        Screen::Subsystems => subsystems::draw(ctx, content, content_pointer, actions),
        Screen::CrewDynasty => crew_dynasty::draw(ctx, content, content_pointer, actions),
        Screen::Contract => {
            contract_systems::draw_active_screen(ctx, content, content_pointer, actions)
        }
        Screen::Market => market::draw(ctx, content, content_pointer, actions),
        Screen::Chronicle => chronicle::draw(ctx, content, content_pointer, actions),
    }
}

fn draw_overlays(ctx: &GameplayCtx<'_>, pointer: Pointer, actions: &mut Vec<UiAction>) {
    navigation::draw_utilities(ctx, pointer, actions);

    // A pending authority decision blocks everything else (GDD §9 step 4):
    // discard screen intents and only accept the modal's.
    if ctx.sim.authority.pending_review.is_some() {
        actions.clear();
        authority_modal::draw(ctx, pointer, actions);
    } else if ctx.sim.pending_event.is_some() {
        actions.clear();
        event_modal::draw(ctx, pointer, actions);
    } else if ctx.sim.pending_dilemma.is_some() {
        actions.clear();
        event_modal::draw_dilemma(ctx, pointer, actions);
    }
    if ctx.sim.survival.warning_active && ctx.sim.terminal.is_none() {
        actions.clear();
        recovery_warning::draw(ctx, pointer, actions);
    }
    if ctx.abort_confirm.get() && !ctx.sim.has_pending_decision() {
        actions.clear();
        mission::draw_abort(ctx, pointer, actions);
    }
    if ctx.tutorial_enabled && ctx.tutorial_open && !ctx.sim.tutorial_dismissed {
        tutorial::draw(ctx, pointer, actions);
    }
}

fn draw_header(ctx: &GameplayCtx<'_>) {
    let rect = Rect::new(16.0, 12.0, logical_width() - 32.0, 58.0);
    term_panel(rect, None);

    let sim = ctx.sim;
    draw_text_glow(
        &ctx.data.config.display_name.to_uppercase(),
        rect.x + 16.0,
        rect.y + 36.0,
        TextStyle::new(24.0, term::primary()),
        0.12,
        2.0,
    );
    draw_ui_text_ex(
        "CUSTODIAN // SHIP INTELLIGENCE",
        rect.x + 16.0,
        rect.y + 54.0,
        TextStyle::new(10.0, term::accent()).params(),
    );

    let leader = sim
        .dynasty
        .leader()
        .map(|l| format!("CAPTAIN {} ({})", l.name, l.age))
        .unwrap_or_else(|| "CAPTAIN // NO LEADER".to_owned());
    let legacy = ctx
        .data
        .legacies
        .get(&sim.legacy.legacy_id)
        .map(|l| l.name.clone())
        .unwrap_or_default();
    // A live run timer while a mission is underway — the pacing gauge for the
    // ~30-min floor / ~1-hr cap (PLAN M4.7).
    let run_seg = if sim.contract.is_some() {
        ctx.run_clock
            .map(|secs| format!(" · RUN {}", format_mmss(secs)))
            .unwrap_or_default()
    } else {
        String::new()
    };
    draw_ui_text_ex(
        &format!(
            "Y{:03} · M{:02}  |  GEN {}  |  {}  |  {}{}",
            sim.year(),
            sim.month(),
            sim.dynasty.generation,
            legacy,
            leader,
            run_seg
        ),
        rect.x + 330.0,
        rect.y + 18.0,
        TextStyle::new(11.0, term::dim()).params(),
    );

    draw_ui_text_ex(
        &format!(
            "Credits {}  Energy {}  Minerals {}  Food {}  Influence {}",
            sim.resources.credits,
            sim.resources.energy,
            sim.resources.minerals,
            sim.resources.food,
            sim.resources.influence
        ),
        rect.x + 330.0,
        rect.y + 44.0,
        TextStyle::new(12.0, term::dim()).params(),
    );
}

fn draw_tabs(ctx: &GameplayCtx<'_>, pointer: Pointer, actions: &mut Vec<UiAction>) {
    navigation::draw(ctx, pointer, actions);
}
