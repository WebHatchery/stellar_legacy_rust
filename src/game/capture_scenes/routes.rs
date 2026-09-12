//! Scene-specific state seeding for deterministic release captures.
//!
//! Each route owns one coherent presentation state so the capture entry point
//! remains a small dispatcher and no single setup function becomes a second
//! game loop.

use super::Game;
use crate::simulation::{contract, tick};
use crate::state::{GameplayState, MenuState, Screen, SimState};
use crate::ui;
use macroquad_toolkit::achievements::Achievements;

pub(super) fn capture(game: &mut Game, scene: &str) {
    match scene {
        "menu" => game.capture_menu(),
        "welcome" => game.capture_welcome(),
        "green" | "founding" | "founding_full" => game.capture_green(scene),
        "crt_off" => game.capture_crt_off(),
        "settings" => game.capture_settings(),
        "help" => game.capture_help(),
        "heritage" => game.capture_heritage(),
        "boot" => game.capture_boot(),
        "log" => game.capture_log(),
        "obligation_watch" => game.capture_obligation_watch(),
        "event"
        | "event_succession"
        | "event_mascot_succession"
        | "event_custodian"
        | "event_fuel"
        | "event_fuel_paused" => game.capture_event(scene),
        "event_obligation" => game.capture_event_obligation(),
        "event_obligation_due" | "event_obligation_unaffordable" => {
            game.capture_event_obligation_due(scene)
        }
        "crew" | "crew_recruitment" | "people_officers" | "people_factions" | "people_council" => {
            game.capture_crew(scene)
        }
        "ship" | "ship_blueprint" | "ship_refitted" | "ship_modules" => game.capture_ship(scene),
        "ship_underway" => game.capture_ship_underway(),
        "ship_underway_corvette" => game.capture_ship_underway_corvette(),
        "ship_underway_ark" => game.capture_ship_underway_ark(),
        "ship_underway_prow" => game.capture_ship_underway_prow(),
        "ship_underway_ring" => game.capture_ship_underway_ring(),
        "subsystems" | "institutions" => game.capture_subsystems(),
        "custody" => game.capture_custody(),
        "market" => game.capture_market(),
        "abort" => game.capture_abort(),
        "contracts" => game.capture_contracts(),
        "prep" | "tutorial" | "tutorial_complete" | "tutorial_navigation" => {
            game.capture_prep(scene)
        }
        "drydock" => game.capture_drydock(),
        "contract_active" | "posture" => game.capture_contract_active(),
        "authority_review" => game.capture_authority_review(),
        "dilemma" => game.capture_dilemma(),
        "dilemma_combat" => game.capture_dilemma_combat(),
        "chronicle" | "mission_archive" | "obligation_history" | "obligation_archive"
        | "milestones" => game.capture_chronicle(scene),
        "gameover" => game.capture_gameover(),
        "debrief" | "debrief_captains" | "debrief_moments" | "debrief_accounting" => {
            game.capture_debrief()
        }
        "dashboard_risk" => game.capture_dashboard_risk(),
        "dashboard_repair" | "ship_repair" => game.capture_dashboard_repair(scene),
        _ => game.capture_default(),
    }
}

impl Game {
    fn capture_menu(&mut self) {
        self.state = crate::state::GameState::Menu(MenuState::new(false));
    }

    fn capture_welcome(&mut self) {
        // The first-run orientation overlay above the new-game picker,
        // where it greets the commander after choosing NEW GAME.
        let mut menu = MenuState::new(false);
        menu.phase = crate::state::MenuPhase::NewGame;
        self.state = crate::state::GameState::Menu(menu);
        self.welcome_open = true;
    }

    fn capture_green(&mut self, scene: &str) {
        // The new-game picker on the green (P1) tube, to verify the recolor.
        if scene == "green" {
            self.display.phosphor = crate::settings::Phosphor::Green;
        }
        self.crt_style = self.display.crt_style();
        ui::term::set_phosphor(self.display.phosphor);
        let mut menu = MenuState::new(true);
        menu.phase = crate::state::MenuPhase::NewGame;
        if scene == "founding_full" {
            menu.selected_factions = crate::state::sim::founding_faction_ids(&self.data);
        }
        self.state = crate::state::GameState::Menu(menu);
    }

