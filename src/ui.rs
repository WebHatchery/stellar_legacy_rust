//! Terminal-styled UI shell: palette, shared widgets, action enum, and the
//! per-frame dispatch into screen modules.
//!
//! UI is a pure view layer: every function reads state and returns
//! `UiAction` intents; nothing here mutates the sim (CODE_STANDARDS §7).

pub mod agenda;
pub mod authority_modal;
pub mod bridge;
pub mod chronicle;
pub mod contract_systems;
pub mod crew_dynasty;
pub mod dashboard;
pub mod debrief;
pub mod event_modal;
pub mod game_over;
pub mod help;
pub mod main_menu;
pub mod market;
pub mod mission;
pub mod prep;
pub mod presentation;
pub mod recovery_warning;
pub mod settings;
pub mod shell;
pub mod ship_builder;
pub mod ship_schematic;
pub mod subsystems;
pub mod time_controls;
pub mod tutorial;
pub mod welcome;
pub mod widgets;

pub use main_menu::*;
pub use shell::*;
pub use widgets::*;

use crate::chronicle::ChronicleStore;
use crate::data::events::EventCategory;
use crate::data::ship_components::ComponentKind;
use crate::data::GameData;
use crate::state::sim::{CommandPosture, GameSpeed, SimState, TradeResource};
use crate::state::{MenuState, Screen};
use macroquad::prelude::*;
use macroquad_toolkit::achievements::Achievements;
use macroquad_toolkit::prelude::*;
use macroquad_toolkit::ui::{
    draw_ui_text_ex, note_neighbour, note_target, touch_area, Pointer, RectExt,
};

pub const LOGICAL_WIDTH: f32 = 1280.0;
pub const LOGICAL_HEIGHT: f32 = 720.0;

pub mod term;

/// Every interaction the UI can request. Game logic applies these in
/// `game.rs`; adding an interaction means adding a variant here, never
/// mutating state from a panel.
#[derive(Debug, Clone, PartialEq)]
pub enum UiAction {
    // Menu
    SelectLegacy(usize),
    /// Toggle a founding faction in the new-game picker (W7).
    ToggleFaction(String),
    StartNewGame,
    ContinueGame,
    DeleteSave,
    /// Step from the main menu into the new-game (legacy/faction) picker.
    GoToNewGame,
    /// Step back from the new-game picker to the main menu.
    BackToMainMenu,
    /// Open the display/settings overlay — from the main menu, or from the
    /// gameplay chrome row where it is the only way in without an F1 key.
    OpenSettings,
    /// Open the help/controls overlay from the gameplay chrome row (F2's twin).
    OpenHelp,
    /// Quit the application from the main menu.
    ExitGame,
    // Global
    SaveGame,
    ToMenu,
    RetireVoyage,
    /// Dismiss the homecoming debrief: clear the sealed report and hand the
    /// ship back to the drydock board for its next charter.
    FileReport,
    SelectScreen(Screen),
    QueueProject {
        project_id: String,
        target_id: Option<String>,
    },
    PauseProject(u64),
    ResumeProject(u64),
    MoveProject {
        sequence_id: u64,
        direction: i32,
    },
    PreviewCancelProject(u64),
    CancelProject(u64),
    DismissCancelProject,
    ReviewRecovery,
    EmergencyStabilise,
    ResumeAfterWarning,
    // Gameplay verbs (GDD §4)
    /// Set the real-time auto-advance rate / pause (real-time loop §1).
    SetSpeed(GameSpeed),
    TogglePause,
    /// Set the voyage-wide operating philosophy shown on the CONTRACT screen.
    SetPosture(CommandPosture),
    /// Turn the current mission for home early (W2). Only emitted underway.
    AbortMission,
    ResolveEvent(usize),
    ResolveDilemma(usize),
    RecruitCrew(String),
    TrainCrew(String),
    DesignateApprentice(String),
    SelectHeir(u32),
    /// Put a charter under consideration in port — never starts it (W4).
    SelectCharter(String),
    CancelSelection,
    /// Commit the selected charter and begin the voyage (W4) — the sole path
    /// that starts a contract.
    Launch,
    /// Refuel to a full tank in drydock (W4).
    Refuel,
    /// Stock spare parts in drydock (W4 provisioning, PREP screen).
    BuyParts(i64),
    /// Advance the guided first-voyage tutorial.
    NextTutorial,
    ReviewProvisions,
    /// Permanently skip the guided tutorial for this campaign.
    SkipTutorial,
    PurchaseComponent(ComponentKind, String),
    FieldRepair(crate::simulation::ship::RepairKind),
    FullRepair,
    InstallSalvage(String),
    CommissionShip(String),
    /// Recruit a fresh people in drydock when short of the founding count (W7).
    RecruitFactionGroup(String),
    /// Subsystem verbs (W5): mend, upgrade (port), or train its knowledge.
    RepairSubsystem(String),
    UpgradeSubsystem(String),
    /// Fit a mission-reward subsystem version the ship has unlocked (2c) — free,
    /// drydock-only, distinct from the bought `UpgradeSubsystem`.
    InstallFitting(String),
    TrainSubsystemKnowledge(String),
    EstablishSchool(String),
    CompileProcedureArchive(String),
    /// Open the touch-driven people picker for this discipline.
    BeginDisciplineCustody(String),
    CancelDisciplineCustody,
    GrantDisciplineCustody {
        subsystem_id: String,
        faction_id: String,
    },
    OpenObligationHistory(String),
    CloseObligationHistory,
    Buy(TradeResource, i64),
    Sell(TradeResource, i64),
    ToggleDelegation(EventCategory),
    /// Resolve the captain's blocking review of a strategic posture.
    ResolveAuthority(crate::state::sim::AuthorityChoice),
}
