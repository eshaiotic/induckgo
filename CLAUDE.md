# InDuckGo — project index

Privacy-first WebKitGTK browser for Linux, written in Rust, built on DuckDuckGo's open-source protections.
This file is an index; the content lives in the linked documents.

## Rules

- [Engineering rules](docs/en/engineering-rules.md) — Object Oriented Programming First, OO Given/When/Then tests, crate boundaries, imports.
- [Zasady inżynierskie (PL)](docs/pl/engineering-rules.md) — polish mirror of the engineering rules.

## Architecture

- [Architecture overview](docs/en/architecture/overview.md) — process per profile/web app, protection pipeline, WebKitGTK limits.
- [Przegląd architektury (PL)](docs/pl/architecture/overview.md) — polish mirror of the architecture overview.
- `docs/assets/diagrams/` — SVG + PNG diagrams, one file per language (`.en` / `.pl`).

## Reports

- [Tracking comparison](docs/en/reports/tracking-comparison.md) — InDuckGo vs Chrome vs Brave: telemetry, trackers, fingerprinting (measured).
- [Porównanie śledzenia (PL)](docs/pl/reports/tracking-comparison.md) — polish mirror of the tracking comparison.
- [Tracking summary (HTML)](docs/en/reports/tracking-summary.html) — one-page tracker and ad blocking summary for non-technical readers.
- [Podsumowanie dla PO (HTML, PL)](docs/pl/reports/tracking-summary.html) — polish mirror of the tracking summary.
- [Product context](PRODUCT.md) — users, positioning and evidence used by design work.
- [Design system](DESIGN.md) — report and error-page visual worlds: colours, type, rules (sidecar `.impeccable/design.json`).

## Code map

- `crates/induckgo-core` — domain logic without GTK/WebKit: protection rules, Guard filter lists, profiles, web apps, zoom, launch, Tor.
- `crates/induckgo-core/tests` — one OO scenario test file per domain type.
- `crates/induckgo-test-report` — dev tool behind `./run-tests`: nextest JUnit + llvm-cov → HTML test report (not packaged).
- `crates/induckgo` — GTK 4 + libadwaita app: adapters that carry out core decisions in WebKitGTK/libsoup/GIO.
- `assets/` — vendored Content Scope Scripts (Apache-2.0), Guard's page script, app icons and the error page template; never the TDS or filter lists.
- `packaging/` — PKGBUILD, rpm spec, deb/AppImage assembly, desktop entry and AppStream metadata.

## Commands

- `cargo test -p induckgo-core` — fast domain tests.
- `./run-tests` — all tests via cargo-nextest + HTML report in `tests/reports/` ([format](docs/en/tests/test-report.md)).
- `cargo clippy --all-targets` — lints (pedantic, `unsafe_code` forbidden).
- `cargo audit` — dependency vulnerability check (zero-vulnerability policy).
- `cargo run -p induckgo -- [--profile NAME | --web-app ID | --tor] [URL…]` — run the browser, a web app or the Tor window.
- `packaging/release.sh [arch] [deb] [appimage] [rpm]` — release packages into `dist/` (deb/AppImage/rpm in podman).
- `./install` / `./uninstall [--purge]` — install or remove the system package (Arch: built with makepkg).
- `git tag vX.Y.Z && git push origin vX.Y.Z` — `.github/workflows/release.yml` publishes the GitHub release (tag = Cargo version).
- `.github/workflows/tests.yml` — fmt, clippy, tests and `cargo audit` on pushes to main and on pull requests.

## Gotchas

- Never call `soup::tld_get_base_domain` (soup3 0.9 binding reads freed memory); use `RegistrableDomainLookup`.
- Changing Guard's list conversion (or upgrading `adblock`): bump `GuardRuleList::CONVERSION_REVISION`, else old compiled lists stay.
- Manual GUI runs: isolate `XDG_DATA_HOME`/`XDG_CACHE_HOME`/`XDG_CONFIG_HOME` so the user's real profiles stay untouched.
- Tor window (`--tor`): never let it load before Tor is connected or fall back to a direct connection (architecture §4.1).
