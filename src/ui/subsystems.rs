//! Subsystems screen (W5): the six ship modules — tier, condition, and the
//! institutional knowledge that gates repair — with the Repair / Upgrade /
//! Train verbs. Pure view: it reads `&SimState` and emits `UiAction` only.

use crate::data::GameData;
use crate::simulation::subsystems::{repair_target_condition, training_target_knowledge};
use crate::state::sim::factions::steward_decay_factor;
use crate::ui::{term, term_bar, term_button, term_panel, GameplayCtx, UiAction};
use macroquad::prelude::*;
use macroquad_toolkit::prelude::*;
use macroquad_toolkit::ui::{draw_text_block, draw_ui_text_ex, occlude, RectExt};

fn priced_action_label(action: &str, cost: i64, available: i64, unit: &str) -> String {
    if available >= cost {
        format!("{action} ({cost}{unit})")
    } else {
        format!("{action} · NEED {cost}{unit}")
    }
}

mod overview;
pub use overview::draw;
pub(crate) use overview::select_compartments;

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
    let in_port = ctx.sim.contract.is_none();

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
    let y = draw_card_identity(ctx, content, id, def, state, school, archived);
    draw_card_bars(ctx, content, id, def, state, y);
    let institution = institution_action(ctx, id, school, archived);
    draw_card_actions(
        ctx,
        content,
        id,
        def,
        state,
        in_port,
        institution,
        pointer,
        actions,
    );
}

fn draw_card_identity(
    ctx: &GameplayCtx<'_>,
    content: Rect,
    id: &str,
    def: &crate::data::subsystems::SubsystemDef,
    state: &crate::state::sim::subsystems::SubsystemState,
    school: Option<&crate::state::sim::institutions::SubsystemSchool>,
    archived: bool,
) -> f32 {
    let pips: String = (1..=3)
        .map(|t| if state.tier >= t { '●' } else { '○' })
        .collect();
    let family = if def.buffers_family.is_empty() {
        "habitat integrity".to_owned()
    } else {
        def.buffers_family.replace('_', " ")
    };
    let mut y = content.y + 32.0;
    draw_ui_text_ex(
        def.fitting_name(state.tier),
        content.x,
        y,
        TextStyle::new(18.0, term::primary()).params(),
    );
    y += 26.0;
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
        TextStyle::new(14.0, term::dim()).params(),
    );
    y + 32.0
}

