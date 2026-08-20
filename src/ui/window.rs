//! Main Application Window for M-Game Solo.
//! Uses native Adw.NavigationSplitView, Adw.ToolbarView, Adw.Banner, and Adw.ToastOverlay for 100% unified GNOME HIG layout.

use gtk4::prelude::*;
use gtk4::{Align, Box, HeaderBar, Label, ListBox, ListBoxRow, Orientation, Stack, MenuButton};
use libadwaita::prelude::*;
use libadwaita::{
    ApplicationWindow, Banner, Breakpoint, NavigationPage, NavigationSplitView, ToastOverlay,
    ToolbarView, WindowTitle,
};
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, RwLock};

use super::censor_view::CensorView;
use super::dsp_view::DspView;
use super::matrix_view::MatrixView;
use super::mixer_view::MixerView;
use super::settings_view::SettingsView;
use super::voice_fx_view::VoiceFxView;
use crate::audio::pipewire::{set_channel_mute, set_channel_volume};
use crate::midi::protocol::{
    build_dsp_msg, build_headphone_mode_msg, build_led_theme_msg, build_mute_msg,
    build_patch_msg, build_phantom_power_msg, build_volume_msg, build_voice_fx_msg,
    Sink, Source,
};
use crate::midi::rawmidi::MidiClient;
use crate::state::MixerState;

pub struct MainWindow {
    pub window: ApplicationWindow,
    pub mixer_view: Rc<RefCell<MixerView>>,
    _toast_overlay: ToastOverlay,
    pub dot_label: Label,
    pub text_label: Label,
    pub banner: Banner,
}

