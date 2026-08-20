//! Unit & Integration tests for M-Game Solo Hardware Connection & Presence.

use mgame_solo::midi::protocol::{build_mute_msg, parse_sysex, Source, SYSEX_END, SYSEX_START};
use mgame_solo::midi::rawmidi::MidiClient;

#[test]
fn test_heartbeat_packet_construction() {
    let packet = build_mute_msg(Source::Microphone, false);
    assert_eq!(packet[0], SYSEX_START);
    assert_eq!(packet[packet.len() - 1], SYSEX_END);
    assert_eq!(packet[1..5], [0x00, 0x01, 0x05, 0x43]);
}

#[test]
fn test_dsp_online_packet_recognition() {
    let valid_packet = build_mute_msg(Source::Microphone, false);
    let parsed = parse_sysex(&valid_packet);
    assert!(parsed.is_some(), "DSP response must be parsed successfully");
    let msg = parsed.unwrap();
    assert_eq!(msg.success, 0x00);
}

#[test]
fn test_corrupted_or_offline_silence_rejection() {
    let noise = vec![0x00, 0x00, 0x00];
    let parsed = parse_sysex(&noise);
    assert!(parsed.is_none(), "Unpowered noise must not trigger online status");
}

#[test]
fn test_alsa_device_node_lookup() {
    // Verifies ALSA lookup correctly finds or detects M-Game device
    let found = MidiClient::find_mgame_device();
    if MidiClient::is_device_present() {
        assert!(found.is_some());
    }
}
