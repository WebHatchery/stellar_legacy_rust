//! Restrained procedural audio for high-value command-state feedback.
//!
//! The tiny WAVs are generated at startup, keeping Windows and WebGL packages
//! identical without adding an external asset pipeline. Cues never carry unique
//! information; every state change also has text and visual feedback.

use macroquad::audio::{load_sound_from_bytes, play_sound, stop_sound, PlaySoundParams, Sound};
use std::f32::consts::TAU;

#[derive(Debug, Clone, Copy)]
pub enum Cue {
    Button,
    Council,
    Resolution,
    Phase,
    Succession,
    Homecoming,
    GameOver,
}

pub struct AudioManager {
    ambience: Option<Sound>,
    button: Option<Sound>,
    council: Option<Sound>,
    resolution: Option<Sound>,
    phase: Option<Sound>,
    succession: Option<Sound>,
    homecoming: Option<Sound>,
    game_over: Option<Sound>,
    ambience_playing: bool,
}

impl AudioManager {
    pub async fn new() -> Self {
        Self {
            ambience: load(&wave(2.0, |t| {
                (t * 55.0 * TAU).sin() * 0.035 + (t * 82.5 * TAU).sin() * 0.018
            }))
            .await,
            button: load(&wave(0.07, |t| {
                (t * 520.0 * TAU).sin() * envelope(t, 0.07) * 0.2
            }))
            .await,
            council: load(&wave(0.42, |t| {
                let frequency = if t < 0.21 { 330.0 } else { 440.0 };
                (t * frequency * TAU).sin() * envelope(t % 0.21, 0.21) * 0.24
            }))
            .await,
            resolution: load(&wave(0.3, |t| {
                ((t * 440.0 * TAU).sin() + (t * 660.0 * TAU).sin()) * envelope(t, 0.3) * 0.1
            }))
            .await,
            phase: load(&wave(0.35, |t| {
                let frequency = 360.0 + t * 500.0;
                (t * frequency * TAU).sin() * envelope(t, 0.35) * 0.18
            }))
            .await,
            succession: load(&wave(0.55, |t| {
                ((t * 220.0 * TAU).sin() + (t * 277.2 * TAU).sin()) * envelope(t, 0.55) * 0.09
            }))
            .await,
            homecoming: load(&wave(0.9, |t| {
                ((t * 261.6 * TAU).sin() + (t * 329.6 * TAU).sin() + (t * 392.0 * TAU).sin())
                    * envelope(t, 0.9)
                    * 0.065
            }))
            .await,
            game_over: load(&wave(0.9, |t| {
                let frequency = 260.0 - t * 150.0;
                (t * frequency * TAU).sin() * envelope(t, 0.9) * 0.16
            }))
            .await,
            ambience_playing: false,
        }
    }

    pub fn cue(&self, cue: Cue, volume: f32) {
        let sound = match cue {
            Cue::Button => &self.button,
            Cue::Council => &self.council,
            Cue::Resolution => &self.resolution,
            Cue::Phase => &self.phase,
            Cue::Succession => &self.succession,
            Cue::Homecoming => &self.homecoming,
            Cue::GameOver => &self.game_over,
        };
        if let Some(sound) = sound {
            play_sound(
                sound,
                PlaySoundParams {
                    looped: false,
                    volume: volume.clamp(0.0, 1.0) * 0.55,
                },
            );
        }
    }

    pub fn update_ambience(&mut self, should_play: bool, volume: f32) {
        let Some(sound) = &self.ambience else {
            return;
        };
        if should_play && !self.ambience_playing {
            play_sound(
                sound,
                PlaySoundParams {
                    looped: true,
                    volume: volume.clamp(0.0, 1.0) * 0.18,
                },
            );
            self.ambience_playing = true;
        } else if !should_play && self.ambience_playing {
            stop_sound(sound);
            self.ambience_playing = false;
        }
    }
}

async fn load(bytes: &[u8]) -> Option<Sound> {
    load_sound_from_bytes(bytes).await.ok()
}

fn envelope(t: f32, duration: f32) -> f32 {
    let attack = (t / 0.015).clamp(0.0, 1.0);
    let release = ((duration - t) / 0.06).clamp(0.0, 1.0);
    attack * release
}

fn wave(duration: f32, sample: impl Fn(f32) -> f32) -> Vec<u8> {
    const RATE: u32 = 22_050;
    let count = (duration * RATE as f32) as u32;
    let data_len = count * 2;
    let mut out = Vec::with_capacity((44 + data_len) as usize);
    out.extend_from_slice(b"RIFF");
    out.extend_from_slice(&(36 + data_len).to_le_bytes());
    out.extend_from_slice(b"WAVEfmt ");
    out.extend_from_slice(&16u32.to_le_bytes());
    out.extend_from_slice(&1u16.to_le_bytes());
    out.extend_from_slice(&1u16.to_le_bytes());
    out.extend_from_slice(&RATE.to_le_bytes());
    out.extend_from_slice(&(RATE * 2).to_le_bytes());
    out.extend_from_slice(&2u16.to_le_bytes());
    out.extend_from_slice(&16u16.to_le_bytes());
    out.extend_from_slice(b"data");
    out.extend_from_slice(&data_len.to_le_bytes());
    for index in 0..count {
        let value = sample(index as f32 / RATE as f32).clamp(-1.0, 1.0);
        out.extend_from_slice(&((value * i16::MAX as f32) as i16).to_le_bytes());
    }
    out
}

#[cfg(test)]
mod tests;