    fn capture_crt_off(&mut self) {
        self.display.crt_enabled = false;
        let mut sim = SimState::new_campaign(
            &self.data,
            "preservers",
            0xC0FFEE,
            &crate::state::sim::founding_faction_ids(&self.data),
        );
        if let Some(template) = self.data.contracts.get("founding_colony") {
            sim.contract = Some(contract::start_contract(template, &sim));
        }
        self.state = crate::state::GameState::Gameplay(Box::new(GameplayState::new(sim)));
    }

    fn capture_settings(&mut self) {
        // Delegate one category so the capture shows both toggle states.
        self.delegation_defaults.mission_milestone = true;
        self.state = crate::state::GameState::Menu(MenuState::new(true));
        self.settings_open = true;
    }

    fn capture_help(&mut self) {
        self.state = crate::state::GameState::Menu(MenuState::new(true));
        self.help_open = true;
    }

    fn capture_heritage(&mut self) {
        // Seed a storied Chronicle so the menu heritage line shows.
        for i in 0..6 {
            self.chronicle.record(crate::chronicle::ChronicleEntry {
                completed_year: 60,
                contract_name: "Founding Charter: Meridian Reach".to_owned(),
                objective: "Colonization".to_owned(),
                legacy_id: "preservers".to_owned(),
                leader_name: "Boro Chartwright".to_owned(),
                generation: i + 1,
                score: 0.95,
                outcome: "Complete".to_owned(),
                duration_years: 60,
                command_posture: crate::state::sim::CommandPosture::Civic,
                charter_approach: None,
                homecoming_recovery: None,
            });
        }
        let mut menu = MenuState::new(true);
        menu.phase = crate::state::MenuPhase::NewGame;
        self.state = crate::state::GameState::Menu(menu);
    }

    fn capture_boot(&mut self) {
        // Freeze the boot log mid-stream for a screenshot.
        self.boot.seek(1.4);
        self.state = crate::state::GameState::Menu(MenuState::new(false));
    }

    fn capture_log(&mut self) {
        // Dashboard with the newest log line frozen mid-stream
        // (cursor-visible phase).
        self.capture_log_reveal = Some(0.5);
        let mut sim = SimState::new_campaign(
            &self.data,
            "preservers",
            0xC0FFEE,
            &crate::state::sim::founding_faction_ids(&self.data),
        );
        if let Some(template) = self.data.contracts.get("founding_colony") {
            sim.contract = Some(contract::start_contract(template, &sim));
        }
        self.state = crate::state::GameState::Gameplay(Box::new(GameplayState::new(sim)));
    }

    fn capture_obligation_watch(&mut self) {
        // A decade-out duty stays visible in the instrument strip while
        // its one-shot watch entry arrives in the ordinary ship's log.
        let mut sim = SimState::new_campaign(
            &self.data,
            "preservers",
            0xC0FFEE,
            &crate::state::sim::founding_faction_ids(&self.data),
        );
        if let Some(event) = self.data.events.get("seed_vault_covenant_offer") {
            crate::simulation::event_resolver::apply_outcome(&mut sim, &self.data, event, 0);
        }
        sim.month_clock = 26 * 12;
        sim.record_obligation_watch();
        if let Some(template) = self.data.contracts.get("founding_colony") {
            sim.contract = Some(contract::start_contract(template, &sim));
        }
        self.state = crate::state::GameState::Gameplay(Box::new(GameplayState::new(sim)));
    }

    fn capture_event(&mut self, scene: &str) {
        let mut sim = SimState::new_campaign(
            &self.data,
            "preservers",
            0xC0FFEE,
            &crate::state::sim::founding_faction_ids(&self.data),
        );
        if scene == "event_fuel_paused" {
            sim.speed = crate::state::sim::GameSpeed::Paused;
        }
        let template_id = if scene.starts_with("event_fuel") {
            sim.ship.fuel = 0.5;
            "relativity_pocket"
        } else if scene == "event_custodian" {
            sim.dynasty.generation = 3;
            "the_emotional_module"
        } else if scene == "event_mascot_succession" {
            "the_mascot_succession"
        } else if scene == "event_succession" {
            sim.dynasty.generation = 3;
            "the_only_captain_they_know"
        } else {
            sim.subsystems.get_mut("engineering_bay").unwrap().knowledge = 0.2;
            "the_last_engineer"
        };
        sim.pending_event = Some(crate::state::sim::PendingEvent {
            template_id: template_id.to_owned(),
            rolled_month_clock: 0,
        });
        self.state = crate::state::GameState::Gameplay(Box::new(GameplayState::new(sim)));
    }

