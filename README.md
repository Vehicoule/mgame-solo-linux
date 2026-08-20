# M-Game Solo Linux Control Center

A desktop application to configure and control the M-Audio M-Game Solo audio interface on Linux.

It provides a modern graphical interface to manage hardware volume faders, audio routing, microphone effects, and LED lighting.

---

## Features

- **Real-Time Volume & Mute Sync**: Moving physical faders or on-screen sliders stays in sync with your system sound settings and keyboard volume keys.
- **Audio Routing**: Separates audio into dedicated channels for Game, Chat, Sampler, and System audio, making it easy to manage stream audio and voice chat independently.
- **Microphone Processing**: Built-in High-Pass Filter, Noise Gate, Compressor, and 3-Band Equalizer.
- **Voice Effects**: Real-time pitch shift, formant alteration, and reverb presets.
- **Hardware Controls**: Toggle +48V phantom power for condenser microphones, switch high-impedance headphone mode, and customize LED lighting colors.
- **Censor & Panic Buttons**: Configure the hardware censor button behavior or mute all audio instantly with one click.

---

## Installation

### Method 1: Pre-built Archive (Recommended)

1. Download the latest `mgame-solo-v1.0.0-x86_64-linux.tar.gz` from the [Releases](https://codeberg.org/Vehicoule/mgame-solo-linux/releases) page.
2. Extract the archive:
   ```bash
   tar -xzf mgame-solo-v1.0.0-x86_64-linux.tar.gz
   cd mgame-solo-v1.0.0-x86_64-linux
   ```
3. Run the installer:
   ```bash
   ./install.sh
   ```
4. Launch **M-Game Solo** from your application launcher or by running `mgame-solo` in a terminal.

---

### Method 2: Flatpak

Download `mgame-solo.flatpak` from the [Releases](https://codeberg.org/Vehicoule/mgame-solo-linux/releases) page and install it with:

```bash
flatpak install --user mgame-solo.flatpak
```

Or build from source manifest:
```bash
flatpak-builder --user --install --force-clean build-dir com.mgame.Solo.yml
```

---

### Method 3: Snap

Download the `.snap` package from [Releases](https://codeberg.org/Vehicoule/mgame-solo-linux/releases) and install:

```bash
sudo snap install --classic --dangerous mgame-solo_*.snap
```

---

### Method 4: Building from Source

#### Prerequisites

Make sure the following packages are installed on your distribution:

- **Debian / Ubuntu**: `build-essential libasound2-dev libgtk-4-dev libadwaita-1-dev pkg-config`
- **Fedora**: `gcc alsa-lib-devel gtk4-devel libadwaita-devel pkgconf-pkg-config`
- **Arch Linux**: `base-devel alsa-lib gtk4 libadwaita pkgconf`
- **Rust Toolchain**: Install via [rustup.rs](https://rustup.rs/)

#### Compile and Install

```bash
git clone https://codeberg.org/Vehicoule/mgame-solo-linux.git
cd mgame-solo-linux
make install
```

To uninstall at any time:
```bash
make uninstall
```

---

## Hardware Permissions (udev)

To allow the application to communicate with the M-Game Solo without requiring root privileges, copy the udev rule file:

```bash
sudo cp data/99-mgame-solo.rules /etc/udev/rules.d/
sudo udevadm control --reload-rules
sudo udevadm trigger
```

Unplug the USB cable and plug it back in once.

---

## License

This project is licensed under the GPL-3.0 License.
