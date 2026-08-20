PREFIX ?= $(HOME)/.local
BINDIR ?= $(PREFIX)/bin
DESKTOPDIR ?= $(PREFIX)/share/applications
ICONDIR ?= $(PREFIX)/share/icons/hicolor/scalable/apps
METAINFODIR ?= $(PREFIX)/share/metainfo
VERSION ?= 1.0.0
DIST_DIR ?= dist
ARCH ?= x86_64

export PKG_CONFIG_PATH := /home/linuxbrew/.linuxbrew/lib/pkgconfig:/home/linuxbrew/.linuxbrew/share/pkgconfig:$(PKG_CONFIG_PATH)
export PATH := $(HOME)/.cargo/bin:/home/linuxbrew/.linuxbrew/bin:$(PATH)

.PHONY: all build release test run install uninstall dist clean

all: build

build:
	cargo build

release:
	cargo build --release

test:
	cargo test -- --nocapture

run: build
	cargo run

install: release
	@echo "Installing M-Game Solo to $(PREFIX)..."
	mkdir -p $(BINDIR) $(DESKTOPDIR) $(ICONDIR) $(METAINFODIR)
	install -m 755 target/release/mgame-solo $(BINDIR)/mgame-solo
	install -m 644 data/com.mgame.Solo.desktop $(DESKTOPDIR)/com.mgame.Solo.desktop
	install -m 644 data/com.mgame.Solo.svg $(ICONDIR)/com.mgame.Solo.svg
	install -m 644 data/com.mgame.Solo.metainfo.xml $(METAINFODIR)/com.mgame.Solo.metainfo.xml
	update-desktop-database $(DESKTOPDIR) 2>/dev/null || true
	gtk-update-icon-cache -f -t $(PREFIX)/share/icons/hicolor 2>/dev/null || true
	@echo "Installation complete! Run 'mgame-solo' from terminal or application launcher."

uninstall:
	rm -f $(BINDIR)/mgame-solo
	rm -f $(DESKTOPDIR)/com.mgame.Solo.desktop
	rm -f $(ICONDIR)/com.mgame.Solo.svg
	rm -f $(METAINFODIR)/com.mgame.Solo.metainfo.xml
	update-desktop-database $(DESKTOPDIR) 2>/dev/null || true
	@echo "Uninstalled M-Game Solo."

dist: release
	@echo "Building universal release tarball for mgame-solo v$(VERSION)..."
	rm -rf $(DIST_DIR)
	mkdir -p $(DIST_DIR)/mgame-solo-v$(VERSION)-$(ARCH)-linux/bin
	mkdir -p $(DIST_DIR)/mgame-solo-v$(VERSION)-$(ARCH)-linux/data
	cp target/release/mgame-solo $(DIST_DIR)/mgame-solo-v$(VERSION)-$(ARCH)-linux/bin/
	cp data/com.mgame.Solo.desktop $(DIST_DIR)/mgame-solo-v$(VERSION)-$(ARCH)-linux/data/
	cp data/com.mgame.Solo.svg $(DIST_DIR)/mgame-solo-v$(VERSION)-$(ARCH)-linux/data/
	cp data/com.mgame.Solo.metainfo.xml $(DIST_DIR)/mgame-solo-v$(VERSION)-$(ARCH)-linux/data/
	cp data/99-mgame-solo.rules $(DIST_DIR)/mgame-solo-v$(VERSION)-$(ARCH)-linux/data/
	cp scripts/install.sh $(DIST_DIR)/mgame-solo-v$(VERSION)-$(ARCH)-linux/install.sh
	cp scripts/uninstall.sh $(DIST_DIR)/mgame-solo-v$(VERSION)-$(ARCH)-linux/uninstall.sh
	cp README.md $(DIST_DIR)/mgame-solo-v$(VERSION)-$(ARCH)-linux/ 2>/dev/null || true
	cd $(DIST_DIR) && tar -czvf mgame-solo-v$(VERSION)-$(ARCH)-linux.tar.gz mgame-solo-v$(VERSION)-$(ARCH)-linux
	cd $(DIST_DIR) && sha256sum mgame-solo-v$(VERSION)-$(ARCH)-linux.tar.gz > mgame-solo-v$(VERSION)-$(ARCH)-linux.tar.gz.sha256
	@echo "Release tarball created at $(DIST_DIR)/mgame-solo-v$(VERSION)-$(ARCH)-linux.tar.gz"

clean:
	cargo clean
	rm -rf $(DIST_DIR)
