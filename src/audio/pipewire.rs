//! PipeWire & WirePlumber Audio Integration for M-Game Solo.
//! Direct pactl & wpctl deterministic audio routing, real-time volume/mute monitoring, and bi-directional sync.

use std::fs;
use std::path::PathBuf;
use std::process::Command;
use log::{info, warn};

const PIPEWIRE_CONFIG: &str = r#"# M-Audio M-Game Solo Channel Isolation Configuration
context.modules = [
    # 1. M-Game Solo (Game) -> Channels 1-2
    {   name = libpipewire-module-loopback
        args = {
            node.description = "M-Game Solo (Game)"
            capture.props = {
                node.name = "mgame_game_sink"
                media.class = "Audio/Sink"
                audio.position = [ FL FR ]
                priority.session = 1500
            }
            playback.props = {
                node.target = "alsa_output.usb-M-Audio_M-Game_Solo-01.analog-surround-71"
                audio.position = [ FL FR ]
                channelmix.disable = true
                stream.dont-remix = true
                node.passive = true
            }
        }
    }
    # 2. M-Game Solo (Chat Out) -> Channels 3-4
    {   name = libpipewire-module-loopback
        args = {
            node.description = "M-Game Solo (Chat Out)"
            capture.props = {
                node.name = "mgame_chat_sink"
                media.class = "Audio/Sink"
                audio.position = [ FL FR ]
                priority.session = 1500
            }
            playback.props = {
                node.target = "alsa_output.usb-M-Audio_M-Game_Solo-01.analog-surround-71"
                audio.position = [ RL RR ]
                channelmix.disable = true
                stream.dont-remix = true
                node.passive = true
            }
        }
    }
    # 3. M-Game Solo (Sampler Out) -> Channels 5-6
    {   name = libpipewire-module-loopback
        args = {
            node.description = "M-Game Solo (Sampler Out)"
            capture.props = {
                node.name = "mgame_sampler_sink"
                media.class = "Audio/Sink"
                audio.position = [ FL FR ]
                priority.session = 1500
            }
            playback.props = {
                node.target = "alsa_output.usb-M-Audio_M-Game_Solo-01.analog-surround-71"
                audio.position = [ FC LFE ]
                channelmix.disable = true
                stream.dont-remix = true
                node.passive = true
            }
        }
    }
    # 4. M-Game Solo (System) -> Channels 7-8 (Master Default Output)
    {   name = libpipewire-module-loopback
        args = {
            node.description = "M-Game Solo (System)"
            capture.props = {
                node.name = "mgame_system_sink"
                media.class = "Audio/Sink"
                audio.position = [ FL FR ]
                priority.session = 3000
                priority.driver = 3000
            }
            playback.props = {
                node.target = "alsa_output.usb-M-Audio_M-Game_Solo-01.analog-surround-71"
                audio.position = [ SL SR ]
                channelmix.disable = true
                stream.dont-remix = true
                node.passive = true
            }
        }
    }
    # 5. M-Game Solo (Stream Mix) <- Channels 1-2
    {   name = libpipewire-module-loopback
        args = {
            node.description = "M-Game Solo (Stream Mix)"
            capture.props = {
                node.target = "alsa_input.usb-M-Audio_M-Game_Solo-01.analog-surround-71"
                audio.position = [ FL FR ]
                channelmix.disable = true
                stream.dont-remix = true
                node.passive = true
            }
            playback.props = {
                node.name = "mgame_stream_source"
                media.class = "Audio/Source"
                audio.position = [ FL FR ]
                priority.session = 1800
            }
        }
    }
    # 6. M-Game Solo (Chat Mic) <- Channels 3-4 (Master Default Input)
    {   name = libpipewire-module-loopback
        args = {
            node.description = "M-Game Solo (Chat Mic)"
            capture.props = {
                node.target = "alsa_input.usb-M-Audio_M-Game_Solo-01.analog-surround-71"
                audio.position = [ RL RR ]
                channelmix.disable = true
                stream.dont-remix = true
                node.passive = true
            }
            playback.props = {
                node.name = "mgame_chat_source"
                media.class = "Audio/Source"
                audio.position = [ FL FR ]
                priority.session = 3000
                priority.driver = 3000
            }
        }
    }
    # 7. M-Game Solo (Sampler In) <- Channels 5-6
    {   name = libpipewire-module-loopback
        args = {
            node.description = "M-Game Solo (Sampler In)"
            capture.props = {
                node.target = "alsa_input.usb-M-Audio_M-Game_Solo-01.analog-surround-71"
                audio.position = [ FC LFE ]
                channelmix.disable = true
                stream.dont-remix = true
                node.passive = true
            }
            playback.props = {
                node.name = "mgame_sampler_source"
                media.class = "Audio/Source"
                audio.position = [ FL FR ]
                priority.session = 1500
            }
        }
    }
    # 8. M-Game Solo (Aux In) <- Channels 7-8
    {   name = libpipewire-module-loopback
        args = {
            node.description = "M-Game Solo (Aux In)"
            capture.props = {
                node.target = "alsa_input.usb-M-Audio_M-Game_Solo-01.analog-surround-71"
                audio.position = [ SL SR ]
                channelmix.disable = true
                stream.dont-remix = true
                node.passive = true
            }
            playback.props = {
                node.name = "mgame_aux_source"
                media.class = "Audio/Source"
                audio.position = [ FL FR ]
                priority.session = 1500
            }
        }
    }
]
"#;

