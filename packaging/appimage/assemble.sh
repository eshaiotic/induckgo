#!/bin/sh
# Assembles InDuckGo-<version>-x86_64.AppImage in /dist from the binary given as the only argument.
# Runs inside the Debian build container (packaging/debian/build.sh); VERSION comes from packaging/release.sh.
#
# An AppImage is a small runtime program followed by a SquashFS image of the application directory; the
# runtime mounts the image and starts its AppRun. The runtime is pinned to a release and verified by hash.
set -eu

runtime_url=https://github.com/AppImage/type2-runtime/releases/download/20251108/runtime-x86_64
runtime_sha256=2fca8b443c92510f1483a883f60061ad09b46b978b2631c807cd873a47ec260d

if [ "$(uname -m)" != x86_64 ]; then
  echo "AppImage: only x86_64 is supported (the pinned runtime is x86_64)" >&2
  exit 1
fi

binary=$1
work=$(mktemp -d)
image="/dist/InDuckGo-${VERSION}-x86_64.AppImage"

curl -fsSL --proto '=https' --tlsv1.2 -o "$work/runtime" "$runtime_url"
echo "$runtime_sha256  $work/runtime" | sha256sum --check --quiet

app=$work/InDuckGo.AppDir
install -Dm0755 /src/packaging/appimage/AppRun "$app/AppRun"
install -Dm0755 "$binary" "$app/usr/bin/induckgo"
install -Dm0644 /src/packaging/linux/io.github.induckgo.InDuckGo.desktop -t "$app"
install -Dm0644 /src/packaging/linux/io.github.induckgo.InDuckGo.desktop -t "$app/usr/share/applications"
install -Dm0644 /src/packaging/linux/io.github.induckgo.InDuckGo.metainfo.xml -t "$app/usr/share/metainfo"
install -Dm0644 /src/assets/icons/io.github.induckgo.InDuckGo.svg -t "$app"
install -Dm0644 /src/assets/icons/io.github.induckgo.InDuckGo.svg -t "$app/usr/share/icons/hicolor/scalable/apps"
install -Dm0644 /src/NOTICE /src/LICENSE -t "$app/usr/share/doc/induckgo"
ln -s io.github.induckgo.InDuckGo.svg "$app/.DirIcon"

mksquashfs "$app" "$work/image.squashfs" -root-owned -noappend -comp zstd -quiet
cat "$work/runtime" "$work/image.squashfs" > "$image"
chmod 0755 "$image"
echo "built $image"