    fn capture_event_obligation(&mut self) {
        let mut sim = SimState::new_campaign(
            &self.data,
            "preservers",
            0xC0FFEE,
            &crate::state::sim::founding_faction_ids(&self.data),
        );
        sim.pending_event = Some(crate::state::sim::PendingEvent {
            template_id: "seed_vault_covenant_offer".to_owned(),
            rolled_month_clock: 0,
        });
        self.state = crate::state::GameState::Gameplay(Box::new(GameplayState::new(sim)));
    }

    fn capture_event_obligation_due(&mut self, scene: &str) {
        let mut sim = SimState::new_campaign(
            &self.data,
            "preservers",
            0xC0FFEE,
            &crate::state::sim::founding_faction_ids(&self.data),
        );
        if let Some(seed) = self.data.events.get("seed_vault_covenant_offer") {
            crate::simulation::event_resolver::apply_outcome(&mut sim, &self.data, seed, 0);
        }
        sim.month_clock = 36 * 12;
        if scene == "event_obligation_unaffordable" {
            sim.resources.food = 100;
            sim.resources.minerals = 0;
        }
        sim.pending_event = Some(crate::state::sim::PendingEvent {
            template_id: "seed_vault_covenant_due".to_owned(),
            rolled_month_clock: sim.month_clock,
        });
        self.state = crate::state::GameState::Gameplay(Box::new(GameplayState::new(sim)));
    }

    fn capture_crew(&mut self, scene: &str) {
        let mut sim = SimState::new_campaign(
            &self.data,
            "preservers",
            0xC0FFEE,
            &crate::state::sim::founding_faction_ids(&self.data),
        );
        sim.resources.credits = if scene == "crew" { 100 } else { 10_000 };
        if scene == "crew_recruitment" {
            sim.factions.pop();
        }
        for (id, approval) in [
            ("ascension_circle", 0.22),
            ("first_flame", 0.52),
            ("hearth_union", 0.82),
        ] {
            if let Some(faction) = sim.factions.iter_mut().find(|f| f.faction_id == id) {
                faction.approval = approval;
            }
        }
        let _ =
            crate::simulation::institutions::designate_apprentice(&mut sim, &self.data, "engineer");
        let mut gameplay = GameplayState::new(sim);
        gameplay.screen = Screen::CrewDynasty;
        self.state = crate::state::GameState::Gameplay(Box::new(gameplay));
    }

    fn capture_ship(&mut self, scene: &str) {
        {
            self.seed_drydock_ship(scene)
        }
        // The SHIP tab under way (real-time loop §5): the procedural blueprint.
        // Three hull classes share one demo sim so the schematic's adaptation
        // to different outlines can be verified side by side.;
    }

    fn capture_ship_underway(&mut self) {
        self.state = self.underway_blueprint_state(None);
    }

    fn capture_ship_underway_corvette(&mut self) {
        self.state = self.underway_blueprint_state(Some("light_corvette"));
    }

    fn capture_ship_underway_ark(&mut self) {
        self.state = self.underway_blueprint_state(Some("generation_ark"));
    }

    fn capture_ship_underway_prow(&mut self) {
        self.state = self.underway_blueprint_state(Some("armored_prow"));
    }

    fn capture_ship_underway_ring(&mut self) {
        self.state = self.underway_blueprint_state(Some("habitat_ring"));
    }

