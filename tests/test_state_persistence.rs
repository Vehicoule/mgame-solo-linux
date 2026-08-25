//! Test suite verifying State persistence, JSON serialization, and defaults across all features.

use mgame_solo::state::MixerState;

#[test]
fn test_default_mixer_state_integrity() {
    let state = MixerState::default();
    
    // Fader channels check
    assert_eq!(*state.faders.get("microphone").unwrap(), 100);
    assert_eq!(*state.faders.get("game").unwrap(), 100);
    assert_eq!(*state.faders.get("chat").unwrap(), 100);
    assert_eq!(*state.faders.get("sampler").unwrap(), 100);
    assert_eq!(*state.faders.get("system").unwrap(), 100);

    // Mute initial states
    assert_eq!(*state.mutes.get("microphone").unwrap(), false);
    assert_eq!(*state.mutes.get("game").unwrap(), false);

    // Routing matrix defaults
    assert_eq!(*state.routing.get("microphone->stream").unwrap(), true);
    assert_eq!(*state.routing.get("microphone->chat").unwrap(), true);
    assert_eq!(*state.routing.get("microphone->main_out").unwrap(), false);
    assert_eq!(*state.routing.get("microphone->phones_out").unwrap(), true);

    // DSP defaults
    assert_eq!(state.equalizer.hpf_enabled, true);
    assert_eq!(state.compressor.enabled, false);
    assert_eq!(state.noise_gate.enabled, false);

    // Voice FX defaults
    assert_eq!(state.voice_fx.pitch_semitones, 0);
    assert_eq!(state.voice_fx.formant_semitones, 0);

    // Device defaults
    assert_eq!(state.phantom_power, false);
    assert_eq!(state.headphone_high_impedance, false);
    assert_eq!(state.led_theme, "Neon Blue");
}

#[test]
fn test_json_roundtrip_all_settings() {
    let mut state = MixerState::default();
    state.faders.insert("microphone".to_string(), 127);
    state.mutes.insert("microphone".to_string(), true);
    state.phantom_power = true;
    state.headphone_high_impedance = true;
    state.led_theme = "Cyberpunk Purple".to_string();
    state.voice_fx.pitch_semitones = 5;
    state.compressor.threshold_db = -18.5;

    let json_str = serde_json::to_string_pretty(&state).expect("Serialization failed");
    let restored: MixerState = serde_json::from_str(&json_str).expect("Deserialization failed");

    assert_eq!(*restored.faders.get("microphone").unwrap(), 127);
    assert_eq!(*restored.mutes.get("microphone").unwrap(), true);
    assert_eq!(restored.phantom_power, true);
    assert_eq!(restored.headphone_high_impedance, true);
    assert_eq!(restored.led_theme, "Cyberpunk Purple");
    assert_eq!(restored.voice_fx.pitch_semitones, 5);
    assert_eq!(restored.compressor.threshold_db, -18.5);
}
