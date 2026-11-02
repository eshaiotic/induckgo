#!/bin/sh
# Builds InDuckGo release packages into dist/, with SHA256SUMS.
#
#   arch      pacman package, built here with makepkg (makepkg.conf flags apply: for this machine)
#   deb       built in a Debian 13 container; installs on Debian 13, Ubuntu 24.04 LTS and newer
#   appimage  the Debian 13 build as an AppImage; the host needs WebKitGTK 6.0, GTK 4.14, libadwaita 1.5
#   rpm       built in a Fedora 44 container
#
# Usage: packaging/release.sh [arch] [deb] [appimage] [rpm]      (no argument: all of them)
# Containers run with rootless podman; CONTAINER_ENGINE=docker works too, but leaves root-owned files.
set -eu

repository=$(cd "$(dirname "$0")/.." && pwd -P)
engine=${CONTAINER_ENGINE:-podman}
debian_image=docker.io/library/rust:1-trixie
fedora_image=registry.fedoraproject.org/fedora:44
version=$(sed -n 's/^version = "\(.*\)"$/\1/p' "$repository/Cargo.toml" | head -n 1)
formats=${*:-arch deb appimage rpm}
work=$repository/target/containers
mkdir -p "$repository/dist" "$work"

# Runs a command in a build container: the repository read-only at /src, dist/ at /dist, scratch at /work,
# and a named volume that keeps downloaded crates between builds.
in_container() {
  image=$1 crates_volume=$2 crates_directory=$3
  shift 3
  "$engine" run --rm \
    --volume "$repository:/src:ro,z" --volume "$repository/dist:/dist:z" --volume "$work:/work:z" \
    --volume "$crates_volume:$crates_directory" \
    --env "VERSION=$version" \
    "$image" "$@"
}

debian_formats=""
for format in $formats; do
  case $format in
    arch)
      (cd "$repository/packaging/arch" &&
        BUILDDIR="$repository/target/makepkg" PKGDEST="$repository/dist" makepkg --syncdeps --force)
      ;;
    deb | appimage) debian_formats="$debian_formats $format" ;;
    rpm)
      git -C "$repository" ls-files -z --cached --others --exclude-standard |
        tar --create --gzip --file="$work/induckgo-$version.tar.gz" --transform="s,^,induckgo-$version/," \
          --directory="$repository" --null --files-from=-
      in_container "$fedora_image" induckgo-crates-fedora /root/.cargo/registry sh /src/packaging/rpm/build.sh
      ;;
    *)
      echo "unknown format: $format (expected arch, deb, appimage or rpm)" >&2
      exit 2
      ;;
  esac
done
if [ -n "$debian_formats" ]; then
  # shellcheck disable=SC2086 # one argument per format
  in_container "$debian_image" induckgo-crates-debian /usr/local/cargo/registry \
    sh /src/packaging/debian/build.sh $debian_formats
fi

(cd "$repository/dist" && sha256sum -- *.deb *.rpm *.AppImage *.pkg.tar.* 2>/dev/null > SHA256SUMS || true)
echo "packages in $repository/dist:"
ls -1 "$repository/dist"
