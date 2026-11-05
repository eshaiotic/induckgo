//! Where InDuckGo keeps its files, shared with the GTK edition so profiles, settings and downloaded
//! protection data carry over.

use std::env;
use std::path::Path;
use std::path::PathBuf;

use induckgo_core::profile::profile_directories::ProfileDirectories;
use induckgo_core::profile::profile_directories::ProfileLocation;
use induckgo_core::profile::profile_id::ProfileId;
use induckgo_core::web_app::web_app_id::WebAppId;

use crate::system_downloads_folder::SystemDownloadsFolder;

#[derive(Debug, Clone)]
pub struct InstallationDirectories {
    data_root: PathBuf,
    cache_root: PathBuf,
    /// Where the desktop looks for `.desktop` files of the person's own applications.
    applications: PathBuf,
    /// Where the desktop looks for the icons of the person's own applications.
    application_icons: PathBuf,
}

impl InstallationDirectories {
    const DIRECTORY: &'static str = "induckgo";

    /// Follows the XDG base directory specification: `$XDG_DATA_HOME` and `$XDG_CACHE_HOME`, else the
    /// `~/.local/share` and `~/.cache` defaults.
    #[must_use]
    pub fn from_environment() -> Self {
        let home = env::var_os("HOME").map(PathBuf::from).unwrap_or_default();
        let data_home = Self::directory_from("XDG_DATA_HOME").unwrap_or_else(|| home.join(".local/share"));
        let cache_home = Self::directory_from("XDG_CACHE_HOME").unwrap_or_else(|| home.join(".cache"));
        Self::under(&data_home, &cache_home)
    }

    /// The layout below the given XDG data and cache homes.
    #[must_use]
    pub fn under(data_home: &Path, cache_home: &Path) -> Self {
        Self {
            data_root: data_home.join(Self::DIRECTORY),
            cache_root: cache_home.join(Self::DIRECTORY),
            applications: data_home.join("applications"),
            application_icons: data_home.join("icons/hicolor/scalable/apps"),
        }
    }

    /// An XDG variable counts only when it is an absolute path, as the specification requires.
    fn directory_from(variable: &str) -> Option<PathBuf> {
        env::var_os(variable).map(PathBuf::from).filter(|path| path.is_absolute())
    }

    #[must_use]
    pub fn profile(&self, id: &ProfileId) -> ProfileLocation {
        ProfileDirectories::new(&self.data_root, &self.cache_root).for_profile(id)
    }

    #[must_use]
    pub fn interface_language_file(&self) -> PathBuf {
        self.data_root.join("interface.json")
    }

    /// Where the sockets that hand a second launch to the running process live: the session's runtime
    /// directory, else a folder of the cache.
    #[must_use]
    pub fn runtime_directory(&self) -> PathBuf {
        Self::directory_from("XDG_RUNTIME_DIR")
            .map_or_else(|| self.cache_root.join("run"), |runtime| runtime.join(Self::DIRECTORY))
    }

    /// The browsing profiles the person created, shared with the GTK edition.
    #[must_use]
    pub fn profile_catalog(&self) -> PathBuf {
        self.data_root.join("profiles.json")
    }

    /// The sites installed as web apps, shared with the GTK edition.
    #[must_use]
    pub fn web_app_catalog(&self) -> PathBuf {
        self.data_root.join("web-apps.json")
    }

    /// The folder holding one installed web app's icon.
    #[must_use]
    pub fn web_app_icon_directory(&self, id: &WebAppId) -> PathBuf {
        self.data_root.join("web-apps").join(id.as_str())
    }

    /// Where the `.desktop` file of an installed web app goes, so the application menu lists it.
    #[must_use]
    pub fn applications_directory(&self) -> &Path {
        &self.applications
    }

    /// Where the icon of the `.desktop` file of an AppImage goes (the icon theme's scalable application icons).
    #[must_use]
    pub fn application_icons_directory(&self) -> PathBuf {
        self.application_icons.clone()
    }

    /// The folder the desktop calls "Downloads".
    #[must_use]
    pub fn system_downloads_folder(&self) -> PathBuf {
        let home = env::var_os("HOME").map(PathBuf::from).unwrap_or_default();
        let config_home = Self::directory_from("XDG_CONFIG_HOME").unwrap_or_else(|| home.join(".config"));
        SystemDownloadsFolder::locate(&home, &config_home)
    }

    /// Downloaded Tracker Data Set and privacy configuration, shared by every profile.
    #[must_use]
    pub fn protection_cache(&self) -> PathBuf {
        self.cache_root.join("protections")
    }
}
