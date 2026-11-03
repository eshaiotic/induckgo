# InDuckGo

A privacy-first web browser for Linux. It rebuilds the protections of the open-source DuckDuckGo macOS
browser on top of **WebKitGTK 6.0** (the Linux port of the same WebKit engine Safari and the DuckDuckGo
macOS app use), with a **GTK 4 + libadwaita** user interface written in **Rust**.

InDuckGo is an independent project. It is not affiliated with or endorsed by Duck Duck Go, Inc.

## What it protects you from

| Protection | How it works |
|---|---|
| Third-party trackers | DuckDuckGo's Tracker Data Set is translated into WebKit content-blocking rules (port of `TrackerRadarKit`). |
| Breakage | DuckDuckGo's privacy configuration supplies per-site exceptions so sites keep working; the shield button (in a web app: *Protect This Site* in its menu) switches protection off for one site. |
| Fingerprinting, GPC | DuckDuckGo Content Scope Scripts run in every frame; InDuckGo Guard switches on the canvas, audio, screen and referrer defences DuckDuckGo leaves off, and sends the `Sec-GPC: 1` header. |
| Ads, trackers, cookie notices | InDuckGo Guard downloads EasyList, EasyPrivacy, uBlock Origin's, Peter Lowe's, Polish and cookie-notice filter lists and applies them to a site's own requests too. |
| Cross-site cookies | Third-party cookies are refused; WebKit Intelligent Tracking Prevention is on. |
| Tracking links | A navigation carrying `utm_*`, `fbclid`, `gclid`, ... is stopped before it leaves the machine and reloaded without them. |
| Plain HTTP | Navigations are upgraded to HTTPS; if a site has no HTTPS, a banner asks before loading it insecurely. |
| Leftover data | The Fire button erases the profile's cookies, storage, cache and favicons in one click. |
| Mixing identities | Every profile and every installed web app runs as its own process with its own cookies and storage. |

There is no extension support — by design, and because WebKitGTK has no extension API.

**Web apps.** *Install Site as App* (the button at the right end of the address bar, or the window menu)
turns any site — your bank, your mailbox — into a desktop application with the site's icon. Each app has
its own isolated session; a link you click inside it to another site opens in the browser, so the app's
login never meets foreign pages.

## Status

Usable work in progress: browsing with tabs, all protections above, profiles, web apps, downloads,
permission prompts and find-in-page work. Not built yet: session restore, bookmarks, history window,
cookie-consent handling, translations. The full list with reasons is in the architecture document.

## Installing

On Arch Linux and its derivatives (CachyOS, EndeavourOS, Manjaro):

```sh
./install                 # builds the pacman package from this checkout and installs it
./uninstall               # removes the package; profiles and web apps are kept
./uninstall --purge       # also deletes all InDuckGo data and web app launchers
```

On Debian 13, Ubuntu 24.04 LTS or newer and on Fedora 44, `./install` installs the `.deb` / `.rpm` that
`packaging/release.sh` built into `dist/`. On any other distribution, use the AppImage:
`chmod +x InDuckGo-*-x86_64.AppImage && ./InDuckGo-*-x86_64.AppImage`. It uses the system's WebKitGTK 6.0,
GTK ≥ 4.14 and libadwaita ≥ 1.5 (oldest tested: Ubuntu 24.04 LTS). Those must be installed, but in return the browser engine keeps receiving
your distribution's security updates.

## Building release packages

```sh
packaging/release.sh                # all formats into dist/, with SHA256SUMS
packaging/release.sh deb appimage   # or only some: arch, deb, appimage, rpm
```

The `.deb` and the AppImage are built in a Debian 13 container and the `.rpm` in a Fedora 44 container
(rootless podman), so they link against the libraries of the systems they target. The Arch package is
built on the host with `makepkg` and its `makepkg.conf` flags. The AppImage runtime is pinned to a release
and checked against its SHA-256.

## Running from source

```sh
sudo pacman -S --needed rust webkitgtk-6.0 gtk4 libadwaita
cargo run -p induckgo                          # browser, default profile
cargo run -p induckgo -- --profile work        # browser, profile "work"
cargo run -p induckgo -- https://example.com/  # open a page
```

Installed web apps start from the desktop's application menu (`induckgo --web-app <id>`).

## Development

```sh
cargo test -p induckgo-core     # domain tests
cargo clippy --all-targets      # lints (pedantic, unsafe code forbidden)
cargo audit                     # dependency vulnerabilities
```

- [Architecture](docs/en/architecture/overview.md) — processes, isolation, the protection pipeline, web apps, limitations.
- [Engineering rules](docs/en/engineering-rules.md) — Object Oriented Programming First, OO Given/When/Then tests.
- Polish mirrors live under [docs/pl](docs/pl).

## Licence

InDuckGo is free software under the [GNU General Public License, version 3 or later](LICENSE)
(`GPL-3.0-or-later`). Attributions are in [NOTICE](NOTICE).

## Licences of third-party data

- Content Scope Scripts 20.0.0 — Apache-2.0, vendored in `assets/content-scope-scripts` with its licence.
- Privacy configuration — Apache-2.0, downloaded at runtime.
- Tracker Data Set — **CC BY-NC-SA 4.0** (non-commercial), downloaded at runtime, never committed.
- InDuckGo Guard's filter lists — GPL-3.0, CC BY-SA 3.0, CC BY 3.0, CC BY-NC-SA 4.0 or their publisher's terms,
  downloaded at runtime, never committed.
- adblock-rust 0.13.5 — MPL-2.0, compiled in unmodified (source: <https://github.com/brave/adblock-rust>).
