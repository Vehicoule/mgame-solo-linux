//! Non-Blocking POSIX RawMIDI Engine for Linux.
//! Accurately connects to /dev/snd/midiC*D0 with real-time ALSA/USB hotplug lifecycle tracking.

use std::fs::{self, OpenOptions};
use std::os::unix::fs::OpenOptionsExt;
use std::os::unix::io::AsFd;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use std::io::{Read, Write};
use log::{info, warn};
use nix::poll::{poll, PollFd, PollFlags, PollTimeout};

use super::protocol::{parse_sysex, MGSysExMessage, SYSEX_END, SYSEX_START};

pub enum MidiEvent {
    Connected(String),
    Disconnected,
    Message(MGSysExMessage),
}

pub struct MidiClient {
    running: Arc<AtomicBool>,
    connected: Arc<AtomicBool>,
    tx_queue: Sender<Vec<u8>>,
    worker_handle: Option<thread::JoinHandle<()>>,
}

impl MidiClient {
    pub fn new<F>(event_callback: F) -> Self
    where
        F: Fn(MidiEvent) + Send + 'static,
    {
        let running = Arc::new(AtomicBool::new(true));
        let connected = Arc::new(AtomicBool::new(Self::is_device_present()));
        let (tx_queue, rx_queue) = channel::<Vec<u8>>();

        let running_clone = Arc::clone(&running);
        let connected_clone = Arc::clone(&connected);
        let worker_handle = thread::spawn(move || {
            Self::worker_loop(running_clone, connected_clone, rx_queue, event_callback);
        });

        Self {
            running,
            connected,
            tx_queue,
            worker_handle: Some(worker_handle),
        }
    }

    pub fn send(&self, bytes: Vec<u8>) {
        let _ = self.tx_queue.send(bytes);
    }

    pub fn is_active(&self) -> bool {
        self.connected.load(Ordering::SeqCst)
    }

    /// Check if M-Game Solo is active via USB sysfs or ALSA device node
    pub fn is_device_present() -> bool {
        // 1. Direct USB sysfs check (instant & authoritative for power switch / plug)
        if let Ok(entries) = fs::read_dir("/sys/bus/usb/devices") {
            for entry in entries.flatten() {
                let p = entry.path();
                let vid_path = p.join("idVendor");
                let pid_path = p.join("idProduct");
                if vid_path.exists() && pid_path.exists() {
                    if let (Ok(vid), Ok(pid)) = (fs::read_to_string(vid_path), fs::read_to_string(pid_path)) {
                        if vid.trim() == "0763" && pid.trim() == "0043" {
                            return true;
                        }
                    }
                }
            }
        }

        // 2. Fallback check on ALSA
        Self::find_mgame_device().is_some()
    }

