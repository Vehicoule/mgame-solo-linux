//! Application State Store & Persistence for M-Game Solo.

use std::collections::HashMap;
use std::fs;
use serde::{Deserialize, Serialize};

use crate::midi::dsp::{CompressorSettings, EqualizerSettings, NoiseGateSettings, VoiceFxSettings};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MixerState {
    pub faders: HashMap<String, u8>,
    pub mutes: HashMap<String, bool>,
    pub routing: HashMap<String, bool>,
    pub phantom_power: bool,
    pub headphone_high_impedance: bool,
    pub censor_mode: String,
    pub led_theme: String,
    pub compressor: CompressorSettings,
    pub noise_gate: NoiseGateSettings,
    pub equalizer: EqualizerSettings,
    pub voice_fx: VoiceFxSettings,
}

impl Default for MixerState {
    fn default() -> Self {
        let mut faders = HashMap::new();
        faders.insert("microphone".to_string(), 100);
        faders.insert("game".to_string(), 100);
        faders.insert("chat".to_string(), 100);
        faders.insert("sampler".to_string(), 100);
        faders.insert("system".to_string(), 100);

        let mut mutes = HashMap::new();
        mutes.insert("microphone".to_string(), false);
        mutes.insert("game".to_string(), false);
        mutes.insert("chat".to_string(), false);
        mutes.insert("sampler".to_string(), false);
        mutes.insert("system".to_string(), false);

        let mut routing = HashMap::new();
        for src in &["microphone", "game", "chat", "sampler", "system"] {
            for snk in &["stream", "chat", "alt_usb", "sampler", "main_out", "phones_out"] {
                let key = format!("{}->{}", src, snk);
                let default_enabled = match (*src, *snk) {
                    ("microphone", "chat") | ("microphone", "phones_out") | ("microphone", "stream") => true,
                    ("game", "phones_out") | ("game", "main_out") | ("game", "stream") => true,
                    ("chat", "phones_out") | ("chat", "main_out") | ("chat", "stream") => true,
                    ("sampler", "phones_out") | ("sampler", "main_out") | ("sampler", "stream") => true,
                    ("system", "phones_out") | ("system", "main_out") | ("system", "stream") => true,
                    _ => false,
                };
                routing.insert(key, default_enabled);
            }
        }

        Self {
            faders,
            mutes,
            routing,
            phantom_power: false,
            headphone_high_impedance: false,
            censor_mode: "Beep Tone".to_string(),
            led_theme: "Neon Blue".to_string(),
            compressor: CompressorSettings::default(),
            noise_gate: NoiseGateSettings::default(),
            equalizer: EqualizerSettings::default(),
            voice_fx: VoiceFxSettings::default(),
        }
    }
}

impl MixerState {
    pub fn load_or_default() -> Self {
        if let Some(config_dir) = directories::BaseDirs::new().map(|d| d.config_dir().to_path_buf()) {
            let state_path = config_dir.join("mgame-solo/state.json");
            if let Ok(content) = fs::read_to_string(state_path) {
                if let Ok(loaded) = serde_json::from_str::<MixerState>(&content) {
                    return loaded;
                }
            }
        }
        Self::default()
    }

    pub fn save(&self) {
        if let Some(config_dir) = directories::BaseDirs::new().map(|d| d.config_dir().to_path_buf()) {
            let dir = config_dir.join("mgame-solo");
            let _ = fs::create_dir_all(&dir);
            let state_path = dir.join("state.json");
            if let Ok(json) = serde_json::to_string_pretty(self) {
                let _ = fs::write(state_path, json);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_state_initialization() {
        let state = MixerState::default();
        assert_eq!(state.faders.get("microphone"), Some(&100));
        assert_eq!(state.faders.get("game"), Some(&100));
        assert_eq!(state.mutes.get("microphone"), Some(&false));
        assert_eq!(state.mutes.get("game"), Some(&false));
        assert_eq!(state.routing.get("microphone->stream"), Some(&true));
        assert_eq!(state.routing.get("microphone->main_out"), Some(&false));
    }

    #[test]
    fn test_json_roundtrip() {
        let mut state = MixerState::default();
        state.faders.insert("microphone".to_string(), 127);
        state.mutes.insert("game".to_string(), true);
        state.phantom_power = true;

        let json = serde_json::to_string(&state).expect("Serialization failed");
        let decoded: MixerState = serde_json::from_str(&json).expect("Deserialization failed");

        assert_eq!(decoded.faders.get("microphone"), Some(&127));
        assert_eq!(decoded.mutes.get("game"), Some(&true));
        assert_eq!(decoded.phantom_power, true);
    }
}
