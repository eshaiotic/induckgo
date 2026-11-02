//! Where InDuckGo keeps its files, following the XDG base directory layout.
//!
//! Data that must survive (`~/.local/share/induckgo`): profile and web app catalogs, each profile's cookies
//! and site storage, web app icons. Data that can be rebuilt (`~/.cache/induckgo`): HTTP caches, the
//! downloaded tracker lists and compiled WebKit rules.

use std::path::Path;
use std::path::PathBuf;

use induckgo_core::profile::profile_directories::ProfileDirectories;
use induckgo_core::profile::profile_directories::ProfileLocation;
use induckgo_core::profile::profile_id::ProfileId;
use induckgo_core::web_app::web_app_id::WebAppId;

pub struct InstallationPaths {
    data_root: PathBuf,
    cache_root: PathBuf,
    applications: PathBuf,
    downloads: PathBuf,
}

impl InstallationPaths {
    const DIRECTORY: &'static str = "induckgo";

    pub fn from_environment() -> Self {
        let data_home = glib::user_data_dir();
        Self {
            data_root: data_home.join(Self::DIRECTORY),
            cache_root: glib::user_cache_dir().join(Self::DIRECTORY),
            applications: data_home.join("applications"),
            downloads: glib::user_special_dir(glib::UserDirectory::Downloads)
                .unwrap_or_else(|| glib::home_dir().join("Downloads")),
        }
    }

    pub fn profile(&self, id: &ProfileId) -> ProfileLocation {
        ProfileDirectories::new(&self.data_root, &self.cache_root).for_profile(id)
    }

    pub fn profile_catalog(&self) -> PathBuf {
        self.data_root.join("profiles.json")
    }

    pub fn web_app_catalog(&self) -> PathBuf {
        self.data_root.join("web-apps.json")
    }

    pub fn web_app_icon(&self, id: &WebAppId) -> PathBuf {
        self.data_root.join("web-apps").join(id.as_str()).join("icon.png")
    }

    pub fn desktop_entry(&self, file_name: &str) -> PathBuf {
        self.applications.join(file_name)
    }

    /// Downloaded Tracker Data Set and privacy configuration, shared by every profile.
    pub fn protection_feed(&self) -> PathBuf {
        self.cache_root.join("protections")
    }

    /// Icon theme directory holding InDuckGo's own icon.
    pub fn icon_theme(&self) -> PathBuf {
        self.data_root.join("icons")
    }

    pub fn downloads(&self) -> &Path {
        &self.downloads
    }
}
