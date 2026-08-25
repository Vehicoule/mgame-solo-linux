//! Native Libadwaita Audio Routing Matrix View for M-Game Solo.
//! Clean 2D Cross-Point Grid with native GTK4 Switches.

use gtk4::prelude::*;
use gtk4::{Align, Box, Grid, Image, Label, Orientation, Switch};
use libadwaita::prelude::*;
use libadwaita::{Clamp, PreferencesGroup};
use std::sync::{Arc, RwLock};

use crate::state::MixerState;

pub struct MatrixView {
    pub container: Box,
}

impl MatrixView {
    pub fn new<F>(state: Arc<RwLock<MixerState>>, on_routing_change: F) -> Self
    where
        F: Fn(&str, &str, bool) + Clone + 'static,
    {
        let container = Box::new(Orientation::Vertical, 16);
        container.set_margin_top(16);
        container.set_margin_bottom(24);
        container.set_margin_start(16);
        container.set_margin_end(16);

        let clamp = Clamp::new();
        clamp.set_maximum_size(860);

        let group = PreferencesGroup::builder()
            .title("Audio Routing Matrix")
            .description("Route physical inputs and virtual PC streams to output destinations.")
            .build();

        // 2D Matrix Table Grid
        let grid = Grid::builder()
            .row_spacing(12)
            .column_spacing(20)
            .margin_top(16)
            .margin_bottom(16)
            .margin_start(16)
            .margin_end(16)
            .halign(Align::Center)
            .build();

        let destinations = [
            ("stream", "Stream Mix", "audio-input-microphone-symbolic", "OBS Broadcast"),
            ("chat", "Chat Out", "audio-headset-symbolic", "Discord / Voice"),
            ("main_out", "Main Out", "audio-speakers-symbolic", "Speakers (Back)"),
            ("phones_out", "Headphones", "audio-headphones-symbolic", "Front Jack"),
        ];

        let sources = [
            ("microphone", "Mic In", "audio-input-microphone-symbolic"),
            ("game", "Game Audio", "input-gaming-symbolic"),
            ("chat", "Chat Audio", "audio-headset-symbolic"),
            ("sampler", "Sampler Audio", "media-playback-start-symbolic"),
            ("system", "System Audio", "video-display-symbolic"),
            ("aux", "Aux In", "audio-card-symbolic"),
        ];

        // Column Headers (Top)
        for (col_idx, (_dest_id, dest_name, icon_name, sub)) in destinations.iter().enumerate() {
            let col_box = Box::new(Orientation::Vertical, 3);
            col_box.set_halign(Align::Center);
            col_box.set_width_request(110);

            let img = Image::from_icon_name(*icon_name);
            img.set_pixel_size(20);
            col_box.append(&img);

            let lbl = Label::builder()
                .label(*dest_name)
                .css_classes(["heading"])
                .halign(Align::Center)
                .build();
            col_box.append(&lbl);

            let sub_lbl = Label::builder()
                .label(*sub)
                .css_classes(["caption", "dim-label"])
                .halign(Align::Center)
                .build();
            col_box.append(&sub_lbl);

            grid.attach(&col_box, (col_idx + 1) as i32, 0, 1, 1);
        }

        // Rows & Switches
        let state_guard = state.read().unwrap();

        for (row_idx, (src_id, src_name, icon_name)) in sources.iter().enumerate() {
            // Row Header (Left)
            let row_header = Box::new(Orientation::Horizontal, 10);
            row_header.set_width_request(150);
            row_header.set_halign(Align::Start);
            row_header.set_valign(Align::Center);

            let img = Image::from_icon_name(*icon_name);
            img.set_pixel_size(20);
            row_header.append(&img);

            let lbl = Label::builder()
                .label(*src_name)
                .css_classes(["heading"])
                .halign(Align::Start)
                .build();
            row_header.append(&lbl);

            grid.attach(&row_header, 0, (row_idx + 1) as i32, 1, 1);

            // Cells (Native GTK4 Switches)
            for (col_idx, (dest_id, _, _, _)) in destinations.iter().enumerate() {
                let key = format!("{}->{}", src_id, dest_id);
                let initial_enabled = *state_guard.routing.get(&key).unwrap_or(&true);

                let sw = Switch::builder()
                    .active(initial_enabled)
                    .halign(Align::Center)
                    .valign(Align::Center)
                    .build();

                let src_str = src_id.to_string();
                let dest_str = dest_id.to_string();
                let cb = on_routing_change.clone();

                sw.connect_active_notify(move |s| {
                    cb(&src_str, &dest_str, s.is_active());
                });

                grid.attach(&sw, (col_idx + 1) as i32, (row_idx + 1) as i32, 1, 1);
            }
        }

        let matrix_card = Box::new(Orientation::Vertical, 0);
        matrix_card.add_css_class("card");
        matrix_card.set_margin_top(8);
        matrix_card.set_margin_bottom(8);
        matrix_card.set_margin_start(4);
        matrix_card.set_margin_end(4);
        matrix_card.append(&grid);

        group.add(&matrix_card);
        clamp.set_child(Some(&group));
        container.append(&clamp);

        Self { container }
    }
}
