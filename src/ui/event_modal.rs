//! Blocking council-decision modals: events (GDD §9 step 4) and legacy
//! dilemmas (GDD §5.5).

use crate::simulation::legacy::pending_dilemma_def;
use crate::ui::{logical_height, logical_width, term, term_button, GameplayCtx, UiAction};
use macroquad::prelude::*;
use macroquad_toolkit::prelude::*;
use macroquad_toolkit::ui::{draw_ui_text_ex, RectExt};

/// Characters-per-second for the terminal reveal of modal body text.
const REVEAL_CPS: f32 = 55.0;

mod council;

pub use council::draw;

/// Blocking legacy-dilemma modal. Options show their success odds up front —
/// the roll is honest, so the interface is too (Pillar 3).
pub fn draw_dilemma(ctx: &GameplayCtx<'_>, pointer: Pointer, actions: &mut Vec<UiAction>) {
    let Some(dilemma) = pending_dilemma_def(ctx.sim, ctx.data) else {
        return;
    };
    macroquad_toolkit::ui::occlude(Rect::new(
        0.0,
        72.0,
        logical_width(),
        logical_height() - 72.0,
    ));
    let legacy_name = ctx
        .data
        .legacies
        .get(&ctx.sim.legacy.legacy_id)
        .map(|l| l.name.clone())
        .unwrap_or_default();

    let header = format!("LEGACY DILEMMA — {}", legacy_name.to_uppercase());
    let content = modal_frame(
        &header,
        countdown_secs(ctx.decision_remaining),
        dilemma.options.len(),
        0.0,
        term::primary(),
    );
    // Drop the title clear of the header divider — at the old offset its caps
    // sat right on the rule and read as cramped.
    let mut y = content.y + 48.0;
    draw_ui_text_ex(
        &dilemma.title,
        content.x,
        y,
        TextStyle::new(22.0, term::primary()).params(),
    );
    y += 20.0;
    draw_typed_block(
        &dilemma.description,
        content.x,
        y,
        content.w,
        ctx.modal_reveal,
    );
    y += 84.0;

    for (i, option) in dilemma.options.iter().enumerate() {
        let card = Rect::new(content.x, y, content.w, 92.0);
        draw_surface(
            card,
            &SurfaceStyle::new(Color::new(0.08, 0.065, 0.015, 1.0)).with_border(1.0, term::faint()),
        );
        let odds = crate::simulation::legacy::dilemma_odds(ctx.sim, ctx.data, option);
        // The shown odds are honest (Pillar 3): a combat-backed Wanderer gamble or
        // a faction-backed one reads higher, a faction-hindered one lower.
        let modifier = odds - option.success_chance;
        let odds_text = if modifier > 0.001 {
            format!(
                "Success odds: {:.0}%  (+{:.0}%)",
                odds * 100.0,
                modifier * 100.0
            )
        } else if modifier < -0.001 {
            format!(
                "Success odds: {:.0}%  ({:.0}%)",
                odds * 100.0,
                modifier * 100.0
            )
        } else {
            format!("Success odds: {:.0}%", odds * 100.0)
        };
        draw_ui_text_ex(
            &odds_text,
            card.x + 14.0,
            card.y + 24.0,
            TextStyle::new(13.0, term::accent()).params(),
        );
        draw_text_block(
            &option.success.log,
            card.x + 14.0,
            card.y + 34.0,
            card.w - 264.0,
            40.0,
            12.0,
            3.0,
            term::faint(),
        );
        if term_button(
            Rect::new(card.right() - 244.0, card.y + 18.0, 230.0, 56.0),
            &format!("[{}] {}", i + 1, option.label.to_uppercase()),
            true,
            pointer,
        ) {
            actions.push(UiAction::ResolveDilemma(i));
        }
        y += 104.0;
    }
}

/// Whole seconds left on the auto-resolve countdown, floored at 0 (real-time
/// loop §2).
fn countdown_secs(remaining: f32) -> i32 {
    remaining.ceil().max(0.0) as i32
}

/// Human phrasing of a population-impact band (real-time loop §3), with a tone:
/// a loss band reads warm-red, a gain accent, a straddle neutral.
fn impact_label(lo: i64, hi: i64) -> (String, Color) {
    if hi <= 0 {
        (
            format!("~ {}–{} souls may be lost", hi.abs(), lo.abs()),
            term::alert(),
        )
    } else if lo >= 0 {
        (format!("~ {lo}–{hi} souls may join"), term::accent())
    } else {
        (format!("~ {lo} to +{hi} souls"), term::dim())
    }
}

