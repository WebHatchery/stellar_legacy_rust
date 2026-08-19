//! Subsystems screen (W5): the six ship modules — tier, condition, and the
//! institutional knowledge that gates repair — with the Repair / Upgrade /
//! Train verbs. Pure view: it reads `&SimState` and emits `UiAction` only.

use crate::data::GameData;
use crate::simulation::subsystems::{repair_target_condition, training_target_knowledge};
use crate::state::sim::factions::steward_decay_factor;
use crate::ui::{term, term_bar, term_button, term_panel, GameplayCtx, UiAction};
use macroquad::prelude::*;
use macroquad_toolkit::prelude::*;
use macroquad_toolkit::ui::{draw_ui_text_ex, RectExt};

fn priced_action_label(action: &str, cost: i64, available: i64, unit: &str) -> String {
    if available >= cost {
        format!("{action} ({cost}{unit})")
    } else {
        format!("NEED {cost}{unit}")
    }
}

pub fn draw(ctx: &GameplayCtx<'_>, area: Rect, pointer: Pointer, actions: &mut Vec<UiAction>) {
    const GAP: f32 = 12.0;
    let col_w = (area.w - GAP) / 2.0;
    let row_h = (area.h - 2.0 * GAP) / 3.0;
    let mut blocked_actions = Vec::new();
    for (i, id) in GameData::sorted_ids(&ctx.data.subsystems)
        .into_iter()
        .enumerate()
    {
        let col = (i % 2) as f32;
        let row = (i / 2) as f32;
        let rect = Rect::new(
            area.x + col * (col_w + GAP),
            area.y + row * (row_h + GAP),
            col_w,
            row_h,
        );
        let action_sink = if ctx.custody_picker.is_some() {
            &mut blocked_actions
        } else {
            &mut *actions
        };
        draw_card(ctx, rect, &id, pointer, action_sink);
    }
    if let Some(subsystem_id) = ctx.custody_picker {
        draw_custody_picker(ctx, area, subsystem_id, pointer, actions);
    }
}

