**English** | [Polski](README.pl.md)

<p align="center">
  <img src="assets/icons/io.github.induckgo.InDuckGo.svg" alt="InDuckGo: an indigo shield with a white duck" width="160">
</p>

# InDuckGo

**A privacy-first web browser for Linux.** It rebuilds the protections of the open-source DuckDuckGo macOS
browser on **QtWebEngine** (Chromium, the engine of Chrome and Brave), with a **Qt 6 Quick** user interface and a
domain core written in **Rust**. No telemetry, no extensions to manage, every profile and installed web app kept
apart from the others. The first generation ran on WebKitGTK; its code and its 1.2.0 packages are still in this
repository while the Qt edition catches up.

[![tests](https://github.com/eshaiotic/induckgo/actions/workflows/tests.yml/badge.svg)](https://github.com/eshaiotic/induckgo/actions/workflows/tests.yml)
[![release](https://img.shields.io/github/v/release/eshaiotic/induckgo?color=6A5ACD)](https://github.com/eshaiotic/induckgo/releases/latest)
![cargo nextest](https://img.shields.io/badge/cargo%20nextest-781%20tests-success)
![stack](https://img.shields.io/badge/stack-Rust%20%E2%80%A2%20Qt%206%20%E2%80%A2%20QtWebEngine-8E7FE8)
![telemetry](https://img.shields.io/badge/telemetry-none-FFB300)
[![licence](https://img.shields.io/badge/licence-GPL--3.0--or--later-2E1A87)](LICENSE)

InDuckGo is an independent project. It is not affiliated with or endorsed by Duck Duck Go, Inc.

---

## Measured against Chrome and Brave, compared with DuckDuckGo

On 2026-11-05 fresh profiles of Google Chrome 157, Brave 1.98 (Chromium 156) and InDuckGo ran at the same time,
each in its own window behind a proxy that recorded only host names, for 5 minutes of doing nothing and then
20 seconds on each of 12 popular sites. Trackers were judged by a neutral list (Disconnect's), each server
counted once per site. The run was done twice (Chrome and Brave in both, so the two runs show how much a single
run varies), because InDuckGo's Qt edition needs protection data that it does not download itself yet.

| | Chrome | Brave | InDuckGo Qt, fresh profile | InDuckGo Qt, protection data on disk | InDuckGo GTK (the 1.2.0 packages) |
|---|---|---|---|---|---|
| Trackers let through on 12 sites | 111 / 109 | 9 / 8 | 90 | 31 | **6** |
| … of which ad servers | 90 / 88 | 7 / 7 | 73 | 31 | **5** |
| Connections in the first 5 minutes | 35 / 36 to 14 hosts | 47 / 53 to 12 hosts | **2 to 2 hosts** | **2 to 2 hosts** | 27 to 8 hosts |

Two numbers per Chrome and Brave cell are the first and the second run.

**How to read it.** The Qt edition, which is what InDuckGo is now built on, blocks much less than the WebKitGTK
edition it replaces. With a fresh profile it blocks almost nothing, because it has no downloader of its own for
DuckDuckGo's tracker list and Guard's filter lists yet (it reads what the GTK edition left in the cache). With the
data on disk it blocks about four trackers in five (31, against Chrome's 109), and still lets through advertising
servers that the GTK edition stopped (on bbc.com, 8 against 1). That gap is a defect to close, tracked in the
[migration plan](docs/en/plans/2026-11-04-qtwebengine-migration-plan.md) (Tasks 8 and 9), and the table is here so
that nobody has to take the project's word for it. Idle traffic is the strong side: the Qt edition contacted only
the DuckDuckGo start page and its ping host; Chrome contacted Google and Brave its own servers (usage ping, P3A).
The GTK edition's 27 connections are its first-run downloads of protection data. Fingerprinting was not measured
again: the Qt edition has no fingerprint protection yet, and the GTK edition's result (noise that differs per
site and per session, measured 2026-11-02) is in the [tracking comparison](docs/en/reports/tracking-comparison.md).
Chrome 157 and Brave 1.98 are different Chromium versions, 157 and 156. One run from one country on one network
is not a ranking; method and limits are in the report.

[tracking-summary.html](docs/en/reports/tracking-summary.html) is a one-page summary for non-technical readers
(download it and open it in a browser).

### Compared with DuckDuckGo's own browser

DuckDuckGo makes no browser for Linux (Windows, macOS, iOS and Android only), so its browser was **not measured**.
The table sets InDuckGo against what DuckDuckGo says about its own browser, in the feature list of its
[comparison page](https://duckduckgo.com/compare-privacy) ("See how DuckDuckGo compares") and in its
[web tracking protections help page](https://duckduckgo.com/duckduckgo-help-pages/privacy/web-tracking-protections),
both read in November 2026. The DuckDuckGo column is DuckDuckGo's own claim, for its Windows and macOS apps
(where they differ, macOS is named); the InDuckGo column is what the Qt edition does today, checked in the code.
For independent numbers on DuckDuckGo's browser, see [privacytests.org](https://privacytests.org/), which tests
DuckDuckGo, Brave and Chrome on desktop and does not test InDuckGo.

| | DuckDuckGo's browser says | InDuckGo (Qt) today |
|---|---|---|
| **Tracking scripts** | | |
| Blocks 3rd-party trackers from loading | yes, from its tracker list | yes, by the same list and by Guard's filter lists, on a site's own requests too; **only when the protection data is on disk** (no downloader in the Qt edition yet) |
| Blocks 3rd-party tracker cookies | yes | tracker-list cookies are refused when the data is on disk; the *Strict* profile (no interface yet) refuses all of them |
| Expires 1st-party cookies | macOS: 24 hours or 7 days; Windows: roadmap | no |
| Prevents fingerprint tracking | listed scripts blocked, listed APIs overridden; canvas and audio defences off on macOS | **not yet** in the Qt edition (the GTK edition had it, with noise per site and session) |
| Protects against CNAME cloaking | a subset of its list on macOS | by the tracker list's cloaked host names, when the data is on disk |
| Protects against referrer tracking | referrer cut to the host name | Chromium's own default only |
| Blocks link tracking | first navigation only on macOS; Windows: roadmap | every top-level navigation, when the data is on disk |
| Blocks Google's AMP tracking | macOS: AMP links replaced | no |
| Blocks Google's sign-in pop-ups | yes | no |
| Blocks embedded Facebook content | yes | Facebook is on the tracker list; no click-to-load placeholder |
| **Ads and pop-ups** | | |
| Blocks cookie pop-ups | closes the pop-ups on its list, choosing the private option | hides them through a filter list, does not answer them; **answering them automatically (rejecting) is the next feature** |
| Blocks targeted and non-tracking ads | says it attempts to block all ads, with a documented Microsoft exception: `bat.bing.com` loads on an advertiser's site after a click on a DuckDuckGo search ad | Guard's filter lists (EasyList, uBlock Origin and others), with no ad-click exception |
| Watches YouTube without ads | yes (Duck Player) | no |
| **Control** | | |
| Global Privacy Control | yes (macOS: the header only to sites known to honour it) | **not yet** in the Qt edition (the GTK edition sent it to every site) |
| Deletes browsing data with one button | Fire button | Burn button |
| Upgrades to HTTPS | the sites on its upgrade list (about 90 % of search clicks) | every navigation except the sites DuckDuckGo's configuration excludes |
| Encrypts DNS queries | yes | not yet (Chromium's secure DNS is not set up) |
| Stores passwords locally, syncs privately | yes | no |
| Usage data sent to its maker | anonymous usage metrics | none |
| **Beyond the browser** | | |
| Private search by default | yes | DuckDuckGo is the default search |
| Scam and phishing protection | Scam Blocker, a Netcraft list checked on the device | no |
| Email protection | free `@duck.com` aliases, sign-up | no |
| VPN | in the paid subscription | not in the Qt edition yet; the GTK edition has a Tor window |
| Removes personal details from data brokers, helps restore an identity | in the paid subscription (the identity service for U.S. residents) | no; planned, see *Next release* below |
| Runs on Linux | no | yes |

In short: DuckDuckGo's browser wins on finished protections today, and InDuckGo wins on what it will not do
(telemetry, ad-click exceptions, paid tiers) and on running on Linux; the finished-protection gap is the work in
front of this project, not something the numbers above hide.

---

## What it protects you from

InDuckGo's engine is now **QtWebEngine** (Chromium), with the interface in Qt Quick. The protections below were
built for the WebKitGTK edition; the last column says how far the Qt edition has them.

| Protection | How it works | In the Qt edition |
|---|---|---|
| Third-party trackers | DuckDuckGo's Tracker Data Set decides, request by request, which third-party requests are refused (port of `TrackerRadarKit`). | works once the data is on disk; no downloader yet |
| Breakage | DuckDuckGo's privacy configuration supplies per-site exceptions so sites keep working; the shield button (in a web app: *Protect This Site* in its menu) switches protection off for one site. | works |
| Fingerprinting, GPC | DuckDuckGo Content Scope Scripts run in every frame; InDuckGo Guard switches on the canvas, audio, screen and referrer defences DuckDuckGo leaves off, and sends the `Sec-GPC: 1` header. | not yet |
| Ads, trackers, cookie notices | InDuckGo Guard downloads EasyList, EasyPrivacy, uBlock Origin's, Peter Lowe's, Polish and cookie-notice filter lists and applies them to a site's own requests too. | works once the lists are on disk; no downloader yet |
| Cross-site cookies | Third-party cookies on the tracker list are refused; the *Strict* profile, which refuses all of them, exists in the code but has no interface yet. | works with data |
| Tracking links | A navigation carrying `utm_*`, `fbclid`, `gclid`, ... is stopped before it leaves the machine and reloaded without them. | works with data |
| Plain HTTP | Navigations are upgraded to HTTPS. | works |
| Leftover data | The Burn button erases the profile's cookies, storage, cache and favicons in one click. | works |
| Mixing identities | Every profile and every installed web app runs as its own process with its own cookies and storage. | works |
| Your IP address | The Tor window sends everything through InDuckGo's own Tor; its session lives in memory only. | not yet (the GTK edition has it) |

There is no extension support, by design.

**Web apps.** *Install Site as App* (the button at the right end of the address bar, or the window menu)
turns any site — your bank, your mailbox — into a desktop application with the site's icon. Each app has
its own isolated session; a link you click inside it to another site opens in the browser, so the app's
login never meets foreign pages. *Installed Apps* in the menu shows them as tiles, with Open and Uninstall.

**Tor window (GTK edition; coming to the Qt edition).** The onion button on the toolbar, *New Tor Window*
(`Alt+Shift+N`) or `induckgo --tor` opens a window whose every request goes through Tor, so sites see a Tor
exit relay instead of your address. InDuckGo starts its own `tor` for it, never a system-wide Tor service,
and shows the connection's progress until Tor is ready; until then no page loads, and nothing ever falls back to a
direct connection. Its cookies, cache and settings live in memory only, WebRTC is off, it searches DuckDuckGo's
onion service and opens `.onion` sites. Where Tor is blocked, *Tor Connection…* offers Snowflake, obfs4 or bridges
of your own. It hides your address, not that you use InDuckGo: where being indistinguishable among Tor users
matters, use Tor Browser.

**Preferences** (`Ctrl+,`), separately in every profile: which toolbar buttons show, whether the address bar is
wide or short and centred, **where the tabs sit** (top, bottom, left or right; top by default), **how the window
buttons look** (the default round buttons on the right, or three coloured dots on the left as on macOS), smooth
mouse-wheel scrolling, the download folder, and the language of the interface (68 languages).

---

## How tracking is cut, and what proves it

A site, an advertiser or the browser's own maker can follow you in seven ways. This table names each way, the
mechanism InDuckGo uses against it, where that mechanism lives in the code, and the check that exercises it. The
decisions live in the engine-neutral domain crate, so the WebKitGTK edition and the Qt edition share them; what the
Qt edition still lacks is listed in the [protection table](#what-it-protects-you-from) above and in the
[migration plan](docs/en/plans/2026-11-04-qtwebengine-migration-plan.md).

| Way of following you | What InDuckGo does | Code (`crates/induckgo-core/src`) | Checked by |
|---|---|---|---|
| The browser reports to its maker | There is no telemetry code at all: no usage ping, crash reporter, experiment seed or account. The only connections the program makes on its own are plain `GET`s for protection data, with no cookie jar and no identifier. | `protection/`, `guard/` (downloads only) | Idle traffic measured at host level: the Qt edition contacts 2 hosts, the DuckDuckGo start page and its ping host ([report §4.1](docs/en/reports/tracking-comparison.md) for the GTK edition) |
| Third-party trackers | DuckDuckGo's Tracker Data Set decides each request (WebKit content-blocking rules in the GTK edition, a request interceptor in the Qt edition), with DuckDuckGo's per-site exceptions so pages keep working. | `protection/tracker_data_set.rs`, `protection/content_blocker_rules_builder.rs`, `network/tracker_request_policy.rs` | Scenario tests `tracker_data_set`, `content_blocker_rules_builder`, `rule_exceptions` |
| Ads and trackers the first list misses | Guard downloads nine community lists (EasyList, EasyPrivacy, uBlock Origin, Peter Lowe's, Polish and cookie-notice lists), compiles them and applies them to a site's own requests too. Cookie notices are hidden. | `guard/guard_engine.rs`, `guard/guard_rule_list.rs`, `network/guard_request_policy.rs` | Tests `guard_engine`, `guard_rule_list`, `guard_request_policy`; GTK edition: 6 trackers on 12 sites against Chrome's 109 (above) |
| Cookies and stored identifiers | Third-party cookies on the tracker list are refused (the GTK edition also had WebKit's Intelligent Tracking Prevention), every profile and web app is its own process with its own storage, and Burn erases a profile in one click. | `storage/cookie_policy.rs`, `storage/tracker_cookie_policy.rs`, `storage/burn_planner.rs` | Tests `cookie_policies`, `burn_planner` |
| Identifiers in links | `utm_*`, `fbclid`, `gclid` and DuckDuckGo's other parameters are stripped from every top-level navigation before the request leaves the machine. | `protection/tracking_parameter_remover.rs`, `network/link_cleaning_request_policy.rs` | Tests `tracking_parameter_remover`, `link_cleaning_request_policy` |
| Fingerprinting | DuckDuckGo's Content Scope Scripts run in every frame, and Guard switches on the canvas, audio, screen and referrer defences that DuckDuckGo's macOS configuration leaves off. The noise changes per site and per session. | `guard/fingerprint_hardening.rs`, `protection/content_scope_script.rs` | Tests `fingerprint_hardening`, `content_scope_script`; probe page in the report (§4.3) |
| What the network and the site's server see | Navigations are upgraded to HTTPS and a banner asks before a site without HTTPS loads; `Sec-GPC: 1` goes to every site. For the IP address, the Tor window routes everything through InDuckGo's own Tor, never falls back to a direct connection, keeps nothing on disk, turns WebRTC off and asks for English like Tor Browser. | `navigation/https_upgrade_policy.rs`, `network/https_upgrade_request_policy.rs`, `tor/` | Tests `https_upgrade_policy`, `https_upgrade_request_policy`, `tor_configuration`, `tor_endpoint`; [architecture §4.1](docs/en/architecture/overview.md) |

Every list and data set is downloaded, never bundled, and size-limited and validated before use. A filter list
can only block or hide: the conversion keeps no scriptlets, so a hostile list could break a site but never run
code in it. The whole suite (`./run-tests`, 682 tests in CI plus the 99 Qt tests run locally, clippy and `cargo audit`) runs on every push.

### The evidence

The measurement behind the [first table](#measured-against-chrome-and-brave-compared-with-duckduckgo) ran Chrome 157,
Brave 1.98 and InDuckGo side by side on 2026-11-05, with fresh profiles, a window each, and a proxy that logs host
names only. Its numbers, repeated here so they sit next to the mechanisms:

- **Trackers reached on 12 sites:** Chrome 111 and 109 in two runs, Brave 9 and 8, InDuckGo's Qt edition 90 on a
  fresh profile and 31 with the protection data on disk, its GTK edition 6, judged by Disconnect's list, which none
  of them uses.
- **Sent to the browser's own maker in 5 idle minutes:** Chrome contacted only Google (14 hosts), Brave mostly its
  own servers (usage ping, P3A), the Qt edition only the DuckDuckGo start page and its ping host (2 hosts).
- **Fingerprint (GTK edition, 2026-11-02):** the same canvas and audio values on every site in Chrome; different per
  site, but equal after a restart, in Brave; different per site *and* per session in InDuckGo.
- **Method, limits and a script-level recipe to repeat it:** [tracking comparison](docs/en/reports/tracking-comparison.md), §3 and §8.

### What this does not prove

The measurement supports "InDuckGo gives its own maker nothing to work with, and, once its protection data is in
place, trackers little". It does not support "InDuckGo is more anonymous than Brave or DuckDuckGo", and the
project does not claim it.

- **The Qt edition blocks less than Brave today** (31 trackers against 8, and 90 on a fresh profile against 9).
  Closing that gap, which the GTK edition had closed (6), is the next piece of work.
- **One run on 12 sites is not a ranking.** Chrome's two runs differ by 2, Brave's by 1; pages change between loads.
- **DuckDuckGo's browser was not measured**, because it does not exist for Linux. The
  [feature table](#compared-with-duckduckgos-own-browser) compares documented behaviour, not traffic.
- **Blocking is not anonymity.** InDuckGo is a Chromium engine like Brave and Chrome, which is good for blending in,
  but the Qt edition does not yet change what a site can read about the browser: no fingerprint noise, no GPC.
  Installed fonts stay visible, as in Brave and Chrome.
- **No phishing or malware warnings** (no Safe Browsing), no email-alias service and no VPN yet. The Tor window,
  once it is back in the Qt edition, hides your address, not that you use InDuckGo; where being indistinguishable
  matters, use Tor Browser.
- **Independent verification is missing:** [privacytests.org](https://privacytests.org/) does not test InDuckGo.

---

## Status

Usable work in progress, on QtWebEngine: browsing with tabs (placed top, bottom, left or right), profiles of
isolated data, installed web apps and their tile page, downloads with a folder of your choice, zoom per site,
find-in-page, per-site protection switch, Burn, 68 interface languages. Not built yet in the Qt edition: the
protection-data downloader, fingerprint and GPC protection, the Tor window, the profile switcher with tiles,
automatic rejection of cookie pop-ups, permission prompts, session restore, bookmarks and history. The full list
with reasons is in the architecture document and the migration plan.

### Next release

Planned, not started, and to be designed before they are built: removal of your personal details from data
brokers' sites, help restoring your identity if it is stolen, and a VPN, all free. DuckDuckGo sells these three
in its paid subscription; InDuckGo aims to include them without charge. They need servers or partners and a legal
set-up, so there is no date. Before them, the cookie pop-up handler: a detector that wakes up after a page loads
and rejects the consent request on its own (Facebook's, for example), whenever the shield is on.

In the future InDuckGo will ship as part of a new Linux distribution. It stays a standalone project and keeps
its packages for other distributions.

---

## Installing

Download a package from the [latest release](https://github.com/eshaiotic/induckgo/releases/latest). **The 1.2.0
packages hold the previous, WebKitGTK edition** (the Qt edition has no packages yet; see *Running from source*, and
the packaging task in the migration plan):

| File | For |
|---|---|
| `induckgo_*_amd64.deb` | Debian 13, Ubuntu 24.04 LTS and newer |
| `induckgo-*.fc44.x86_64.rpm` | Fedora 44 |
| `induckgo-*-x86_64.pkg.tar.zst` | Arch Linux, CachyOS, EndeavourOS, Manjaro |
| `InDuckGo-*-x86_64.AppImage` | any other distribution with WebKitGTK 6.0, GTK ≥ 4.14 and libadwaita ≥ 1.5 |

Every release carries `SHA256SUMS` and a build provenance attestation, which proves the file was built by
this repository's GitHub Actions:

```sh
sha256sum --check --ignore-missing SHA256SUMS
gh attestation verify induckgo_*_amd64.deb --repo eshaiotic/induckgo
```

The packages use the system's WebKitGTK, GTK and libadwaita, so the browser engine keeps receiving your
distribution's security updates. For the Tor window, the Arch and Fedora packages pull in the distribution's
`tor` (no Tor service is started); the `.deb` carries Debian's `tor` itself, because Debian's package would
start a system-wide Tor service, and recommends `obfs4proxy` and `snowflake-client` for bridges; the AppImage
carries all three. The AppImage needs no installation:
`chmod +x InDuckGo-*-x86_64.AppImage && ./InDuckGo-*-x86_64.AppImage`.

From a checkout of this repository:

```sh
./install                 # Arch: builds the pacman package and installs it; Debian/Ubuntu/Fedora: installs dist/
./uninstall               # removes the package; profiles and web apps are kept
./uninstall --purge       # also deletes all InDuckGo data and web app launchers
```

---

## Building release packages

```sh
packaging/release.sh                # all formats into dist/, with SHA256SUMS
packaging/release.sh deb appimage   # or only some: arch, deb, appimage, rpm
```

The `.deb` and the AppImage are built in a Debian 13 container and the `.rpm` in a Fedora 44 container
(rootless podman), so they link against the libraries of the systems they target. The Arch package is
built on the host with `makepkg` and its `makepkg.conf` flags. The AppImage runtime is pinned to a release
and checked against its SHA-256.

Pushing a version tag publishes a release: `git tag vX.Y.Z && git push origin vX.Y.Z`. The
[release workflow](.github/workflows/release.yml) runs the same scripts (the Arch package in an Arch Linux
container), refuses dependencies with known vulnerabilities (`cargo audit`), and uploads the four packages
with `SHA256SUMS` and their attestations. The tag must match the version in `Cargo.toml`.

---

## Running from source

The Qt edition needs Qt 6 with QtWebEngine, QtWebChannel and Quick Controls (on Arch: `qt6-base qt6-declarative
qt6-webengine qt6-webchannel qt6-svg`), a C++ compiler and Rust.

```sh
sudo pacman -S --needed rust qt6-base qt6-declarative qt6-webengine qt6-webchannel qt6-svg
cargo run --release -p induckgo-qt                          # browser, default profile
cargo run --release -p induckgo-qt -- --profile work        # browser, profile "work"
cargo run --release -p induckgo-qt -- https://example.com/  # open a page
```

The previous edition still builds and runs: `sudo pacman -S --needed webkitgtk-6.0 gtk4 libadwaita tor`, then
`cargo run -p induckgo` (it also has `--tor` for the Tor window).

Installed web apps start from the desktop's application menu (`induckgo-qt --web-app <id>`).

---

## Testing

Stack: **cargo-nextest** — 781 object-oriented Given/When/Then scenarios: 612 for the domain in
`crates/induckgo-core/tests/` (74 files, one per domain type), 99 for the Qt edition and its engine adapter,
61 for the report tool in `crates/induckgo-test-report/tests/` and 9 for the benchmark tool in
`crates/induckgo-bench/tests/`. The Qt crates need Qt 6, so CI runs the other 682; run the Qt ones with
`QT_QPA_PLATFORM=offscreen cargo test -p induckgo-qt -p induckgo-engine-qt`.

```sh
./run-tests                     # all tests + HTML report in tests/reports/ (opens in the browser)
cargo test -p induckgo-core     # domain tests only (fast, no Qt, GTK or WebKit needed)
cargo clippy --all-targets      # lints (pedantic, unsafe code forbidden)
cargo fmt --check               # formatting
cargo audit                     # dependency vulnerabilities (zero-vulnerability policy)
```

**Test report:** `./run-tests` always writes `tests/reports/test-report-YYYYMMDD-HHmm.html` — a run-history
dashboard with the latest run, the last 20 runs, failure rankings, features and the slowest tests, plus
`induckgo-core`'s coverage when `cargo-llvm-cov` is installed. See [HTML test report](docs/en/tests/test-report.md).

**Test architecture:**
- The domain (`induckgo-core`) has no Qt, GTK or WebKit dependency, so every protection decision — rule
  building, filter list conversion, HTTPS upgrade, link cleaning, profiles, web apps, zoom — is tested
  without a display.
- Each scenario is an object: `given_…` builds it, `when_…` acts and returns an outcome, `then_…` asserts
  one thing, following [Object Oriented Programming First](docs/en/engineering-rules.md):
  `LinkCleaningScenario::given_tracking_parameters_protection().when_link_is_opened(url).then_opens_unchanged()`.
- A failing test is retried once: one that then passes is reported as FLAKY, a defect to fix, never ignored.
- The [tests workflow](.github/workflows/tests.yml) runs fmt, clippy, `cargo audit` and `./run-tests` on
  every push to `main` and on every pull request, and keeps each run's report as an artifact.

---

## Project structure

```
crates/induckgo-core/    domain logic without Qt/GTK/WebKit: protection rules, Guard filter lists,
                         profiles, web apps, zoom, launch, tab placement and window button style
crates/induckgo-core/tests/
                         one OO scenario file per domain type
crates/induckgo-qt/      the browser: Qt Quick interface (QML) on QtWebEngine, bound to Rust with cxx-qt
crates/induckgo-engine-qt/
                         the QtWebEngine adapter: request and cookie policies, C++ shim
crates/induckgo/         the previous edition: GTK 4 + libadwaita on WebKitGTK (what the 1.2.0 packages hold)
crates/induckgo-test-report/
                         developer tool behind ./run-tests: the HTML test report (never packaged)
assets/                  vendored Content Scope Scripts (Apache-2.0), Guard's page script, icons,
                         error page template; never the Tracker Data Set or filter lists
packaging/               PKGBUILD, rpm spec, deb/AppImage assembly, desktop entry, AppStream metadata
docs/en, docs/pl         architecture, engineering rules and reports (English canonical, Polish mirror)
```

- [Architecture](docs/en/architecture/overview.md) — processes, isolation, the protection pipeline, web apps, limitations.
- [Engineering rules](docs/en/engineering-rules.md) — Object Oriented Programming First, OO Given/When/Then tests.

---

## Licence

InDuckGo is free software under the [GNU General Public License, version 3 or later](LICENSE)
(`GPL-3.0-or-later`). Attributions are in [NOTICE](NOTICE).

### Licences of third-party data

- Content Scope Scripts 20.0.0 — Apache-2.0, vendored in `assets/content-scope-scripts` with its licence.
- Privacy configuration — Apache-2.0, downloaded at runtime.
- Tracker Data Set — **CC BY-NC-SA 4.0** (non-commercial), downloaded at runtime, never committed.
- InDuckGo Guard's filter lists — GPL-3.0, CC BY-SA 3.0, CC BY 3.0, CC BY-NC-SA 4.0 or their publisher's terms,
  downloaded at runtime, never committed.
- adblock-rust 0.13.5 — MPL-2.0, compiled in unmodified (source: <https://github.com/brave/adblock-rust>).
