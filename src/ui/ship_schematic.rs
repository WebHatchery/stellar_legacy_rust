//! Procedural ship schematic (mission SHIP tab): a blueprint of the vessel
//! built from its actual loadout and subsystems, with each module highlighted by
//! condition, tier, and crew manning.
//!
//! Two halves, split so the layout is testable without a live frame:
//! [`build`] is a pure, deterministic function that reads `&SimState` and returns
//! a [`ShipSchematic`] of positioned glyphs (no macroquad calls, no RNG); [`draw`]
//! and [`draw_status_strip`] render that spec. Because `build` reads the sim fresh
//! each frame, the picture reacts on its own to anything that changes mid-mission —
//! a subsystem repaired or upgraded, a salvaged part field-installed, or a crew
//! member added by an event — with no wiring beyond the per-frame call.

mod draw;

pub use draw::*;

use crate::data::ship_components::{ComponentKind, ComponentStats};
use crate::data::GameData;
use crate::simulation::ship::loadout_stats;
use crate::state::sim::SimState;
use crate::ui::{term, term_bar};
use macroquad::prelude::*;
use macroquad_toolkit::prelude::*;
use macroquad_toolkit::ui::draw_ui_text_ex;

/// Which class of ship part a glyph stands for. Bridge/Engine/Weapon are the
/// installed components; Subsystem is one of the six module families (W5).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModuleKind {
    Bridge,
    Subsystem,
    Engine,
    Weapon,
}

/// One highlighted module on the schematic: where it sits, what it is, and the
/// three signals the highlight encodes — condition, tier, and manning.
#[derive(Debug, Clone, PartialEq)]
pub struct ModuleGlyph {
    /// Subsystem id, or `"bridge"` / component id for the loadout glyphs.
    pub id: String,
    pub label: String,
    /// Standardized 3-letter tag drawn inside the box — the scannable identity,
    /// independent of the long external caption.
    pub code: String,
    pub rect: Rect,
    pub kind: ModuleKind,
    /// 0-1 physical condition (subsystem condition; component integrity/fuel).
    pub condition: f32,
    /// 0..=3 upgrade tier (subsystems only; components stay 0).
    pub tier: u32,
    /// A matching crew post is aboard.
    pub manned: bool,
    /// Schematic deck range for the caption, e.g. `DECKS 3-4`. Deterministic,
    /// derived from the module's placed row and longitudinal section, and never
    /// read by the sim.
    pub deck_lo: u8,
    pub deck_hi: u8,
}

/// A fully-placed schematic: the hull silhouette, an optional habitat ring, the
/// module glyphs, and the live ship-status numbers for the bottom strip.
#[derive(Debug, Clone, PartialEq)]
pub struct ShipSchematic {
    pub hull_id: String,
    pub hull_name: String,
    /// Closed hull outline in absolute logical coordinates.
    pub outline: Vec<Vec2>,
    /// The primary corridor (bow end → stern end) that every compartment branches
    /// off — the schematic's structural spine.
    pub corridor: (Vec2, Vec2),
    /// Central spun-gravity ring (centre, radius) for hulls that carry one.
    pub ring: Option<(Vec2, f32)>,
    pub modules: Vec<ModuleGlyph>,
    pub hull_integrity: f32,
    pub life_support: f32,
    pub fuel: f32,
    pub spare_parts: i64,
    /// Aggregate installed-loadout stats — drives the combat/modifiers tiles.
    pub stats: ComponentStats,
}

/// The crew posts that "man" a subsystem. A module with one of these aboard reads
/// as crewed; the rest stand dim. Kept here (not in data) because it is a UI
/// affordance, not balance.
fn subsystem_crew_posts(id: &str) -> &'static [&'static str] {
    match id {
        "agriculture" => &["agronomist"],
        "medical_bay" => &["medic"],
        "engineering_bay" => &["engineer"],
        // Life support is kept by the same hands that keep the reactor.
        "life_support_habitat" => &["engineer"],
        "security" => &["security_chief"],
        "education_culture" => &["scientist"],
        _ => &[],
    }
}

/// The standardized 3-letter tag stamped inside a subsystem compartment. Kept
/// stable so a room reads the same across every hull it appears on.
fn subsystem_code(id: &str) -> &'static str {
    match id {
        "agriculture" => "AGR",
        "education_culture" => "EDU",
        "engineering_bay" => "ENG",
        "life_support_habitat" => "LSH",
        "medical_bay" => "MED",
        "security" => "SEC",
        _ => "SYS",
    }
}

