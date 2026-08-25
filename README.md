# M-Game Solo for Linux

<div align="center">

<a href="https://github.com/Vehicoule/mgame-solo-linux/releases/latest">
<img src="https://img.shields.io/github/v/release/Vehicoule/mgame-solo-linux?style=for-the-badge&label=Latest%20release" alt="Latest release">
</a>
<a href="https://www.gnu.org/licenses/gpl-3.0">
<img src="https://img.shields.io/badge/License-GPL--3.0--or--later-blue?style=for-the-badge" alt="License">
</a>

</div>

Native companion app and mixer control center for the **M-Audio M-Game Solo** streaming audio interface.
Real-time fader sync, mic processing DSP, voice effects, LED themes and PipeWire channel routing — all in one GTK4 app.

---

## 📦 Download & Install

**Click a button to download the latest build:**

<div align="center">

<a href="https://github.com/Vehicoule/mgame-solo-linux/releases/latest/download/mgame-solo-x86_64-linux.tar.gz">
<img src="https://img.shields.io/badge/Download-.tar.gz-4A90D9?style=for-the-badge&logo=gnu-bash&logoColor=white" alt="Download tarball">
</a>
<a href="https://github.com/Vehicoule/mgame-solo-linux/releases/latest/download/mgame-solo.flatpak">
<img src="https://img.shields.io/badge/Download-.Flatpak-F2913D?style=for-the-badge&logo=flathub&logoColor=white" alt="Download Flatpak">
</a>
<a href="https://github.com/Vehicoule/mgame-solo-linux/releases/latest/download/mgame-solo_amd64.snap">
<img src="https://img.shields.io/badge/Download-.Snap-82BFA1?style=for-the-badge&logo=snapcraft&logoColor=white" alt="Download Snap">
</a>

</div>

### Tarball — works on any distro

```bash
tar xzf mgame-solo-x86_64-linux.tar.gz
cd mgame-solo-v*-linux
./install.sh          # add sudo if you want /usr/local instead of ~/.local
```

Installs the binary, desktop entry, icon and udev rules. Uninstall anytime with `./uninstall.sh`.

### Flatpak — sandboxed, any distro with Flatpak

```bash
flatpak install ./mgame-solo.flatpak
```

Requires the [Flathub](https://flathub.org) remote (preconfigured on most distros) for the GNOME runtime.

### Snap

```bash
sudo snap install mgame-solo_amd64.snap --dangerous
```

`--dangerous` is only needed because the package is installed locally instead of from the Snap Store.

---

## 🔨 Build from source

Requirements: Rust (`rustup`), `pkg-config`, GTK4 and libadwaita development packages
(`libgtk-4-dev`, `libadwaita-1-dev` on Debian/Ubuntu).

```bash
git clone https://github.com/Vehicoule/mgame-solo-linux
cd mgame-solo-linux
make            # debug build
make test       # run tests
make install    # install to ~/.local (override with PREFIX=...)
make dist       # create release tarball in dist/
```

---

## 📄 License

[GPL-3.0-or-later](https://www.gnu.org/licenses/gpl-3.0)
