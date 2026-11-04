//! Where InDuckGo keeps its files, shared with the GTK edition so profiles, settings and downloaded
//! protection data carry over.

use std::env;
use std::path::PathBuf;

use induckgo_core::profile::profile_directories::ProfileDirectories;
use induckgo_core::profile::profile_directories::ProfileLocation;
use induckgo_core::profile::profile_id::ProfileId;

use crate::system_downloads_folder::SystemDownloadsFolder;

pub struct InstallationDirectories {
    data_root: PathBuf,
    cache_root: PathBuf,
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
        Self { data_root: data_home.join(Self::DIRECTORY), cache_root: cache_home.join(Self::DIRECTORY) }
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
