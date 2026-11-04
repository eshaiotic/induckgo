#!/bin/sh
# Assembles dist/InDuckGo-<version>-x86_64.AppImage for the Qt/Chromium build (version 1.2).
#
# Unlike the WebKitGTK AppImage, this one carries its engine: Qt 6, QtWebEngine (Chromium) and the QML modules
# the window uses are copied from the machine that builds it. The image therefore runs on any x86_64 Linux whose
# glibc is at least as new as the build machine's, with only the graphics stack (Mesa, X11/Wayland) taken from
# the host. Security updates of the engine arrive with a new InDuckGo release, not with the distribution.
#
# Usage: packaging/appimage/assemble-qt.sh      (after `cargo build --release -p induckgo-qt`)
set -eu

repository=$(cd "$(dirname "$0")/../.." && pwd -P)
version=$(sed -n 's/^version = "\(.*\)"$/\1/p' "$repository/Cargo.toml" | head -n 1)
binary=$repository/target/release/induckgo-qt
qt=/usr/lib/qt6
runtime_url=https://github.com/AppImage/type2-runtime/releases/download/20251108/runtime-x86_64
runtime_sha256=2fca8b443c92510f1483a883f60061ad09b46b978b2631c807cd873a47ec260d

[ -x "$binary" ] || { echo "build first: cargo build --release -p induckgo-qt" >&2; exit 1; }
[ "$(uname -m)" = x86_64 ] || { echo "only x86_64 is supported" >&2; exit 1; }

work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT
app=$work/InDuckGo.AppDir
mkdir -p "$app/usr/bin" "$app/usr/lib" "$app/usr/lib/qt6/plugins" "$app/usr/lib/qt6/qml" "$app/usr/share/qt6"

install -m0755 "$binary" "$app/usr/bin/induckgo"
install -m0755 "$qt/QtWebEngineProcess" "$app/usr/lib/qt6/QtWebEngineProcess"
cp -a /usr/share/qt6/resources "$app/usr/share/qt6/resources"
mkdir -p "$app/usr/share/qt6/translations"
cp -a /usr/share/qt6/translations/qtwebengine_locales "$app/usr/share/qt6/translations/qtwebengine_locales"

# Qt plugins the window needs: platforms (X11, Wayland, offscreen), image formats, TLS, input methods.
for plugin in platforms imageformats tls platforminputcontexts xcbglintegrations iconengines \
  wayland-decoration-client wayland-graphics-integration-client wayland-shell-integration; do
  if [ -d "$qt/plugins/$plugin" ]; then cp -a "$qt/plugins/$plugin" "$app/usr/lib/qt6/plugins/$plugin"; fi
done
# The desktop's own file chooser and colour scheme come through the portal; without it Qt draws its own dialog.
mkdir -p "$app/usr/lib/qt6/plugins/platformthemes"
if [ -e "$qt/plugins/platformthemes/libqxdgdesktopportal.so" ]; then
  cp "$qt/plugins/platformthemes/libqxdgdesktopportal.so" "$app/usr/lib/qt6/plugins/platformthemes/"
fi
# QML modules: Qt Quick with Controls, Layouts, Window, Templates, QtQml and the web view.
for module in QtQml QtQuick QtWebEngine QtCore Qt; do
  if [ -d "$qt/qml/$module" ]; then cp -a "$qt/qml/$module" "$app/usr/lib/qt6/qml/$module"; fi
done
# Only the Controls styles that need nothing beyond Qt itself.
find "$app/usr/lib/qt6/qml/QtQuick/Controls" -mindepth 1 -maxdepth 1 -type d \
  ! -name Basic ! -name Fusion ! -name impl ! -name designer -exec rm -rf {} +

# Every shared library any of those files needs, except what must come from the host.
excluded='^(linux-vdso|ld-linux|libc|libm|libdl|libpthread|librt|libresolv|libutil|libanl|libnss_[a-z]+|libGL|libEGL|libGLX|libGLdispatch|libOpenGL|libvulkan|libdrm|libgbm|libwayland-egl|libwayland-client|libwayland-server|libwayland-cursor|libX11|libX11-xcb|libxcb|libXau|libXdmcp|libnvidia[a-z-]*|libcuda)\.'
find "$app" -type f \( -name '*.so' -o -name '*.so.*' -o -name induckgo -o -name QtWebEngineProcess \) |
  while read -r file; do ldd "$file" 2>/dev/null; done |
  awk '/=> \// {print $3}' | sort -u |
  while read -r library; do
    name=$(basename "$library")
    printf '%s\n' "$name" | grep -Eq "$excluded" && continue
    [ -e "$app/usr/lib/$name" ] || cp -L "$library" "$app/usr/lib/$name"
  done
# Libraries the copied ones need in turn (Qt's own are resolved above; this closes the loop once more).
find "$app/usr/lib" -maxdepth 1 -type f -name '*.so*' |
  while read -r file; do ldd "$file" 2>/dev/null; done |
  awk '/=> \// {print $3}' | sort -u |
  while read -r library; do
    name=$(basename "$library")
    printf '%s\n' "$name" | grep -Eq "$excluded" && continue
    [ -e "$app/usr/lib/$name" ] || cp -L "$library" "$app/usr/lib/$name"
  done
strip --strip-unneeded "$app"/usr/lib/*.so* 2>/dev/null || true

install -Dm0755 "$repository/packaging/appimage/AppRun.qt" "$app/AppRun"
install -Dm0644 "$repository/packaging/linux/io.github.induckgo.InDuckGo.desktop" -t "$app"
install -Dm0644 "$repository/packaging/linux/io.github.induckgo.InDuckGo.desktop" -t "$app/usr/share/applications"
install -Dm0644 "$repository/packaging/linux/io.github.induckgo.InDuckGo.metainfo.xml" -t "$app/usr/share/metainfo"
install -Dm0644 "$repository/assets/icons/io.github.induckgo.InDuckGo.svg" -t "$app"
install -Dm0644 "$repository/assets/icons/io.github.induckgo.InDuckGo.svg" -t "$app/usr/share/icons/hicolor/scalable/apps"
install -Dm0644 "$repository/NOTICE" "$repository/LICENSE" -t "$app/usr/share/doc/induckgo"
ln -s io.github.induckgo.InDuckGo.svg "$app/.DirIcon"

curl -fsSL --proto '=https' --tlsv1.2 -o "$work/runtime" "$runtime_url"
echo "$runtime_sha256  $work/runtime" | sha256sum --check --quiet
mksquashfs "$app" "$work/image.squashfs" -root-owned -noappend -comp zstd -quiet
mkdir -p "$repository/dist"
image=$repository/dist/InDuckGo-${version}-x86_64.AppImage
cat "$work/runtime" "$work/image.squashfs" > "$image"
chmod 0755 "$image"
(cd "$repository/dist" && sha256sum -- *.deb *.rpm *.AppImage *.pkg.tar.* 2>/dev/null > SHA256SUMS || true)
echo "built $image ($(du -h "$image" | cut -f1))"
