//! Microphone Processing (DSP) View for M-Game Solo.
//! Implements Compressor / Limiter, Noise Gate, 80 Hz HPF, and 3-Band Equalizer controls.

use gtk4::prelude::*;
use gtk4::{Align, Orientation, Scale, ScrolledWindow, Switch};
use libadwaita::prelude::*;
use libadwaita::{ActionRow, PreferencesGroup, PreferencesPage};
use std::sync::{Arc, RwLock};

use crate::state::MixerState;

pub struct DspView {
    pub container: ScrolledWindow,
}

impl DspView {
    pub fn new<FDsp>(state: Arc<RwLock<MixerState>>, on_dsp_change: FDsp) -> Self
    where
        FDsp: Fn(&str, f64) + Clone + 'static,
    {
        let page = PreferencesPage::new();
        let state_guard = state.read().unwrap();

        // 1. High Pass Filter
        let hpf_group = PreferencesGroup::builder()
            .title("High-Pass Filter (HPF)")
            .description("Cuts sub-bass rumble, desk thumps, and mechanical vibrations below 80 Hz.")
            .build();

        let hpf_switch = Switch::builder()
            .active(state_guard.equalizer.hpf_enabled)
            .valign(Align::Center)
            .build();

        let on_dsp = on_dsp_change.clone();
        hpf_switch.connect_active_notify(move |s| {
            on_dsp("hpf_enabled", if s.is_active() { 1.0 } else { 0.0 });
        });

        let hpf_row = ActionRow::builder()
            .title("80 Hz High-Pass Filter")
            .subtitle("Standard 18 dB/oct roll-off")
            .activatable_widget(&hpf_switch)
            .build();
        hpf_row.add_suffix(&hpf_switch);
        hpf_group.add(&hpf_row);
        page.add(&hpf_group);

        // 2. Compressor / Limiter
        let comp_group = PreferencesGroup::builder()
            .title("Compressor & Limiter")
            .description("Evens out vocal dynamics to prevent clipping and ensure consistent loudness.")
            .build();

        let comp_switch = Switch::builder()
            .active(state_guard.compressor.enabled)
            .valign(Align::Center)
            .build();

        let on_dsp = on_dsp_change.clone();
        comp_switch.connect_active_notify(move |s| {
            on_dsp("comp_enabled", if s.is_active() { 1.0 } else { 0.0 });
        });

        let comp_row = ActionRow::builder()
            .title("Enable Compressor")
            .activatable_widget(&comp_switch)
            .build();
        comp_row.add_suffix(&comp_switch);
        comp_group.add(&comp_row);

        // Threshold Scale
        let thresh_scale = Scale::with_range(Orientation::Horizontal, -40.0, 0.0, 1.0);
        thresh_scale.set_value(state_guard.compressor.threshold_db);
        thresh_scale.set_hexpand(true);
        let on_dsp = on_dsp_change.clone();
        thresh_scale.connect_value_changed(move |s| {
            on_dsp("comp_threshold", s.value());
        });
        let thresh_row = ActionRow::builder()
            .title("Threshold")
            .subtitle("Level above which compression occurs (-40 to 0 dB)")
            .build();
        thresh_row.add_suffix(&thresh_scale);
        comp_group.add(&thresh_row);

        // Ratio Scale
        let ratio_scale = Scale::with_range(Orientation::Horizontal, 1.0, 20.0, 0.5);
        ratio_scale.set_value(state_guard.compressor.ratio);
        ratio_scale.set_hexpand(true);
        let on_dsp = on_dsp_change.clone();
        ratio_scale.connect_value_changed(move |s| {
            on_dsp("comp_ratio", s.value());
        });
        let ratio_row = ActionRow::builder()
            .title("Ratio")
            .subtitle("Compression ratio (1:1 to 20:1)")
            .build();
        ratio_row.add_suffix(&ratio_scale);
        comp_group.add(&ratio_row);

        page.add(&comp_group);

        // 3. Noise Gate
        let gate_group = PreferencesGroup::builder()
            .title("Noise Gate")
            .description("Mutes ambient room noise, keyboard clicks, and PC fan hum when not speaking.")
            .build();

        let gate_switch = Switch::builder()
            .active(state_guard.noise_gate.enabled)
            .valign(Align::Center)
            .build();

        let on_dsp = on_dsp_change.clone();
        gate_switch.connect_active_notify(move |s| {
            on_dsp("gate_enabled", if s.is_active() { 1.0 } else { 0.0 });
        });

        let gate_row = ActionRow::builder()
            .title("Enable Noise Gate")
            .activatable_widget(&gate_switch)
            .build();
        gate_row.add_suffix(&gate_switch);
        gate_group.add(&gate_row);

        let gate_thresh = Scale::with_range(Orientation::Horizontal, -60.0, 0.0, 1.0);
        gate_thresh.set_value(state_guard.noise_gate.threshold_db);
        gate_thresh.set_hexpand(true);
        let on_dsp = on_dsp_change.clone();
        gate_thresh.connect_value_changed(move |s| {
            on_dsp("gate_threshold", s.value());
        });
        let gate_thresh_row = ActionRow::builder()
            .title("Gate Threshold")
            .subtitle("Signals below this level are attenuated (-60 to 0 dB)")
            .build();
        gate_thresh_row.add_suffix(&gate_thresh);
        gate_group.add(&gate_thresh_row);

        page.add(&gate_group);

        // 4. Parametric Equalizer
        let eq_group = PreferencesGroup::builder()
            .title("3-Band Equalizer")
            .description("Tune microphone tone across Lows (80 Hz), Mids (2.5 kHz), and Highs (10 kHz).")
            .build();

        let eq_switch = Switch::builder()
            .active(state_guard.equalizer.enabled)
            .valign(Align::Center)
            .build();

        let on_dsp = on_dsp_change.clone();
        eq_switch.connect_active_notify(move |s| {
            on_dsp("eq_enabled", if s.is_active() { 1.0 } else { 0.0 });
        });

        let eq_row = ActionRow::builder()
            .title("Enable Equalizer")
            .activatable_widget(&eq_switch)
            .build();
        eq_row.add_suffix(&eq_switch);
        eq_group.add(&eq_row);

        // Low Gain
        let low_scale = Scale::with_range(Orientation::Horizontal, -12.0, 12.0, 0.5);
        low_scale.set_value(state_guard.equalizer.low_gain_db);
        low_scale.set_hexpand(true);
        let on_dsp = on_dsp_change.clone();
        low_scale.connect_value_changed(move |s| {
            on_dsp("eq_low", s.value());
        });
        let low_row = ActionRow::builder().title("Low Gain (80 Hz)").build();
        low_row.add_suffix(&low_scale);
        eq_group.add(&low_row);

        // Mid Gain
        let mid_scale = Scale::with_range(Orientation::Horizontal, -12.0, 12.0, 0.5);
        mid_scale.set_value(state_guard.equalizer.mid_gain_db);
        mid_scale.set_hexpand(true);
        let on_dsp = on_dsp_change.clone();
        mid_scale.connect_value_changed(move |s| {
            on_dsp("eq_mid", s.value());
        });
        let mid_row = ActionRow::builder().title("Mid Gain (2.5 kHz)").build();
        mid_row.add_suffix(&mid_scale);
        eq_group.add(&mid_row);

        // High Gain
        let high_scale = Scale::with_range(Orientation::Horizontal, -12.0, 12.0, 0.5);
        high_scale.set_value(state_guard.equalizer.high_gain_db);
        high_scale.set_hexpand(true);
        let on_dsp = on_dsp_change.clone();
        high_scale.connect_value_changed(move |s| {
            on_dsp("eq_high", s.value());
        });
        let high_row = ActionRow::builder().title("High Gain (10 kHz)").build();
        high_row.add_suffix(&high_scale);
        eq_group.add(&high_row);

        page.add(&eq_group);

        let scrolled = ScrolledWindow::builder()
            .hexpand(true)
            .vexpand(true)
            .child(&page)
            .build();

        Self { container: scrolled }
    }
}
