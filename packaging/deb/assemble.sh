#!/bin/sh
# Assembles induckgo_<version>-1_<arch>.deb in /dist from the binary given as the only argument.
# Runs inside the Debian build container (packaging/debian/build.sh); VERSION comes from packaging/release.sh.
set -eu

binary=$1
architecture=$(dpkg --print-architecture)
package="/dist/induckgo_${VERSION}-1_${architecture}.deb"
root=$(mktemp -d)

install -Dm0755 "$binary" "$root/usr/bin/induckgo"
install -Dm0644 /src/packaging/linux/io.github.induckgo.InDuckGo.desktop -t "$root/usr/share/applications"
install -Dm0644 /src/packaging/linux/io.github.induckgo.InDuckGo.metainfo.xml -t "$root/usr/share/metainfo"
install -Dm0644 /src/assets/icons/io.github.induckgo.InDuckGo.svg -t "$root/usr/share/icons/hicolor/scalable/apps"
install -Dm0644 /src/NOTICE /src/README.md -t "$root/usr/share/doc/induckgo"
cat > "$root/usr/share/doc/induckgo/copyright" <<'EOF'
Format: https://www.debian.org/doc/packaging-manuals/copyright-format/1.0/
Upstream-Name: InDuckGo

Files: *
Copyright: 2026 Michał Ligus
License: GPL-3.0-or-later

Files: assets/content-scope-scripts/*
Copyright: 2010-2021 Duck Duck Go, Inc.
License: Apache-2.0

License: GPL-3.0-or-later
 On Debian systems, the full text of the GNU General Public License, version 3 can be found in
 /usr/share/common-licenses/GPL-3.

License: Apache-2.0
 On Debian systems, the full text of the Apache License, Version 2.0 can be found in
 /usr/share/common-licenses/Apache-2.0. See /usr/share/doc/induckgo/NOTICE for attributions.
EOF

# Library dependencies with minimum versions, read from the binary by dpkg-shlibdeps. Qt Quick loads its QML
# modules, QtWebEngine its helper process and the interface its SVG icons at run time, so no library link names
# them: they are listed here, and the list is checked by installing the package in a clean container.
qt_runtime="qml6-module-qtquick, qml6-module-qtquick-controls, qml6-module-qtquick-layouts,
 qml6-module-qtquick-dialogs, qml6-module-qtquick-effects, qml6-module-qtquick-window,
 qml6-module-qtquick-templates, qml6-module-qtqml, qml6-module-qtqml-workerscript, qml6-module-qtwebengine,
 qml6-module-qtwebchannel, qml6-module-qtcore, libqt6webenginecore6-bin, qt6-svg-plugins, qt6-qpa-plugins,
 qt6-wayland"
work=$(mktemp -d)
mkdir "$work/debian"
printf 'Source: induckgo\n\nPackage: induckgo\nArchitecture: any\n' > "$work/debian/control"
libraries=$(cd "$work" && dpkg-shlibdeps -O "$root/usr/bin/induckgo" 2>/dev/null |
  sed -n 's/^shlibs:Depends=//p')

mkdir "$root/DEBIAN"
cat > "$root/DEBIAN/control" <<EOF
Package: induckgo
Version: ${VERSION}-1
Architecture: ${architecture}
Maintainer: Michał Ligus <eshaiotic@eshaiotic.io>
Installed-Size: $(du -sk "$root/usr" | cut -f1)
Depends: ${libraries}, ${qt_runtime}, hicolor-icon-theme
Section: web
Priority: optional
Description: privacy-first web browser on QtWebEngine (Chromium)
 InDuckGo blocks third-party trackers with DuckDuckGo's open-source
 protections, upgrades connections to HTTPS and removes tracking parameters
 from links. Every profile and every site installed as an app runs in its
 own process with its own cookies and storage.
EOF

dpkg-deb --root-owner-group --build "$root" "$package" >/dev/null
echo "built $package"
