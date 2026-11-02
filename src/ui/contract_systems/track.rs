//! The active charter as a voyage track: phase bands from home berth to the
//! operation site and back, the ship's position, milestone diamonds, and
//! success metrics as gauges with their target ticked. Replaces the separate
//! PROGRESS bar, phase strip, `[ ]` milestone list and "1.00/0.85 (w 21%)"
//! metric labels (docs/UI_TEXT_REVIEW.md, item 4).

use super::outlook::mission_time;
use super::reward_hint;
use crate::state::sim::ActiveContract;
use crate::ui::chart::{self, Band};
use crate::ui::{term, term_bar};
use macroquad::prelude::*;
use macroquad_toolkit::prelude::*;
use macroquad_toolkit::ui::draw_ui_text_ex;

const TRACK_H: f32 = 64.0;
const METRIC_STRIDE: f32 = 50.0;

/// `detail` discloses reached milestones and the success metrics; without it
/// the view keeps to route, objective and the milestones still ahead.
pub(super) fn draw(contract: &ActiveContract, operation_site: &str, content: Rect, detail: bool) {
    let mut y = content.y + 42.0;
    draw_ui_text_ex(
        &contract.name,
        content.x,
        y,
        TextStyle::new(19.0, term::accent()).params(),
    );
    y += 26.0;
    draw_ui_text_ex(
        &{
            let mut line = format!(
                "{} · YEAR {} / {}",
                contract.objective.label().to_uppercase(),
                contract.months_elapsed / 12,
                contract.target_duration_years
            );
            if let Some((phase, eta)) = contract.next_phase_eta() {
                line.push_str(&format!(
                    " · {} IN {}",
                    phase.label().to_uppercase(),
                    mission_time(eta).to_uppercase()
                ));
            }
            line
        },
        content.x,
        y,
        TextStyle::new(16.0, term::dim()).params(),
    );
    y += 22.0;
    let track = Rect::new(content.x, y, content.w, TRACK_H);
    draw_track(contract, track);
    y = draw_places(contract, operation_site, track) + 18.0;
    y = draw_milestones(contract, content, y, detail) + 22.0;
    term_bar(
        Rect::new(content.x, y, content.w, 22.0),
        contract.objective_fraction(),
        term::accent(),
        "OBJECTIVE",
        &format!(
            "{:.0} / {:.0} {}",
            contract.objective_progress, contract.objective_target, contract.objective_unit
        ),
    );
    if detail {
        draw_metrics(contract, content, y + 58.0);
    }
}

/// Phase bands with the elapsed span washed in, the ship as a bright mast at
/// its position, and each milestone as a diamond on the centre line.
fn draw_track(contract: &ActiveContract, track: Rect) {
    let labels: Vec<String> = contract
        .phases
        .iter()
        .map(|phase| phase.kind.label().to_uppercase())
        .collect();
    let bands: Vec<Band<'_>> = contract
        .phases
        .iter()
        .zip(&labels)
        .map(|(phase, label)| Band {
            label,
            span: phase.years as f32,
        })
        .collect();
    chart::bands(track, &bands);
    let progress = contract.progress();
    let elapsed = term::accent();
    draw_rectangle(
        track.x,
        track.y,
        track.w * progress,
        track.h,
        Color::new(elapsed.r, elapsed.g, elapsed.b, 0.18),
    );
    let mid = track.y + track.h * 0.66;
    draw_line(track.x, mid, track.right(), mid, 1.0, term::faint());
    let next = contract.milestones.iter().position(|m| !m.reached);
    for (index, milestone) in contract.milestones.iter().enumerate() {
        let x = track.x + track.w * milestone.progress_threshold.clamp(0.0, 1.0);
        let color = if milestone.reached {
            term::accent()
        } else if Some(index) == next {
            term::primary()
        } else {
            term::dim()
        };
        chart::marker(vec2(x, mid), 8.0, color);
    }
    let ship_x = track.x + track.w * progress;
    draw_rectangle(
        ship_x - 1.5,
        track.y - 6.0,
        3.0,
        track.h + 12.0,
        term::accent(),
    );
    draw_triangle(
        vec2(ship_x - 8.0, track.y - 14.0),
        vec2(ship_x + 8.0, track.y - 14.0),
        vec2(ship_x, track.y - 4.0),
        term::accent(),
    );
}

