//! Compact project selection keeps the exact existing operations in one detail panel.
use super::*;

pub(super) fn draw_work_board(
    ctx: &GameplayCtx<'_>,
    area: Rect,
    pointer: Pointer,
    actions: &mut Vec<UiAction>,
) {
    term_panel(
        area,
        Some(&format!(
            "Agenda · {} active · {} waiting",
            ctx.sim.projects.active_count(),
            ctx.sim.projects.waiting_count()
        )),
    );
    let choices = catalogue_choices(ctx);
    let jobs = &ctx.sim.projects.jobs;
    let count = jobs.len() + choices.len();
    let selected = ctx
        .presentation
        .selected_agenda
        .get()
        .min(count.saturating_sub(1));
    let view = Rect::new(area.x + 16.0, area.y + 46.0, area.w - 32.0, area.h - 264.0);
    let content_h = count as f32 * 66.0;
    let mut scroll = ctx.agenda_scroll.get();
    scroll.update_at(view, content_h, pointer.position);
    let list_pointer = if scroll.absorbs_press() {
        pointer.suppressed()
    } else {
        pointer
    };
    for index in 0..count {
        let rect = Rect::new(
            view.x,
            view.y + index as f32 * 66.0 - scroll.offset(),
            view.w - GUTTER,
            56.0,
        );
        if !is_fully_visible(rect, view) {
            continue;
        }
        let label = if let Some(job) = jobs.get(index) {
            let name = ctx
                .data
                .projects
                .get(&job.project_id)
                .map_or(job.project_id.as_str(), |def| def.name.as_str());
            if job.status == ProjectStatus::Running {
                let progress = ctx
                    .data
                    .projects
                    .get(&job.project_id)
                    .map_or(0.0, |def| job.progress(def.duration_months));
                format!("{name} · Running {:.0}%", progress * 100.0)
            } else if let Some((position, count)) =
                ctx.sim.projects.waiting_position(job.sequence_id)
            {
                format!("{name} · Waiting {position}/{count} · {:?}", job.status)
            } else {
                format!("{} · {:?}", name, job.status)
            }
        } else {
            let choice = &choices[index - jobs.len()];
            let Some(def) = ctx.data.projects.get(&choice.project_id) else {
                continue;
            };
            let target = choice
                .target_id
                .as_deref()
                .map(|id| id.replace('_', " "))
                .unwrap_or_default();
            format!(
                "{} · {}{}",
                def.name,
                target,
                if choice.eligible {
                    ""
                } else {
                    " · unavailable"
                }
            )
        };
        if term_button(rect, &label, true, list_pointer) {
            ctx.presentation.selected_agenda.set(index);
        }
        if index == selected {
            draw_rectangle(rect.x, rect.y, 4.0, rect.h, term::primary());
        }
    }
    scroll.draw_scrollbar_with(
        view,
        content_h,
        term::surface_inset(),
        term::dim(),
        term::primary(),
    );
    ctx.agenda_scroll.set(scroll);
    let detail = Rect::new(area.x + 16.0, area.bottom() - 204.0, area.w - 32.0, 188.0);
    if let Some(job) = jobs.get(selected) {
        draw_job(ctx, job, detail, pointer, actions);
    } else if let Some(choice) = choices.get(selected.saturating_sub(jobs.len())) {
        draw_choice(ctx, choice, detail, pointer, actions);
    }
}
