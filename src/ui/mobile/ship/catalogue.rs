//! Drydock purchases with explicit installed state and complete prices.
use super::*;
use crate::data::ResourceDelta;

pub(super) fn build(ctx: &GameplayCtx<'_>, form: &mut Form) {
    form.heading("Drydock catalogue");
    form.text("Purchase and fit replaces the part only. Commissioning a hull also fully restores hull, air and fuel, restocks spare parts and raises morale and unity.");
    for (label, kind, current) in [
        (
            "Hulls",
            ComponentKind::Hull,
            Some(ctx.sim.ship.hull.as_str()),
        ),
        (
            "Engines",
            ComponentKind::Engine,
            Some(ctx.sim.ship.engine.as_str()),
        ),
        (
            "Weapons",
            ComponentKind::Weapon,
            ctx.sim.ship.weapon.as_deref(),
        ),
    ] {
        form.heading(label);
        for part in ctx
            .data
            .ship_components
            .list(kind)
            .iter()
            .filter(|p| !p.acquisition.is_mission_only())
        {
            let installed = current == Some(part.id.as_str());
            form.heading(&format!(
                "{}{}",
                part.name,
                if installed { " · Installed" } else { "" }
            ));
            form.text(&part.description);
            form.text(&format!(
                "Cargo {} · Crew capacity {} · Speed {} · Combat {} · Fuel regeneration {}",
                part.stats.cargo,
                part.stats.crew_capacity,
                part.stats.speed,
                part.stats.combat,
                part.stats.fuel_regen
            ));
            if installed {
                form.text("This part is already fitted. Use Repairs above for a full refit.");
                continue;
            }
            if ctx.sim.ship.salvage.contains(&part.id) {
                form.text("Already in your salvage hold. Use Recovered fittings below to install it free in port.");
                continue;
            }
            if kind == ComponentKind::Hull {
                let cm = &ctx.data.config.commission;
                let cost = ResourceDelta {
                    credits: part.cost.credits + cm.premium_credits,
                    minerals: part.cost.minerals + cm.premium_minerals,
                    energy: part.cost.energy,
                    ..Default::default()
                };
                form.text(&format!(
                    "Commission bonus: up to {} spare parts · morale +{:.0} percentage points · unity +{:.0} percentage points",
                    ctx.data.config.repair.full_parts_restock, cm.hope_morale * 100.0, cm.hope_unity * 100.0
                ));
                purchase(
                    ctx,
                    form,
                    "Commission with full refit",
                    cost,
                    UiAction::CommissionShip(part.id.clone()),
                );
            }
            purchase(
                ctx,
                form,
                "Purchase and fit",
                part.cost,
                UiAction::PurchaseComponent(kind, part.id.clone()),
            );
        }
    }
}

pub(super) fn purchase(
    ctx: &GameplayCtx<'_>,
    form: &mut Form,
    label: &str,
    cost: ResourceDelta,
    action: UiAction,
) {
    let resources = &ctx.sim.resources;
    let amounts = [
        ("credits", cost.credits, resources.credits),
        ("energy", cost.energy, resources.energy),
        ("minerals", cost.minerals, resources.minerals),
        ("food", cost.food, resources.food),
        ("influence", cost.influence, resources.influence),
    ];
    let price = amounts
        .iter()
        .filter(|(_, cost, _)| *cost != 0)
        .map(|(unit, cost, _)| format!("{cost} {unit}"))
        .collect::<Vec<_>>()
        .join(" · ");
    let missing = amounts
        .iter()
        .filter(|(_, cost, held)| cost > held && *cost > 0)
        .map(|(unit, cost, held)| format!("{} {unit}", cost - held))
        .collect::<Vec<_>>();
    if !missing.is_empty() {
        form.text(&format!("{label} needs {} more.", missing.join(" · ")));
    }
    form.action(
        &format!(
            "{label} · {}",
            if price.is_empty() { "Free" } else { &price }
        ),
        missing.is_empty() && ctx.sim.contract.is_none(),
        action,
    );
}
