#!/bin/sh
# Copies Debian's tor (and, with --transports, obfs4proxy and snowflake-client, and with --libraries the
# libraries they load except the C library) into <prefix>/lib/induckgo/tor, where the Tor window looks first,
# and each copied package's licence into <prefix>/share/doc/induckgo/third-party.
# Usage: tor-bundle.sh <prefix> [--transports] [--libraries]       (runs in the Debian build container)
set -eu

prefix=$1
shift
bundle=$prefix/lib/induckgo/tor
licences=$prefix/share/doc/induckgo/third-party
tor=$(dpkg -L tor | grep -E '/s?bin/tor$')
programs="$tor"
for option in "$@"; do
  case $option in
    --transports) programs="$programs /usr/bin/obfs4proxy /usr/bin/snowflake-client" ;;
    --libraries) ;;
    *) echo "unknown option $option" >&2; exit 1 ;;
  esac
done

copy_licence() {
  package=$(dpkg -S "$(readlink -f "$1")" 2>/dev/null | head -n 1 | cut -d: -f1) || true
  if [ -n "$package" ]; then
    install -Dm0644 "/usr/share/doc/$package/copyright" "$licences/$package.copyright"
  fi
}

for program in $programs; do
  install -Dm0755 "$program" "$bundle/$(basename "$program")"
  copy_licence "$program"
done

case " $* " in
  *" --libraries "*)
    for library in $(ldd $programs | awk '$3 ~ /^\// { print $3 }' | sort -u); do
      case $(basename "$library") in
        libc.so.* | libm.so.* | libpthread.so.* | libdl.so.* | librt.so.* | ld-linux*) ;;
        *)
          install -Dm0644 "$library" "$bundle/lib/$(basename "$library")"
          copy_licence "$library"
          ;;
      esac
    done
    ;;
esac
