//! Censor Button & Panic Mute View for M-Game Solo.
//! Configure the physical swear/censor button behavior and emergency panic mute.

use gtk4::prelude::*;
use gtk4::{Align, Button, DropDown, ScrolledWindow, StringList};
use libadwaita::prelude::*;
use libadwaita::{ActionRow, PreferencesGroup, PreferencesPage};
use std::sync::{Arc, RwLock};

use crate::state::MixerState;

pub struct CensorView {
    pub container: ScrolledWindow,
}

impl CensorView {
    pub fn new<FCensor, FPanic>(
        state: Arc<RwLock<MixerState>>,
        on_censor_mode: FCensor,
        on_panic_mute: FPanic,
    ) -> Self
    where
        FCensor: Fn(&str) + Clone + 'static,
        FPanic: Fn() + Clone + 'static,
    {
        let page = PreferencesPage::new();
        let state_guard = state.read().unwrap();

        let censor_group = PreferencesGroup::builder()
            .title("Hardware Censor Button (!)")
            .description("Configure what happens when the physical (!) button on your M-Game Solo is pressed.")
            .build();

        let modes = StringList::new(&["1 kHz Bleep Tone", "Mute Microphone to Stream", "Trigger Sampler Sound #1"]);
        let current_index = match state_guard.censor_mode.as_str() {
            "Mute Stream" => 1,
            "Sampler Sound" => 2,
            _ => 0,
        };

        let mode_drop = DropDown::builder()
            .model(&modes)
            .selected(current_index)
            .valign(Align::Center)
            .build();

        let on_cen = on_censor_mode.clone();
        mode_drop.connect_selected_notify(move |d| {
            let mode_str = match d.selected() {
                1 => "Mute Stream",
                2 => "Sampler Sound",
                _ => "1 kHz Bleep Tone",
            };
            on_cen(mode_str);
        });

        let mode_row = ActionRow::builder()
            .title("Censor Action")
            .subtitle("Real-time hardware action triggered by (!) button")
            .activatable_widget(&mode_drop)
            .build();
        mode_row.add_suffix(&mode_drop);
        censor_group.add(&mode_row);
        page.add(&censor_group);

        // Emergency Panic Mute
        let panic_group = PreferencesGroup::builder()
            .title("Emergency Controls")
            .description("Immediately cut all audio outputs in case of live broadcast emergencies.")
            .build();

        let panic_btn = Button::builder()
            .label("PANIC MUTE ALL")
            .css_classes(["destructive-action", "pill"])
            .valign(Align::Center)
            .build();

        panic_btn.connect_clicked(move |_| {
            on_panic_mute();
        });

        let panic_row = ActionRow::builder()
            .title("Panic Mute")
            .subtitle("Mutes all microphone, game, chat, sampler, and system outputs instantly")
            .activatable_widget(&panic_btn)
            .build();
        panic_row.add_suffix(&panic_btn);
        panic_group.add(&panic_row);
        page.add(&panic_group);

        let scrolled = ScrolledWindow::builder()
            .hexpand(true)
            .vexpand(true)
            .child(&page)
            .build();

        Self { container: scrolled }
    }
}
