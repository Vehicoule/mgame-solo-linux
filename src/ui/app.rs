//! Application Lifecycle & Hardware Event Dispatcher for M-Game Solo.
//! Full bi-directional real-time sync with PipeWire system sound settings and hardware SysEx.

use gtk4::gdk::Display;
use gtk4::prelude::*;
use gtk4::CssProvider;
use libadwaita::Application;
use std::rc::Rc;
use std::sync::{Arc, RwLock};
use std::time::Duration;
use log::{info, warn};

use super::window::MainWindow;
use crate::audio::pipewire::{
    ensure_pipewire_configuration, set_channel_mute, set_channel_volume, spawn_pipewire_event_listener,
};
use crate::midi::protocol::{
    build_dsp_msg, build_headphone_mode_msg, build_led_theme_msg, build_mute_msg,
    build_patch_msg, build_phantom_power_msg, build_volume_msg, build_voice_fx_msg,
    MsgType, Sink, Source,
};
use crate::midi::rawmidi::{MidiClient, MidiEvent};
use crate::state::MixerState;

const STYLE_CSS: &str = include_str!("style.css");

pub struct MGameApp {
    pub app: Application,
}

impl MGameApp {
    pub fn new() -> Self {
        let app = Application::builder()
            .application_id("com.mgame.Solo")
            .build();

        app.connect_startup(|_| {
            let _ = ensure_pipewire_configuration();

            // Load global CSS styling
            let provider = CssProvider::new();
            provider.load_from_string(STYLE_CSS);
            if let Some(display) = Display::default() {
                gtk4::style_context_add_provider_for_display(
                    &display,
                    &provider,
                    gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
                );
            }
        });

        app.connect_activate(Self::build_ui);

        Self { app }
    }