fn draw_card_bars(
    ctx: &GameplayCtx<'_>,
    content: Rect,
    id: &str,
    def: &crate::data::subsystems::SubsystemDef,
    state: &crate::state::sim::subsystems::SubsystemState,
    y: f32,
) {
    term_bar(
        Rect::new(content.x, y, content.w, 18.0),
        state.condition,
        term::accent(),
        "CONDITION",
        &format!("{:.0}%", state.condition * 100.0),
    );
    let can_mend = state.knowledge >= def.repair_knowledge_required;
    term_bar(
        Rect::new(content.x, y + 24.0, content.w, 18.0),
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
    let Some(culture) =
        crate::simulation::culture::descriptor(ctx.data, id, &state.culture.descriptor_id)
    else {
        return;
    };
    let custodian = state
        .culture
        .custodian_faction_id
        .as_deref()
        .and_then(|faction_id| ctx.data.factions.get(faction_id))
        .map_or("NO LOCAL CUSTODIAN", |faction| faction.name.as_str());
    let memory = state
        .culture
        .remembered_event
        .as_deref()
        .unwrap_or("No compartment memory recorded.");
    let grievance = state
        .culture
        .grievance
        .as_deref()
        .unwrap_or("No active grievance.");
    draw_ui_text_ex(
        "LOCAL CULTURE",
        content.x,
        y + 58.0,
        TextStyle::new(13.0, term::primary()).params(),
    );
    draw_text_block(
        &format!(
            "{} · CUSTODIAN {}\n{}\nMEMORY {}\nGRIEVANCE {}",
            culture.label,
            custodian,
            crate::simulation::culture::effect_summary(culture),
            memory,
            grievance
        ),
        content.x,
        y + 66.0,
        content.w,
        76.0,
        10.0,
        2.0,
        term::dim(),
    );
}

fn institution_action(
    ctx: &GameplayCtx<'_>,
    id: &str,
    school: Option<&crate::state::sim::institutions::SubsystemSchool>,
    archived: bool,
) -> (String, bool, UiAction) {
    let cfg = &ctx.data.config;
    match school {
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
    }
}

fn draw_card_actions(
    ctx: &GameplayCtx<'_>,
    content: Rect,
    id: &str,
    def: &crate::data::subsystems::SubsystemDef,
    state: &crate::state::sim::subsystems::SubsystemState,
    in_port: bool,
    institution: (String, bool, UiAction),
    pointer: Pointer,
    actions: &mut Vec<UiAction>,
) {
    let bw = (content.w - 3.0 * 8.0) / 4.0;
    let by = content.bottom() - 44.0;
    draw_repair_button(
        ctx, content.x, by, bw, id, def, state, in_port, pointer, actions,
    );
    draw_upgrade_button(
        ctx,
        content.x + bw + 8.0,
        by,
        bw,
        id,
        def,
        state,
        in_port,
        pointer,
        actions,
    );
    draw_train_button(
        ctx,
        content.x + 2.0 * (bw + 8.0),
        by,
        bw,
        id,
        state,
        pointer,
        actions,
    );
    let (label, enabled, action) = institution;
    if term_button(
        Rect::new(content.x + 3.0 * (bw + 8.0), by, bw, 44.0),
        &label,
        enabled,
        pointer,
    ) {
        actions.push(action);
    }
}

fn draw_repair_button(
    ctx: &GameplayCtx<'_>,
    x: f32,
    y: f32,
    width: f32,
    id: &str,
    def: &crate::data::subsystems::SubsystemDef,
    state: &crate::state::sim::subsystems::SubsystemState,
    in_port: bool,
    pointer: Pointer,
    actions: &mut Vec<UiAction>,
) {
    let can_mend = state.knowledge >= def.repair_knowledge_required;
    let ceiling = if in_port {
        1.0
    } else {
        ctx.data.config.repair.field_ceiling
    };
    let repair_ok = can_mend
        && state.condition < ceiling
        && ctx.sim.ship.spare_parts >= def.repair_parts_cost
        && ctx.sim.resources.minerals >= def.repair_minerals_cost;
    let label = if !can_mend {
        format!(
            "REPAIR · NEED {:.0}% KNOWLEDGE",
            def.repair_knowledge_required * 100.0
        )
    } else if state.condition >= ceiling {
        "SOUND".to_owned()
    } else if ctx.sim.ship.spare_parts < def.repair_parts_cost
        || ctx.sim.resources.minerals < def.repair_minerals_cost
    {
        format!(
            "REPAIR · NEED {} parts / {} minerals",
            def.repair_parts_cost, def.repair_minerals_cost
        )
    } else {
        format!(
            "REPAIR ({}p·{}min)",
            def.repair_parts_cost, def.repair_minerals_cost
        )
    };
    if term_button(Rect::new(x, y, width, 44.0), &label, repair_ok, pointer) {
        actions.push(UiAction::RepairSubsystem(id.to_owned()));
    }
}

fn draw_upgrade_button(
    ctx: &GameplayCtx<'_>,
    x: f32,
    y: f32,
    width: f32,
    id: &str,
    def: &crate::data::subsystems::SubsystemDef,
    state: &crate::state::sim::subsystems::SubsystemState,
    in_port: bool,
    pointer: Pointer,
    actions: &mut Vec<UiAction>,
) {
    let next = def.next_fitting(state.tier);
    let label = match next {
        Some(t)
            if in_port
                && (ctx.sim.resources.credits < t.cost.credits
                    || ctx.sim.resources.minerals < t.cost.minerals) =>
        {
            format!(
                "UPGRADE · NEED {} credits / {} minerals",
                t.cost.credits, t.cost.minerals
            )
        }
        Some(t) if in_port => format!("UPGRADE ({}cr)", t.cost.credits),
        Some(_) => "UPGRADE · PORT".to_owned(),
        None => "MAX TIER".to_owned(),
    };
    let enabled = in_port
        && next.is_some_and(|t| {
            ctx.sim.resources.credits >= t.cost.credits
                && ctx.sim.resources.minerals >= t.cost.minerals
        });
    if term_button(Rect::new(x, y, width, 44.0), &label, enabled, pointer) {
        actions.push(UiAction::UpgradeSubsystem(id.to_owned()));
    }
}

fn draw_train_button(
    ctx: &GameplayCtx<'_>,
    x: f32,
    y: f32,
    width: f32,
    id: &str,
    state: &crate::state::sim::subsystems::SubsystemState,
    pointer: Pointer,
    actions: &mut Vec<UiAction>,
) {
    let cost = ctx.data.config.subsystems.train_cost_credits;
    let target = training_target_knowledge(ctx.sim, ctx.data, id).unwrap_or(state.knowledge);
    let complete = target <= state.knowledge + f32::EPSILON;
    let enabled = !complete && ctx.sim.resources.credits >= cost;
    let label = if complete {
        "MASTERED".to_owned()
    } else if ctx.sim.resources.credits < cost {
        format!("TRAIN · NEED {cost} credits")
    } else {
        format!("TRAIN TO {:.0}% · {cost}cr", target * 100.0)
    };
    if term_button(Rect::new(x, y, width, 44.0), &label, enabled, pointer) {
        actions.push(UiAction::TrainSubsystemKnowledge(id.to_owned()));
    }
}
#[allow(dead_code, unused_imports)]
mod tests {
    include!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/unit/ui/subsystems/tests.rs"
    ));
}
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
    occlude(area);
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
    let _modal_region = Region::on(modal, term::panel());
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
        draw_custody_candidate(ctx, content, subsystem_id, state, y, pointer, actions);
        y += 82.0;
    }
}

fn draw_custody_candidate(
    ctx: &GameplayCtx<'_>,
    content: Rect,
    subsystem_id: &str,
    state: &crate::state::sim::factions::FactionState,
    y: f32,
    pointer: Pointer,
    actions: &mut Vec<UiAction>,
) {
    let Some(faction) = ctx.data.factions.get(&state.faction_id) else {
        return;
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
    draw_text_block(
        &format!(
            "{} members · approval {:.0}% → {:.0}% · CARE ×{care_factor:.2} · {craft}",
            state.members,
            state.approval * 100.0,
            approval_after * 100.0
        ),
        row.x + 12.0,
        row.y + 38.0,
        row.w - 236.0,
        32.0,
        13.0,
        2.0,
        term::dim(),
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
}
