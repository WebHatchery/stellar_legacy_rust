//! The homecoming's voyage curve: population year by year from launch to
//! berth, the final score as a gauge, and the marks as diamonds. Replaces the
//! "Population 1000 → 326 (-674)", "Final score 0.83" and "Marks reached 4/4"
//! report lines (docs/UI_TEXT_REVIEW.md, item 8).

use super::*;
use crate::ui::chart;

pub(super) fn draw(report: &VoyageDebrief, area: Rect) {
    let content = column_panel(area, "Voyage curve");
    let mut y = content.y + 18.0;
    let change = report.population_change();
    let tone = if change < 0 {
        term::alert()
    } else {
        term::accent()
    };
    heading(
        content,
        y,
        "POPULATION",
        &format!("{} → {}", report.population_start, report.population_end),
        tone,
    );
    let plot = Rect::new(content.x, y + 14.0, content.w, (content.h * 0.5).max(160.0));
    draw_population(report, plot, tone);
    y = plot.bottom() + 24.0;
    let years = TextStyle::new(16.0, term::dim());
    draw_ui_text_ex(
        &format!("Y{}", report.began_year),
        content.x,
        y,
        years.params(),
    );
    let end = format!("Y{}", report.ended_year);
    let w = measure_text_size(&end, years).width;
    draw_ui_text_ex(&end, content.right() - w, y, years.params());

    y += 48.0;
    heading(
        content,
        y,
        "FINAL SCORE",
        &format!("{:.2} · {}", report.score, report.outcome.to_uppercase()),
        term::primary(),
    );
    chart::gauge(
        Rect::new(content.x, y + 12.0, content.w, 20.0),
        report.score,
        None,
        term::accent(),
    );

    y += 74.0;
    let (reached, total) = report.milestones_reached();
    heading(
        content,
        y,
        "MARKS",
        &format!("{reached} / {total}"),
        term::primary(),
    );
    for (index, milestone) in report.milestones.iter().enumerate() {
        let color = if milestone.reached {
            term::accent()
        } else {
            term::dim()
        };
        chart::marker(
            vec2(content.x + 10.0 + index as f32 * 30.0, y + 26.0),
            9.0,
            color,
        );
    }
}

fn heading(content: Rect, y: f32, label: &str, value: &str, tone: Color) {
    draw_ui_text_ex(
        label,
        content.x,
        y,
        TextStyle::new(16.0, term::dim()).params(),
    );
    let style = TextStyle::new(16.0, tone);
    let w = measure_text_size(value, style).width;
    draw_ui_text_ex(value, content.right() - w, y, style.params());
}

/// The recorded track, or a straight line from launch to berth for a report
/// sealed before the track existed. A faint line marks the launch head count.
fn draw_population(report: &VoyageDebrief, plot: Rect, tone: Color) {
    draw_rectangle(plot.x, plot.y, plot.w, plot.h, term::surface_inset());
    draw_rectangle_lines(plot.x, plot.y, plot.w, plot.h, 1.0, term::faint());
    let values: Vec<f32> = if report.population_track.len() >= 2 {
        report.population_track.iter().map(|v| *v as f32).collect()
    } else {
        vec![report.population_start as f32, report.population_end as f32]
    };
    let hi = values.iter().copied().fold(1.0, f32::max) * 1.1;
    let inner = plot.inset(6.0);
    let launch = chart::line_points(inner, &[report.population_start as f32], 0.0, hi);
    if let Some(start) = launch.first() {
        draw_line(inner.x, start.y, inner.right(), start.y, 1.0, term::dim());
    }
    let points = chart::line_points(inner, &values, 0.0, hi);
    chart::polyline(&points, 2.5, tone);
    if let Some(last) = points.last() {
        draw_circle(last.x, last.y, 5.0, tone);
    }
}