    fn build_ui(app: &Application) {
        let state = Arc::new(RwLock::new(MixerState::load_or_default()));

        let (async_tx, async_rx) = async_channel::unbounded::<MidiEvent>();
        let (pw_tx, pw_rx) = async_channel::unbounded::<(String, u8, bool)>();

        let async_tx_clone = async_tx.clone();
        let midi = Arc::new(MidiClient::new(move |evt| {
            let _ = async_tx_clone.send_blocking(evt);
        }));

        // Spawn PipeWire real-time subscriber (GNOME Settings / Keyboard Hotkeys / pavucontrol)
        let pw_tx_clone = pw_tx.clone();
        spawn_pipewire_event_listener(move |ch, vol, muted| {
            let _ = pw_tx_clone.send_blocking((ch, vol, muted));
        });

        let main_window = Rc::new(MainWindow::new(app, Arc::clone(&state), Arc::clone(&midi)));
        main_window.window.present();

        // 100ms Active UI Loop Watcher tracking direct USB / ALSA presence
        let window_poll = Rc::downgrade(&main_window);
        glib::timeout_add_local(Duration::from_millis(100), move || {
            if let Some(win) = window_poll.upgrade() {
                let present = MidiClient::is_device_present();
                win.set_hardware_connected(present);
                glib::ControlFlow::Continue
            } else {
                glib::ControlFlow::Break
            }
        });

        // 1. PipeWire System Events -> UI & Hardware Sync Loop
        let window_pw = Rc::downgrade(&main_window);
        let state_pw = Arc::clone(&state);
        let midi_pw = Arc::clone(&midi);
        glib::spawn_future_local(async move {
            while let Ok((ch_name, vol, muted)) = pw_rx.recv().await {
                if let Some(win) = window_pw.upgrade() {
                    // Update state
                    if let Ok(mut st) = state_pw.write() {
                        st.faders.insert(ch_name.clone(), vol);
                        st.mutes.insert(ch_name.clone(), muted);
                    }

                    // Update UI Strip
                    if let Some(strip) = win.mixer_view.borrow().strips.get(&ch_name) {
                        strip.set_value(vol);
                        strip.set_muted(muted);
                    }

                    // Push to Hardware LEDs & DSP
                    let src = match ch_name.as_str() {
                        "microphone" => Some(Source::Microphone),
                        "game" => Some(Source::Game),
                        "chat" => Some(Source::Chat),
                        "sampler" => Some(Source::Sampler),
                        "system" => Some(Source::System),
                        _ => None,
                    };
                    if let Some(s) = src {
                        for snk in [Sink::PhonesOut, Sink::MainOut, Sink::Stream] {
                            midi_pw.send(build_volume_msg(s, snk, vol));
                        }
                        midi_pw.send(build_mute_msg(s, muted));
                    }
                }
            }
        });

        // 2. Hardware MIDI Events -> PipeWire & UI Sync Loop
        let window_weak = Rc::downgrade(&main_window);
        let state_midi = Arc::clone(&state);
        let midi_init = Arc::clone(&midi);

        glib::spawn_future_local(async move {
            while let Ok(evt) = async_rx.recv().await {
                if let Some(win) = window_weak.upgrade() {
                    match evt {
                        MidiEvent::Connected(dev) => {
                            info!("Hardware connected at {}", dev);
                            win.set_hardware_connected(true);

                            let state_guard = state_midi.read().unwrap();
                            
                            // 1. Initial fader volumes & patches
                            for (ch_name, &val) in &state_guard.faders {
                                let src = match ch_name.as_str() {
                                    "microphone" => Source::Microphone,
                                    "game" => Source::Game,
                                    "chat" => Source::Chat,
                                    "sampler" => Source::Sampler,
                                    "system" => Source::System,
                                    _ => continue,
                                };
                                for snk in [Sink::PhonesOut, Sink::MainOut, Sink::Stream] {
                                    midi_init.send(build_patch_msg(src, snk, true));
                                    midi_init.send(build_volume_msg(src, snk, val));
                                }
                                set_channel_volume(ch_name, val);
                            }
                            
                            // 2. Initial mutes
                            for (ch_name, &muted) in &state_guard.mutes {
                                set_channel_mute(ch_name, muted);
                            }

                            // 3. Initial DSP Settings
                            midi_init.send(build_dsp_msg(0x10, if state_guard.equalizer.hpf_enabled { 1 } else { 0 }));
                            midi_init.send(build_dsp_msg(0x20, if state_guard.compressor.enabled { 1 } else { 0 }));
                            let comp_v = ((state_guard.compressor.threshold_db + 40.0) / 40.0 * 127.0) as u8;
                            midi_init.send(build_dsp_msg(0x21, comp_v));
                            let comp_r = ((state_guard.compressor.ratio - 1.0) / 19.0 * 127.0) as u8;
                            midi_init.send(build_dsp_msg(0x22, comp_r));
                            midi_init.send(build_dsp_msg(0x30, if state_guard.noise_gate.enabled { 1 } else { 0 }));
                            let gate_v = ((state_guard.noise_gate.threshold_db + 60.0) / 60.0 * 127.0) as u8;
                            midi_init.send(build_dsp_msg(0x31, gate_v));
                            midi_init.send(build_dsp_msg(0x40, if state_guard.equalizer.enabled { 1 } else { 0 }));
                            let eq_l = ((state_guard.equalizer.low_gain_db + 12.0) / 24.0 * 127.0) as u8;
                            midi_init.send(build_dsp_msg(0x41, eq_l));
                            let eq_m = ((state_guard.equalizer.mid_gain_db + 12.0) / 24.0 * 127.0) as u8;
                            midi_init.send(build_dsp_msg(0x42, eq_m));
                            let eq_h = ((state_guard.equalizer.high_gain_db + 12.0) / 24.0 * 127.0) as u8;
                            midi_init.send(build_dsp_msg(0x43, eq_h));

                            // 4. Initial Voice FX
                            midi_init.send(build_voice_fx_msg(0x00, if state_guard.voice_fx.enabled { 1 } else { 0 }));
                            let pitch_v = ((state_guard.voice_fx.pitch_semitones as f64 + 12.0) / 24.0 * 127.0) as u8;
                            midi_init.send(build_voice_fx_msg(0x01, pitch_v));
                            let formant_v = ((state_guard.voice_fx.formant_semitones as f64 + 12.0) / 24.0 * 127.0) as u8;
                            midi_init.send(build_voice_fx_msg(0x02, formant_v));
                            midi_init.send(build_voice_fx_msg(0x03, state_guard.voice_fx.reverb_preset));
                            let rev_m = (state_guard.voice_fx.reverb_mix_percent as f64 / 100.0 * 127.0) as u8;
                            midi_init.send(build_voice_fx_msg(0x04, rev_m));

                            // 5. Initial Hardware & LEDs
                            midi_init.send(build_phantom_power_msg(state_guard.phantom_power));
                            midi_init.send(build_headphone_mode_msg(state_guard.headphone_high_impedance));
                            let theme_idx = match state_guard.led_theme.as_str() {
                                "Cyberpunk Purple" => 1,
                                "Crimson Red" => 2,
                                "Emerald Green" => 3,
                                "Sunset Amber" => 4,
                                "Monochrome White" => 5,
                                _ => 0,
                            };
                            midi_init.send(build_led_theme_msg(theme_idx));
                        }
                        MidiEvent::Disconnected => {
                            warn!("Hardware disconnected");
                            win.set_hardware_connected(false);
                        }
                        MidiEvent::Message(msg) => {
                            // Strict filtering: Ignore hardware error replies (status 0x7F)
                            if msg.success != 0x00 {
                                continue;
                            }

                            // Hardware Censor Button (!) pressed
                            if msg.source == Source::Button6 {
                                if let Ok(st) = state_midi.read() {
                                    if st.censor_mode == "Mute Stream" {
                                        set_channel_mute("microphone", true);
                                    }
                                }
                                continue;
                            }

                            let ch_name_opt = match msg.source {
                                Source::Microphone | Source::Button1 => Some("microphone"),
                                Source::Game | Source::Button2 => Some("game"),
                                Source::Chat | Source::Button3 => Some("chat"),
                                Source::Sampler | Source::Button4 => Some("sampler"),
                                Source::System | Source::Button5 => Some("system"),
                                _ => None,
                            };

                            if msg.msg_type == MsgType::Volume {
                                if let Some(ch_name) = ch_name_opt {
                                    if msg.payload.len() >= 2 {
                                        let vol = msg.payload[1];
                                        if let Ok(mut st) = state_midi.write() {
                                            st.faders.insert(ch_name.to_string(), vol);
                                        }
                                        set_channel_volume(ch_name, vol);
                                        if let Some(strip) = win.mixer_view.borrow().strips.get(ch_name) {
                                            strip.set_value(vol);
                                        }
                                    }
                                }
                            } else if msg.msg_type == MsgType::Button {
                                // Only accept primary destination mutes (Stream=0x00, Phones=0x05)
                                if msg.sink != Sink::Stream && msg.sink != Sink::PhonesOut {
                                    continue;
                                }

                                if let Some(ch_name) = ch_name_opt {
                                    if !msg.payload.is_empty() {
                                        let muted = if msg.payload[0] == 0x44 && msg.payload.len() >= 2 {
                                            msg.payload[1] != 0
                                        } else if msg.payload.len() >= 2 && (msg.payload[1] == 0 || msg.payload[1] == 1) {
                                            msg.payload[1] != 0
                                        } else {
                                            false
                                        };

                                        if let Ok(mut st) = state_midi.write() {
                                            st.mutes.insert(ch_name.to_string(), muted);
                                        }
                                        set_channel_mute(ch_name, muted);
                                        if let Some(strip) = win.mixer_view.borrow().strips.get(ch_name) {
                                            strip.set_muted(muted);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        });
    }

    pub fn run(&self) -> glib::ExitCode {
        self.app.run()
    }
}
