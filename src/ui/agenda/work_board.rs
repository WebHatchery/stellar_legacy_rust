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
    let detail_height = if selected < jobs.len() { 188.0 } else { 316.0 };
    let view = Rect::new(
        area.x + 16.0,
        area.y + 46.0,
        area.w - 32.0,
        area.h - detail_height - 76.0,
    );
    let content_h = count as f32 * 66.0;
    let mut scroll = ctx.agenda_scroll.get();
    scroll.update_at(view, content_h, pointer.position);
    let list_pointer = if scroll.absorbs_press() {
        pointer.suppressed()
    } else {
        pointer
    };
    let selected_row = draw_work_list(ctx, view, list_pointer, &choices, selected, &mut scroll);
    scroll.draw_scrollbar_with(
        view,
        content_h,
        term::surface_inset(),
        term::dim(),
        term::primary(),
    );
    if let Some(index) = selected_row {
        // Keep the chosen row in view when the catalogue detail grows next frame.
        scroll.set_offset(index as f32 * 66.0);
    }
    ctx.agenda_scroll.set(scroll);
    let detail = Rect::new(
        area.x + 16.0,
        area.bottom() - detail_height - 16.0,
        area.w - 32.0,
        detail_height,
    );
    if let Some(job) = jobs.get(selected) {
        draw_job(ctx, job, detail, pointer, actions);
    } else if let Some(choice) = choices.get(selected.saturating_sub(jobs.len())) {
        draw_choice(ctx, choice, detail, pointer, actions);
    }
}

fn draw_work_list(
    ctx: &GameplayCtx<'_>,
    view: Rect,
    pointer: Pointer,
    choices: &[CatalogueChoice],
    selected: usize,
    scroll: &mut macroquad_toolkit::ui::ScrollArea,
) -> Option<usize> {
    let jobs = &ctx.sim.projects.jobs;
    let count = jobs.len() + choices.len();
    let mut selected_row = None;
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
        let label = work_row_label(ctx, choices, index);
        if term_button(rect, &label, true, pointer) {
            ctx.presentation.selected_agenda.set(index);
            if index != selected {
                selected_row = Some(index);
            }
        }
        if index == selected {
            draw_rectangle(rect.x, rect.y, 4.0, rect.h, term::primary());
        }
    }
    selected_row
}

fn work_row_label(ctx: &GameplayCtx<'_>, choices: &[CatalogueChoice], index: usize) -> String {
    let jobs = &ctx.sim.projects.jobs;
    if let Some(job) = jobs.get(index) {
        let name = ctx
            .data
            .projects
            .get(&job.project_id)
            .map_or(job.project_id.as_str(), |def| def.name.as_str());
        let name = job.target_id.as_deref().map_or_else(
            || name.to_owned(),
            |target| format!("{name} · {}", target_label(ctx.data, Some(target))),
        );
        return if job.status == ProjectStatus::Running {
            let progress = ctx
                .data
                .projects
                .get(&job.project_id)
                .map_or(0.0, |def| job.progress(def.duration_months));
            format!("{name} · Running {:.0}%", progress * 100.0)
        } else if let Some((position, count)) = ctx.sim.projects.waiting_position(job.sequence_id) {
            format!("{name} · Waiting {position}/{count} · {:?}", job.status)
        } else {
            format!("{} · {:?}", name, job.status)
        };
    }
    let choice = &choices[index - jobs.len()];
    let Some(def) = ctx.data.projects.get(&choice.project_id) else {
        return String::new();
    };
    let target = target_label(ctx.data, choice.target_id.as_deref());
    format!(
        "{} · {}{}",
        def.name,
        target,
        if choice.eligible {
            ""
        } else if choice.existing_job.is_some() {
            " · on Agenda"
        } else {
            " · unavailable"
        }
    )
}
