//! Readiness rows and aftermath notices for the Custodian Agenda.

use super::*;

/// Vertical stretch for the readiness rows: 1 at 100% text and below, up to
/// 1.5 at 150%, so each row's stride and blocks grow with its type instead
/// of clipping.
fn k() -> f32 {
    macroquad_toolkit::ui::ui_text_scale().max(1.0)
}

/// One readiness or issue row's stride at the current text size.
fn row_stride() -> f32 {
    184.0 * k()
}

pub(super) fn draw_readiness(
    ctx: &GameplayCtx<'_>,
    area: Rect,
    pointer: Pointer,
    actions: &mut Vec<UiAction>,
) {
    term_panel(area, Some("READINESS / CURRENT CONDITIONS"));
    let mut view = area.inset(18.0);
    view.y += 22.0;
    view.h -= 22.0;
    let model = readiness::forecast(ctx.sim, ctx.data);
    let content_h = model.rows.len() as f32 * row_stride()
        + 180.0 * k()
        + ctx.sim.issues.active.len() as f32 * row_stride();
    let mut scroll = ctx.agenda_readiness_scroll.get();
    scroll.update_at(view, content_h, pointer.position);
    let pointer = if scroll.absorbs_press() {
        pointer.suppressed()
    } else {
        pointer
    };
    let mut y = view.y - scroll.offset();
    y = draw_readiness_rows(ctx, &model, view, pointer, actions, y);
    y = draw_readiness_summary(&model, view, y);
    draw_active_issues(ctx, view, pointer, actions, y);
    scroll.draw_scrollbar_with(
        view,
        content_h,
        term::surface_inset(),
        term::dim(),
        term::primary(),
    );
    ctx.agenda_readiness_scroll.set(scroll);
}

fn draw_readiness_rows(
    ctx: &GameplayCtx<'_>,
    model: &readiness::ReadinessModel,
    view: Rect,
    pointer: Pointer,
    actions: &mut Vec<UiAction>,
    mut y: f32,
) -> f32 {
    for row in &model.rows {
        let rect = Rect::new(view.x, y, view.w - 12.0, row_stride() - 6.0);
        if is_fully_visible(rect, view) {
            // Concern on the left and band right-aligned when both fit; at a
            // large text size the band leads the evidence line instead.
            let style = TextStyle::new(13.0, band_color(row.band));
            draw_ui_text_ex(&row.concern, rect.x, rect.y + 18.0 * k(), style.params());
            let fits = measure_text_size(&row.concern, style).width
                + measure_text_size(row.band.label(), style).width
                + 16.0
                <= rect.w;
            let evidence = if fits {
                draw_text_right(row.band.label(), rect.right(), rect.y + 18.0 * k(), style);
                row.evidence_line(" / ")
            } else {
                format!("{} / {}", row.band.label(), row.evidence_line(" / "))
            };
            draw_text_block(
                &evidence,
                rect.x,
                rect.y + 26.0 * k(),
                rect.w,
                46.0 * k(),
                12.0,
                2.0,
                term::dim(),
            );
            draw_readiness_action(ctx, row, rect, pointer, actions);
        }
        y += row_stride();
    }
    y
}

fn draw_readiness_action(
    ctx: &GameplayCtx<'_>,
    row: &readiness::ReadinessRow,
    rect: Rect,
    pointer: Pointer,
    actions: &mut Vec<UiAction>,
) {
    if row.id == "life_support"
        && ctx.sim.ship.life_support <= ctx.data.config.survival.critical_warning_threshold
        && !ctx.sim.survival.emergency_used
    {
        let cfg = &ctx.data.config.survival;
        let button = Rect::new(rect.x, rect.y + 82.0 * k(), rect.w, 60.0);
        if term_button(
            button,
            "STABILISE AIR",
            crate::simulation::survival::emergency_availability(ctx.sim, ctx.data).is_ok(),
            pointer,
        ) {
            actions.push(UiAction::EmergencyStabilise);
        }
        draw_ui_text_ex(
            &format!(
                "Cost: {} energy / {} minerals / {} parts",
                cfg.emergency_resource_cost.energy,
                cfg.emergency_resource_cost.minerals,
                cfg.emergency_parts_cost
            ),
            rect.x,
            rect.y + 82.0 * k() + 60.0 + 18.0 * k(),
            TextStyle::new(11.0, term::dim()).params(),
        );
    } else if let Some(id) = &row.recommended_project {
        queue_response(
            ctx,
            id,
            row.recommended_target.clone(),
            Rect::new(rect.x, rect.y + 82.0 * k(), rect.w, 60.0 + 32.0 * k()),
            pointer,
            actions,
        );
    }
}