/// A short single-phrase caption for the diagram — the full formal name lives on
/// the Subsystems tab. Short so adjacent captions never collide, even when a lean
/// hull packs the rooms close together.
fn subsystem_short(id: &str) -> &'static str {
    match id {
        "agriculture" => "AGRICULTURE",
        "education_culture" => "ARCHIVES",
        "engineering_bay" => "ENGINEERING",
        "life_support_habitat" => "LIFE SUPPORT",
        "medical_bay" => "MEDICAL",
        "security" => "SECURITY",
        _ => "SYSTEM",
    }
}

/// The bridge is stood by the ship's command staff.
const BRIDGE_POSTS: &[&str] = &["commander", "navigator"];

/// Fixed distance from the corridor to each deck's room centres. Rooms are placed
/// relative to this, not to the hull size, so the layout survives any hull class.
const ROW_OFFSET: f32 = 48.0;
/// The vertical slot each room occupies (tallest tier-3 box plus headroom).
const ROOM_SLOT_H: f32 = 46.0;

fn any_post_aboard(sim: &SimState, posts: &[&str]) -> bool {
    sim.crew
        .iter()
        .any(|c| posts.contains(&c.archetype_id.as_str()))
}

/// The schematic has six longitudinal deck sections on each side of its
/// corridor. Number a compartment from the row and column where it is actually
/// drawn, rather than assigning a flavour number from its id.
fn deck_range_for_slot(upper: bool, col: usize, cols: usize) -> (u8, u8) {
    let cols = cols.max(1);
    let section = (col.saturating_mul(6) / cols).min(5) as u8;
    let row_start = if upper { 1 } else { 7 };
    let lo = row_start + section;
    (lo, lo + 1)
}

fn component_deck_range(kind: ModuleKind) -> (u8, u8) {
    match kind {
        ModuleKind::Bridge => (1, 2),
        ModuleKind::Weapon => (6, 7),
        ModuleKind::Engine => (11, 12),
        ModuleKind::Subsystem => (1, 2),
    }
}

#[derive(Debug, Clone, Copy)]
enum Silhouette {
    Standard,
    Barge,
    Ark,
}

/// Hull silhouette parameters, chosen by hull id so each ship reads distinct.
/// Only the *outline* is shaped here; the compartment grid is placed independently
/// (rooms hug the corridor at a fixed offset), so no hull can make a room overflow.
struct Profile {
    /// 0 blunt … 1 needle prow.
    nose: f32,
    /// Mid-body fullness — how full the shoulders are (broad freighter vs. lean
    /// corvette).
    bulge: f32,
    /// Stern width fraction.
    tail: f32,
    /// Fraction of the available width the hull spans — a corvette is short, an
    /// ark long.
    length: f32,
    /// Multiplier on the hull's half-height above the room-enclosing minimum, so
    /// bulky classes stand taller without ever pinching the compartments.
    height: f32,
    /// Carries a central spun-gravity ring.
    ring: bool,
    /// Selects the hull's readable construction language where the generic
    /// profile would make the barge and ark look alike.
    silhouette: Silhouette,
}

struct HullGeometry {
    outline: Vec<Vec2>,
    ring: Option<(Vec2, f32)>,
    cy: f32,
    row_offset: f32,
    max_h: f32,
    sx0: f32,
    sx1: f32,
}

impl HullGeometry {
    fn x_at(&self, fraction: f32) -> f32 {
        self.sx0 + fraction * (self.sx1 - self.sx0)
    }
}

fn hull_profile(hull_id: &str) -> Profile {
    match hull_id {
        "colony_barge" => Profile {
            nose: 0.2,
            bulge: 1.0,
            tail: 0.75,
            length: 0.94,
            height: 1.12,
            ring: false,
            silhouette: Silhouette::Barge,
        },
        "light_corvette" => Profile {
            nose: 0.95,
            bulge: 0.4,
            tail: 0.36,
            length: 0.72,
            height: 0.98,
            ring: false,
            silhouette: Silhouette::Standard,
        },
        "generation_ark" => Profile {
            nose: 0.3,
            bulge: 1.0,
            tail: 0.85,
            length: 1.0,
            height: 1.2,
            ring: true,
            silhouette: Silhouette::Ark,
        },
        "habitat_ring" => Profile {
            nose: 0.45,
            bulge: 0.72,
            tail: 0.6,
            length: 0.9,
            height: 1.16,
            ring: true,
            silhouette: Silhouette::Standard,
        },
        "armored_prow" => Profile {
            nose: 1.0,
            bulge: 0.6,
            tail: 0.5,
            length: 0.84,
            height: 1.0,
            ring: false,
            silhouette: Silhouette::Standard,
        },
        _ => Profile {
            nose: 0.5,
            bulge: 0.75,
            tail: 0.6,
            length: 0.88,
            height: 1.05,
            ring: false,
            silhouette: Silhouette::Standard,
        },
    }
}