impl MainWindow {
    pub fn new(app: &libadwaita::Application, state: Arc<RwLock<MixerState>>, midi: Arc<MidiClient>) -> Self {
        let split_view = NavigationSplitView::new();
        split_view.set_min_sidebar_width(220.0);
        split_view.set_max_sidebar_width(260.0);

        let is_present = MidiClient::is_device_present();

        // =========================================================================
        // 1. SIDEBAR (Unified ToolbarView + Custom Dot/Status Box)
        // =========================================================================
        let sidebar_toolbar = ToolbarView::new();

        let title_box = Box::new(Orientation::Vertical, 2);
        title_box.set_valign(Align::Center);
        title_box.set_halign(Align::Center);

        let app_title_lbl = Label::builder()
            .label("M-Game Solo")
            .css_classes(["heading"])
            .halign(Align::Center)
            .build();
        title_box.append(&app_title_lbl);

        let status_box = Box::new(Orientation::Horizontal, 0);
        status_box.set_halign(Align::Center);
        status_box.set_valign(Align::Center);

        let dot_classes: &[&str] = if is_present {
            &["status-dot", "connected"]
        } else {
            &["status-dot", "disconnected"]
        };

        let dot_label = Label::builder()
            .label("●")
            .css_classes(dot_classes)
            .build();
        status_box.append(&dot_label);

        let text_label = Label::builder()
            .label(if is_present { "Connected" } else { "Disconnected" })
            .css_classes(["status-text"])
            .build();
        status_box.append(&text_label);

        title_box.append(&status_box);

        let sidebar_header = HeaderBar::builder()
            .title_widget(&title_box)
            .show_title_buttons(false)
            .build();

        let menu = gio::Menu::new();
        menu.append(Some("About M-Game Solo"), Some("app.about"));
        menu.append(Some("Quit"), Some("app.quit"));

        let menu_btn = MenuButton::builder()
            .icon_name("open-menu-symbolic")
            .menu_model(&menu)
            .build();
        sidebar_header.pack_end(&menu_btn);

        sidebar_toolbar.add_top_bar(&sidebar_header);

        let sidebar_list = ListBox::builder()
            .css_classes(["navigation-sidebar"])
            .margin_top(6)
            .margin_bottom(8)
            .margin_start(6)
            .margin_end(6)
            .selection_mode(gtk4::SelectionMode::Single)
            .build();

        // Section 1: Mixer
        sidebar_list.append(&Self::create_header_row("Mixer"));
        sidebar_list.append(&Self::create_nav_row("mixer", "Mixer Console", "audio-volume-high-symbolic"));
        sidebar_list.append(&Self::create_nav_row("routing", "Audio Routing", "network-transmit-receive-symbolic"));

        // Section 2: Processing
        sidebar_list.append(&Self::create_header_row("Processing"));
        sidebar_list.append(&Self::create_nav_row("dsp", "Mic Processing", "audio-input-microphone-symbolic"));
        sidebar_list.append(&Self::create_nav_row("voice_fx", "Voice Effects", "audio-speakers-symbolic"));
        sidebar_list.append(&Self::create_nav_row("censor", "Censor & Panic", "dialog-warning-symbolic"));

        // Section 3: Device
        sidebar_list.append(&Self::create_header_row("Device"));
        sidebar_list.append(&Self::create_nav_row("settings", "Hardware & LEDs", "emblem-system-symbolic"));

        sidebar_toolbar.set_content(Some(&sidebar_list));

        let sidebar_page = NavigationPage::new(&sidebar_toolbar, "Navigation");
        split_view.set_sidebar(Some(&sidebar_page));

        // =========================================================================
        // 2. MAIN CONTENT (Unified ToolbarView + Banner)
        // =========================================================================
        let content_toolbar = ToolbarView::new();

        let title_widget = WindowTitle::new("Mixer Console", "");

        let content_header = HeaderBar::builder()
            .title_widget(&title_widget)
            .show_title_buttons(true)
            .build();

        content_toolbar.add_top_bar(&content_header);

        // Hardware Status Banner
        let banner = Banner::builder()
            .title("M-Game Solo is disconnected. Connect USB cable to enable hardware control.")
            .revealed(!is_present)
            .build();
        content_toolbar.add_top_bar(&banner);

        // Content Stack
        let content_stack = Stack::new();
        content_stack.set_transition_type(gtk4::StackTransitionType::Crossfade);

        // 1. Mixer View
        let midi_vol = Arc::clone(&midi);
        let state_vol = Arc::clone(&state);
        let on_volume = move |channel: &str, val: u8| {
            if let Ok(mut st) = state_vol.write() {
                st.faders.insert(channel.to_string(), val);
                st.save();
            }
            set_channel_volume(channel, val);

            let src = match channel {
                "microphone" => Some(Source::Microphone),
                "game" => Some(Source::Game),
                "chat" => Some(Source::Chat),
                "sampler" => Some(Source::Sampler),
                "system" => Some(Source::System),
                _ => None,
            };
            if let Some(s) = src {
                for snk in [Sink::PhonesOut, Sink::MainOut, Sink::Stream] {
                    midi_vol.send(build_volume_msg(s, snk, val));
                }
            }
        };

        let midi_mut = Arc::clone(&midi);
        let state_mut = Arc::clone(&state);
        let on_mute = move |channel: &str, muted: bool| {
            if let Ok(mut st) = state_mut.write() {
                st.mutes.insert(channel.to_string(), muted);
                st.save();
            }
            set_channel_mute(channel, muted);

            let src = match channel {
                "microphone" => Some(Source::Microphone),
                "game" => Some(Source::Game),
                "chat" => Some(Source::Chat),
                "sampler" => Some(Source::Sampler),
                "system" => Some(Source::System),
                _ => None,
            };
            if let Some(s) = src {
                midi_mut.send(build_mute_msg(s, muted));
            }
        };

        let mixer_view = MixerView::new(Arc::clone(&state), on_volume, on_mute);
        let mixer_view_rc = Rc::new(RefCell::new(mixer_view));
        content_stack.add_named(&mixer_view_rc.borrow().container, Some("mixer"));

        // 2. Matrix View
        let midi_rout = Arc::clone(&midi);
        let state_rout = Arc::clone(&state);
        let on_routing = move |src_name: &str, snk_name: &str, enabled: bool| {
            let key = format!("{}->{}", src_name, snk_name);
            if let Ok(mut st) = state_rout.write() {
                st.routing.insert(key, enabled);
                st.save();
            }
            let src_enum = match src_name {
                "microphone" => Some(Source::Microphone),
                "game" => Some(Source::Game),
                "chat" => Some(Source::Chat),
                "sampler" => Some(Source::Sampler),
                "system" => Some(Source::System),
                "aux" => Some(Source::Aux),
                _ => None,
            };
            let snk_enum = match snk_name {
                "stream" => Some(Sink::Stream),
                "chat" => Some(Sink::Chat),
                "main_out" => Some(Sink::MainOut),
                "phones_out" => Some(Sink::PhonesOut),
                _ => None,
            };
            if let (Some(s), Some(k)) = (src_enum, snk_enum) {
                midi_rout.send(build_patch_msg(s, k, enabled));
            }
        };
        let matrix_view = MatrixView::new(Arc::clone(&state), on_routing);
        content_stack.add_named(&matrix_view.container, Some("routing"));

        // 3. DSP View
        let state_dsp = Arc::clone(&state);
        let midi_dsp = Arc::clone(&midi);
        let on_dsp = move |param: &str, val: f64| {
            if let Ok(mut st) = state_dsp.write() {
                match param {
                    "hpf_enabled" => {
                        st.equalizer.hpf_enabled = val > 0.5;
                        midi_dsp.send(build_dsp_msg(0x10, if val > 0.5 { 1 } else { 0 }));
                    }
                    "comp_enabled" => {
                        st.compressor.enabled = val > 0.5;
                        midi_dsp.send(build_dsp_msg(0x20, if val > 0.5 { 1 } else { 0 }));
                    }
                    "comp_threshold" => {
                        st.compressor.threshold_db = val;
                        let v = ((val + 40.0) / 40.0 * 127.0) as u8;
                        midi_dsp.send(build_dsp_msg(0x21, v));
                    }
                    "comp_ratio" => {
                        st.compressor.ratio = val;
                        let v = ((val - 1.0) / 19.0 * 127.0) as u8;
                        midi_dsp.send(build_dsp_msg(0x22, v));
                    }
                    "gate_enabled" => {
                        st.noise_gate.enabled = val > 0.5;
                        midi_dsp.send(build_dsp_msg(0x30, if val > 0.5 { 1 } else { 0 }));
                    }
                    "gate_threshold" => {
                        st.noise_gate.threshold_db = val;
                        let v = ((val + 60.0) / 60.0 * 127.0) as u8;
                        midi_dsp.send(build_dsp_msg(0x31, v));
                    }
                    "eq_enabled" => {
                        st.equalizer.enabled = val > 0.5;
                        midi_dsp.send(build_dsp_msg(0x40, if val > 0.5 { 1 } else { 0 }));
                    }
                    "eq_low" => {
                        st.equalizer.low_gain_db = val;
                        let v = ((val + 12.0) / 24.0 * 127.0) as u8;
                        midi_dsp.send(build_dsp_msg(0x41, v));
                    }
                    "eq_mid" => {
                        st.equalizer.mid_gain_db = val;
                        let v = ((val + 12.0) / 24.0 * 127.0) as u8;
                        midi_dsp.send(build_dsp_msg(0x42, v));
                    }
                    "eq_high" => {
                        st.equalizer.high_gain_db = val;
                        let v = ((val + 12.0) / 24.0 * 127.0) as u8;
                        midi_dsp.send(build_dsp_msg(0x43, v));
                    }
                    _ => {}
                }
                st.save();
            }
        };
        let dsp_view = DspView::new(Arc::clone(&state), on_dsp);
        content_stack.add_named(&dsp_view.container, Some("dsp"));

        // 4. Voice FX View
        let midi_fx = Arc::clone(&midi);
        let state_fx = Arc::clone(&state);
        let on_fx = move |param: &str, val: f64| {
            if let Ok(mut st) = state_fx.write() {
                match param {
                    "voice_fx_enabled" => {
                        st.voice_fx.enabled = val > 0.5;
                        midi_fx.send(build_voice_fx_msg(0x00, if val > 0.5 { 1 } else { 0 }));
                    }
                    "pitch_semitones" => {
                        st.voice_fx.pitch_semitones = val as i8;
                        let raw_pitch = ((val + 12.0) / 24.0 * 127.0) as u8;
                        midi_fx.send(build_voice_fx_msg(0x01, raw_pitch));
                    }
                    "formant_semitones" => {
                        st.voice_fx.formant_semitones = val as i8;
                        let raw_formant = ((val + 12.0) / 24.0 * 127.0) as u8;
                        midi_fx.send(build_voice_fx_msg(0x02, raw_formant));
                    }
                    "reverb_preset" => {
                        st.voice_fx.reverb_preset = val as u8;
                        midi_fx.send(build_voice_fx_msg(0x03, val as u8));
                    }
                    "reverb_mix" => {
                        st.voice_fx.reverb_mix_percent = val as u8;
                        let raw_mix = (val / 100.0 * 127.0) as u8;
                        midi_fx.send(build_voice_fx_msg(0x04, raw_mix));
                    }
                    _ => {}
                }
                st.save();
            }
        };
        let voice_fx_view = VoiceFxView::new(Arc::clone(&state), on_fx);
        content_stack.add_named(&voice_fx_view.container, Some("voice_fx"));

        // 5. Censor View
        let midi_panic = Arc::clone(&midi);
        let state_censor = Arc::clone(&state);
        let on_censor_mode = move |mode: &str| {
            if let Ok(mut st) = state_censor.write() {
                st.censor_mode = mode.to_string();
                st.save();
            }
        };
        let on_panic_mute = move || {
            for src in [Source::Microphone, Source::Game, Source::Chat, Source::Sampler, Source::System] {
                midi_panic.send(build_mute_msg(src, true));
            }
            set_channel_mute("microphone", true);
            set_channel_mute("game", true);
            set_channel_mute("chat", true);
            set_channel_mute("sampler", true);
            set_channel_mute("system", true);
        };
        let censor_view = CensorView::new(Arc::clone(&state), on_censor_mode, on_panic_mute);
        content_stack.add_named(&censor_view.container, Some("censor"));

        // 6. Settings View
        let midi_ph = Arc::clone(&midi);
        let midi_hp = Arc::clone(&midi);
        let midi_th = Arc::clone(&midi);
        let state_set = Arc::clone(&state);
        let on_phantom = move |enabled: bool| {
            if let Ok(mut st) = state_set.write() {
                st.phantom_power = enabled;
                st.save();
            }
            midi_ph.send(build_phantom_power_msg(enabled));
        };
        let state_hp = Arc::clone(&state);
        let on_headphones = move |high_imp: bool| {
            if let Ok(mut st) = state_hp.write() {
                st.headphone_high_impedance = high_imp;
                st.save();
            }
            midi_hp.send(build_headphone_mode_msg(high_imp));
        };
        let state_th = Arc::clone(&state);
        let on_theme = move |theme: &str| {
            if let Ok(mut st) = state_th.write() {
                st.led_theme = theme.to_string();
                st.save();
            }
            let idx = match theme {
                "Cyberpunk Purple" => 1,
                "Crimson Red" => 2,
                "Emerald Green" => 3,
                "Sunset Amber" => 4,
                "Monochrome White" => 5,
                _ => 0, // Neon Blue
            };
            midi_th.send(build_led_theme_msg(idx));
        };
        let settings_view = SettingsView::new(Arc::clone(&state), on_phantom, on_headphones, on_theme);
        content_stack.add_named(&settings_view.container, Some("settings"));

        // Connect Sidebar Selection to Content Stack
        let stack_clone = content_stack.clone();
        let title_clone = title_widget.clone();
        sidebar_list.connect_row_selected(move |_, row_opt| {
            if let Some(row) = row_opt {
                let name = row.widget_name();
                if name.is_empty() || name == "header" {
                    return;
                }
                stack_clone.set_visible_child_name(&name);
                let title_text = match name.as_str() {
                    "routing" => "Audio Routing",
                    "dsp" => "Mic Processing",
                    "voice_fx" => "Voice Effects",
                    "censor" => "Censor & Panic",
                    "settings" => "Hardware & LEDs",
                    _ => "Mixer Console",
                };
                title_clone.set_title(title_text);
            }
        });

        // Select the first selectable row ("Mixer Console" at index 1) on startup
        if let Some(row) = sidebar_list.row_at_index(1) {
            sidebar_list.select_row(Some(&row));
        }

        content_toolbar.set_content(Some(&content_stack));

        let content_page = NavigationPage::new(&content_toolbar, "Content");
        split_view.set_content(Some(&content_page));

        let toast_overlay = ToastOverlay::new();
        toast_overlay.set_child(Some(&split_view));

        let window = ApplicationWindow::builder()
            .application(app)
            .title("M-Audio M-Game Solo")
            .default_width(1040)
            .default_height(640)
            .content(&toast_overlay)
            .build();

        let breakpoint = Breakpoint::new(libadwaita::BreakpointCondition::new_length(
            libadwaita::BreakpointConditionLengthType::MaxWidth,
            720.0,
            libadwaita::LengthUnit::Px,
        ));
        let val = true.to_value();
        breakpoint.add_setter(&split_view, "collapsed", Some(&val));
        window.add_breakpoint(breakpoint);

        Self {
            window,
            mixer_view: mixer_view_rc,
            _toast_overlay: toast_overlay,
            dot_label,
            text_label,
            banner,
        }
    }

