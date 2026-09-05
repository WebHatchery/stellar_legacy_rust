//! Restrained procedural audio for high-value command-state feedback.
//!
//! The tiny WAVs are generated at startup, keeping Windows and WebGL packages
//! identical without adding an external asset pipeline. Cues never carry unique
//! information; every state change also has text and visual feedback.

use macroquad::audio::PlaySoundParams;
use macroquad_toolkit::audio::SoundManager;
use std::f32::consts::TAU;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Cue {
    Ambience,
    Button,
    Council,
    Resolution,
    Phase,
    Succession,
    Homecoming,
    GameOver,
}

pub struct AudioManager {
    sounds: SoundManager<Cue>,
    ambience_playing: bool,
}

impl AudioManager {
    pub async fn new() -> Self {
        let generated = [
            (
                Cue::Ambience,
                wave(2.0, |t| {
                    (t * 55.0 * TAU).sin() * 0.035 + (t * 82.5 * TAU).sin() * 0.018
                }),
            ),
            (
                Cue::Button,
                wave(0.07, |t| (t * 520.0 * TAU).sin() * envelope(t, 0.07) * 0.2),
            ),
            (
                Cue::Council,
                wave(0.42, |t| {
                    let frequency = if t < 0.21 { 330.0 } else { 440.0 };
                    (t * frequency * TAU).sin() * envelope(t % 0.21, 0.21) * 0.24
                }),
            ),
            (
                Cue::Resolution,
                wave(0.3, |t| {
                    ((t * 440.0 * TAU).sin() + (t * 660.0 * TAU).sin()) * envelope(t, 0.3) * 0.1
                }),
            ),
            (
                Cue::Phase,
                wave(0.35, |t| {
                    let frequency = 360.0 + t * 500.0;
                    (t * frequency * TAU).sin() * envelope(t, 0.35) * 0.18
                }),
            ),
            (
                Cue::Succession,
                wave(0.55, |t| {
                    ((t * 220.0 * TAU).sin() + (t * 277.2 * TAU).sin()) * envelope(t, 0.55) * 0.09
                }),
            ),
            (
                Cue::Homecoming,
                wave(0.9, |t| {
                    ((t * 261.6 * TAU).sin() + (t * 329.6 * TAU).sin() + (t * 392.0 * TAU).sin())
                        * envelope(t, 0.9)
                        * 0.065
                }),
            ),
            (
                Cue::GameOver,
                wave(0.9, |t| {
                    let frequency = 260.0 - t * 150.0;
                    (t * frequency * TAU).sin() * envelope(t, 0.9) * 0.16
                }),
            ),
        ];
        let mut sounds = SoundManager::new();
        for (cue, bytes) in generated {
            let _ = sounds.load_sound_bytes(cue, &bytes).await;
        }
        Self {
            sounds,
            ambience_playing: false,
        }
    }

    pub fn cue(&self, cue: Cue, volume: f32) {
        self.sounds.play_raw(
            cue,
            PlaySoundParams {
                looped: false,
                volume: volume.clamp(0.0, 1.0) * 0.55,
            },
        );
    }
    pub fn update_ambience(&mut self, should_play: bool, volume: f32) {
        if !self.sounds.has_sound(Cue::Ambience) {
            return;
        }
        if should_play && !self.ambience_playing {
            self.sounds.play_raw(
                Cue::Ambience,
                PlaySoundParams {
                    looped: true,
                    volume: volume.clamp(0.0, 1.0) * 0.18,
                },
            );
            self.ambience_playing = true;
        } else if !should_play && self.ambience_playing {
            self.sounds.stop_raw(Cue::Ambience);
            self.ambience_playing = false;
        }
    }
}

fn envelope(t: f32, duration: f32) -> f32 {
    let attack = (t / 0.015).clamp(0.0, 1.0);
    let release = ((duration - t) / 0.06).clamp(0.0, 1.0);
    attack * release
}

fn wave(duration: f32, sample: impl Fn(f32) -> f32) -> Vec<u8> {
    const RATE: u32 = 22_050;
    let count = (duration * RATE as f32) as u32;
    let pcm: Vec<i16> = (0..count)
        .map(|index| {
            let value = sample(index as f32 / RATE as f32).clamp(-1.0, 1.0);
            (value * i16::MAX as f32) as i16
        })
        .collect();
    macroquad_toolkit::synth::wav_bytes(
        &pcm,
        &macroquad_toolkit::synth::SynthConfig {
            sample_rate: RATE,
            ..Default::default()
        },
    )
}

#[cfg(test)]
mod tests;
