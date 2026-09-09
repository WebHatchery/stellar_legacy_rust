//! Explain the condition and consumables behind immediate ship repairs.
use super::*;
use crate::simulation::ship::{field_repair_target, full_repair_needed, RepairKind};

pub(super) fn build(ctx: &GameplayCtx<'_>, form: &mut Form) {
    let sim = ctx.sim;
    let ship = &sim.ship;
    let cfg = &ctx.data.config.repair;
    form.heading("Repairs");
    form.text(&format!(
        "Available: {} credits · {} minerals · {} spare parts",
        sim.resources.credits, sim.resources.minerals, ship.spare_parts
    ));
    if sim.contract.is_none() {
        form.text(&format!("Full refit restores hull, air and fuel to 100%, and supplies spare parts up to {}. Existing parts above that amount are kept.", cfg.full_parts_restock));
        let needed = full_repair_needed(sim, &ctx.data.config);
        let credits = (cfg.full_credits_cost - sim.resources.credits).max(0);
        let minerals = (cfg.full_minerals_cost - sim.resources.minerals).max(0);
        if !needed {
            form.text("Already fully refitted.");
        } else if credits > 0 || minerals > 0 {
            form.text(&format!(
                "Full refit needs {} more credits and {} more minerals.",
                credits, minerals
            ));
        }
        form.action(
            &format!(
                "Full repair & refit · {} credits · {} minerals",
                cfg.full_credits_cost, cfg.full_minerals_cost
            ),
            needed && credits == 0 && minerals == 0,
            UiAction::FullRepair,
        );
    } else {
        form.text("A full refit is available in port between voyages.");
    }
    form.text(&format!("Field repairs restore one system at a time, up to {:.0}% condition. Each repair spends {} spare parts and {} minerals.", cfg.field_ceiling * 100.0, cfg.field_parts_cost, cfg.field_minerals_cost));
    let parts = (cfg.field_parts_cost - ship.spare_parts).max(0);
    let minerals = (cfg.field_minerals_cost - sim.resources.minerals).max(0);
    let mut repairable = false;
    for (label, kind, current) in [
        ("Hull", RepairKind::Hull, ship.hull_integrity),
        ("Air", RepairKind::LifeSupport, ship.life_support),
    ] {
        if current >= cfg.field_ceiling {
            form.text(&format!(
                "{label} {:.0}% · Field repairs cannot improve this further",
                current * 100.0
            ));
            continue;
        }
        repairable = true;
        let target = field_repair_target(current, &ctx.data.config);
        form.action(
            &format!(
                "Repair {label} · {:.0}% → {:.0}%",
                current * 100.0,
                target * 100.0
            ),
            parts == 0 && minerals == 0,
            UiAction::FieldRepair(kind),
        );
    }
    if repairable && (parts > 0 || minerals > 0) {
        form.text(&format!(
            "One field repair needs {} more spare parts and {} more minerals.",
            parts, minerals
        ));
    }
}
