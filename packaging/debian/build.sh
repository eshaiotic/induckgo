#!/bin/sh
# Runs inside the Debian 13 build container started by packaging/release.sh. Builds the binary once and
# assembles each requested format from it. Arguments: one or more of "deb", "appimage".
#
# Debian 13 is the oldest supported base, so packages built here also run on newer systems.
set -eu

export DEBIAN_FRONTEND=noninteractive
apt-get update -qq
# tor, obfs4proxy and snowflake-client are copied into the packages for the Tor window (see
# packaging/debian/tor-bundle.sh); in the container their services never start.
apt-get install -y -qq --no-install-recommends \
  dpkg-dev libadwaita-1-dev libgtk-4-dev libsoup-3.0-dev libwebkitgtk-6.0-dev squashfs-tools \
  tor obfs4proxy snowflake-client >/dev/null

export CARGO_TARGET_DIR=/work/debian-13
cargo build --release --locked -p induckgo --manifest-path /src/Cargo.toml

for format in "$@"; do
  sh "/src/packaging/$format/assemble.sh" "$CARGO_TARGET_DIR/release/induckgo"
done