pub(crate) fn known_effects(
    outcome: &crate::data::events::EventOutcome,
    population_range: Option<(i64, i64)>,
) -> (String, Color) {
    let mut effects = Vec::new();
    let r = outcome.resource_delta;
    for (label, value) in [
        ("Credits", r.credits),
        ("Energy", r.energy),
        ("Minerals", r.minerals),
        ("Food", r.food),
        ("Influence", r.influence),
    ] {
        if value != 0 {
            effects.push(format!("{label} {value:+}"));
        }
    }
    let s = outcome.ship_delta;
    for (label, value) in [
        ("hull", s.hull_integrity),
        ("life", s.life_support),
        ("fuel", s.fuel),
    ] {
        if value.abs() > f32::EPSILON {
            effects.push(format!("{label} {:+.0}%", value * 100.0));
        }
    }
    if s.spare_parts != 0 {
        effects.push(format!("parts {:+}", s.spare_parts));
    }
    let p = outcome.population_delta;
    for (label, value) in [
        ("morale", p.morale),
        ("unity", p.unity),
        ("stability", p.stability),
        ("loyalty", p.legacy_loyalty),
        ("adapt", p.adaptation),
        ("drift", p.cultural_drift),
    ] {
        if value.abs() > f32::EPSILON {
            effects.push(format!("{label} {:+.0}%", value * 100.0));
        }
    }
    for delta in &outcome.faction_approval_deltas {
        if delta.delta.abs() > f32::EPSILON {
            effects.push(format!(
                "{} approval {:+.0}%",
                delta.id.replace('_', " "),
                delta.delta * 100.0
            ));
        }
    }
    for delta in &outcome.reputation_deltas {
        if delta.delta.abs() > f32::EPSILON {
            let label = if delta.id == "custodian_empathy" {
                "AI empathy".to_owned()
            } else {
                format!("{} reputation", delta.id.replace('_', " "))
            };
            effects.push(format!("{label} {:+.0}%", delta.delta * 100.0));
        }
    }
    for delta in &outcome.subsystem_deltas {
        let name = delta.id.replace('_', " ");
        if delta.condition.abs() > f32::EPSILON {
            effects.push(format!("{name} condition {:+.0}%", delta.condition * 100.0));
        }
        if delta.knowledge.abs() > f32::EPSILON {
            effects.push(format!("{name} knowledge {:+.0}%", delta.knowledge * 100.0));
        }
    }
    if outcome.objective_progress_delta.abs() > f32::EPSILON {
        let pct = outcome.objective_progress_delta * 100.0;
        let amount = if pct.abs() < 0.1 {
            format!("{pct:+.2}%")
        } else if pct.abs() < 1.0 {
            format!("{pct:+.1}%")
        } else {
            format!("{pct:+.0}%")
        };
        effects.push(format!("objective {amount}"));
    }
    if outcome.force_return {
        effects.push("forced return".to_owned());
    }
    if outcome.faction_loss.is_some() {
        effects.push("faction may leave".to_owned());
    }
    if outcome.designate_heir {
        effects.push("names the ready heir".to_owned());
    }
    if let Some(followup) = &outcome.schedule_followup {
        effects.push(format!("follow-up in {}y", followup.delay_years));
    } else if !outcome.long_term_consequences.is_empty() {
        effects.push("future consequence".to_owned());
    }

    let mut color = term::accent();
    let mut text = if effects.is_empty() {
        "KNOWN: no immediate material change".to_owned()
    } else {
        format!("KNOWN: {}", effects.join(" · "))
    };
    if let Some((lo, hi)) = population_range {
        let (impact, impact_color) = impact_label(lo, hi);
        text.push_str(&format!(" · UNCERTAIN: {impact}"));
        color = impact_color;
    }
    (text, color)
}

/// Dim the world and draw the modal surface with `header` centered in the title
/// band and the human fallback `countdown` beside it. Tall cards move the
/// countdown below the header so it cannot collide with the global time controls
/// above the gameplay shell (real-time loop §2); returns the content rect.
fn modal_frame(
    header: &str,
    countdown: i32,
    option_count: usize,
    extra_height: f32,
    accent: Color,
) -> Rect {
    draw_rectangle(
        0.0,
        0.0,
        logical_width(),
        logical_height(),
        Color::new(0.0, 0.0, 0.0, 0.75),
    );

    // Taller cards (bigger buttons) and the lower title need more room; a wider
    // frame lets the option labels breathe on two comfortable lines.
    let height = 210.0 + option_count as f32 * 104.0 + extra_height;
    let rect = Rect::new(
        logical_width() / 2.0 - 350.0,
        (logical_height() - height) / 2.0,
        700.0,
        height,
    );
    let header_h = 40.0;
    draw_surface(
        rect,
        &SurfaceStyle::new(Color::new(0.06, 0.05, 0.012, 1.0))
            .with_border(2.0, accent)
            .with_header(header_h, term::panel_header())
            .with_header_divider(1.0, accent),
    );
    draw_text_centered_in_box_ex(
        header,
        rect.x,
        rect.y,
        rect.w,
        header_h,
        TextStyle::new(15.0, accent),
    );
    // Tall cards can reach the shell's top controls. Move their countdown just
    // below the header; the body is left-aligned, so this right-side label stays
    // clear of the event title and advisor copy.
    let countdown_y = if rect.y < 84.0 {
        rect.y + header_h + 18.0
    } else {
        rect.y + header_h * 0.5 + 4.0
    };
    draw_text_right(
        &format!("CAPTAIN FALLBACK {countdown}s"),
        rect.right() - 16.0,
        countdown_y,
        TextStyle::new(11.0, accent),
    );
    rect.inset(26.0)
}

/// Word-wrapped body text revealed left-to-right terminal style, with a
/// blinking underscore cursor while it is still typing.
fn draw_typed_block(text: &str, x: f32, y: f32, w: f32, reveal: f32) {
    let shown = typed_prefix(text, reveal, REVEAL_CPS);
    let cursor = if !is_fully_typed(text, reveal, REVEAL_CPS) && blink(reveal, 2.5) {
        "_"
    } else {
        ""
    };
    draw_text_block(
        &format!("{shown}{cursor}"),
        x,
        y,
        w,
        70.0,
        14.0,
        4.0,
        term::dim(),
    );
}

#[cfg(test)]
mod tests;