    fn capture_subsystems(&mut self) {
        // The underway subsystems screen (W5) with mixed tiers, worn
        // condition, and knowledge dipping below a repair threshold.
        let mut sim = SimState::new_campaign(
            &self.data,
            "preservers",
            0xC0FFEE,
            &crate::state::sim::founding_faction_ids(&self.data),
        );
        if let Some(s) = sim.subsystems.get_mut("medical_bay") {
            s.tier = 2;
            s.condition = 0.44;
            s.knowledge = 0.22;
        }
        if let Some(s) = sim.subsystems.get_mut("engineering_bay") {
            s.tier = 1;
            s.condition = 0.71;
        }
        if let Some(s) = sim.subsystems.get_mut("agriculture") {
            s.tier = 3;
        }
        sim.resources.credits = 20_000;
        sim.resources.influence = 100;
        let _ = crate::simulation::institutions::establish_or_support_school(
            &mut sim,
            &self.data,
            "agriculture",
        );
        let _ =
            crate::simulation::institutions::compile_archive(&mut sim, &self.data, "agriculture");
        let _ = crate::simulation::institutions::grant_custodianship(
            &mut sim,
            &self.data,
            "agriculture",
            "hearth_union",
        );
        let _ = crate::simulation::institutions::establish_or_support_school(
            &mut sim,
            &self.data,
            "medical_bay",
        );
        sim.resources.credits = 100;
        sim.resources.influence = 5;
        if let Some(template) = self.data.contracts.get("deep_vein_survey") {
            sim.contract = Some(contract::start_contract(template, &sim));
        }
        let mut gameplay = GameplayState::new(sim);
        gameplay.screen = Screen::Subsystems;
        self.state = crate::state::GameState::Gameplay(Box::new(gameplay));
    }

    fn capture_custody(&mut self) {
        let mut sim = SimState::new_campaign(
            &self.data,
            "preservers",
            0xC0FFEE,
            &crate::state::sim::founding_faction_ids(&self.data),
        );
        sim.resources.credits = 20_000;
        sim.resources.influence = 100;
        let _ = crate::simulation::institutions::establish_or_support_school(
            &mut sim,
            &self.data,
            "medical_bay",
        );
        let _ =
            crate::simulation::institutions::compile_archive(&mut sim, &self.data, "medical_bay");
        let mut gameplay = GameplayState::new(sim);
        gameplay.screen = Screen::Subsystems;
        self.state = crate::state::GameState::Gameplay(Box::new(gameplay));
        self.custody_picker = Some("medical_bay".to_owned());
    }

    fn capture_market(&mut self) {
        let mut sim = SimState::new_campaign(
            &self.data,
            "wanderers",
            0xC0FFEE,
            &crate::state::sim::founding_faction_ids(&self.data),
        );
        // Exercise every quoted-term modifier in one deterministic frame:
        // a well-regarded hull gets favorable name terms, critically low
        // food/energy draws a need premium, and bare coffers draw a
        // distress-sale discount.
        sim.reputation.insert("mercy".to_owned(), 0.8);
        sim.resources.credits = self.data.config.distress_credit_floor - 500;
        sim.resources.food = (self.data.config.low_food_threshold - 100).max(0);
        sim.resources.energy = (self.data.config.low_energy_threshold - 100).max(0);
        let _ = crate::simulation::market::sell(
            &mut sim,
            crate::state::sim::TradeResource::Minerals,
            50,
        );
        let mut gameplay = GameplayState::new(sim);
        gameplay.screen = Screen::Market;
        self.state = crate::state::GameState::Gameplay(Box::new(gameplay));
    }

    fn capture_abort(&mut self) {
        let mut sim = SimState::new_campaign(
            &self.data,
            "preservers",
            5,
            &crate::state::sim::founding_faction_ids(&self.data),
        );
        sim.contract = Some(contract::start_contract(
            self.data.contracts.get("deep_vein_survey").unwrap(),
            &sim,
        ));
        self.abort_confirm.set(true);
        let mut g = GameplayState::new(sim);
        g.screen = Screen::Contract;
        self.state = crate::state::GameState::Gameplay(Box::new(g));
    }

