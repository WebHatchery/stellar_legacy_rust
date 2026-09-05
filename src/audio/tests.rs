use super::*;

#[test]
fn procedural_cues_are_valid_pcm_wav_payloads() {
    let bytes = wave(0.1, |t| (t * 440.0 * TAU).sin() * 0.1);
    assert_eq!(&bytes[0..4], b"RIFF");
    assert_eq!(&bytes[8..12], b"WAVE");
    assert_eq!(&bytes[36..40], b"data");
    assert!(bytes.len() > 44);
}

#[test]
fn toolkit_encoding_preserves_the_legacy_header_clipping_and_quantization() {
    let bytes = wave(4.0 / 22_050.0, |t| {
        [-2.0, -0.5, 0.5, 2.0][(t * 22_050.0).round() as usize]
    });
    // Canonical legacy mono 22,050 Hz PCM16 file, with samples
    // -32767, -16383, 16383, 32767 (truncated toward zero, not rounded).
    assert_eq!(
        bytes,
        [
            82, 73, 70, 70, 44, 0, 0, 0, 87, 65, 86, 69, 102, 109, 116, 32, 16, 0, 0, 0, 1, 0, 1,
            0, 34, 86, 0, 0, 68, 172, 0, 0, 2, 0, 16, 0, 100, 97, 116, 97, 8, 0, 0, 0, 1, 128, 1,
            192, 255, 63, 255, 127,
        ]
    );
}
