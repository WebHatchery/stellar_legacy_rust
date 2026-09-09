//! Player-facing details for recovered fittings.
use super::*;
use crate::simulation::ship::{install_eligibility, InstallEligibility};

pub(super) fn build(ctx: &GameplayCtx<'_>, form: &mut Form) {
    form.heading("Recovered fittings");
    if ctx.sim.ship.salvage.is_empty() {
        form.text(
            "The salvage hold is empty. Parts recovered on voyages appear here for installation.",
        );
        return;
    }
    let port = ctx.sim.contract.is_none();
    let cfg = &ctx.data.config.field_install;
    for id in &ctx.sim.ship.salvage {
        let Some((kind, part)) = ctx.data.ship_components.find_any(id) else {
            form.text("A recovered fitting is not recognised by the current ship catalogue.");
            continue;
        };
        form.heading(&part.name);
        form.text(&part.description);
        let current = match kind {
            ComponentKind::Hull => Some(ctx.sim.ship.hull.as_str()),
            ComponentKind::Engine => Some(ctx.sim.ship.engine.as_str()),
            ComponentKind::Weapon => ctx.sim.ship.weapon.as_deref(),
        };
        let current_name = current
            .and_then(|id| ctx.data.ship_components.find(kind, id))
            .map_or("Empty slot", |part| part.name.as_str());
        form.text(&format!("Replaces: {current_name}\nCargo {} · Crew capacity {} · Speed {} · Combat {} · Fuel regeneration {}", part.stats.cargo, part.stats.crew_capacity, part.stats.speed, part.stats.combat, part.stats.fuel_regen));
        if port {
            form.text("Drydock installation is free.");
        } else if part.field_installable {
            form.text(&format!("Underway installation: {} spare parts · {} minerals. Requires an engineer with skill {} or higher.", cfg.parts_cost, cfg.minerals_cost, cfg.skill_required));
        }
        let eligibility = install_eligibility(ctx.sim, ctx.data, id);
        match eligibility {
            InstallEligibility::Ready => {}
            InstallEligibility::NeedsDrydock => form.text("This fitting needs a drydock. Install it in port between voyages."),
            InstallEligibility::NeedsEngineer => form.text("No engineer aboard meets the required skill. Check People → Officers to train your engineer or install in port."),
            InstallEligibility::NeedsConsumables => form.text(&format!("Missing {} spare parts and {} minerals for installation.", (cfg.parts_cost - ctx.sim.ship.spare_parts).max(0), (cfg.minerals_cost - ctx.sim.resources.minerals).max(0))),
            InstallEligibility::NotSalvaged => form.text("This fitting is no longer available in the salvage hold."),
        }
        form.action(
            &format!("Install {}", part.name),
            eligibility == InstallEligibility::Ready,
            UiAction::InstallSalvage(id.clone()),
        );
    }
}
