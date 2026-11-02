//! What an event outcome is known to change, as structured terms. The council
//! modal lays the terms of every option side by side in a comparison grid.

use crate::data::events::EventOutcome;
use crate::ui::term;
use macroquad::prelude::Color;

/// One known change: a row label, its value as shown (empty for a flag such
/// as "forced return"), and whether it helps (+1), hurts (-1) or is neutral.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct EffectTerm {
    pub label: String,
    pub value: String,
    pub tone: i8,
}

impl EffectTerm {
    fn signed(label: impl Into<String>, value: String, amount: f32) -> Self {
        Self {
            label: label.into(),
            value,
            tone: amount.signum() as i8,
        }
    }

    fn flag(label: &str, tone: i8) -> Self {
        Self {
            label: label.to_owned(),
            value: String::new(),
            tone,
        }
    }

    pub fn color(&self) -> Color {
        match self.tone {
            1 => term::accent(),
            -1 => term::alert(),
            _ => term::dim(),
        }
    }
}

fn pct(value: f32) -> String {
    format!("{:+.0}%", value * 100.0)
}

/// Every known change of `outcome`, in a stable order: stores, ship, people,
/// peoples' approval, reputation, compartments, objective, then flags.
pub(crate) fn effect_terms(
    outcome: &EventOutcome,
    fuel_preview: Option<(f32, f32)>,
) -> Vec<EffectTerm> {
    let mut terms = Vec::new();
    let r = outcome.resource_delta;
    for (label, value) in [
        ("Credits", r.credits),
        ("Energy", r.energy),
        ("Minerals", r.minerals),
        ("Food", r.food),
        ("Influence", r.influence),
    ] {
        if value != 0 {
            terms.push(EffectTerm::signed(
                label,
                format!("{value:+}"),
                value as f32,
            ));
        }
    }
    let s = outcome.ship_delta;
    for (label, value) in [("hull", s.hull_integrity), ("life", s.life_support)] {
        if value.abs() > f32::EPSILON {
            terms.push(EffectTerm::signed(label, pct(value), value));
        }
    }
    if s.spare_parts != 0 {
        terms.push(EffectTerm::signed(
            "parts",
            format!("{:+}", s.spare_parts),
            s.spare_parts as f32,
        ));
    }
    if let Some((before, after)) = fuel_preview {
        terms.push(EffectTerm::signed(
            "fuel tank",
            format!("{:.1}% → {:.1}%", before * 100.0, after * 100.0),
            after - before,
        ));
    }
    append_people_terms(&mut terms, outcome);
    append_flag_terms(&mut terms, outcome);
    terms
}

fn append_people_terms(terms: &mut Vec<EffectTerm>, outcome: &EventOutcome) {
    let p = outcome.population_delta;
    for (label, value, good_when_rising) in [
        ("morale", p.morale, true),
        ("unity", p.unity, true),
        ("stability", p.stability, true),
        ("loyalty", p.legacy_loyalty, true),
        ("adapt", p.adaptation, true),
        // Drift away from the founding reads as a cost when it climbs.
        ("drift", p.cultural_drift, false),
    ] {
        if value.abs() > f32::EPSILON {
            let tone_amount = if good_when_rising { value } else { -value };
            terms.push(EffectTerm::signed(label, pct(value), tone_amount));
        }
    }
    for delta in &outcome.faction_approval_deltas {
        if delta.delta.abs() > f32::EPSILON {
            terms.push(EffectTerm::signed(
                format!("{} approval", delta.id.replace('_', " ")),
                pct(delta.delta),
                delta.delta,
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
            // A reputation is a standing, not a good: it stays neutral-toned.
            terms.push(EffectTerm {
                label,
                value: pct(delta.delta),
                tone: 0,
            });
        }
    }
    for delta in &outcome.subsystem_deltas {
        let name = delta.id.replace('_', " ");
        if delta.condition.abs() > f32::EPSILON {
            terms.push(EffectTerm::signed(
                format!("{name} condition"),
                pct(delta.condition),
                delta.condition,
            ));
        }
        if delta.knowledge.abs() > f32::EPSILON {
            terms.push(EffectTerm::signed(
                format!("{name} knowledge"),
                pct(delta.knowledge),
                delta.knowledge,
            ));
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
        terms.push(EffectTerm::signed("objective", amount, pct));
    }
}

fn append_flag_terms(terms: &mut Vec<EffectTerm>, outcome: &EventOutcome) {
    if outcome.force_return {
        terms.push(EffectTerm::flag("forced return", -1));
    }
    if outcome.faction_loss.is_some() {
        terms.push(EffectTerm::flag("faction may leave", -1));
    }
    if outcome.designate_heir {
        terms.push(EffectTerm::flag("names the ready heir", 1));
    }
    if let Some(followup) = &outcome.schedule_followup {
        terms.push(EffectTerm {
            label: "follow-up".to_owned(),
            value: format!("in {}y", followup.delay_years),
            tone: 0,
        });
    } else if !outcome.long_term_consequences.is_empty() {
        terms.push(EffectTerm::flag("future consequence", 0));
    }
}

/// Human phrasing of a population-impact band (real-time loop §3), with a tone:
/// a loss band reads warm-red, a gain accent, a straddle neutral.
pub(crate) fn impact_label(lo: i64, hi: i64) -> (String, Color) {
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
