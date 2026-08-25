//! Golden Tests for PipeWire Channel Mapping Configuration & Volume Parsing.

#[path = "../src/audio/pipewire.rs"]
mod pipewire;

use pipewire::{ensure_pipewire_configuration, parse_volume_percentage, set_channel_mute, set_channel_volume};

#[test]
fn test_pipewire_configuration_contains_all_four_pairs() {
    assert!(ensure_pipewire_configuration().is_ok());

    let config_dir = directories::BaseDirs::new().unwrap().config_dir().join("pipewire/pipewire.conf.d/50-m-game-solo.conf");
    let content = std::fs::read_to_string(config_dir).expect("Config file should exist");

    // Output Sinks
    assert!(content.contains("\"M-Game Solo (Game)\""));
    assert!(content.contains("\"M-Game Solo (Chat Out)\""));
    assert!(content.contains("\"M-Game Solo (Sampler Out)\""));
    assert!(content.contains("\"M-Game Solo (System)\""));

    // Input Sources
    assert!(content.contains("\"M-Game Solo (Stream Mix)\""));
    assert!(content.contains("\"M-Game Solo (Chat Mic)\""));
    assert!(content.contains("\"M-Game Solo (Sampler In)\""));
    assert!(content.contains("\"M-Game Solo (Aux In)\""));

    // Channel Isolation Directives
    assert!(content.contains("channelmix.disable = true"));
    assert!(content.contains("stream.dont-remix = true"));
}

#[test]
fn test_pipewire_volume_and_mute_execution() {
    set_channel_volume("system", 100);
    set_channel_mute("system", false);
    set_channel_volume("microphone", 80);
    set_channel_mute("microphone", true);
}

#[test]
fn test_pipewire_volume_parsing() {
    let mock_out = "Volume: front-left: 65536 / 100% / 0.00 dB,   front-right: 65536 / 100% / 0.00 dB\n        balance 0.00";
    let vol = parse_volume_percentage(mock_out);
    assert_eq!(vol, Some(100));

    let mock_out_80 = "Volume: front-left: 52428 /  80% / -5.81 dB,   front-right: 52428 /  80% / -5.81 dB";
    let vol_80 = parse_volume_percentage(mock_out_80);
    assert_eq!(vol_80, Some(80));

    let mock_out_over = "Volume: front-left: 98304 / 150% / +3.52 dB";
    let vol_over = parse_volume_percentage(mock_out_over);
    assert_eq!(vol_over, Some(150)); // Clamped to 150
}