    /// Locate M-Game Solo MIDI node in /proc/asound/cards
    pub fn find_mgame_device() -> Option<String> {
        if let Ok(cards_txt) = fs::read_to_string("/proc/asound/cards") {
            let mut current_card_num: Option<u32> = None;
            for line in cards_txt.lines() {
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    continue;
                }
                if let Some(first_char) = trimmed.chars().next() {
                    if first_char.is_ascii_digit() {
                        let parts: Vec<&str> = trimmed.split_whitespace().collect();
                        if let Some(num_str) = parts.first() {
                            current_card_num = num_str.parse::<u32>().ok();
                        }
                    }
                }
                if (line.contains("M-Game Solo") || line.contains("M-Audio M-Game Solo") || line.contains("Solo")) && current_card_num.is_some() {
                    let card_idx = current_card_num.unwrap();
                    let midi_path = format!("/dev/snd/midiC{}D0", card_idx);
                    if Path::new(&midi_path).exists() {
                        return Some(midi_path);
                    }
                }
            }
        }
        None
    }

    fn worker_loop<F>(
        running: Arc<AtomicBool>,
        connected: Arc<AtomicBool>,
        rx_queue: Receiver<Vec<u8>>,
        event_callback: F,
    ) where
        F: Fn(MidiEvent) + Send + 'static,
    {
        let mut rx_buf = [0u8; 1024];
        let mut sysex_buf = Vec::with_capacity(128);

        while running.load(Ordering::SeqCst) {
            if !Self::is_device_present() {
                if connected.swap(false, Ordering::SeqCst) {
                    event_callback(MidiEvent::Disconnected);
                }
                thread::sleep(Duration::from_millis(150));
                continue;
            }

            let dev_path = match Self::find_mgame_device() {
                Some(p) => p,
                None => {
                    thread::sleep(Duration::from_millis(150));
                    continue;
                }
            };

            let file_res = OpenOptions::new()
                .read(true)
                .write(true)
                .custom_flags(nix::libc::O_NONBLOCK)
                .open(&dev_path);

            let mut file = match file_res {
                Ok(f) => {
                    info!("Connected to M-Game MIDI device: {}", dev_path);
                    connected.store(true, Ordering::SeqCst);
                    event_callback(MidiEvent::Connected(dev_path.clone()));
                    f
                }
                Err(_) => {
                    if connected.swap(false, Ordering::SeqCst) {
                        event_callback(MidiEvent::Disconnected);
                    }
                    thread::sleep(Duration::from_millis(200));
                    continue;
                }
            };

            while running.load(Ordering::SeqCst) {
                if !Self::is_device_present() {
                    warn!("M-Game Solo unplugged or powered off");
                    break;
                }

                // Send pending outgoing packets
                while let Ok(msg) = rx_queue.try_recv() {
                    if let Err(e) = file.write_all(&msg) {
                        warn!("Error writing SysEx to hardware: {}", e);
                    }
                }

                let mut poll_fds = [PollFd::new(file.as_fd(), PollFlags::POLLIN | PollFlags::POLLERR | PollFlags::POLLHUP | PollFlags::POLLNVAL)];
                match poll(&mut poll_fds, PollTimeout::from(60u16)) {
                    Ok(n) if n > 0 => {
                        let revents = poll_fds[0].revents().unwrap_or(PollFlags::empty());
                        if revents.contains(PollFlags::POLLERR) || revents.contains(PollFlags::POLLHUP) || revents.contains(PollFlags::POLLNVAL) {
                            warn!("MIDI device disconnected (POLLHUP/ERR/NVAL)");
                            break;
                        }
                        if revents.contains(PollFlags::POLLIN) {
                            match file.read(&mut rx_buf) {
                                Ok(n) if n > 0 => {
                                    for &byte in &rx_buf[..n] {
                                        if byte == SYSEX_START {
                                            sysex_buf.clear();
                                            sysex_buf.push(byte);
                                        } else if byte == SYSEX_END {
                                            if !sysex_buf.is_empty() && sysex_buf[0] == SYSEX_START {
                                                sysex_buf.push(byte);
                                                if let Some(parsed) = parse_sysex(&sysex_buf) {
                                                    event_callback(MidiEvent::Message(parsed));
                                                }
                                                sysex_buf.clear();
                                            }
                                        } else if !sysex_buf.is_empty() {
                                            sysex_buf.push(byte);
                                            if sysex_buf.len() > 256 {
                                                sysex_buf.clear();
                                            }
                                        }
                                    }
                                }
                                Ok(_) => {}
                                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {}
                                Err(e) => {
                                    warn!("Error reading from MIDI (device disconnected): {}", e);
                                    break;
                                }
                            }
                        }
                    }
                    Ok(_) => {}
                    Err(e) => {
                        warn!("Poll error on MIDI descriptor: {}", e);
                        break;
                    }
                }
            }

            if connected.swap(false, Ordering::SeqCst) {
                event_callback(MidiEvent::Disconnected);
            }
            thread::sleep(Duration::from_millis(150));
        }
    }
}

impl Drop for MidiClient {
    fn drop(&mut self) {
        self.running.store(false, Ordering::SeqCst);
        if let Some(handle) = self.worker_handle.take() {
            let _ = handle.join();
        }
    }
}
