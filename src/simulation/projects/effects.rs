use super::*;
use crate::data::PopulationDelta;
pub(super) fn deliver_stage(
    sim: &mut SimState,
    data: &GameData,
    index: usize,
    definition: &ProjectDefinition,
    stage: u32,
    target_id: Option<&str>,
) {
    let job = &mut sim.projects.jobs[index];
    job.delivered_stages = stage;
    job.delivered_months.push(sim.month_clock);

    if definition.divisible || stage == definition.stage_count() {
        apply_effect(
            sim,
            data,
            definition,
            target_id,
            stage == definition.stage_count(),
        );
    }
}

fn apply_effect(
    sim: &mut SimState,
    data: &GameData,
    definition: &ProjectDefinition,
    target_id: Option<&str>,
    final_stage: bool,
) {
    let stages = if definition.divisible {
        definition.stage_count() as f32
    } else {
        1.0
    };
    let effect = &definition.effect;
    match definition.kind {
        ProjectKind::ServiceSubsystem | ProjectKind::SteriliseGrowingSystems => {
            if let Some(id) = target_id.or(Some("agriculture")) {
                if let Some(state) = sim.subsystems.get_mut(id) {
                    state.condition =
                        (state.condition + effect.condition_gain / stages).clamp(0.0, 1.0);
                }
                let maintenance_id = format!("maintenance:{id}");
                if sim.subsystems.get(id).is_some_and(|state| {
                    state.condition > data.config.projects.maintenance_condition_threshold
                }) {
                    issues::resolve_issue(sim, &maintenance_id, &definition.name);
                }
            }
        }
        ProjectKind::RestoreHull => {
            sim.ship.hull_integrity =
                (sim.ship.hull_integrity + effect.hull_gain / stages).clamp(0.0, 1.0);
        }
        ProjectKind::OverhaulLifeSupport => {
            sim.ship.life_support =
                (sim.ship.life_support + effect.life_support_gain / stages).clamp(0.0, 1.0);
        }
        ProjectKind::TrainReplacementCohort => {
            if let Some(id) = target_id {
                if let Some(state) = sim.subsystems.get_mut(id) {
                    state.knowledge =
                        (state.knowledge + effect.knowledge_gain / stages).clamp(0.0, 1.0);
                }
            }
        }
        ProjectKind::OptimiseHydroponics => {
            sim.projects.hydroponics_bonus = (sim.projects.hydroponics_bonus
                + effect.food_production_bonus / stages)
                .min(data.config.projects.maximum_hydroponics_bonus);
        }
        ProjectKind::RestoreCrewQuarters => {
            sim.population.apply(&PopulationDelta {
                morale: effect.morale_recovery / stages,
                unity: effect.unity_recovery / stages,
                ..Default::default()
            });
        }
        ProjectKind::EstablishSeedProgramme => {
            if final_stage {
                if let Some(capability) = &effect.capability {
                    if !sim.projects.has_capability(capability) {
                        sim.projects.capabilities.push(capability.clone());
                        sim.push_log(format!("Capability completed: {capability}."));
                    }
                }
            }
        }
    }
}