    pub fn set_hardware_connected(&self, connected: bool) {
        if connected {
            self.dot_label.set_css_classes(&["status-dot", "connected"]);
            self.text_label.set_text("Connected");
            self.banner.set_revealed(false);
        } else {
            self.dot_label.set_css_classes(&["status-dot", "disconnected"]);
            self.text_label.set_text("Disconnected");
            self.banner.set_revealed(true);
        }
    }

    fn create_header_row(heading: &str) -> ListBoxRow {
        let row = ListBoxRow::new();
        row.set_widget_name("header");
        row.set_selectable(false);
        row.set_activatable(false);
        row.add_css_class("sidebar-header-row");

        let lbl = Label::builder()
            .label(heading)
            .css_classes(["sidebar-heading"])
            .halign(Align::Start)
            .build();

        row.set_child(Some(&lbl));
        row
    }

    fn create_nav_row(id: &str, label: &str, icon: &str) -> ListBoxRow {
        let row = ListBoxRow::new();
        row.add_css_class("sidebar-row");
        row.set_widget_name(id);

        let h_box = Box::new(Orientation::Horizontal, 12);
        h_box.set_margin_top(8);
        h_box.set_margin_bottom(8);
        h_box.set_margin_start(10);
        h_box.set_margin_end(10);

        let img = gtk4::Image::from_icon_name(icon);
        img.set_pixel_size(18);

        let lbl = Label::new(Some(label));
        lbl.set_halign(Align::Start);
        lbl.set_hexpand(true);

        h_box.append(&img);
        h_box.append(&lbl);
        row.set_child(Some(&h_box));
        row
    }
}