pub fn get_pipewire_config_path() -> Option<PathBuf> {
    directories::BaseDirs::new().map(|dirs| {
        dirs.config_dir()
            .join("pipewire/pipewire.conf.d/50-m-game-solo.conf")
    })
}

pub fn ensure_pipewire_configuration() -> Result<(), Box<dyn std::error::Error>> {
    let mut modified = false;

    if let Some(pw_path) = get_pipewire_config_path() {
        if let Some(parent) = pw_path.parent() {
            fs::create_dir_all(parent)?;
        }
        let needs_write = match fs::read_to_string(&pw_path) {
            Ok(content) => content != PIPEWIRE_CONFIG,
            Err(_) => true,
        };
        if needs_write {
            info!("Provisioning PipeWire loopback config at {:?}", pw_path);
            fs::write(&pw_path, PIPEWIRE_CONFIG)?;
            modified = true;
        }
    }

    if modified {
        let _ = Command::new("systemctl")
            .args(["--user", "restart", "pipewire.service", "wireplumber.service"])
            .status();
        std::thread::sleep(std::time::Duration::from_millis(500));
    }

    // Set Master Defaults: Output -> System, Input -> Chat Mic
    let _ = Command::new("pactl")
        .args(["set-default-sink", "mgame_system_sink"])
        .status();
    let _ = Command::new("pactl")
        .args(["set-default-source", "mgame_chat_source"])
        .status();

    Ok(())
}

pub fn set_channel_volume(channel: &str, fader_val: u8) {
    let vol_percent = format!("{}%", fader_val);
    match channel {
        "microphone" => {
            let _ = Command::new("pactl").args(["set-source-volume", "mgame_chat_source", &vol_percent]).status();
            let _ = Command::new("pactl").args(["set-source-volume", "mgame_stream_source", &vol_percent]).status();
            let _ = Command::new("pactl").args(["set-source-volume", "@DEFAULT_SOURCE@", &vol_percent]).status();
            let _ = Command::new("pactl").args(["set-source-volume", "alsa_input.usb-M-Audio_M-Game_Solo-01.analog-surround-71", &vol_percent]).status();
        }
        "game" => {
            let _ = Command::new("pactl").args(["set-sink-volume", "mgame_game_sink", &vol_percent]).status();
        }
        "chat" => {
            let _ = Command::new("pactl").args(["set-sink-volume", "mgame_chat_sink", &vol_percent]).status();
        }
        "sampler" => {
            let _ = Command::new("pactl").args(["set-sink-volume", "mgame_sampler_sink", &vol_percent]).status();
            let _ = Command::new("pactl").args(["set-source-volume", "mgame_sampler_source", &vol_percent]).status();
        }
        "system" => {
            let _ = Command::new("pactl").args(["set-sink-volume", "mgame_system_sink", &vol_percent]).status();
            let _ = Command::new("pactl").args(["set-sink-volume", "@DEFAULT_SINK@", &vol_percent]).status();
        }
        _ => {}
    }
}

