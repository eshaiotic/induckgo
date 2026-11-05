//! `QtWebEngine` as the application sees it.

use std::path::Path;
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

    /// Makes Chromium ask these policies about every request and cookie, and keep everything it stores under
    /// `storage` and `cache`. Call it once the application exists; a second call changes nothing and returns
    /// `false`.
    pub fn install_policies(
        request: Arc<dyn RequestPolicy>,
        cookies: Arc<dyn CookiePolicy>,
        storage: &Path,
        cache: &Path,
    ) -> bool {
        let is_first = EnginePolicies::new(request, cookies).install();
        if is_first {
            ffi::install_hooks(&storage.to_string_lossy(), &cache.to_string_lossy());
        }
        is_first
    }

    /// Gives every window the icon stored at `resource_path` in the Qt resources. Call it once the application
    /// exists.
    pub fn set_window_icon(resource_path: &str) {
        ffi::set_window_icon(resource_path);
    }

    /// The text of the Qt resource at `path` (`:/qtwebchannel/qwebchannel.js`); `None` when there is none.
    #[must_use]
    pub fn resource_text(path: &str) -> Option<String> {
        Some(ffi::read_qt_resource(path)).filter(|text| !text.is_empty())
    }

    /// The PNG `bytes` decoded and encoded again by Qt; `None` when Qt cannot decode them.
    #[must_use]
    pub fn reencode_png(bytes: &[u8]) -> Option<Vec<u8>> {
        Some(ffi::reencode_png(bytes)).filter(|png| !png.is_empty())
    }

    /// The Chromium version inside the installed `QtWebEngine`; `None` if it cannot be read.
    #[must_use]
    pub fn chromium_version() -> Option<EngineVersion> {
        EngineVersion::parse(&ffi::chromium_version())
    }
}
