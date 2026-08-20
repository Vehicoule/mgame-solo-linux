//! Audio DSP Curves, Calibration Tables, and Value Converters for M-Game Solo.

use serde::{Deserialize, Serialize};

/// Convert percentage slider value (0..150%) to logarithmic dB representation.
/// Unity Gain (0.0 dB) is 100%. Max Boost (+3.5 dB) is 150%. Zero is -inf dB.
pub fn fader_val_to_db(val: u8) -> f64 {
    if val == 0 {
        return f64::NEG_INFINITY;
    }
    if val == 100 {
        return 0.0;
    }
    let normalized = val as f64 / 100.0;
    let db = 20.0 * normalized.log10();
    (db.max(-60.0) * 10.0).round() / 10.0
}

/// Convert dB value back to percentage slider value (0..150%)
#[allow(dead_code)]
pub fn db_to_fader_val(db: f64) -> u8 {
    if db <= -60.0 || db.is_infinite() {
        return 0;
    }
    if (db - 0.0).abs() < 0.05 {
        return 100;
    }
    let normalized = 10.0f64.powf(db / 20.0);
    ((normalized * 100.0 / 5.0).round() * 5.0).clamp(0.0, 150.0) as u8
}

/// Format fader dB for clean UI display
pub fn format_db(db: f64) -> String {
    if db.is_infinite() || db <= -60.0 {
        "-inf dB".to_string()
    } else if db > 0.0 {
        format!("+{:.1} dB", db)
    } else {
        format!("{:.1} dB", db)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CompressorSettings {
    pub enabled: bool,
    pub threshold_db: f64, // -40.0 .. 0.0 dB
    pub ratio: f64,        // 1.0 .. 20.0:1
    pub attack_ms: f64,    // 0.1 .. 100.0 ms
    pub release_ms: f64,   // 10.0 .. 1000.0 ms
}

impl Default for CompressorSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            threshold_db: -20.0,
            ratio: 4.0,
            attack_ms: 10.0,
            release_ms: 100.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NoiseGateSettings {
    pub enabled: bool,
    pub threshold_db: f64, // -60.0 .. 0.0 dB
    pub depth_db: f64,     // -60.0 .. 0.0 dB
    pub hold_ms: f64,      // 0.0 .. 500.0 ms
    pub release_ms: f64,   // 10.0 .. 500.0 ms
}

impl Default for NoiseGateSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            threshold_db: -45.0,
            depth_db: -40.0,
            hold_ms: 50.0,
            release_ms: 150.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EqualizerSettings {
    pub enabled: bool,
    pub hpf_enabled: bool,
    pub hpf_freq_hz: f64,  // 80.0 Hz standard
    pub low_gain_db: f64,  // -12.0 .. +12.0 dB (80 Hz Low Shelf)
    pub mid_gain_db: f64,  // -12.0 .. +12.0 dB (2.5 kHz Peak)
    pub high_gain_db: f64, // -12.0 .. +12.0 dB (10 kHz High Shelf)
}

impl Default for EqualizerSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            hpf_enabled: true,
            hpf_freq_hz: 80.0,
            low_gain_db: 0.0,
            mid_gain_db: 0.0,
            high_gain_db: 0.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VoiceFxSettings {
    pub enabled: bool,
    pub pitch_semitones: i8,   // -12 .. +12
    pub formant_semitones: i8, // -12 .. +12
    pub reverb_preset: u8,     // 0: Off, 1: Small Room, 2: Large Hall, 3: Space
    pub reverb_mix_percent: u8,// 0 .. 100%
}

impl Default for VoiceFxSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            pitch_semitones: 0,
            formant_semitones: 0,
            reverb_preset: 0,
            reverb_mix_percent: 20,
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    pub fn test_zero_fader_is_inf() {
        let db = fader_val_to_db(0);
        assert!(db.is_infinite());
        assert!(db.is_sign_negative());
    }

    #[test]
    pub fn test_unity_gain_calibration() {
        let db = fader_val_to_db(100);
        assert_eq!(db, 0.0);
    }

    #[test]
    pub fn test_max_gain_calibration() {
        let db = fader_val_to_db(150);
        assert_eq!(db, 3.5);
    }
}
