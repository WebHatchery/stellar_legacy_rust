//! The selected compartment's card. Immediate care leads: fitting, what a
//! repair can reach, condition and knowledge, urgent institutional trouble and
//! the repair/upgrade/train actions. School, archive, custody and culture sit
//! behind a Continuity & culture disclosure, reflowed at reading size.

use super::*;
use crate::state::sim::institutions::SubsystemSchool;

/// Years before a school's support runs out at which the care view warns.
const SCHOOL_WARNING_YEARS: u32 = 5;

#[derive(Clone, Copy)]
pub(super) struct SubsystemDetails<'a> {
    pub id: &'a str,
    pub def: &'a crate::data::subsystems::SubsystemDef,
    pub state: &'a crate::state::sim::subsystems::SubsystemState,
}

pub(super) fn draw_card(
    ctx: &GameplayCtx<'_>,
    rect: Rect,
    id: &str,
    pointer: Pointer,
    actions: &mut Vec<UiAction>,
) {
    let (Some(def), Some(state)) = (ctx.data.subsystems.get(id), ctx.sim.subsystems.get(id)) else {
        return;
    };
    term_panel(rect, Some(&def.name.to_uppercase()));
    let content = rect.inset(14.0);
    let school = ctx
        .sim
        .subsystem_schools
        .iter()
        .find(|school| school.subsystem_id == id);
    let archived = ctx
        .sim
        .procedure_archives
        .iter()
        .any(|archive| archive.subsystem_id == id);
    let details = SubsystemDetails { id, def, state };
    let continuity = ctx.presentation.continuity_open.get();
    if continuity {
        draw_continuity(ctx, content, details, school, archived);
    } else {
        let y = draw_identity(ctx, content, details);
        draw_bars(content, details, y);
        draw_warnings(ctx, content, details, school, y + 54.0);
    }
    let institution = institution_action(ctx, id, school, archived);
    draw_actions(
        ctx,
        content,
        details,
        institution,
        continuity,
        pointer,
        actions,
    );
}

/// Fitting, tier and what a repair can reach right now.
fn draw_identity(ctx: &GameplayCtx<'_>, content: Rect, subsystem: SubsystemDetails<'_>) -> f32 {
    let SubsystemDetails { id, def, state } = subsystem;
    let pips: String = (1..=3)
        .map(|t| if state.tier >= t { '●' } else { '○' })
        .collect();
    let mut y = content.y + 40.0;
    draw_ui_text_ex(
        &format!("{} · TIER {pips}", def.fitting_name(state.tier)),
        content.x,
        y,
        TextStyle::new(20.0, term::primary()).params(),
    );
    y += 30.0;
    let (line, tone) = if state.knowledge < def.repair_knowledge_required {
        (
            format!(
                "! Repair unavailable: knowledge {:.0}% is below the {:.0}% this system needs. Train first.",
                state.knowledge * 100.0,
                def.repair_knowledge_required * 100.0
            ),
            term::alert(),
        )
    } else {
        let target = repair_target_condition(ctx.sim, ctx.data, id).unwrap_or(state.condition);
        (
            format!("The crew can repair this system to {:.0}%.", target * 100.0),
            term::dim(),
        )
    };
    draw_ui_text_ex(&line, content.x, y, TextStyle::new(17.0, tone).params());
    y + 18.0
}

fn draw_bars(content: Rect, subsystem: SubsystemDetails<'_>, y: f32) {
    let SubsystemDetails { def, state, .. } = subsystem;
    term_bar(
        Rect::new(content.x, y, content.w, 22.0),
        state.condition,
        term::accent(),
        "CONDITION",
        &format!("{:.0}%", state.condition * 100.0),
    );
    let can_mend = state.knowledge >= def.repair_knowledge_required;
    term_bar(
        Rect::new(content.x, y + 28.0, content.w, 22.0),
        state.knowledge,
        if can_mend {
            term::accent()
        } else {
            term::alert()
        },
        "KNOWLEDGE",
        &format!(
            "{:.0}%  (need {:.0}%)",
            state.knowledge * 100.0,
            def.repair_knowledge_required * 100.0
        ),
    );
}

