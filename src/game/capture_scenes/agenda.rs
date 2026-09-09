use super::*;
impl Game {
    pub(super) fn capture_agenda(&mut self, scene: &str) {
        use crate::simulation::projects;
        let mut sim = SimState::new_campaign(
            &self.data,
            "preservers",
            11,
            &crate::state::sim::founding_faction_ids(&self.data),
        );
        sim.contract = Some(contract::start_contract(
            self.data.contracts.get("deep_vein_survey").unwrap(),
            &sim,
        ));
        sim.ship.hull_integrity = 0.25;
        sim.ship.life_support = if scene == "agenda_recovery" {
            0.05
        } else {
            0.4
        };
        sim.population.morale = 0.3;
        let id =
            projects::queue_project(&mut sim, &self.data, "restore_crew_quarters", None).unwrap();
        for month in 1..=30 {
            sim.month_clock = month;
            let snapshot = projects::capture_month(&sim, &self.data);
            projects::advance_captured_month(&mut sim, &self.data, snapshot);
        }
        projects::pause_project(&mut sim, &self.data, id).unwrap();
        projects::queue_project(&mut sim, &self.data, "restore_hull", None).unwrap();
        if matches!(scene, "agenda_queue" | "agenda_catalogue") {
            projects::queue_project(&mut sim, &self.data, "overhaul_life_support", None).unwrap();
            let selected = projects::queue_project(
                &mut sim,
                &self.data,
                "train_replacement_cohort",
                Some("agriculture".into()),
            )
            .unwrap();
            projects::queue_project(
                &mut sim,
                &self.data,
                "optimise_hydroponics",
                Some("agriculture".into()),
            )
            .unwrap();
            self.presentation.selected_agenda.set(
                sim.projects
                    .jobs
                    .iter()
                    .position(|job| job.sequence_id == selected)
                    .unwrap(),
            );
            if scene == "agenda_catalogue" {
                self.presentation
                    .selected_agenda
                    .set(sim.projects.jobs.len());
            }
        }
        if scene == "agenda_review_blocked" {
            for _ in 0..self.data.config.projects.pause_grace_months + 2 {
                sim.month_clock += 1;
                let snapshot = projects::capture_month(&sim, &self.data);
                projects::advance_captured_month(&mut sim, &self.data, snapshot);
            }
            sim.resources.minerals = 0;
        }
        if scene.contains("review") || scene.contains("cancel") {
            self.project_cancel_confirm.set(Some(id));
            self.presentation
                .project_cancellation
                .set(scene.contains("cancel").then_some(id));
        } else {
            self.project_cancel_confirm.set(None);
        }
        let mut state = GameplayState::new(sim);
        state.screen = Screen::Agenda;
        self.state = crate::state::GameState::Gameplay(Box::new(state));
    }
}
