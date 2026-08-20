//! Mixer Console View for M-Game Solo.
//! 5 responsive channel strips with unified PreferencesGroup header and Reset button.

use gtk4::prelude::*;
use gtk4::{Box, Button, Orientation, PolicyType, ScrolledWindow};
use libadwaita::prelude::*;
use libadwaita::PreferencesGroup;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use super::channel_strip::ChannelStrip;
use crate::state::MixerState;

pub struct MixerView {
    pub container: ScrolledWindow,
    pub strips: HashMap<String, ChannelStrip>,
}

impl MixerView {
    pub fn new<FVolume, FMute>(
        state: Arc<RwLock<MixerState>>,
        on_volume: FVolume,
        on_mute: FMute,
    ) -> Self
    where
        FVolume: Fn(&str, u8) + Clone + 'static,
        FMute: Fn(&str, bool) + Clone + 'static,
    {
        let scrolled = ScrolledWindow::builder()
            .hscrollbar_policy(PolicyType::Never)
            .vscrollbar_policy(PolicyType::Automatic)
            .hexpand(true)
            .vexpand(true)
            .build();

        let root_box = Box::new(Orientation::Vertical, 16);
        root_box.set_margin_top(16);
        root_box.set_margin_bottom(24);
        root_box.set_margin_start(16);
        root_box.set_margin_end(16);

        let group = PreferencesGroup::builder()
            .title("Mixer Console")
            .description("Control hardware faders, software levels, and mute states.")
            .build();

        let reset_btn = Button::builder()
            .label("Reset to 0 dB")
            .icon_name("view-refresh-symbolic")
            .css_classes(["flat", "pill"])
            .tooltip_text("Reset all faders to unity gain (0.0 dB)")
            .build();

        group.set_header_suffix(Some(&reset_btn));

        // 5 Fader Strips Flex Box
        let flex_box = Box::new(Orientation::Horizontal, 12);
        flex_box.set_homogeneous(true);
        flex_box.set_hexpand(true);
        flex_box.set_vexpand(true);
        flex_box.set_margin_top(12);

        let channels = [
            ("microphone", "Mic In", "audio-input-microphone-symbolic"),
            ("game", "Game", "input-gaming-symbolic"),
            ("chat", "Chat", "audio-headset-symbolic"),
            ("sampler", "Sampler", "media-playback-start-symbolic"),
            ("system", "System", "video-display-symbolic"),
        ];

        let state_guard = state.read().unwrap();
        let mut strips = HashMap::new();

        for (id, name, icon) in channels {
            let initial_val = *state_guard.faders.get(id).unwrap_or(&100);
            let initial_mute = *state_guard.mutes.get(id).unwrap_or(&false);

            let on_vol = on_volume.clone();
            let on_mut = on_mute.clone();

            let strip = ChannelStrip::new(
                id,
                name,
                icon,
                initial_val,
                initial_mute,
                on_vol,
                on_mut,
            );

            flex_box.append(&strip.container);
            strips.insert(id.to_string(), strip);
        }

        group.add(&flex_box);
        root_box.append(&group);
        scrolled.set_child(Some(&root_box));

        // Connect Reset Button
        let strips_clone = strips.clone();
        let on_vol_all = on_volume.clone();
        reset_btn.connect_clicked(move |_| {
            for (ch_id, strip) in &strips_clone {
                strip.set_value(100);
                on_vol_all(ch_id, 100);
            }
        });

        Self {
            container: scrolled,
            strips,
        }
    }
}
