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
- [Podsumowanie dla PO (HTML, PL)](docs/pl/reports/tracking-summary.html) — one-page tracker and ad blocking summary for non-technical readers.
- [Product context](PRODUCT.md) — users, positioning and evidence used by design work.

## Code map

- `crates/induckgo-core` — domain logic without GTK/WebKit: protection rules, Guard filter lists, profiles, web apps, zoom, launch.
- `crates/induckgo-core/tests` — one OO scenario test file per domain type.
- `crates/induckgo` — GTK 4 + libadwaita app: adapters that carry out core decisions in WebKitGTK/libsoup/GIO.
- `assets/` — vendored Content Scope Scripts (Apache-2.0), Guard's page script, app icons and the error page template; never the TDS or filter lists.
- `packaging/` — PKGBUILD, rpm spec, deb/AppImage assembly, desktop entry and AppStream metadata.

## Commands

- `cargo test -p induckgo-core` — fast domain tests.
- `cargo clippy --all-targets` — lints (pedantic, `unsafe_code` forbidden).
- `cargo audit` — dependency vulnerability check (zero-vulnerability policy).
- `cargo run -p induckgo -- [--profile NAME | --web-app ID] [URL…]` — run the browser or an installed web app.
- `packaging/release.sh [arch] [deb] [appimage] [rpm]` — release packages into `dist/` (deb/AppImage/rpm in podman).
- `./install` / `./uninstall [--purge]` — install or remove the system package (Arch: built with makepkg).

## Gotchas

- Never call `soup::tld_get_base_domain` (soup3 0.9 binding reads freed memory); use `RegistrableDomainLookup`.
- Changing Guard's list conversion (or upgrading `adblock`): bump `GuardRuleList::CONVERSION_REVISION`, else old compiled lists stay.
- Manual GUI runs: isolate `XDG_DATA_HOME`/`XDG_CACHE_HOME`/`XDG_CONFIG_HOME` so the user's real profiles stay untouched.
