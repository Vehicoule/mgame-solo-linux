//! Native Libadwaita Channel Strip Widget for M-Game Solo.
//! Bi-directionally synchronized with PipeWire system sound settings (0..150% with 5% stepped increments).

use gtk4::prelude::*;
use gtk4::{Align, Box, Button, GestureClick, Image, Label, Orientation, Scale};
use std::cell::{Cell, RefCell};
use std::rc::Rc;

use crate::midi::dsp::{fader_val_to_db, format_db};

#[derive(Clone)]
pub struct ChannelStrip {
    pub container: Box,
    scale: Scale,
    db_label: Label,
    mute_button: Button,
    mute_icon: Image,
    is_muted: Rc<RefCell<bool>>,
    current_val: Rc<RefCell<u8>>,
    suppress_callbacks: Rc<Cell<bool>>,
}

impl ChannelStrip {
    pub fn new<FVolume, FMute>(
        channel_id: &str,
        display_name: &str,
        icon_name: &str,
        initial_val: u8,
        initial_mute: bool,
        on_volume_change: FVolume,
        on_mute_toggle: FMute,
    ) -> Self
    where
        FVolume: Fn(&str, u8) + 'static,
        FMute: Fn(&str, bool) + 'static,
    {
        let container = Box::new(Orientation::Vertical, 10);
        container.add_css_class("channel-card");
        container.set_hexpand(true);
        container.set_vexpand(true);
        container.set_margin_start(4);
        container.set_margin_end(4);
        container.set_margin_top(4);
        container.set_margin_bottom(4);

        let suppress_callbacks = Rc::new(Cell::new(false));

        // Header: Icon + Title + dB/Percentage Badge
        let header_box = Box::new(Orientation::Vertical, 4);
        header_box.set_halign(Align::Center);
        header_box.set_margin_top(2);

        let icon = Image::from_icon_name(icon_name);
        icon.set_pixel_size(32);
        header_box.append(&icon);

        let name_label = Label::builder()
            .label(display_name)
            .css_classes(["heading"])
            .halign(Align::Center)
            .build();
        header_box.append(&name_label);

        let stepped_initial = ((initial_val as f64 / 5.0).round() * 5.0).clamp(0.0, 150.0) as u8;
        let initial_badge_text = Self::format_badge_text(stepped_initial, initial_mute);

        let db_label = Label::builder()
            .label(&initial_badge_text)
            .css_classes(if initial_mute { vec!["db-badge", "muted"] } else { vec!["db-badge"] })
            .halign(Align::Center)
            .build();
        header_box.append(&db_label);

        container.append(&header_box);

        // Vertical Fader Scale (0% to 150% in 5% increments)
        let scale = Scale::with_range(Orientation::Vertical, 0.0, 150.0, 5.0);
        scale.set_inverted(true); // 150% on top, 0% on bottom
        scale.set_increments(5.0, 10.0);
        scale.set_value(stepped_initial as f64);
        scale.set_height_request(220);
        scale.set_vexpand(true);
        scale.set_halign(Align::Center);
        scale.set_draw_value(false);
        scale.add_css_class("fader-slider");
        scale.add_css_class("accent");

        container.append(&scale);

        // Double-click to reset individual fader to Unity Gain (100% = 0.0 dB)
        let gesture = GestureClick::new();
        let scale_clone = scale.clone();
        gesture.connect_pressed(move |_, n_press, _, _| {
            if n_press == 2 {
                scale_clone.set_value(100.0);
            }
        });
        scale.add_controller(gesture);

        // Mute Button: Dynamic speaker icon
        let initial_icon = Self::get_volume_icon(stepped_initial, initial_mute);
        let mute_icon = Image::from_icon_name(initial_icon);
        mute_icon.set_pixel_size(18);
        mute_icon.set_halign(Align::Center);
        mute_icon.set_valign(Align::Center);

        let mute_button = Button::builder()
            .child(&mute_icon)
            .css_classes(if initial_mute {
                vec!["destructive-action", "pill", "channel-mute-btn"]
            } else {
                vec!["pill", "channel-mute-btn"]
            })
            .halign(Align::Center)
            .valign(Align::Center)
            .width_request(56)
            .height_request(36)
            .margin_bottom(2)
            .tooltip_text(if initial_mute { "Unmute Channel" } else { "Mute Channel" })
            .build();

        container.append(&mute_button);

        let is_muted = Rc::new(RefCell::new(initial_mute));
        let current_val = Rc::new(RefCell::new(stepped_initial));

        let ch_id_vol = channel_id.to_string();
        let db_label_clone = db_label.clone();
        let is_muted_vol = is_muted.clone();
        let current_val_vol = current_val.clone();
        let suppress_vol = suppress_callbacks.clone();
        let mute_icon_vol = mute_icon.clone();
        scale.connect_value_changed(move |s| {
            let raw = s.value();
            let stepped = ((raw / 5.0).round() * 5.0).clamp(0.0, 150.0);
            let val = stepped as u8;

            if (raw - stepped).abs() > 0.001 {
                s.set_value(stepped);
                return;
            }

            *current_val_vol.borrow_mut() = val;
            let muted = *is_muted_vol.borrow();
            db_label_clone.set_text(&Self::format_badge_text(val, muted));
            mute_icon_vol.set_icon_name(Some(Self::get_volume_icon(val, muted)));

            if !suppress_vol.get() {
                on_volume_change(&ch_id_vol, val);
            }
        });

        let ch_id_mute = channel_id.to_string();
        let is_muted_btn = is_muted.clone();
        let db_label_btn = db_label.clone();
        let current_val_btn = current_val.clone();
        let mute_btn_clone = mute_button.clone();
        let mute_icon_clone = mute_icon.clone();
        let suppress_btn = suppress_callbacks.clone();
        mute_button.connect_clicked(move |_| {
            let new_mute = !*is_muted_btn.borrow();
            *is_muted_btn.borrow_mut() = new_mute;
            let val = *current_val_btn.borrow();

            mute_icon_clone.set_icon_name(Some(Self::get_volume_icon(val, new_mute)));
            db_label_btn.set_text(&Self::format_badge_text(val, new_mute));

            if new_mute {
                mute_btn_clone.add_css_class("destructive-action");
                mute_btn_clone.set_tooltip_text(Some("Unmute Channel"));
                db_label_btn.add_css_class("muted");
            } else {
                mute_btn_clone.remove_css_class("destructive-action");
                mute_btn_clone.set_tooltip_text(Some("Mute Channel"));
                db_label_btn.remove_css_class("muted");
            }

            if !suppress_btn.get() {
                on_mute_toggle(&ch_id_mute, new_mute);
            }
        });

        Self {
            container,
            scale,
            db_label,
            mute_button,
            mute_icon,
            is_muted,
            current_val,
            suppress_callbacks,
        }
    }

