//! Browser-independent domain logic of InDuckGo.
//!
//! Nothing in this crate knows about GTK or WebKit: it turns DuckDuckGo's open data (Tracker Data Set,
//! privacy configuration) and user intent (typed addresses, profiles, installed web apps) into plain
//! values the `induckgo` application hands to WebKitGTK. That keeps the rules fast to test and easy to
//! audit.

pub mod document;
pub mod download;
pub mod guard;
pub mod identity;
pub mod language;
pub mod launch;
pub mod navigation;
pub mod page_zoom;
pub mod profile;
pub mod protection;
pub mod rendering;
pub mod tor;
pub mod web_app;