    fn capture_contracts(&mut self) {
        // No active contract, so the available-charters list is shown.
        let sim = SimState::new_campaign(
            &self.data,
            "wanderers",
            0xC0FFEE,
            &crate::state::sim::founding_faction_ids(&self.data),
        );
        let mut gameplay = GameplayState::new(sim);
        gameplay.screen = Screen::Drydock;
        self.state = crate::state::GameState::Gameplay(Box::new(gameplay));
    }

    fn capture_prep(&mut self, scene: &str) {
        // A charter under consideration in port (W4): the PREP screen,
        // with deliberately mixed provisioning so shortfalls show red.
        let mut sim = SimState::new_campaign(
            &self.data,
            "wanderers",
            0xC0FFEE,
            &crate::state::sim::founding_faction_ids(&self.data),
        );
        if let Some(event) = self.data.events.get("sanctuary_berths_asked").cloned() {
            crate::simulation::event_resolver::apply_outcome(&mut sim, &self.data, &event, 0);
        }
        if scene.starts_with("tutorial") {
            self.tutorial_open = true;
            self.display.tutorial_enabled = true;
            sim.tutorial_step = if scene == "tutorial_complete" {
                self.data.config.tutorial.guided_steps.len()
            } else if scene == "tutorial_navigation" {
                0
            } else {
                2
            };
        }
        sim.selected_charter = Some("the_hard_contract".to_owned());
        sim.ship.fuel = 0.6;
        sim.resources.food = 800;
        sim.ship.spare_parts = 45;
        let mut gameplay = GameplayState::new(sim);
        gameplay.screen = Screen::Drydock;
        self.state = crate::state::GameState::Gameplay(Box::new(gameplay));
    }

    fn capture_drydock(&mut self) {
        // Home from a mission (M4.6): no active contract, a worn ship,
        // and a concluded charter in the Chronicle → the Homecoming banner.
        let mut sim = SimState::new_campaign(
            &self.data,
            "wanderers",
            0xC0FFEE,
            &crate::state::sim::founding_faction_ids(&self.data),
        );
        sim.ship.hull_integrity = 0.46;
        sim.ship.life_support = 0.58;
        sim.ship.spare_parts = 3;
        self.chronicle.record(crate::chronicle::ChronicleEntry {
            completed_year: 41,
            contract_name: "Deep Vein Survey: Karst Belt".to_owned(),
            objective: "Mining".to_owned(),
            legacy_id: "wanderers".to_owned(),
            leader_name: "Sella Voss".to_owned(),
            generation: 2,
            score: 0.82,
            outcome: "Partial".to_owned(),
            duration_years: 40,
            command_posture: crate::state::sim::CommandPosture::Steady,
            charter_approach: None,
            homecoming_recovery: None,
        });
        self.capture_run_secs = Some(2280.0); // 38m — the run just flown
        let mut gameplay = GameplayState::new(sim);
        gameplay.screen = Screen::Drydock;
        self.state = crate::state::GameState::Gameplay(Box::new(gameplay));
    }

    fn capture_contract_active(&mut self) {
        // A charter a dozen years in, to show progress + drive assist.
        let mut sim = SimState::new_campaign(
            &self.data,
            "adaptors",
            0xC0FFEE,
            &crate::state::sim::founding_faction_ids(&self.data),
        );
        if let Some(template) = self.data.contracts.get("deep_vein_survey") {
            sim.contract = Some(contract::start_contract(template, &sim));
        }
        sim.resources.food = 1_000_000;
        for _ in 0..12 {
            sim.pending_event = None;
            sim.pending_dilemma = None;
            tick::advance_year(&mut sim, &self.data);
        }
        sim.pending_event = None;
        sim.pending_dilemma = None;
        sim.command_posture = crate::state::sim::CommandPosture::Expeditionary;
        sim.command_posture_locked_until = sim.month_clock.saturating_add(12);
        sim.ship.fuel = 0.0;
        self.capture_run_secs = Some(1140.0); // 19m into the run (live timer)
        let mut gameplay = GameplayState::new(sim);
        gameplay.screen = Screen::Contract;
        self.state = crate::state::GameState::Gameplay(Box::new(gameplay));
    }

