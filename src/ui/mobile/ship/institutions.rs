//! Funding and continuity for a compartment's discipline.
use super::*;

pub(super) fn build(ctx: &GameplayCtx<'_>, form: &mut Form, id: &str) {
    let sim = ctx.sim;
    let cfg = &ctx.data.config.crew;
    let school = sim
        .subsystem_schools
        .iter()
        .find(|school| school.subsystem_id == id);
    let archive = sim
        .procedure_archives
        .iter()
        .find(|archive| archive.subsystem_id == id);
    form.heading("Preserve this discipline");
    form.text(&format!("A funded school reduces knowledge loss at succession and officer departure by {:.0}%. It preserves knowledge rather than immediately training it.", cfg.school_decay_reduction * 100.0));
    if let Some(school) = school {
        form.text(&format!(
            "School founded in Year {} · {} Year {}",
            school.founded_year,
            if school.supported_until_year >= sim.year() {
                "Funded through"
            } else {
                "Funding lapsed after"
            },
            school.supported_until_year
        ));
    } else {
        form.text("No school established.");
    }
    let until = school
        .map_or(sim.year(), |s| s.supported_until_year.max(sim.year()))
        .saturating_add(cfg.school_support_years);
    let cost = if school.is_some() {
        cfg.school_upkeep_credits
    } else {
        cfg.school_cost_credits
    };
    let label = if school.is_some() {
        "Extend school funding"
    } else {
        "Establish school"
    };
    funded(
        ctx,
        form,
        &format!("{label} through Year {until}"),
        cost,
        UiAction::EstablishSchool(id.to_owned()),
    );
    if let Some(archive) = archive {
        form.text(&format!(
            "Procedure archive compiled in Year {} · {} departing experts recorded",
            archive.compiled_year,
            archive.preserved_experts.len()
        ));
    } else if school.is_some() {
        form.text(&format!("A procedure archive reduces the remaining knowledge loss when an officer leaves by {:.0}%, and records the experts whose knowledge it preserves.", cfg.archive_loss_reduction * 100.0));
        funded(
            ctx,
            form,
            "Compile archive",
            cfg.archive_cost_credits,
            UiAction::CompileProcedureArchive(id.to_owned()),
        );
    }
    if let Some(school) = school {
        if let Some(custodian) = &school.custodian_faction_id {
            let name = ctx
                .data
                .factions
                .get(custodian)
                .map_or(custodian.as_str(), |f| f.name.as_str());
            form.text(&format!("Discipline custodians: {name}"));
        } else {
            form.text("Appoint one people as custodians. Their approval affects the compartment's care while the school is funded. Custody cannot be reassigned through this control.");
            let missing = (cfg.custody_influence_cost - sim.resources.influence).max(0);
            if missing > 0 {
                form.text(&format!("Custody requires {missing} more influence."));
            }
            form.action(
                &format!(
                    "Choose discipline custodians · {} influence",
                    cfg.custody_influence_cost
                ),
                missing == 0,
                UiAction::BeginDisciplineCustody(id.to_owned()),
            );
        }
    }
}

fn funded(ctx: &GameplayCtx<'_>, form: &mut Form, label: &str, cost: i64, action: UiAction) {
    let missing = (cost - ctx.sim.resources.credits).max(0);
    if missing > 0 {
        form.text(&format!("{label} requires {missing} more credits."));
    }
    form.action(&format!("{label} · {cost} credits"), missing == 0, action);
}
