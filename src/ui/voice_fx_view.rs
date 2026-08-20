//! Voice Effects View for M-Game Solo.
//! Real-time Pitch Shifter, Formant Shifter (Vocal Tract), and Reverb Processor.

use gtk4::prelude::*;
use gtk4::{Align, DropDown, Orientation, Scale, ScrolledWindow, StringList, Switch};
use libadwaita::prelude::*;
use libadwaita::{ActionRow, PreferencesGroup, PreferencesPage};
use std::sync::{Arc, RwLock};

use crate::state::MixerState;

pub struct VoiceFxView {
    pub container: ScrolledWindow,
}

impl VoiceFxView {
    pub fn new<FFx>(state: Arc<RwLock<MixerState>>, on_fx_change: FFx) -> Self
    where
        FFx: Fn(&str, f64) + Clone + 'static,
    {
        let page = PreferencesPage::new();
        let state_guard = state.read().unwrap();

        let fx_group = PreferencesGroup::builder()
            .title("Real-Time Voice Transformer")
            .description("Hardware DSP pitch shifting and formant warping for creative vocal effects.")
            .build();

        let fx_switch = Switch::builder()
            .active(state_guard.voice_fx.enabled)
            .valign(Align::Center)
            .build();

        let on_fx = on_fx_change.clone();
        fx_switch.connect_active_notify(move |s| {
            on_fx("voice_fx_enabled", if s.is_active() { 1.0 } else { 0.0 });
        });

        let fx_row = ActionRow::builder()
            .title("Enable Voice Effects Engine")
            .activatable_widget(&fx_switch)
            .build();
        fx_row.add_suffix(&fx_switch);
        fx_group.add(&fx_row);

        // Pitch Shift Scale (-12 .. +12 semitones)
        let pitch_scale = Scale::with_range(Orientation::Horizontal, -12.0, 12.0, 1.0);
        pitch_scale.set_value(state_guard.voice_fx.pitch_semitones as f64);
        pitch_scale.set_hexpand(true);
        let on_fx = on_fx_change.clone();
        pitch_scale.connect_value_changed(move |s| {
            on_fx("pitch_semitones", s.value());
        });
        let pitch_row = ActionRow::builder()
            .title("Pitch Shift")
            .subtitle("Raise or lower vocal pitch (-12 to +12 semitones)")
            .build();
        pitch_row.add_suffix(&pitch_scale);
        fx_group.add(&pitch_row);

        // Formant Shift Scale (-12 .. +12 semitones)
        let formant_scale = Scale::with_range(Orientation::Horizontal, -12.0, 12.0, 1.0);
        formant_scale.set_value(state_guard.voice_fx.formant_semitones as f64);
        formant_scale.set_hexpand(true);
        let on_fx = on_fx_change.clone();
        formant_scale.connect_value_changed(move |s| {
            on_fx("formant_semitones", s.value());
        });
        let formant_row = ActionRow::builder()
            .title("Formant Shift")
            .subtitle("Alter vocal tract size / gender character (-12 to +12 semitones)")
            .build();
        formant_row.add_suffix(&formant_scale);
        fx_group.add(&formant_row);

        page.add(&fx_group);

        // Reverb Group
        let rev_group = PreferencesGroup::builder()
            .title("Reverb Engine")
            .description("Add acoustic space and dimension to your voice.")
            .build();

        let presets = StringList::new(&["Off", "Small Room", "Large Hall", "Space / Shimmer"]);
        let rev_drop = DropDown::builder()
            .model(&presets)
            .selected(state_guard.voice_fx.reverb_preset as u32)
            .valign(Align::Center)
            .build();

        let on_fx = on_fx_change.clone();
        rev_drop.connect_selected_notify(move |d| {
            on_fx("reverb_preset", d.selected() as f64);
        });

        let rev_preset_row = ActionRow::builder()
            .title("Reverb Preset")
            .activatable_widget(&rev_drop)
            .build();
        rev_preset_row.add_suffix(&rev_drop);
        rev_group.add(&rev_preset_row);

        let mix_scale = Scale::with_range(Orientation::Horizontal, 0.0, 100.0, 1.0);
        mix_scale.set_value(state_guard.voice_fx.reverb_mix_percent as f64);
        mix_scale.set_hexpand(true);
        let on_fx = on_fx_change.clone();
        mix_scale.connect_value_changed(move |s| {
            on_fx("reverb_mix", s.value());
        });
        let mix_row = ActionRow::builder()
            .title("Wet / Dry Mix")
            .subtitle("Reverb effect intensity (0% to 100%)")
            .build();
        mix_row.add_suffix(&mix_scale);
        rev_group.add(&mix_row);

        page.add(&rev_group);

        let scrolled = ScrolledWindow::builder()
            .hexpand(true)
            .vexpand(true)
            .child(&page)
            .build();

        Self { container: scrolled }
    }
}
