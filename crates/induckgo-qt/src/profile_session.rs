//! The state of the profile this process runs: the shield's list, the downloads and the Fire button.
//!
//! QML creates the bridge objects itself, so the session is installed once at start-up and the bridge finds it
//! here.

use std::sync::Mutex;
use std::sync::MutexGuard;
use std::sync::OnceLock;

use induckgo_core::profile::profile_directories::ProfileLocation;

use crate::burn_restart::BurnRestart;
use crate::download_center::DownloadCenter;
use crate::site_protection::SiteProtection;
use crate::site_zoom_store::SiteZoomStore;

pub struct ProfileSession {
    pub protection: SiteProtection,
    pub burn: BurnRestart,
    downloads: Mutex<DownloadCenter>,
    zoom: Mutex<SiteZoomStore>,
}

static CURRENT: OnceLock<ProfileSession> = OnceLock::new();

impl ProfileSession {
    #[must_use]
    pub fn new(
        location: &ProfileLocation,
        protection: SiteProtection,
        downloads: DownloadCenter,
        zoom: SiteZoomStore,
    ) -> Self {
        Self {
            protection,
            burn: BurnRestart::new(location.clone()),
            downloads: Mutex::new(downloads),
            zoom: Mutex::new(zoom),
        }
    }

    /// Makes this the session of the process; a second call is ignored.
    pub fn install(self) {
        let _ = CURRENT.set(self);
    }

    #[must_use]
    pub fn current() -> Option<&'static Self> {
        CURRENT.get()
    }

    pub fn zoom(&self) -> MutexGuard<'_, SiteZoomStore> {
        self.zoom.lock().unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    pub fn downloads(&self) -> MutexGuard<'_, DownloadCenter> {
        self.downloads.lock().unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}