/// Build the schematic for the current ship inside `frame` (the content area the
/// silhouette should fill). Pure and deterministic: given the same sim it returns
/// byte-identical geometry.
pub fn build(sim: &SimState, data: &GameData, frame: Rect) -> ShipSchematic {
    let profile = hull_profile(&sim.ship.hull);
    let hull_name = data
        .ship_components
        .find(ComponentKind::Hull, &sim.ship.hull)
        .map(|c| c.name.clone())
        .unwrap_or_else(|| sim.ship.hull.clone());
    let geometry = hull_geometry(&profile, frame);
    let mut modules = anchor_modules(sim, data, &geometry);
    modules.extend(subsystem_modules(sim, data, &geometry));

    ShipSchematic {
        hull_id: sim.ship.hull.clone(),
        hull_name,
        outline: geometry.outline.clone(),
        // Corridor runs between the bridge and engine boxes as its terminals,
        // rather than passing through them.
        corridor: (
            vec2(geometry.x_at(0.05) + 60.0, geometry.cy),
            vec2(geometry.x_at(0.95) - 64.0, geometry.cy),
        ),
        ring: geometry.ring,
        modules,
        hull_integrity: sim.ship.hull_integrity,
        life_support: sim.ship.life_support,
        fuel: sim.ship.fuel,
        spare_parts: sim.ship.spare_parts,
        stats: loadout_stats(sim, data),
    }
}

fn hull_geometry(profile: &Profile, frame: Rect) -> HullGeometry {
    let cy = frame.y + frame.h * 0.5;
    let cx = frame.x + frame.w * 0.5;
    let row_offset = ROW_OFFSET;
    let max_h = (row_offset + ROOM_SLOT_H * 0.5 + 20.0) * profile.height;
    let hull_span = frame.w * 0.86 * profile.length;
    let sx0 = cx - hull_span * 0.5;
    let sx1 = cx + hull_span * 0.5;
    let geometry = HullGeometry {
        outline: Vec::new(),
        ring: None,
        cy,
        row_offset,
        max_h,
        sx0,
        sx1,
    };
    let shoulder = 0.82 + profile.bulge * 0.13;
    let bow = 0.12 + (1.0 - profile.nose) * 0.26;
    let tail = 0.22 + profile.tail * 0.30;
    let stations: Vec<(f32, f32)> = match profile.silhouette {
        Silhouette::Barge => vec![
            (0.00, 0.50),
            (0.08, 0.74),
            (0.18, 0.92),
            (0.30, 0.96),
            (0.76, 0.96),
            (0.88, 0.90),
            (1.00, 0.90),
        ],
        Silhouette::Ark => vec![
            (0.00, 0.36),
            (0.10, 0.62),
            (0.22, 0.78),
            (0.32, 0.78),
            (0.38, 0.96),
            (0.62, 0.96),
            (0.68, 0.78),
            (0.78, 0.78),
            (0.90, 0.68),
            (1.00, 0.68),
        ],
        Silhouette::Standard => vec![
            (0.00, bow),
            (0.12, (bow + shoulder) * 0.5),
            (0.30, shoulder),
            (0.50, 1.0),
            (0.70, shoulder),
            (0.88, (tail + shoulder) * 0.5),
            (1.00, tail),
        ],
    };
    let mut outline: Vec<Vec2> = stations
        .iter()
        .map(|&(t, f)| vec2(geometry.x_at(t), cy - f * max_h))
        .collect();
    for &(t, f) in stations.iter().rev() {
        outline.push(vec2(geometry.x_at(t), cy + f * max_h));
    }
    HullGeometry {
        outline,
        ring: profile
            .ring
            .then(|| (vec2(cx, cy), row_offset + ROOM_SLOT_H * 0.5 + 10.0)),
        ..geometry
    }
}

