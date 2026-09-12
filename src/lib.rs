//! Public library surface for Stellar Legacy's deterministic game logic.
//!
//! The binary is intentionally only a Macroquad runtime shell. Keeping the
//! game modules here lets unit tests exercise the same state machine and UI
//! helpers that the published executable uses, while the small public surface
//! remains useful to capture and integration harnesses.

use macroquad::prelude::*;
use macroquad_toolkit::capture;

mod achievements;
mod audio;
mod boot;
mod chronicle;
mod data;
mod game;
mod heritage;
mod save;
mod settings;
mod simulation;
mod state;
mod support;
mod ui;

pub use data::GameData;
pub use game::Game;

use macroquad_toolkit::ui::{
    begin_target_audit, begin_target_frame, end_target_audit, neighbours_warm, overlapping_targets,
    smallest_touchable_width, take_audit, undersized_targets, Finding,
};

/// Print what the last captured scene's controls would demand of a touchscreen.
///
/// Three separate faults, in the order they matter. Two controls whose grown
/// hit areas overlap are ambiguous, and a press landing on the wrong control is
/// worse than one landing on nothing. A control too small to *press* is one the
/// standard says a finger cannot reliably reach. A control too small to *see* is
/// one nobody presses on purpose — reachable, but not findable.
fn report_touch_targets(scene: &str) {
    if !neighbours_warm() {
        return;
    }
    let clashes = overlapping_targets();
    assert!(
        clashes.is_empty(),
        "capture[{scene}] has ambiguous touch targets: {clashes:?}"
    );
    for (a, b, area) in &clashes {
        println!("touch[{scene}] AMBIGUOUS: \"{a}\" and \"{b}\" share {area:.0}px²");
    }
    if let Some((width, label)) = smallest_touchable_width(if ui::mobile::active() {
        ui::mobile::size().0
    } else {
        ui::logical_width()
    }) {
        let viewport = if ui::mobile::active() {
            ui::mobile::size().0
        } else {
            ui::logical_width()
        };
        assert!(
            width <= viewport + 0.5,
            "capture[{scene}] target {label:?} needs {width:.0}px, viewport is {viewport:.0}px"
        );
        println!(
            "touch[{scene}] every control clears 44px at {width:.0}px wide (worst: \"{label}\")"
        );
    }
    let under = undersized_targets();
    for (drawn, label) in under.iter().take(8) {
        println!("touch[{scene}] drawn {drawn:.0}px: \"{label}\"");
    }
    if under.len() > 8 {
        println!(
            "touch[{scene}] ...and {} more drawn under 44px",
            under.len() - 8
        );
    }
}

/// Capture scenes are release evidence, so accessibility findings stop the
/// capture rather than becoming another stale text report.
fn assert_capture_accessibility(scene: &str) {
    let text_findings = take_audit();
    if text_findings.is_empty() {
        return;
    }
    let details = text_findings
        .iter()
        .map(|finding| match finding {
            Finding::Overflow { .. } | Finding::LowContrast { .. } | Finding::Collision { .. } => {
                format!("{}: {}", finding.text(), finding.describe())
            }
        })
        .collect::<Vec<_>>();
    panic!(
        "capture[{scene}] text/modal audit failed: {}",
        details.join("; ")
    );
}

/// Window configuration shared by the desktop and capture runtime.
pub fn window_conf() -> Conf {
    capture::capture_window_conf(
        "STELLAR_LEGACY",
        "Stellar Legacy",
        ui::LOGICAL_WIDTH as i32,
        ui::LOGICAL_HEIGHT as i32,
    )
}

/// Run the game loop and the optional screenshot/capture harness.
pub async fn run() {
    macroquad_toolkit::crash::install_crash_log("stellar_legacy");
    let mut game = Game::new().await;

    if let Some(configs) = capture::CaptureConfig::all_from_env("STELLAR_LEGACY") {
        for config in configs {
            game.begin_capture_scene(&config.scene);
            begin_target_audit();
            macroquad_toolkit::ui::begin_audit();
            macroquad_toolkit::ui::begin_collision_audit();
            let mut frame_no = 0;
            capture::run_capture_once(&config, |dt| {
                frame_no += 1;
                begin_target_frame();
                // Text extents belong to one rendered frame. Clearing the
                // audit here prevents the same button from colliding with its
                // own label on the next capture frame.
                macroquad_toolkit::ui::begin_audit();
                macroquad_toolkit::ui::begin_collision_audit();
                game.update(dt);
                game.draw();
                if frame_no >= config.frames {
                    end_target_audit();
                    report_touch_targets(&config.scene);
                    assert_capture_accessibility(&config.scene);
                }
            })
            .await;
        }
        return;
    }

    loop {
        let dt = get_frame_time().min(0.1);
        game.update(dt);
        game.draw();
        next_frame().await;
    }
}