    pub fn set_value(&self, val: u8) {
        let stepped = ((val as f64 / 5.0).round() * 5.0).clamp(0.0, 150.0) as u8;
        *self.current_val.borrow_mut() = stepped;
        self.suppress_callbacks.set(true);
        self.scale.set_value(stepped as f64);
        self.suppress_callbacks.set(false);

        let muted = *self.is_muted.borrow();
        self.db_label.set_text(&Self::format_badge_text(stepped, muted));
        self.mute_icon.set_icon_name(Some(Self::get_volume_icon(stepped, muted)));
    }

    pub fn set_muted(&self, muted: bool) {
        *self.is_muted.borrow_mut() = muted;
        let val = *self.current_val.borrow();

        self.mute_icon.set_icon_name(Some(Self::get_volume_icon(val, muted)));
        self.db_label.set_text(&Self::format_badge_text(val, muted));

        if muted {
            self.mute_button.add_css_class("destructive-action");
            self.mute_button.set_tooltip_text(Some("Unmute Channel"));
            self.db_label.add_css_class("muted");
        } else {
            self.mute_button.remove_css_class("destructive-action");
            self.mute_button.set_tooltip_text(Some("Mute Channel"));
            self.db_label.remove_css_class("muted");
        }
    }

    fn format_badge_text(val: u8, muted: bool) -> String {
        if muted {
            "MUTED".to_string()
        } else {
            let db = fader_val_to_db(val);
            let pct = val;
            format!("{} ({}%)", format_db(db), pct)
        }
    }

    fn get_volume_icon(val: u8, muted: bool) -> &'static str {
        if muted || val == 0 {
            "audio-volume-muted-symbolic"
        } else if val < 40 {
            "audio-volume-low-symbolic"
        } else if val < 80 {
            "audio-volume-medium-symbolic"
        } else {
            "audio-volume-high-symbolic"
        }
    }
}