/// Home berth, the operation site and the berth again, under the band edges
/// where the ship arrives at each. Returns the y below the labels.
fn draw_places(contract: &ActiveContract, operation_site: &str, track: Rect) -> f32 {
    let total = contract.target_duration_years.max(1) as f32;
    let site_x = contract
        .phases
        .iter()
        .take_while(|phase| phase.kind != crate::data::contracts::ContractPhase::Operation)
        .map(|phase| phase.years as f32)
        .sum::<f32>()
        / total
        * track.w
        + track.x;
    let y = track.bottom() + 22.0;
    let style = TextStyle::new(16.0, term::dim());
    draw_ui_text_ex("HOME BERTH", track.x, y, style.params());
    draw_ui_text_ex(
        &operation_site.to_uppercase(),
        site_x + 4.0,
        y,
        style.params(),
    );
    let remaining = format!(
        "BERTH IN {}",
        mission_time(contract.mission_months_remaining()).to_uppercase()
    );
    let w = measure_text_size(&remaining, style).width;
    draw_ui_text_ex(&remaining, track.right() - w, y, style.params());
    y
}

/// One line per milestone: its diamond, name and reward, and on the right
/// either REACHED or the mission time until it.
fn draw_milestones(contract: &ActiveContract, content: Rect, mut y: f32, detail: bool) -> f32 {
    // Only the next milestone carries a countdown; later ones wait their turn.
    let next = contract.next_milestone_eta();
    let reached = contract.milestones.iter().filter(|m| m.reached).count();
    if !detail && reached > 0 {
        y += 26.0;
        chart::marker(vec2(content.x + 8.0, y - 6.0), 7.0, term::accent());
        draw_ui_text_ex(
            &format!(
                "{reached} milestone{} reached",
                if reached == 1 { "" } else { "s" }
            ),
            content.x + 26.0,
            y,
            TextStyle::new(16.0, term::accent()).params(),
        );
    }
    for milestone in contract
        .milestones
        .iter()
        .filter(|milestone| detail || !milestone.reached)
    {
        y += 26.0;
        let (color, status) = if milestone.reached {
            (term::accent(), "REACHED".to_owned())
        } else {
            match next {
                Some((upcoming, eta)) if upcoming.id == milestone.id => (
                    term::primary(),
                    format!("IN {}", mission_time(eta).to_uppercase()),
                ),
                _ => (term::dim(), String::new()),
            }
        };
        chart::marker(vec2(content.x + 8.0, y - 6.0), 7.0, color);
        draw_ui_text_ex(
            &format!("{}{}", milestone.name, reward_hint(&milestone.reward)),
            content.x + 26.0,
            y,
            TextStyle::new(16.0, color).params(),
        );
        let w = measure_text_size(&status, TextStyle::new(16.0, color)).width;
        draw_ui_text_ex(
            &status,
            content.right() - w,
            y,
            TextStyle::new(16.0, color).params(),
        );
    }
    y
}

/// Each scored metric as a gauge with its target ticked; a metric short of its
/// target reads in the alert colour.
fn draw_metrics(contract: &ActiveContract, content: Rect, mut y: f32) {
    draw_ui_text_ex(
        "SUCCESS METRICS",
        content.x,
        y,
        TextStyle::new(16.0, term::primary()).params(),
    );
    y += 12.0;
    for metric in &contract.metrics {
        let met = metric.current >= metric.target;
        let tone = if met { term::accent() } else { term::alert() };
        let scale = metric.target.max(1.0);
        draw_ui_text_ex(
            &metric.name.to_uppercase(),
            content.x,
            y + 18.0,
            TextStyle::new(16.0, term::dim()).params(),
        );
        chart::gauge(
            Rect::new(content.x, y + 26.0, content.w, 14.0),
            metric.current / scale,
            Some(metric.target / scale),
            tone,
        );
        y += METRIC_STRIDE;
    }
}