fn draw_card(
    ctx: &GameplayCtx<'_>,
    rect: Rect,
    id: &str,
    pointer: Pointer,
    actions: &mut Vec<UiAction>,
) {
    let (Some(def), Some(state)) = (ctx.data.subsystems.get(id), ctx.sim.subsystems.get(id)) else {
        return;
    };
    let cfg = &ctx.data.config;
    let in_port = ctx.sim.contract.is_none();

    term_panel(rect, Some(&def.name.to_uppercase()));
    let content = rect.inset(14.0);
    let mut y = content.y + 32.0;

    // The version currently fitted, its tier pips, and the family it buffers.
    let pips: String = (1..=3)
        .map(|t| if state.tier >= t { '●' } else { '○' })
        .collect();
    let family = if def.buffers_family.is_empty() {
        "habitat integrity".to_owned()
    } else {
        def.buffers_family.replace('_', " ")
    };
    draw_ui_text_ex(
        def.fitting_name(state.tier),
        content.x,
        y,
        TextStyle::new(13.0, term::primary()).params(),
    );
    y += 16.0;
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
    let repair_target = repair_target_condition(ctx.sim, ctx.data, id).unwrap_or(state.condition);
    let mend = format!("REPAIR TO {:.0}%", repair_target * 100.0);
    let detail = school.map_or_else(
        || format!("TIER {pips} · buffers {family} · {mend}"),
        |school| {
            let support = if school.supported_until_year >= ctx.sim.year() {
                format!("SCHOOL→Y{}", school.supported_until_year)
            } else {
                "SCHOOL LAPSED".to_owned()
            };
            let archive = if archived { "ARCHIVED" } else { "NO ARCHIVE" };
            let custodian = school
                .custodian_faction_id
                .as_deref()
                .and_then(|faction_id| ctx.data.factions.get(faction_id))
                .map(|faction| faction.name.as_str())
                .unwrap_or("NO CUSTODIAN");
            format!("TIER {pips} · {support} · {archive} · {custodian} · {mend}")
        },
    );
    draw_ui_text_ex(
        &detail,
        content.x,
        y,
        TextStyle::new(11.0, term::dim()).params(),
    );
    y += 22.0;

    term_bar(
        Rect::new(content.x, y, content.w, 18.0),
        state.condition,
        term::accent(),
        "CONDITION",
        &format!("{:.0}%", state.condition * 100.0),
    );
    y += 24.0;

    // Knowledge — red when it has fallen below the repair threshold.
    let can_mend = state.knowledge >= def.repair_knowledge_required;
    term_bar(
        Rect::new(content.x, y, content.w, 18.0),
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

    // Institutional continuity stays attached to the discipline it protects.
    // The single focused verb advances from school, to archive, to faction
    // custody, then becomes the school's periodic recommitment.
    let (institution_label, institution_ok, institution_action) = match school {
        None => (
            priced_action_label(
                "SCHOOL",
                cfg.crew.school_cost_credits,
                ctx.sim.resources.credits,
                "cr",
            ),
            ctx.sim.resources.credits >= cfg.crew.school_cost_credits,
            UiAction::EstablishSchool(id.to_owned()),
        ),
        Some(_) if !archived => (
            priced_action_label(
                "ARCHIVE",
                cfg.crew.archive_cost_credits,
                ctx.sim.resources.credits,
                "cr",
            ),
            ctx.sim.resources.credits >= cfg.crew.archive_cost_credits,
            UiAction::CompileProcedureArchive(id.to_owned()),
        ),
        Some(school) if school.custodian_faction_id.is_none() => (
            priced_action_label(
                "CUSTODY",
                cfg.crew.custody_influence_cost,
                ctx.sim.resources.influence,
                "inf",
            ),
            ctx.sim.resources.influence >= cfg.crew.custody_influence_cost,
            UiAction::BeginDisciplineCustody(id.to_owned()),
        ),
        Some(_) => (
            priced_action_label(
                &format!("EXTEND +{}Y", cfg.crew.school_support_years),
                cfg.crew.school_upkeep_credits,
                ctx.sim.resources.credits,
                "cr",
            ),
            ctx.sim.resources.credits >= cfg.crew.school_upkeep_credits,
            UiAction::EstablishSchool(id.to_owned()),
        ),
    };
    // --- Verbs: Repair / Upgrade (port) / Train ---
    let bw = (content.w - 3.0 * 8.0) / 4.0;
    let by = content.bottom() - 40.0;
    let ceiling = if in_port {
        1.0
    } else {
        cfg.repair.field_ceiling
    };
    let repair_ok = can_mend
        && state.condition < ceiling
        && ctx.sim.ship.spare_parts >= def.repair_parts_cost
        && ctx.sim.resources.minerals >= def.repair_minerals_cost;
    let repair_label = if !can_mend {
        format!("NEED {:.0}% KNOW", def.repair_knowledge_required * 100.0)
    } else if state.condition >= ceiling {
        "SOUND".to_owned()
    } else if ctx.sim.ship.spare_parts < def.repair_parts_cost
        || ctx.sim.resources.minerals < def.repair_minerals_cost
    {
        format!(
            "NEED {}p·{}min",
            def.repair_parts_cost, def.repair_minerals_cost
        )
    } else {
        format!(
            "REPAIR ({}p·{}min)",
            def.repair_parts_cost, def.repair_minerals_cost
        )
    };
    if term_button(
        Rect::new(content.x, by, bw, 40.0),
        &repair_label,
        repair_ok,
        pointer,
    ) {
        actions.push(UiAction::RepairSubsystem(id.to_owned()));
    }

    // Upgrade: port-only, pays the next fitting's cost, caps at the top version.
    let next = def.next_fitting(state.tier);
    let upgrade_label = match next {
        Some(t)
            if in_port
                && (ctx.sim.resources.credits < t.cost.credits
                    || ctx.sim.resources.minerals < t.cost.minerals) =>
        {
            format!("NEED {}cr·{}min", t.cost.credits, t.cost.minerals)
        }
        Some(t) if in_port => format!("UPGRADE ({}cr)", t.cost.credits),
        Some(_) => "UPGRADE · PORT".to_owned(),
        None => "MAX TIER".to_owned(),
    };
    let upgrade_ok = in_port
        && next.is_some_and(|t| {
            ctx.sim.resources.credits >= t.cost.credits
                && ctx.sim.resources.minerals >= t.cost.minerals
        });
    if term_button(
        Rect::new(content.x + bw + 8.0, by, bw, 40.0),
        &upgrade_label,
        upgrade_ok,
        pointer,
    ) {
        actions.push(UiAction::UpgradeSubsystem(id.to_owned()));
    }

    // Train: anytime, raises this subsystem's knowledge.
    let training_target =
        training_target_knowledge(ctx.sim, ctx.data, id).unwrap_or(state.knowledge);
    let training_complete = training_target <= state.knowledge + f32::EPSILON;
    let train_ok =
        !training_complete && ctx.sim.resources.credits >= cfg.subsystems.train_cost_credits;
    let train_label = if training_complete {
        "MASTERED".to_owned()
    } else if ctx.sim.resources.credits < cfg.subsystems.train_cost_credits {
        format!("NEED {}cr", cfg.subsystems.train_cost_credits)
    } else {
        format!(
            "TRAIN TO {:.0}% · {}cr",
            training_target * 100.0,
            cfg.subsystems.train_cost_credits
        )
    };
    if term_button(
        Rect::new(content.x + 2.0 * (bw + 8.0), by, bw, 40.0),
        &train_label,
        train_ok,
        pointer,
    ) {
        actions.push(UiAction::TrainSubsystemKnowledge(id.to_owned()));
    }
    if term_button(
        Rect::new(content.x + 3.0 * (bw + 8.0), by, bw, 40.0),
        &institution_label,
        institution_ok,
        pointer,
    ) {
        actions.push(institution_action);
    }
}

#[cfg(test)]
mod tests;

fn draw_custody_picker(
    ctx: &GameplayCtx<'_>,
    area: Rect,
    subsystem_id: &str,
    pointer: Pointer,
    actions: &mut Vec<UiAction>,
) {
    let Some(subsystem) = ctx.data.subsystems.get(subsystem_id) else {
        actions.push(UiAction::CancelDisciplineCustody);
        return;
    };
    let candidates: Vec<_> = ctx
        .sim
        .factions
        .iter()
        .filter(|faction| faction.is_aboard())
        .collect();
    draw_rectangle(
        area.x,
        area.y,
        area.w,
        area.h,
        Color::new(0.0, 0.0, 0.0, 0.82),
    );
    let height = 148.0 + candidates.len() as f32 * 82.0;
    let modal = Rect::new(
        area.x + (area.w - 760.0) * 0.5,
        area.y + (area.h - height) * 0.5,
        760.0,
        height,
    );
    draw_surface(
        modal,
        &SurfaceStyle::new(term::panel())
            .with_border(2.0, term::accent())
            .with_header(48.0, term::panel_header())
            .with_header_divider(1.0, term::accent()),
    );
    draw_text_centered_in_box_ex(
        &format!("GRANT CUSTODY // {}", subsystem.name.to_uppercase()),
        modal.x,
        modal.y,
        modal.w,
        48.0,
        TextStyle::new(15.0, term::accent()),
    );
    if term_button(
        Rect::new(modal.right() - 110.0, modal.y + 2.0, 102.0, 44.0),
        "CANCEL",
        true,
        pointer,
    ) {
        actions.push(UiAction::CancelDisciplineCustody);
    }
    let content = modal.inset(18.0);
    draw_text_block(
        &format!(
            "Choose the people who will hold this discipline across successions. The grant costs {} influence and raises that people's approval by {:.0}%. CARE below ×1.00 slows annual wear while the school is supported.",
            ctx.data.config.crew.custody_influence_cost,
            ctx.data.config.crew.custody_approval_gain * 100.0
        ),
        content.x,
        content.y + 36.0,
        content.w,
        46.0,
        12.0,
        3.0,
        term::dim(),
    );
    let mut y = content.y + 86.0;
    for state in candidates {
        let Some(faction) = ctx.data.factions.get(&state.faction_id) else {
            continue;
        };
        let row = Rect::new(content.x, y, content.w, 72.0);
        draw_surface(
            row,
            &SurfaceStyle::new(term::surface_inset()).with_border(1.0, term::faint()),
        );
        let native = faction.tended_subsystem == subsystem_id;
        let approval_after = (state.approval + ctx.data.config.crew.custody_approval_gain).min(1.0);
        let care_factor = steward_decay_factor(ctx.data, approval_after);
        let craft = if native {
            "NATIVE CRAFT".to_owned()
        } else {
            let tended = ctx
                .data
                .subsystems
                .get(&faction.tended_subsystem)
                .map(|definition| definition.name.as_str())
                .unwrap_or("no named discipline");
            format!("CROSS-DISCIPLINE · tends {tended}")
        };
        draw_ui_text_ex(
            &faction.name,
            row.x + 12.0,
            row.y + 24.0,
            TextStyle::new(
                15.0,
                if native {
                    term::accent()
                } else {
                    term::primary()
                },
            )
            .params(),
        );
        draw_ui_text_ex(
            &format!(
                "{} members · approval {:.0}% → {:.0}% · CARE ×{care_factor:.2} · {craft}",
                state.members,
                state.approval * 100.0,
                approval_after * 100.0
            ),
            row.x + 12.0,
            row.y + 48.0,
            TextStyle::new(11.0, term::dim()).params(),
        );
        let enabled = ctx.sim.resources.influence >= ctx.data.config.crew.custody_influence_cost;
        if term_button(
            Rect::new(row.right() - 212.0, row.y + 14.0, 198.0, 44.0),
            "GRANT CUSTODY",
            enabled,
            pointer,
        ) {
            actions.push(UiAction::GrantDisciplineCustody {
                subsystem_id: subsystem_id.to_owned(),
                faction_id: state.faction_id.clone(),
            });
        }
        y += 82.0;
    }
}