    fn capture_authority_review(&mut self) {
        // A civic captain objects to expeditionary tempo while the crew
        // is strained. This is the reproducible disagreement capture:
        // the modal shows the enforced compromise and emergency gate.
        let mut sim = SimState::new_campaign(
            &self.data,
            "preservers",
            0xC0FFEE,
            &crate::state::sim::founding_faction_ids(&self.data),
        );
        if let Some(template) = self.data.contracts.get("deep_vein_survey") {
            sim.contract = Some(contract::start_contract(template, &sim));
        }
        sim.population.morale = 0.52;
        sim.population.unity = 0.56;
        sim.authority.captain_priority = crate::state::sim::CaptainPriority::Civic;
        sim.authority.captain_name = "Captain Ilyan Vale".to_owned();
        sim.authority.pending_review = crate::state::sim::authority::posture_review(
            &sim,
            crate::state::sim::CommandPosture::Expeditionary,
        );
        let mut gameplay = GameplayState::new(sim);
        gameplay.screen = Screen::Contract;
        self.state = crate::state::GameState::Gameplay(Box::new(gameplay));
    }

    fn capture_dilemma(&mut self) {
        let mut sim = SimState::new_campaign(
            &self.data,
            "preservers",
            0xC0FFEE,
            &crate::state::sim::founding_faction_ids(&self.data),
        );
        sim.pending_dilemma = Some(crate::state::sim::PendingDilemma {
            dilemma_id: "archive_purge".to_owned(),
            rolled_month_clock: 0,
        });
        self.state = crate::state::GameState::Gameplay(Box::new(GameplayState::new(sim)));
    }

    fn capture_dilemma_combat(&mut self) {
        // Wanderer convoy raid with a weapon installed — combat lifts
        // the shown odds.
        let mut sim = SimState::new_campaign(
            &self.data,
            "wanderers",
            0xC0FFEE,
            &crate::state::sim::founding_faction_ids(&self.data),
        );
        sim.ship.weapon = Some("mass_driver".to_owned());
        sim.pending_dilemma = Some(crate::state::sim::PendingDilemma {
            dilemma_id: "convoy_raid".to_owned(),
            rolled_month_clock: 0,
        });
        self.state = crate::state::GameState::Gameplay(Box::new(GameplayState::new(sim)));
    }

    fn capture_chronicle(&mut self, scene: &str) {
        // Seed a storied Chronicle and unlock the matching milestones.
        self.achievements = Achievements::from_definitions(crate::achievements::definitions());
        let mut sim = SimState::new_campaign(
            &self.data,
            "preservers",
            0xC0FFEE,
            &crate::state::sim::founding_faction_ids(&self.data),
        );
        sim.dynasty.generation = 5;
        sim.month_clock = 120 * 12;
        for event_id in [
            "sanctuary_berths_asked",
            "station_foundation_request",
            "aboard_compact_offer",
        ] {
            if let Some(event) = self.data.events.get(event_id).cloned() {
                crate::simulation::event_resolver::apply_outcome(&mut sim, &self.data, &event, 0);
            }
        }
        // The ledger must prove that active duties beyond the visible
        // rows remain reachable, including a mixture of timed and open
        // promises. Capture-only copies avoid altering authored data.
        let obligation_samples = sim.obligations.clone();
        for (index, original) in obligation_samples.iter().cycle().take(3).enumerate() {
            let mut sample = original.clone();
            sample.id = format!("capture-obligation-{index}");
            sample.authored_id = format!("capture-duty-{index}");
            sample.title = [
                "The Cartographers' Passage",
                "Rain for the Glass Gardens",
                "Witnesses for the Far Compact",
            ][index]
                .to_owned();
            sample.due_year = (index != 1).then_some(116 + index as u32 * 18);
            sim.obligations.push(sample);
        }
        let heir = sim
            .dynasty
            .members
            .iter()
            .find(|member| !member.is_leader)
            .map(|member| member.name.clone());
        if let Some(heir) = heir {
            sim.dynasty.end_reign(sim.year());
            for member in &mut sim.dynasty.members {
                member.is_leader = member.name == heir;
            }
            sim.dynasty.begin_reign(sim.year());
            sim.inherit_obligations();
        }
        if scene == "obligation_archive" {
            sim.apply_obligation_operation(&crate::state::sim::ObligationOperation::Fulfil {
                authored_id: "sanctuary_berths".to_owned(),
                note: "Every promised family crossed the ramp.".to_owned(),
            });
            sim.apply_obligation_operation(&crate::state::sim::ObligationOperation::Default {
                authored_id: "station_aid".to_owned(),
                note: "The promised return survey was abandoned.".to_owned(),
            });
            self.obligation_resolved_tab.set(true);
        }
        // More entries than the panel can hold, so the capture shows the
        // state the log's scroll exists for rather than a short list that
        // never reaches it.
        let archive_entries = if scene == "mission_archive" { 6 } else { 14 };
        for i in 0..archive_entries {
            self.chronicle.record(crate::chronicle::ChronicleEntry {
                completed_year: 40 + i * 20,
                contract_name: "Deep Vein Survey: Karst Belt".to_owned(),
                objective: "Mining".to_owned(),
                legacy_id: "preservers".to_owned(),
                leader_name: "Boro Chartwright".to_owned(),
                generation: i + 1,
                score: 0.92,
                outcome: if i % 2 == 0 { "Complete" } else { "Partial" }.to_owned(),
                duration_years: 40,
                command_posture: crate::state::sim::CommandPosture::Steady,
                charter_approach: None,
                homecoming_recovery: None,
            });
        }
        for id in crate::achievements::evaluate(&sim, &self.chronicle) {
            self.achievements.unlock(id);
        }
        let mut gameplay = GameplayState::new(sim);
        gameplay.screen = Screen::Chronicle;
        if scene == "mission_archive" {
            self.chronicle_records_tab.set(false);
        }
        if scene == "obligation_history" {
            self.obligation_detail = gameplay
                .sim
                .obligations
                .first()
                .map(|obligation| obligation.id.clone());
        }
        self.state = crate::state::GameState::Gameplay(Box::new(gameplay));
    }

