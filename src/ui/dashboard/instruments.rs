//! Compact voyage instruments and truthful reserve summaries.
use super::*;

/// Keep an obligation identifiable inside the narrow instrument tile.
fn short_duty(title: &str) -> String {
    title
        .split_whitespace()
        .filter(|word| !word.eq_ignore_ascii_case("the"))
        .take(2)
        .collect::<Vec<_>>()
        .join(" ")
        .to_uppercase()
}

/// The bottom command strip complements (rather than duplicates) the primary
/// vital meters: mission state, the next interruption, the most urgent risk,
/// a causal maintenance label, and the live clock posture.
pub(super) fn draw_systems_strip(ctx: &GameplayCtx<'_>, rect: Rect) {
    term_panel(rect, None);
    let sim = ctx.sim;
    let inner = rect.inset(10.0);

    let contract = sim.contract.as_ref();
    let (phase, objective) = contract
        .map(|contract| {
            (
                contract.phase.label().to_uppercase(),
                format!("{:.0}% BANKED", contract.objective_fraction() * 100.0),
            )
        })
        .unwrap_or_else(|| ("DRYDOCK".to_owned(), "SELECT A WRIT".to_owned()));
    let pending_decision = sim
        .pending_event
        .as_ref()
        .and_then(|pending| ctx.data.events.get(&pending.template_id))
        .map(|event| event.title.to_uppercase())
        .or_else(|| {
            sim.pending_dilemma
                .as_ref()
                .map(|_| "LEGACY DILEMMA".to_owned())
        });
    let (decision_label, next_decision, decision_tone) = if let Some(title) = pending_decision {
        ("NEXT DECISION", title, term::alert())
    } else if let Some(obligation) = sim.next_timed_obligation() {
        let due_year = obligation.due_year.unwrap_or(sim.year());
        let remaining = due_year.saturating_sub(sim.year());
        let timing = if remaining == 0 {
            "DUE NOW".to_owned()
        } else {
            format!("IN {remaining}Y")
        };
        (
            "NEXT DUTY",
            format!("{timing} · {}", short_duty(&obligation.title)),
            if remaining <= 1 {
                term::alert()
            } else if remaining <= 10 {
                term::accent()
            } else {
                term::primary()
            },
        )
    } else {
        (
            "NEXT DECISION",
            if contract.is_some() {
                "CLOCK RUNNING".to_owned()
            } else {
                "NO ACTIVE COUNCIL".to_owned()
            },
            term::primary(),
        )
    };
    let (risk_label, risk_value) = primary_risk(sim, &ctx.data.config);
    let risk_label = risk_label.replace("ALL SYSTEMS SOUND", "Sound");
    let weakest = weakest_module_readout(sim, ctx.data).replace("ALL MODULES SOUND", "Sound");
    let posture = if contract.is_some() {
        sim.command_posture.label().to_owned()
    } else {
        "NO VOYAGE".to_owned()
    };
    let cells: [(GaugeIcon, &str, String, Color); 6] = [
        (GaugeIcon::Fuel, "MISSION PHASE", phase, term::primary()),
        (GaugeIcon::Maint, "OBJECTIVE", objective, term::accent()),
        (
            GaugeIcon::Alert,
            decision_label,
            next_decision,
            decision_tone,
        ),
        (
            GaugeIcon::Life,
            "PRIMARY RISK",
            risk_label,
            if risk_value < 0.35 {
                term::alert()
            } else {
                term::accent()
            },
        ),
        (GaugeIcon::Hull, "WEAKEST MODULE", weakest, term::dim()),
        (
            GaugeIcon::People,
            "COMMAND POSTURE",
            posture,
            term::accent(),
        ),
    ];
    let n = cells.len();
    let cw = inner.w / n as f32;
    for (i, (icon, label, value, color)) in cells.into_iter().enumerate() {
        let cell = Rect::new(inner.x + i as f32 * cw, inner.y, cw, inner.h);
        status_badge(cell, icon, label, &value, color);
    }
}

/// Honest subsystem triage for the instrument strip. Condition alone cannot
/// establish that a module is declining, so the dashboard names the weakest
/// module without a trend arrow and stays quiet while every module is sound.
pub(super) fn weakest_module_readout(sim: &SimState, data: &GameData) -> String {
    sim.subsystems
        .iter()
        .min_by(|a, b| a.1.condition.total_cmp(&b.1.condition))
        .and_then(|(id, state)| {
            if state.condition >= 0.85 {
                return Some("ALL MODULES SOUND".to_owned());
            }
            data.subsystems.get(id).map(|definition| {
                let short = definition
                    .name
                    .split(" & ")
                    .next()
                    .unwrap_or(&definition.name);
                format!("{short} {:.0}%", state.condition * 100.0)
            })
        })
        .unwrap_or_else(|| "ALL MODULES SOUND".to_owned())
}

/// Exact weakest survival reserve for the Dashboard instrument strip. Scores
/// share a 0-1 safety scale; once all are comfortably above danger, the readout
/// stops inventing a problem and reports the ship sound.
pub(super) fn primary_risk(sim: &SimState, config: &GameConfig) -> (String, f32) {
    let yearly_food = config.food_per_person_per_year * sim.population.count.max(1) as f32;
    let food_years = if yearly_food > 0.0 {
        sim.resources.food as f32 / yearly_food
    } else {
        10.0
    };
    let energy_score = if config.low_energy_threshold > 0 {
        sim.resources.energy as f32 / config.low_energy_threshold as f32
    } else {
        1.0
    };
    let risks = [
        (
            sim.ship.hull_integrity,
            format!("HULL {:.0}%", sim.ship.hull_integrity * 100.0),
        ),
        (
            sim.ship.life_support,
            format!("AIR {:.0}%", sim.ship.life_support * 100.0),
        ),
        (sim.ship.fuel, format!("FUEL {:.0}%", sim.ship.fuel * 100.0)),
        (
            (food_years / 10.0).clamp(0.0, 1.0),
            format!("FOOD {food_years:.1}Y"),
        ),
        (
            energy_score.clamp(0.0, 1.0),
            format!(
                "ENERGY {}/{}",
                sim.resources.energy, config.low_energy_threshold
            ),
        ),
    ];
    let (score, label) = risks
        .into_iter()
        .min_by(|a, b| a.0.total_cmp(&b.0))
        .unwrap();
    if score >= 0.75 {
        ("ALL SYSTEMS SOUND".to_owned(), score)
    } else {
        (label, score)
    }
}
