//! Ground-Truth SysEx Protocol Definitions & Builders for M-Audio M-Game Solo.

pub const SYSEX_START: u8 = 0xF0;
pub const SYSEX_END: u8 = 0xF7;
pub const MANUFACTURER_ID: [u8; 3] = [0x00, 0x01, 0x05];
pub const PRODUCT_ID: u8 = 0x43;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum LengthByte {
    Tiny = 0x00,
    Short = 0x01,
    Long = 0x02,
    Longer = 0x04,
    Bigger = 0x06,
    Huge = 0x0B,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Source {
    Microphone = 0x00,
    Game = 0x02,
    Chat = 0x04,
    Sampler = 0x08,
    System = 0x0A,
    Aux = 0x0C,
    VoiceFx = 0x0E,
    Banking = 0x10,
    Button1 = 0x14,
    Button2 = 0x16,
    Button3 = 0x18,
    Button4 = 0x1A,
    Button5 = 0x1C,
    Button6 = 0x1E, // Censor
}

impl Source {
    pub fn from_u8(val: u8) -> Option<Self> {
        match val {
            0x00 => Some(Self::Microphone),
            0x02 => Some(Self::Game),
            0x04 => Some(Self::Chat),
            0x08 => Some(Self::Sampler),
            0x0A => Some(Self::System),
            0x0C => Some(Self::Aux),
            0x0E => Some(Self::VoiceFx),
            0x10 => Some(Self::Banking),
            0x14 => Some(Self::Button1),
            0x16 => Some(Self::Button2),
            0x18 => Some(Self::Button3),
            0x1A => Some(Self::Button4),
            0x1C => Some(Self::Button5),
            0x1E => Some(Self::Button6),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Sink {
    Stream = 0x00,
    Chat = 0x01,
    AltUsb = 0x02,
    Sampler = 0x03,
    MainOut = 0x04,
    PhonesOut = 0x05,
}

impl Sink {
    pub fn from_u8(val: u8) -> Option<Self> {
        match val {
            0x00 => Some(Self::Stream),
            0x01 => Some(Self::Chat),
            0x02 => Some(Self::AltUsb),
            0x03 => Some(Self::Sampler),
            0x04 => Some(Self::MainOut),
            0x05 => Some(Self::PhonesOut),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum MsgType {
    Patch = 0x00,
    Volume = 0x01,
    Button = 0x02,
    VoiceFx = 0x04,
    Dsp = 0x05,
    Led = 0x06,
}

impl MsgType {
    pub fn from_u8(val: u8) -> Option<Self> {
        match val {
            0x00 => Some(Self::Patch),
            0x01 => Some(Self::Volume),
            0x02 => Some(Self::Button),
            0x04 => Some(Self::VoiceFx),
            0x05 => Some(Self::Dsp),
            0x06 => Some(Self::Led),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MGSysExMessage {
    pub source: Source,
    pub sink: Sink,
    pub msg_type: MsgType,
    pub payload: Vec<u8>,
    pub length: LengthByte,
    pub success: u8,
}

pub fn compute_checksum(body: &[u8]) -> u8 {
    let sum: u32 = body.iter().map(|&b| b as u32).sum();
    ((128 - (sum % 128)) % 128) as u8
}

pub fn verify_checksum(body: &[u8]) -> bool {
    if body.is_empty() {
        return false;
    }
    let data = &body[..body.len() - 1];
    let expected = body[body.len() - 1];
    compute_checksum(data) == expected
}

impl MGSysExMessage {
    pub fn new(source: Source, sink: Sink, msg_type: MsgType, payload: Vec<u8>, length: LengthByte) -> Self {
        Self {
            source,
            sink,
            msg_type,
            payload,
            length,
            success: 0x00,
        }
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut body = Vec::with_capacity(20);
        body.extend_from_slice(&MANUFACTURER_ID);
        body.push(PRODUCT_ID);
        body.push(self.success);
        body.push(self.length as u8);
        body.push(self.source as u8);
        body.push(self.sink as u8);
        body.push(self.msg_type as u8);
        body.extend_from_slice(&self.payload);

        let chk = compute_checksum(&body);

        let mut packet = Vec::with_capacity(body.len() + 2);
        packet.push(SYSEX_START);
        packet.extend_from_slice(&body);
        packet.push(chk);
        packet.push(SYSEX_END);
        packet
    }
}

pub fn parse_sysex(raw: &[u8]) -> Option<MGSysExMessage> {
    if raw.len() < 11 || raw[0] != SYSEX_START || raw[raw.len() - 1] != SYSEX_END {
        return None;
    }

    let body = &raw[1..raw.len() - 1];
    if !verify_checksum(body) {
        return None;
    }

    if body[..3] != MANUFACTURER_ID || body[3] != PRODUCT_ID {
        return None;
    }

    let success = body[4];
    let length = match body[5] {
        0x00 => LengthByte::Tiny,
        0x01 => LengthByte::Short,
        0x02 => LengthByte::Long,
        0x04 => LengthByte::Longer,
        0x06 => LengthByte::Bigger,
        0x0B => LengthByte::Huge,
        _ => LengthByte::Short,
    };

    let source = Source::from_u8(body[6])?;
    let sink = Sink::from_u8(body[7])?;
    let msg_type = MsgType::from_u8(body[8])?;
    let payload = body[9..body.len() - 1].to_vec();

    Some(MGSysExMessage {
        source,
        sink,
        msg_type,
        payload,
        length,
        success,
    })
}

// -------------------------------------------------------------
// Ground-Truth SysEx Packet Builders (Verified against Hardware)
// -------------------------------------------------------------

pub fn build_volume_msg(source: Source, sink: Sink, value_7bit: u8) -> Vec<u8> {
    let val = value_7bit.min(127);
    let payload = vec![0x00, val, 0x00, 0x00, 0x00];
    MGSysExMessage::new(source, sink, MsgType::Volume, payload, LengthByte::Short).encode()
}

pub fn build_patch_msg(source: Source, sink: Sink, enabled: bool) -> Vec<u8> {
    let state_byte = if enabled { 0x73 } else { 0x00 };
    let payload = vec![0x44, state_byte, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
    MGSysExMessage::new(source, sink, MsgType::Patch, payload, LengthByte::Long).encode()
}

pub fn build_mute_msg(source: Source, muted: bool) -> Vec<u8> {
    let val = if muted { 0x01 } else { 0x00 };
    // Multi-destination mute payload (Stream, Chat, AltUsb, Sampler, Main, Phones)
    let payload = vec![0x44, val, val, val, val, val, val, 0x00, 0x00];
    MGSysExMessage::new(source, Sink::Stream, MsgType::Button, payload, LengthByte::Long).encode()
}

pub fn build_phantom_power_msg(enabled: bool) -> Vec<u8> {
    let val = if enabled { 0x01 } else { 0x00 };
    let payload = vec![0x08, val, 0x00, 0x00, 0x00];
    MGSysExMessage::new(Source::Microphone, Sink::Stream, MsgType::Button, payload, LengthByte::Short).encode()
}

pub fn build_headphone_mode_msg(high_impedance: bool) -> Vec<u8> {
    let val = if high_impedance { 0x01 } else { 0x00 };
    let payload = vec![0x09, val, 0x00, 0x00, 0x00];
    MGSysExMessage::new(Source::Banking, Sink::PhonesOut, MsgType::Button, payload, LengthByte::Short).encode()
}

pub fn build_voice_fx_msg(param: u8, val: u8) -> Vec<u8> {
    let payload = vec![0x00, param, val & 0x7F, 0x00, 0x00];
    MGSysExMessage::new(Source::VoiceFx, Sink::Stream, MsgType::VoiceFx, payload, LengthByte::Short).encode()
}

pub fn build_dsp_msg(param: u8, val: u8) -> Vec<u8> {
    let payload = vec![0x00, param, val & 0x7F, 0x00, 0x00];
    MGSysExMessage::new(Source::Microphone, Sink::Stream, MsgType::Dsp, payload, LengthByte::Short).encode()
}

pub fn build_led_theme_msg(theme_idx: u8) -> Vec<u8> {
    let payload = vec![0x00, theme_idx & 0x0F, 0x00, 0x00, 0x00];
    MGSysExMessage::new(Source::Microphone, Sink::Stream, MsgType::Led, payload, LengthByte::Short).encode()
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn test_checksum_golden_values() {
        let raw = [0xF0, 0x00, 0x01, 0x05, 0x43, 0x00, 0x02, 0x00, 0x00, 0x02, 0x44, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x00, 0x00, 0x69, 0xF7];
        let body = &raw[1..raw.len() - 1];
        assert!(verify_checksum(body));
        let expected_chk = compute_checksum(&body[..body.len() - 1]);
        assert_eq!(expected_chk, 0x69);
    }

    #[test]
    fn test_phantom_power_packet() {
        let pkt = build_phantom_power_msg(true);
        assert_eq!(pkt[0], 0xF0);
        assert_eq!(pkt[7], Source::Microphone as u8);
        assert_eq!(pkt[9], MsgType::Button as u8);
        assert_eq!(pkt[10], 0x08);
        assert_eq!(pkt[11], 0x01);
    }

    #[test]
    fn test_patch_packet_golden() {
        let pkt = build_patch_msg(Source::Microphone, Sink::Stream, true);
        assert_eq!(pkt[0], 0xF0);
        assert_eq!(pkt[7], Source::Microphone as u8);
        assert_eq!(pkt[8], Sink::Stream as u8);
        assert_eq!(pkt[9], MsgType::Patch as u8);
        assert_eq!(*pkt.last().unwrap(), 0xF7);
    }

    #[test]
    fn test_volume_packet_roundtrip() {
        let pkt = build_volume_msg(Source::Game, Sink::Stream, 100);
        let parsed = parse_sysex(&pkt).expect("Should parse valid volume message");
        assert_eq!(parsed.source, Source::Game);
        assert_eq!(parsed.sink, Sink::Stream);
        assert_eq!(parsed.msg_type, MsgType::Volume);
        assert_eq!(parsed.payload[1], 100);
    }
}