    fn capture_gameover(&mut self) {
        let mut sim = SimState::new_campaign(
            &self.data,
            "preservers",
            0xC0FFEE,
            &crate::state::sim::founding_faction_ids(&self.data),
        );
        sim.month_clock = 148 * 12;
        sim.dynasty.generation = 6;
        sim.legacy.tradition_points = 210;
        sim.dynasty.extinct = true;
        self.state = crate::state::GameState::Gameplay(Box::new(GameplayState::new(sim)));
    }

    fn capture_debrief(&mut self) {
        self.fly_to_homecoming("founding_colony");
    }

    fn capture_dashboard_risk(&mut self) {
        let mut sim = SimState::new_campaign(
            &self.data,
            "preservers",
            0xC0FFEE,
            &crate::state::sim::founding_faction_ids(&self.data),
        );
        sim.resources.energy = self.data.config.low_energy_threshold / 5;
        if let Some(template) = self.data.contracts.get("founding_colony") {
            sim.contract = Some(contract::start_contract(template, &sim));
        }
        self.state = crate::state::GameState::Gameplay(Box::new(GameplayState::new(sim)));
    }

    fn capture_dashboard_repair(&mut self, scene: &str) {
        {
            let mut sim = SimState::new_campaign(
                &self.data,
                "preservers",
                0xC0FFEE,
                &crate::state::sim::founding_faction_ids(&self.data),
            );
            sim.ship.hull_integrity = 0.38;
            sim.ship.life_support = 0.61;
            if let Some(template) = self.data.contracts.get("founding_colony") {
                sim.contract = Some(contract::start_contract(template, &sim));
            }
            let mut gameplay = GameplayState::new(sim);
            if scene == "ship_repair" {
                gameplay.screen = Screen::ShipBuilder;
            }
            self.state = crate::state::GameState::Gameplay(Box::new(gameplay));
        }
        // "gameplay" and anything else: a fresh campaign on the dashboard.;
    }

    fn capture_default(&mut self) {
        let sim = SimState::new_campaign(
            &self.data,
            "preservers",
            0xC0FFEE,
            &crate::state::sim::founding_faction_ids(&self.data),
        );
        self.state = crate::state::GameState::Gameplay(Box::new(GameplayState::new(sim)));
    }
}