/// Institutional trouble that bears on the care decision stays visible, one
/// line each, though the full record sits behind the disclosure.
fn draw_warnings(
    ctx: &GameplayCtx<'_>,
    content: Rect,
    subsystem: SubsystemDetails<'_>,
    school: Option<&SubsystemSchool>,
    mut y: f32,
) {
    let year = ctx.sim.year();
    let mut lines = Vec::new();
    if let Some(grievance) = subsystem.state.culture.grievance.as_deref() {
        lines.push(format!("! Grievance: {grievance}"));
    }
    if let Some(school) = school {
        if school.supported_until_year < year {
            lines.push("! The school has lapsed; its knowledge is no longer protected.".to_owned());
        } else if school.supported_until_year - year <= SCHOOL_WARNING_YEARS {
            lines.push(format!(
                "! School support ends in year {}.",
                school.supported_until_year
            ));
        }
    }
    for line in lines {
        draw_ui_text_ex(
            &line,
            content.x,
            y + 18.0,
            TextStyle::new(17.0, term::alert()).params(),
        );
        y += 26.0;
    }
}

/// School, archive, custody and the compartment's culture.
fn draw_continuity(
    ctx: &GameplayCtx<'_>,
    content: Rect,
    subsystem: SubsystemDetails<'_>,
    school: Option<&SubsystemSchool>,
    archived: bool,
) {
    let SubsystemDetails { id, def, state } = subsystem;
    draw_ui_text_ex(
        "CONTINUITY & CULTURE",
        content.x,
        content.y + 40.0,
        TextStyle::new(20.0, term::primary()).params(),
    );
    let faction_name = |faction_id: Option<&str>| {
        faction_id
            .and_then(|faction_id| ctx.data.factions.get(faction_id))
            .map(|faction| faction.name.clone())
    };
    let institutions = match school {
        None => "No school. Knowledge drifts unless the crew trains.".to_owned(),
        Some(school) => {
            let support = if school.supported_until_year >= ctx.sim.year() {
                format!("School supported to year {}", school.supported_until_year)
            } else {
                "School lapsed".to_owned()
            };
            let archive = if archived {
                "procedures archived"
            } else {
                "no procedure archive"
            };
            let custodian = faction_name(school.custodian_faction_id.as_deref()).map_or_else(
                || "no custodian".to_owned(),
                |name| format!("custodian {name}"),
            );
            format!("{support} · {archive} · {custodian}")
        }
    };
    let family = if def.buffers_family.is_empty() {
        "habitat integrity".to_owned()
    } else {
        def.buffers_family.replace('_', " ")
    };
    let mut lines = vec![institutions, format!("Buffers {family}.")];
    if let Some(culture) =
        crate::simulation::culture::descriptor(ctx.data, id, &state.culture.descriptor_id)
    {
        let custodian = faction_name(state.culture.custodian_faction_id.as_deref())
            .unwrap_or_else(|| "no local custodian".to_owned());
        lines.push(format!("Culture: {} · {custodian}", culture.label));
        lines.push(crate::simulation::culture::effect_summary(culture));
        if let Some(memory) = state.culture.remembered_event.as_deref() {
            lines.push(format!("Remembered: {memory}"));
        }
        if let Some(grievance) = state.culture.grievance.as_deref() {
            lines.push(format!("Grievance: {grievance}"));
        }
    }
    draw_text_block(
        &lines.join("\n"),
        content.x,
        content.y + 56.0,
        content.w,
        content.h - 110.0,
        17.0,
        6.0,
        term::dim(),
    );
}

fn draw_actions(
    ctx: &GameplayCtx<'_>,
    content: Rect,
    subsystem: SubsystemDetails<'_>,
    institution: (String, bool, UiAction),
    continuity: bool,
    pointer: Pointer,
    actions: &mut Vec<UiAction>,
) {
    let SubsystemDetails { id, def, state } = subsystem;
    let bw = (content.w - 3.0 * 8.0) / 4.0;
    let by = content.bottom() - 44.0;
    let slot = |i: f32| Rect::new(content.x + i * (bw + 8.0), by, bw, 44.0);
    if continuity {
        let (label, enabled, action) = institution;
        if term_button(slot(0.0), &label, enabled, pointer) {
            actions.push(action);
        }
        if term_button(slot(3.0), "Back to care", true, pointer) {
            ctx.presentation.continuity_open.set(false);
        }
        return;
    }
    draw_repair_button(ctx, slot(0.0), id, def, state, pointer, actions);
    draw_upgrade_button(ctx, slot(1.0), id, def, state, pointer, actions);
    draw_train_button(ctx, slot(2.0), id, state, pointer, actions);
    if term_button(slot(3.0), "Continuity & culture", true, pointer) {
        ctx.presentation.continuity_open.set(true);
    }
}
