//! Puts a running AppImage into the application menu, so the desktop can show its icon.
//!
//! A Wayland desktop gives a window the icon of the `.desktop` file named like the window's `app_id`; a package
//! installs that file, an AppImage does not. Without it the window shows a generic Wayland icon. The entry lives in
//! the person's own data directory and names the AppImage file, so moving the file only needs one more start.
//! A packaged InDuckGo already owns the entry system-wide and is left alone.

use std::fs;
use std::io;
use std::path::Path;
use std::path::PathBuf;

use induckgo_core::launch::appimage_menu_entry::AppImageMenuEntry;

use crate::installation_directories::InstallationDirectories;

pub struct AppImageIntegration {
    applications: PathBuf,
    icons: PathBuf,
    system_data_directories: Vec<PathBuf>,
}

impl AppImageIntegration {
    /// The `.desktop` file name the window's `app_id` selects.
    pub const ENTRY_FILE: &'static str = "io.github.induckgo.InDuckGo.desktop";
    const ICON_FILE: &'static str = "io.github.induckgo.InDuckGo.svg";
    const PACKAGED_ENTRY: &'static str = include_str!("../../../packaging/linux/io.github.induckgo.InDuckGo.desktop");
    const ICON: &'static str = include_str!("../../../assets/icons/io.github.induckgo.InDuckGo.svg");
    const DEFAULT_SYSTEM_DATA_DIRECTORIES: &'static str = "/usr/local/share:/usr/share";

    #[must_use]
    pub fn new(directories: &InstallationDirectories, system_data_directories: Vec<PathBuf>) -> Self {
        Self {
            applications: directories.applications_directory().to_path_buf(),
            icons: directories.application_icons_directory(),
            system_data_directories,
        }
    }

    /// Looks for an installed package in `$XDG_DATA_DIRS`, or in the specification's default directories.
    #[must_use]
    pub fn for_environment(directories: &InstallationDirectories) -> Self {
        let configured = std::env::var_os("XDG_DATA_DIRS").filter(|value| !value.is_empty());
        let listed = configured.unwrap_or_else(|| Self::DEFAULT_SYSTEM_DATA_DIRECTORIES.into());
        Self::new(directories, std::env::split_paths(&listed).filter(|path| path.is_absolute()).collect())
    }

    /// Writes the menu entry and the icon for `appimage`, unless a package owns the entry or the files are current.
    ///
    /// # Errors
    /// Returns the I/O error when the person's data directory cannot be written.
    pub fn install(&self, appimage: &Path) -> io::Result<()> {
        if self.is_installed_system_wide() {
            return Ok(());
        }
        let icon = self.icons.join(Self::ICON_FILE);
        Self::write_when_changed(&icon, Self::ICON)?;
        let entry = AppImageMenuEntry::new(Self::PACKAGED_ENTRY, appimage, &icon).render();
        Self::write_when_changed(&self.applications.join(Self::ENTRY_FILE), &entry)
    }

    fn is_installed_system_wide(&self) -> bool {
        self.system_data_directories
            .iter()
            .any(|directory| directory.join("applications").join(Self::ENTRY_FILE).exists())
    }

    fn write_when_changed(path: &Path, content: &str) -> io::Result<()> {
        if fs::read_to_string(path).is_ok_and(|current| current == content) {
            return Ok(());
        }
        if let Some(directory) = path.parent() {
            fs::create_dir_all(directory)?;
        }
        fs::write(path, content)
    }
}