fn anchor_modules(sim: &SimState, data: &GameData, geometry: &HullGeometry) -> Vec<ModuleGlyph> {
    let mut modules = Vec::new();
    modules.push(component_glyph(
        "bridge",
        "COMMAND BRIDGE",
        "CMD",
        Rect::new(geometry.x_at(0.05) - 6.0, geometry.cy - 18.0, 66.0, 36.0),
        ModuleKind::Bridge,
        sim.ship.hull_integrity,
        any_post_aboard(sim, BRIDGE_POSTS),
    ));
    let engine = data
        .ship_components
        .find(ComponentKind::Engine, &sim.ship.engine);
    let engine_label = engine
        .map(|component| component.name.to_uppercase())
        .unwrap_or_else(|| sim.ship.engine.to_uppercase());
    let engine_height = 38.0 + engine.map_or(0, |c| c.stats.speed.clamp(0, 6)) as f32 * 4.0;
    modules.push(component_glyph(
        &sim.ship.engine,
        &engine_label,
        "DRV",
        Rect::new(
            geometry.x_at(0.95) - 64.0,
            geometry.cy - engine_height / 2.0,
            72.0,
            engine_height,
        ),
        ModuleKind::Engine,
        sim.ship.fuel,
        any_post_aboard(sim, &["engineer"]),
    ));
    if let Some(weapon_id) = sim.ship.weapon.as_deref() {
        modules.push(weapon_glyph(sim, data, geometry, weapon_id));
    }
    modules
}

fn weapon_glyph(
    sim: &SimState,
    data: &GameData,
    geometry: &HullGeometry,
    weapon_id: &str,
) -> ModuleGlyph {
    let weapon = data.ship_components.find(ComponentKind::Weapon, weapon_id);
    let label = weapon
        .map(|component| component.name.to_uppercase())
        .unwrap_or_else(|| weapon_id.to_uppercase());
    let width = 60.0 + weapon.map_or(0, |c| c.stats.combat.clamp(0, 12)) as f32 * 3.0;
    component_glyph(
        weapon_id,
        &label,
        "WPN",
        Rect::new(
            geometry.x_at(0.5) - width / 2.0,
            geometry.cy - geometry.max_h - 30.0,
            width,
            24.0,
        ),
        ModuleKind::Weapon,
        sim.ship.hull_integrity,
        false,
    )
}

fn subsystem_modules(sim: &SimState, data: &GameData, geometry: &HullGeometry) -> Vec<ModuleGlyph> {
    let ids = GameData::sorted_ids(&data.subsystems);
    let cols = ids.len().div_ceil(2).max(1);
    ids.iter()
        .enumerate()
        .filter_map(|(i, id)| subsystem_glyph(sim, id, i, cols, geometry))
        .collect()
}

fn subsystem_glyph(
    sim: &SimState,
    id: &str,
    index: usize,
    cols: usize,
    geometry: &HullGeometry,
) -> Option<ModuleGlyph> {
    let state = sim.subsystems.get(id)?;
    let col = index % cols;
    let upper = index < cols;
    let t = 0.20 + (col as f32 + 0.5) / cols as f32 * 0.60;
    let visual_tier = state.tier.min(3) as f32;
    let width = 58.0 + visual_tier * 9.0;
    let height = 30.0 + visual_tier * 5.0;
    let band_y = if upper {
        geometry.cy - geometry.row_offset
    } else {
        geometry.cy + geometry.row_offset
    };
    let rect = Rect::new(
        geometry.x_at(t) - width * 0.5,
        band_y - height * 0.5,
        width,
        height,
    );
    let (deck_lo, deck_hi) = deck_range_for_slot(upper, col, cols);
    Some(ModuleGlyph {
        id: id.to_owned(),
        label: subsystem_short(id).to_owned(),
        code: subsystem_code(id).to_owned(),
        rect,
        kind: ModuleKind::Subsystem,
        condition: state.condition,
        tier: state.tier,
        manned: any_post_aboard(sim, subsystem_crew_posts(id)),
        deck_lo,
        deck_hi,
    })
}

fn component_glyph(
    id: &str,
    label: &str,
    code: &str,
    rect: Rect,
    kind: ModuleKind,
    condition: f32,
    manned: bool,
) -> ModuleGlyph {
    let (deck_lo, deck_hi) = component_deck_range(kind);
    ModuleGlyph {
        id: id.to_owned(),
        label: label.to_owned(),
        code: code.to_owned(),
        rect,
        kind,
        condition,
        tier: 0,
        manned,
        deck_lo,
        deck_hi,
    }
}
#[allow(dead_code, unused_imports)]
mod tests {
    include!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/unit/ui/ship_schematic/tests.rs"
    ));
}
