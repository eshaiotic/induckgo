//! `QtWebEngine` as the application sees it.

use std::sync::Arc;

use induckgo_core::engine::engine_version::EngineVersion;
use induckgo_core::network::request_policy::RequestPolicy;
use induckgo_core::storage::cookie_policy::CookiePolicy;

use crate::engine_bridge::ffi;
use crate::engine_policies::EnginePolicies;

pub struct QtWebEngine;

impl QtWebEngine {
    /// Prepares `QtWebEngine`. Call it before the `QGuiApplication` is created.
    pub fn initialize() {
        ffi::initialize_web_engine();
    }

    /// Makes Chromium ask these policies about every request and cookie. Call it once the application exists.
    /// A second call changes nothing and returns `false`.
    pub fn install_policies(request: Arc<dyn RequestPolicy>, cookies: Arc<dyn CookiePolicy>) -> bool {
        let is_first = EnginePolicies::new(request, cookies).install();
        if is_first {
            ffi::install_hooks();
        }
        is_first
    }

    /// The Chromium version inside the installed `QtWebEngine`; `None` if it cannot be read.
    #[must_use]
    pub fn chromium_version() -> Option<EngineVersion> {
        EngineVersion::parse(&ffi::chromium_version())
    }
}
