//! Readiness rows and aftermath notices for the Custodian Agenda.

use super::*;

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
    let content_h =
        model.rows.len() as f32 * 184.0 + 180.0 + ctx.sim.issues.active.len() as f32 * 184.0;
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
        let rect = Rect::new(view.x, y, view.w - 12.0, 178.0);
        if is_fully_visible(rect, view) {
            draw_ui_text_ex(
                &format!("{} / {}", row.concern, row.band.label()),
                rect.x,
                rect.y + 18.0,
                TextStyle::new(13.0, band_color(row.band)).params(),
            );
            draw_text_block(
                &format!("{} / {}", row.evidence, row.trend),
                rect.x,
                rect.y + 26.0,
                rect.w,
                46.0,
                12.0,
                2.0,
                term::dim(),
            );
            draw_readiness_action(ctx, row, rect, pointer, actions);
        }
        y += 184.0;
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
        let button = Rect::new(rect.x, rect.y + 82.0, rect.w, 60.0);
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
            rect.y + 160.0,
            TextStyle::new(11.0, term::dim()).params(),
        );
    } else if let Some(id) = &row.recommended_project {
        queue_response(
            ctx,
            id,
            row.recommended_target.clone(),
            Rect::new(rect.x, rect.y + 82.0, rect.w, 92.0),
            pointer,
            actions,
        );
    }
}

fn draw_readiness_summary(model: &readiness::ReadinessModel, view: Rect, y: f32) -> f32 {
    let summary = Rect::new(view.x, y, view.w - 12.0, 174.0);
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
    y + 180.0
}

fn draw_active_issues(
    ctx: &GameplayCtx<'_>,
    view: Rect,
    pointer: Pointer,
    actions: &mut Vec<UiAction>,
    mut y: f32,
) {
    for issue in &ctx.sim.issues.active {
        let rect = Rect::new(view.x, y, view.w - 12.0, 178.0);
        if is_fully_visible(rect, view) {
            let due = issue.due_month.map_or("No deadline".into(), |m| {
                format!("Deadline Y{}.{}", m / 12, m % 12 + 1)
            });
            draw_text_block(
                &format!("{} / {} / {}", issue.id, issue.severity.label(), due),
                rect.x,
                rect.y + 4.0,
                rect.w,
                64.0,
                13.0,
                3.0,
                term::alert(),
            );
            if let Some(id) = issue.recovery_project_ids.first() {
                queue_response(
                    ctx,
                    id,
                    Some(issue.target.clone()),
                    Rect::new(rect.x, rect.y + 80.0, rect.w, 92.0),
                    pointer,
                    actions,
                );
            }
        }
        y += 184.0;
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
            28.0,
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