fn draw_readiness_summary(model: &readiness::ReadinessModel, view: Rect, y: f32) -> f32 {
    let summary = Rect::new(view.x, y, view.w - 12.0, 174.0 * k());
    if is_fully_visible(summary, view) {
        draw_text_block(
            &format!(
                "FOOD / YEAR: {} output; {} use and toll; {} spoilage; {:+} net. Gross reserve uses population consumption. FUEL: {} travel months, {:.2} burn, {:.2} scoop/year. Estimates hold current conditions steady. Future project effects are excluded until delivered.",
                model.food.annual_output,
                model.food.annual_consumption,
                model.food.annual_spoilage,
                model.food.net_per_year,
                model.fuel.remaining_travel_months,
                model.fuel.remaining_burn,
                model.fuel.annual_scoop
            ),
            summary.x,
            summary.y + 8.0,
            summary.w,
            summary.h - 16.0,
            13.0,
            4.0,
            term::dim(),
        );
    }
    y + 180.0 * k()
}

fn draw_active_issues(
    ctx: &GameplayCtx<'_>,
    view: Rect,
    pointer: Pointer,
    actions: &mut Vec<UiAction>,
    mut y: f32,
) {
    for issue in &ctx.sim.issues.active {
        let rect = Rect::new(view.x, y, view.w - 12.0, row_stride() - 6.0);
        if is_fully_visible(rect, view) {
            let due = issue.due_month.map_or("No deadline".into(), |m| {
                format!("Deadline Y{}.{}", m / 12, m % 12 + 1)
            });
            draw_text_block(
                &format!("{} / {} / {}", issue.id, issue.severity.label(), due),
                rect.x,
                rect.y + 4.0,
                rect.w,
                64.0 * k(),
                13.0,
                3.0,
                term::alert(),
            );
            if let Some(id) = issue.recovery_project_ids.first() {
                queue_response(
                    ctx,
                    id,
                    Some(issue.target.clone()),
                    Rect::new(rect.x, rect.y + 80.0 * k(), rect.w, 60.0 + 32.0 * k()),
                    pointer,
                    actions,
                );
            }
        }
        y += row_stride();
    }
}

fn queue_response(
    ctx: &GameplayCtx<'_>,
    id: &str,
    target: Option<String>,
    rect: Rect,
    pointer: Pointer,
    actions: &mut Vec<UiAction>,
) {
    let Some(definition) = ctx.data.projects.get(id) else {
        return;
    };
    let target = if matches!(
        definition.target,
        ProjectTarget::None | ProjectTarget::Social
    ) {
        None
    } else {
        target
    };
    let existing = project_sim::live_job(ctx.sim, id, target.as_deref());
    let check = project_sim::queue_check(ctx.sim, ctx.data, definition, target.as_deref());
    let button = Rect::new(rect.x, rect.y, rect.w, 60.0);
    if term_button(
        button,
        &format!(
            "{} {}",
            if existing.is_some() {
                "REVIEW"
            } else {
                "QUEUE"
            },
            definition.name.to_uppercase()
        ),
        check.eligible || existing.is_some(),
        pointer,
    ) {
        if let Some(job) = existing {
            actions.push(UiAction::PreviewCancelProject(job.sequence_id));
        } else {
            actions.push(UiAction::QueueProject {
                project_id: id.into(),
                target_id: target,
            });
        }
    }
    if !check.eligible {
        draw_text_block(
            if existing.is_some() {
                "Recovery is already on the Agenda."
            } else {
                &check.reason
            },
            rect.x,
            rect.y + 62.0,
            rect.w,
            30.0 * k() * k(),
            11.0,
            1.0,
            if existing.is_some() {
                term::dim()
            } else {
                term::alert()
            },
        );
    }
}
