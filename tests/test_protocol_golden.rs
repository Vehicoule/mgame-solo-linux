//! Golden Tests & Protocol Verification for M-Game Solo.

#[path = "../src/midi/protocol.rs"]
mod protocol;

#[path = "../src/midi/dsp.rs"]
mod dsp;

use protocol::*;

#[test]
fn test_all_mute_packets_golden() {
    // 1. Microphone Mute
    let mic_mute = build_mute_msg(Source::Microphone, true);
    assert_eq!(mic_mute[0], 0xF0);
    assert_eq!(&mic_mute[1..4], &[0x00, 0x01, 0x05]);
    assert_eq!(mic_mute[4], 0x43);
    assert_eq!(mic_mute[7], Source::Microphone as u8);
    assert_eq!(mic_mute[8], Sink::Stream as u8);
    assert_eq!(mic_mute[9], MsgType::Button as u8);
    assert_eq!(mic_mute[10], 0x44); // Multi-destination prefix
    assert_eq!(mic_mute[11], 0x01); // muted = true
    assert_eq!(*mic_mute.last().unwrap(), 0xF7);

    // 2. Game Mute
    let game_mute = build_mute_msg(Source::Game, false);
    assert_eq!(game_mute[7], Source::Game as u8);
    assert_eq!(game_mute[11], 0x00); // muted = false

    // 3. Chat Mute
    let chat_mute = build_mute_msg(Source::Chat, true);
    assert_eq!(chat_mute[7], Source::Chat as u8);

    // 4. Sampler Mute
    let sampler_mute = build_mute_msg(Source::Sampler, true);
    assert_eq!(sampler_mute[7], Source::Sampler as u8);

    // 5. System Mute
    let system_mute = build_mute_msg(Source::System, true);
    assert_eq!(system_mute[7], Source::System as u8);
}

#[test]
fn test_dsp_and_voice_fx_packets() {
    // 80 Hz HPF
    let hpf = build_dsp_msg(0x10, 1);
    assert_eq!(hpf[9], MsgType::Dsp as u8);
    assert_eq!(hpf[11], 0x10);
    assert_eq!(hpf[12], 1);

    // Pitch shift
    let pitch = build_voice_fx_msg(0x01, 64);
    assert_eq!(pitch[9], MsgType::VoiceFx as u8);
    assert_eq!(pitch[11], 0x01);
    assert_eq!(pitch[12], 64);

    // LED Theme
    let theme = build_led_theme_msg(2);
    assert_eq!(theme[9], MsgType::Led as u8);
    assert_eq!(theme[11], 2);
}

#[test]
fn test_live_hardware_mute_decoding_golden() {
    // Real packet captured from M-Game Solo when Mic is muted:
    let raw_mic_muted = [0xF0, 0x00, 0x01, 0x05, 0x43, 0x00, 0x02, 0x00, 0x00, 0x02, 0x44, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x00, 0x00, 0x69, 0xF7];
    let parsed = parse_sysex(&raw_mic_muted).expect("Should parse valid mic mute packet");
    assert_eq!(parsed.source, Source::Microphone);
    assert_eq!(parsed.sink, Sink::Stream);
    assert_eq!(parsed.msg_type, MsgType::Button);
    assert_eq!(parsed.payload[0], 0x44);
    assert_eq!(parsed.payload[1], 0x01); // Muted!

    // Real packet captured from M-Game Solo when Mic is unmuted:
    let raw_mic_unmuted = [0xF0, 0x00, 0x01, 0x05, 0x43, 0x00, 0x02, 0x00, 0x00, 0x02, 0x44, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x6F, 0xF7];
    let parsed_unmuted = parse_sysex(&raw_mic_unmuted).expect("Should parse valid mic unmute packet");
    assert_eq!(parsed_unmuted.source, Source::Microphone);
    assert_eq!(parsed_unmuted.payload[1], 0x00); // Unmuted!
}

#[test]
fn test_error_packet_rejection_golden() {
    // Real ACK/error packet (status 0x7F) from M-Game Solo:
    let raw_err = [0xF0, 0x00, 0x01, 0x05, 0x43, 0x7F, 0x00, 0x04, 0x05, 0x02, 0x00, 0x2D, 0xF7];
    let parsed = parse_sysex(&raw_err).expect("Checksum is valid");
    assert_eq!(parsed.success, 0x7F); // Daemon MUST ignore this packet because success != 0x00
}
