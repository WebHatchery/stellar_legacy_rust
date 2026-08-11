use super::*;

#[test]
fn procedural_cues_are_valid_pcm_wav_payloads() {
    let bytes = wave(0.1, |t| (t * 440.0 * TAU).sin() * 0.1);
    assert_eq!(&bytes[0..4], b"RIFF");
    assert_eq!(&bytes[8..12], b"WAVE");
    assert_eq!(&bytes[36..40], b"data");
    assert!(bytes.len() > 44);
}
