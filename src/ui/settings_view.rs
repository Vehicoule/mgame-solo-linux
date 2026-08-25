//! Hardware Settings & LED Customization View for M-Game Solo.
//! 48V Phantom Power, High-Impedance Headphone Mode, and LED Color Scheme controls.

use gtk4::{Align, DropDown, ScrolledWindow, StringList, Switch};
use libadwaita::prelude::*;
use libadwaita::{ActionRow, PreferencesGroup, PreferencesPage};
use std::sync::{Arc, RwLock};

use crate::state::MixerState;

pub struct SettingsView {
    pub container: ScrolledWindow,
}

impl SettingsView {
    pub fn new<FPhantom, FHeadphones, FTheme>(
        state: Arc<RwLock<MixerState>>,
        on_phantom: FPhantom,
        on_headphones: FHeadphones,
        on_theme: FTheme,
    ) -> Self
    where
        FPhantom: Fn(bool) + Clone + 'static,
        FHeadphones: Fn(bool) + Clone + 'static,
        FTheme: Fn(&str) + Clone + 'static,
    {
        let page = PreferencesPage::new();
        let state_guard = state.read().unwrap();

        // 1. Microphone Hardware Power
        let mic_group = PreferencesGroup::builder()
            .title("Hardware Preamplifier")
            .description("Configure hardware microphone inputs and power circuitry.")
            .build();

        let phantom_switch = Switch::builder()
            .active(state_guard.phantom_power)
            .valign(Align::Center)
            .build();

        let on_ph = on_phantom.clone();
        phantom_switch.connect_active_notify(move |s| {
            on_ph(s.is_active());
        });

        let phantom_row = ActionRow::builder()
            .title("+48V Phantom Power")
            .subtitle("Required for XLR condenser studio microphones (e.g. Rode NT1, AT2020). Keep OFF for dynamic mics.")
            .activatable_widget(&phantom_switch)
            .build();
        phantom_row.add_suffix(&phantom_switch);
        mic_group.add(&phantom_row);
        page.add(&mic_group);

        // 2. Headphone Output Amplifier
        let hp_group = PreferencesGroup::builder()
            .title("Headphone Amplifier")
            .description("High-current headphone amp gain stage configuration.")
            .build();

        let hp_switch = Switch::builder()
            .active(state_guard.headphone_high_impedance)
            .valign(Align::Center)
            .build();

        let on_hp = on_headphones.clone();
        hp_switch.connect_active_notify(move |s| {
            on_hp(s.is_active());
        });

        let hp_row = ActionRow::builder()
            .title("High-Impedance Boost Mode")
            .subtitle("Provides additional output voltage for 250Ω / 600Ω studio headphones (e.g. DT 990 Pro, HD 650)")
            .activatable_widget(&hp_switch)
            .build();
        hp_row.add_suffix(&hp_switch);
        hp_group.add(&hp_row);
        page.add(&hp_group);

        // 3. LED Lighting Themes
        let led_group = PreferencesGroup::builder()
            .title("Hardware Lighting & Themes")
            .description("Select color styling for the fader LEDs, meter rings, and button backlights.")
            .build();

        let themes = StringList::new(&[
            "Neon Blue",
            "Cyberpunk Purple",
            "Crimson Red",
            "Emerald Green",
            "Sunset Amber",
            "Monochrome White",
        ]);

        let current_theme_index = match state_guard.led_theme.as_str() {
            "Cyberpunk Purple" => 1,
            "Crimson Red" => 2,
            "Emerald Green" => 3,
            "Sunset Amber" => 4,
            "Monochrome White" => 5,
            _ => 0,
        };

        let theme_drop = DropDown::builder()
            .model(&themes)
            .selected(current_theme_index)
            .valign(Align::Center)
            .build();

        let on_th = on_theme.clone();
        theme_drop.connect_selected_notify(move |d| {
            let th_str = match d.selected() {
                1 => "Cyberpunk Purple",
                2 => "Crimson Red",
                3 => "Emerald Green",
                4 => "Sunset Amber",
                5 => "Monochrome White",
                _ => "Neon Blue",
            };
            on_th(th_str);
        });

        let theme_row = ActionRow::builder()
            .title("LED Lighting Preset")
            .subtitle("Syncs immediately to hardware LED controllers")
            .activatable_widget(&theme_drop)
            .build();
        theme_row.add_suffix(&theme_drop);
        led_group.add(&theme_row);
        page.add(&led_group);

        let scrolled = ScrolledWindow::builder()
            .hexpand(true)
            .vexpand(true)
            .child(&page)
            .build();

        Self { container: scrolled }
    }
}
