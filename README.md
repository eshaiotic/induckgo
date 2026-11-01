# InDuckGo

A privacy-first web browser for Linux. It rebuilds the protections of the open-source DuckDuckGo macOS
browser on top of **WebKitGTK 6.0** (the Linux port of the same WebKit engine Safari and the DuckDuckGo
macOS app use), with a **GTK4 + libadwaita** user interface written in **Python**.

InDuckGo is an independent project. It is not affiliated with or endorsed by Duck Duck Go, Inc.

## What it protects you from

| Protection | How it works |
|---|---|
| Third-party trackers | DuckDuckGo's Tracker Data Set is translated into WebKit content-blocking rules (port of `TrackerRadarKit`). |
| Breakage | DuckDuckGo's privacy configuration supplies per-site exceptions so sites keep working. |
| Fingerprinting, GPC | DuckDuckGo Content Scope Scripts are injected into every frame. |
| Cross-site cookies | Third-party cookies are refused; WebKit Intelligent Tracking Prevention is on. |
| Tracking links | `utm_*`, `fbclid`, `gclid`, ... are stripped from links before navigation. |
| Plain HTTP | Navigations are upgraded to HTTPS, falling back to HTTP only when the site has no HTTPS. |
| Leftover data | The Fire button erases cookies, storage, cache and history in one click. |

There is no extension support — by design, and because WebKitGTK has no extension API.

## Status

Work in progress. The browser-independent core (`crates/induckgo-core`: tracker-blocking rules, privacy
configuration, link cleaning, address bar, HTTPS upgrades) is implemented and tested; the GTK application
crate is next. Planned: isolated profiles and installing any site as its own isolated app (PWA).

## Development

```sh
sudo pacman -S --needed rust webkitgtk-6.0 gtk4 libadwaita
cargo test -p induckgo-core     # domain tests
cargo clippy --all-targets      # lints
cargo audit                     # dependency vulnerabilities
```

Documentation index: [CLAUDE.md](CLAUDE.md).

## Licences of third-party data

- Content Scope Scripts — Apache-2.0, to be vendored with the application crate.
- Privacy configuration — Apache-2.0, downloaded at runtime.
- Tracker Data Set — **CC BY-NC-SA 4.0** (non-commercial), downloaded at runtime, never committed.
