//! InDuckGo 1.2: the Qt window on `QtWebEngine`. See `docs/en/specs/2026-11-04-qtwebengine-migration-design.md`.

pub mod browser_application;
// `cxx-qt` generates `unsafe` code and a boxed constructor in its bridge; this is the only module that may contain them.
#[allow(unsafe_code, clippy::unnecessary_box_returns)]
pub mod browser_bridge;
pub mod installation_directories;
pub mod interface_catalogue;
pub mod profile_policies;
pub mod protection_documents;
pub mod startup_notice;
pub mod system_locales;
