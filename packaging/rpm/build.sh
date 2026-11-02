#!/bin/sh
# Runs inside the Fedora build container started by packaging/release.sh: installs the spec's build
# dependencies and builds the RPM from /work/induckgo-<version>.tar.gz into /dist.
set -eu

dnf -y -q builddep /src/packaging/rpm/induckgo.spec >/dev/null
dnf -y -q install rpm-build >/dev/null

topdir=$(mktemp -d)
mkdir -p "$topdir/SOURCES"
cp "/work/induckgo-${VERSION}.tar.gz" "$topdir/SOURCES/"
rpmbuild --quiet -bb --define "_topdir $topdir" /src/packaging/rpm/induckgo.spec
for package in "$topdir"/RPMS/*/induckgo-*.rpm; do
  cp "$package" /dist/
  echo "built /dist/$(basename "$package")"
done
