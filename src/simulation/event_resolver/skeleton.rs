//! Seeded campaign skeleton (W6): the major beats of a mission, laid out at
//! LAUNCH from the mission seed so a centuries-long voyage reads as a generated
//! campaign rather than a random-event stream. Same seed ⇒ same schedule.
//!
//! The families themselves are authored content (JSON); only the *pool
//! structure* — which families belong to which phase — is mechanics, and lives
//! here as a constant table.

use crate::data::contracts::ContractPhase;
use crate::data::CampaignSkeletonConfig;
use crate::state::sim::{ActiveContract, CampaignBeat};
use macroquad_toolkit::rng::SeededRng;

fn pool_for_phase(cfg: &CampaignSkeletonConfig, phase: ContractPhase) -> &[String] {
    match phase {
        ContractPhase::Travel | ContractPhase::Preparation => &cfg.travel_pool,
        ContractPhase::Operation => &cfg.operation_pool,
        ContractPhase::Return | ContractPhase::Completion => &cfg.return_pool,
    }
}

/// Lay out the campaign beats for `contract` (W6): one beat per full
/// `months_per_window` of mission duration, each placed uniformly at random
/// within its own window (skipping the first `skip_months` overall), drawing a
/// family from the phase pool active at that month, the any-phase families, and
/// — depending where in the voyage the beat lands — the founding-era or
/// homecoming-era pool (content-depth era layering). Deterministic for a given
/// rng state.
pub fn generate_beats(
    rng: &mut SeededRng,
    contract: &ActiveContract,
    cfg: &CampaignSkeletonConfig,
) -> Vec<CampaignBeat> {
    let total_months = contract.total_months();
    let windows = total_months / cfg.months_per_window;
    let early_cutoff = (total_months as f32 * cfg.early_fraction) as u32;
    let late_cutoff = (total_months as f32 * cfg.late_fraction) as u32;
    let mut beats = Vec::with_capacity(windows as usize);
    for i in 0..windows {
        let window_start = i * cfg.months_per_window;
        let lo = window_start.max(cfg.skip_months);
        let hi = window_start + cfg.months_per_window;
        if lo >= hi {
            continue;
        }
        let month = lo + rng.below((hi - lo) as usize) as u32;
        let (_, phase) = contract.phase_at(month + 1);
        // Build the eligible draw for this beat: phase pool + any-phase, plus the
        // era pool for where it lands. Order is deterministic, so a fixed rng
        // state yields a fixed schedule.
        let mut draw: Vec<&str> = pool_for_phase(cfg, phase)
            .iter()
            .chain(cfg.any_pool.iter())
            .map(String::as_str)
            .collect();
        if month < early_cutoff {
            draw.extend(cfg.early_pool.iter().map(String::as_str));
        } else if month < late_cutoff {
            // The deep middle: neither founding nor homecoming, tinted by the
            // era no one aboard remembers beginning (content-depth round 4).
            draw.extend(cfg.mid_pool.iter().map(String::as_str));
        }
        if month >= late_cutoff {
            draw.extend(cfg.late_pool.iter().map(String::as_str));
        }
        // The charter's own bias (content-depth round 7): its families ride in
        // every window's draw, weighting the campaign toward the mission's flavor.
        draw.extend(contract.beat_families.iter().map(String::as_str));
        let family = draw[rng.below(draw.len())].to_owned();
        beats.push(CampaignBeat {
            month_clock: month,
            family,
            fired: false,
        });
    }
    beats
}

#[cfg(test)]
mod tests;
