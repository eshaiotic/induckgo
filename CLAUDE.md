# InDuckGo — project index

Privacy-first WebKitGTK browser for Linux, written in Rust, built on DuckDuckGo's open-source protections.
This file is an index; the content lives in the linked documents.

## Rules

- [Engineering rules](docs/en/engineering-rules.md) — Object Oriented Programming First, OO Given/When/Then tests, crate boundaries, imports.
- [Zasady inżynierskie (PL)](docs/pl/engineering-rules.md) — polish mirror of the engineering rules.

## Code map

- `crates/induckgo-core` — domain logic without GTK/WebKit: tracker rules, privacy config, navigation.
- `crates/induckgo-core/tests` — one OO scenario test file per domain type.

## Commands

- `cargo test -p induckgo-core` — fast domain tests.
- `cargo clippy --all-targets` — lints (pedantic, `unsafe_code` forbidden).
- `cargo audit` — dependency vulnerability check (zero-vulnerability policy).