pub fn set_channel_mute(channel: &str, muted: bool) {
    let mute_val = if muted { "1" } else { "0" };
    match channel {
        "microphone" => {
            let _ = Command::new("pactl").args(["set-source-mute", "mgame_chat_source", mute_val]).status();
            let _ = Command::new("pactl").args(["set-source-mute", "mgame_stream_source", mute_val]).status();
            let _ = Command::new("pactl").args(["set-source-mute", "@DEFAULT_SOURCE@", mute_val]).status();
            let _ = Command::new("pactl").args(["set-source-mute", "alsa_input.usb-M-Audio_M-Game_Solo-01.analog-surround-71", mute_val]).status();
        }
        "game" => {
            let _ = Command::new("pactl").args(["set-sink-mute", "mgame_game_sink", mute_val]).status();
        }
        "chat" => {
            let _ = Command::new("pactl").args(["set-sink-mute", "mgame_chat_sink", mute_val]).status();
        }
        "sampler" => {
            let _ = Command::new("pactl").args(["set-sink-mute", "mgame_sampler_sink", mute_val]).status();
            let _ = Command::new("pactl").args(["set-source-mute", "mgame_sampler_source", mute_val]).status();
        }
        "system" => {
            let _ = Command::new("pactl").args(["set-sink-mute", "mgame_system_sink", mute_val]).status();
            let _ = Command::new("pactl").args(["set-sink-mute", "@DEFAULT_SINK@", mute_val]).status();
        }
        _ => {}
    }
}

pub fn get_channel_volume(channel: &str) -> Option<u8> {
    let (cmd, target) = match channel {
        "microphone" => ("get-source-volume", "alsa_input.usb-M-Audio_M-Game_Solo-01.analog-surround-71"),
        "game" => ("get-sink-volume", "mgame_game_sink"),
        "chat" => ("get-sink-volume", "mgame_chat_sink"),
        "sampler" => ("get-sink-volume", "mgame_sampler_sink"),
        "system" => ("get-sink-volume", "mgame_system_sink"),
        _ => return None,
    };
    let output = Command::new("pactl").args([cmd, target]).output().ok()?;
    if !output.status.success() {
        return None;
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    parse_volume_percentage(&stdout)
}

pub fn get_channel_mute(channel: &str) -> Option<bool> {
    let (cmd, target) = match channel {
        "microphone" => ("get-source-mute", "alsa_input.usb-M-Audio_M-Game_Solo-01.analog-surround-71"),
        "game" => ("get-sink-mute", "mgame_game_sink"),
        "chat" => ("get-sink-mute", "mgame_chat_sink"),
        "sampler" => ("get-sink-mute", "mgame_sampler_sink"),
        "system" => ("get-sink-mute", "mgame_system_sink"),
        _ => return None,
    };
    let output = Command::new("pactl").args([cmd, target]).output().ok()?;
    if !output.status.success() {
        return None;
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    Some(stdout.to_lowercase().contains("yes"))
}

pub fn parse_volume_percentage(output: &str) -> Option<u8> {
    for part in output.split('/') {
        let trimmed = part.trim();
        if let Some(num_str) = trimmed.strip_suffix('%') {
            if let Ok(val) = num_str.trim().parse::<u32>() {
                return Some(val.min(150) as u8);
            }
        }
    }
    None
}

/// Spawns a dedicated background thread monitoring real-time system audio changes (via pactl subscribe)
pub fn spawn_pipewire_event_listener<F>(event_tx: F) -> std::thread::JoinHandle<()>
where
    F: Fn(String, u8, bool) + Send + 'static,
{
    std::thread::spawn(move || {
        use std::io::{BufRead, BufReader};
        use std::process::Stdio;

        loop {
            if let Ok(mut child) = Command::new("pactl")
                .arg("subscribe")
                .stdout(Stdio::piped())
                .stderr(Stdio::null())
                .spawn()
            {
                if let Some(stdout) = child.stdout.take() {
                    let reader = BufReader::new(stdout);
                    for line in reader.lines() {
                        if let Ok(l) = line {
                            if l.contains("sink") || l.contains("source") {
                                for ch in &["microphone", "game", "chat", "sampler", "system"] {
                                    if let (Some(vol), Some(muted)) = (get_channel_volume(ch), get_channel_mute(ch)) {
                                        event_tx(ch.to_string(), vol, muted);
                                    }
                                }
                            }
                        }
                    }
                }
                let _ = child.wait();
            }
            std::thread::sleep(std::time::Duration::from_millis(500));
        }
    })
}
