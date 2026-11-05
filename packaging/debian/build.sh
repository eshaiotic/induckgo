#!/bin/sh
# Runs inside the Debian 13 build container started by packaging/release.sh. Builds the binary once and
# assembles each requested format from it. Arguments: one or more of "deb", "appimage".
#
# Debian 13 (Qt 6.8, glibc 2.41) is the oldest supported base, so packages built here also run on newer systems.
set -eu

export DEBIAN_FRONTEND=noninteractive
apt-get update -qq
# The build needs Qt's headers and code generators (moc, qmltyperegistrar, rcc); the AppImage also needs the
# runtime pieces it copies (QML modules, QtWebEngineProcess, SVG support) and squashfs-tools.
apt-get install -y -qq --no-install-recommends \
  build-essential clang dpkg-dev pkg-config qmake6 qt6-base-dev qt6-base-dev-tools qt6-declarative-dev \
  qt6-declarative-dev-tools qt6-webengine-dev qt6-webengine-dev-tools qt6-webchannel-dev qt6-svg-dev \
  qml6-module-qtquick qml6-module-qtquick-controls qml6-module-qtquick-layouts qml6-module-qtquick-dialogs \
  qml6-module-qtquick-effects qml6-module-qtquick-window qml6-module-qtquick-templates qml6-module-qtqml \
  qml6-module-qtqml-workerscript qml6-module-qtwebengine qml6-module-qtwebchannel qml6-module-qtcore \
  libqt6webenginecore6-bin qt6-svg-plugins qt6-qpa-plugins qt6-wayland squashfs-tools >/dev/null

export CARGO_TARGET_DIR=/work/debian-13
cargo build --release --locked -p induckgo-qt --manifest-path /src/Cargo.toml

for format in "$@"; do
  sh "/src/packaging/$format/assemble.sh" "$CARGO_TARGET_DIR/release/induckgo-qt"
done
