//! PipeWire & WirePlumber Audio Integration for M-Game Solo.
//! Direct pactl & wpctl deterministic audio routing, real-time volume/mute monitoring, and bi-directional sync.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::io::Write;
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

fn is_flatpak() -> bool {
    Path::new("/.flatpak-info").exists()
}

fn is_snap() -> bool {
    std::env::var("SNAP").is_ok()
}

fn get_host_home() -> Option<PathBuf> {
    if is_snap() {
        if let Ok(real) = std::env::var("SNAP_REAL_HOME") {
            return Some(PathBuf::from(real));
        }
    }
    std::env::var("HOME").ok().map(PathBuf::from)
}

pub fn get_pipewire_config_path() -> Option<PathBuf> {
    directories::BaseDirs::new().map(|dirs| {
        dirs.config_dir()
            .join("pipewire/pipewire.conf.d/50-m-game-solo.conf")
    })
}

fn get_host_pipewire_config_path() -> Option<PathBuf> {
    get_host_home().map(|h| h.join(".config/pipewire/pipewire.conf.d/50-m-game-solo.conf"))
}

fn write_config_to_path(path: &Path) -> Result<bool, Box<dyn std::error::Error>> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let needs_write = match fs::read_to_string(path) {
        Ok(content) => content != PIPEWIRE_CONFIG,
        Err(_) => true,
    };
    if needs_write {
        info!("Provisioning PipeWire loopback config at {:?}", path);
        fs::write(path, PIPEWIRE_CONFIG)?;
        return Ok(true);
    }
    Ok(false)
}

fn write_config_via_host_spawn(path: &Path) -> Result<bool, Box<dyn std::error::Error>> {
    let path_str = path.to_string_lossy().to_string();
    let parent_str = path.parent().unwrap().to_string_lossy().to_string();

    // Ensure parent exists on host
    let st = Command::new("flatpak-spawn")
        .args(["--host", "mkdir", "-p", &parent_str])
        .status();
    if st.is_err() {
        return Err("flatpak-spawn mkdir failed".into());
    }

    // Check existing content via host cat
    let existing = Command::new("flatpak-spawn")
        .args(["--host", "cat", &path_str])
        .output();
    if let Ok(out) = existing {
        if out.status.success() {
            let content = String::from_utf8_lossy(&out.stdout);
            if content == PIPEWIRE_CONFIG {
                return Ok(false);
            }
        }
    }

    info!("Provisioning PipeWire config via host spawn at {:?}", path);
    let mut child = Command::new("flatpak-spawn")
        .args(["--host", "tee", &path_str])
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;
    {
        let stdin = child.stdin.as_mut().ok_or("Failed to open stdin")?;
        stdin.write_all(PIPEWIRE_CONFIG.as_bytes())?;
    }
    let status = child.wait()?;
    if !status.success() {
        return Err("flatpak-spawn tee failed".into());
    }
    Ok(true)
}

fn restart_pipewire_services() {
    let mut success = false;

    // Try direct systemctl first (works natively and in flatpak if --talk-name=org.freedesktop.systemd1 is granted)
    if let Ok(st) = Command::new("systemctl")
        .args(["--user", "restart", "pipewire.service", "wireplumber.service"])
        .status()
    {
        if st.success() {
            success = true;
        }
    }

    // Fallback via host spawn for flatpak
    if !success && is_flatpak() {
        if let Ok(st) = Command::new("flatpak-spawn")
            .args(["--host", "systemctl", "--user", "restart", "pipewire.service", "wireplumber.service"])
            .status()
        {
            if st.success() {
                success = true;
            }
        }
    }

    if success {
        std::thread::sleep(std::time::Duration::from_millis(800));
        // Wait for nodes to appear (up to 5s)
        for _ in 0..10 {
            if check_mgame_nodes_present() {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(500));
        }
    } else {
        warn!("Failed to restart PipeWire via systemctl; virtual devices may require manual restart or re-login");
    }
}

fn check_mgame_nodes_present() -> bool {
    if let Ok(out) = Command::new("pactl").args(["list", "sinks", "short"]).output() {
        let txt = String::from_utf8_lossy(&out.stdout);
        return txt.contains("mgame_game_sink") && txt.contains("mgame_system_sink");
    }
    false
}

pub fn ensure_pipewire_configuration() -> Result<(), Box<dyn std::error::Error>> {
    let mut modified = false;

    // 1) Sandboxed/native config path (for native builds)
    if let Some(pw_path) = get_pipewire_config_path() {
        match write_config_to_path(&pw_path) {
            Ok(m) => modified |= m,
            Err(e) => warn!("Failed to write pipewire config to {:?}: {}", pw_path, e),
        }
    }

    // 2) Host config path when sandboxed (flatpak/snap)
    if is_flatpak() || is_snap() {
        if let Some(host_path) = get_host_pipewire_config_path() {
            // Avoid double-write if paths are identical
            let sandboxed = get_pipewire_config_path();
            if Some(&host_path) != sandboxed.as_ref() {
                let res = if is_flatpak() {
                    write_config_via_host_spawn(&host_path).or_else(|_| write_config_to_path(&host_path))
                } else {
                    write_config_to_path(&host_path)
                };
                match res {
                    Ok(m) => modified |= m,
                    Err(e) => warn!("Failed to write host pipewire config to {:?}: {}", host_path, e),
                }
            }
        }
    }

    if modified {
        restart_pipewire_services();
    } else {
        // Even if not modified, ensure nodes are actually present; if missing, force restart
        if !check_mgame_nodes_present() {
            info!("PipeWire config present but virtual sinks missing — triggering restart");
            restart_pipewire_services();
        }
    }

    // Set Master Defaults: Output -> System, Input -> Chat Mic (wait for nodes)
    for _ in 0..5 {
        if check_mgame_nodes_present() {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(400));
    }

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
