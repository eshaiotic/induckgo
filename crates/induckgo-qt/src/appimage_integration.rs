//! Puts a running AppImage into the application menu, so the desktop can show its icon.
//!
//! A Wayland desktop gives a window the icon of the `.desktop` file named like the window's `app_id`; a package
//! installs that file, an AppImage does not. Without it the window shows a generic Wayland icon. The entry lives in
//! the person's own data directory and names the AppImage file, so moving the file only needs one more start.
//! A packaged InDuckGo already owns the entry system-wide and is left alone.
//!
//! The entry makes the desktop run the AppImage file whenever the person clicks it, so it is written only for a
//! file that nobody else can replace: the path is resolved first (the entry names the real file), and the file
//! and every folder above it belong to root or to the person and are not writable by the group or by others.
//! The one exception is a sticky folder of root's (`/tmp`) above the file's own folder, where others cannot rename
//! what they do not own. A file directly in `/tmp` does not qualify: after a clean-up anyone could put a file
//! of that name in its place. An entry the person wrote themselves is never overwritten.

use std::fs;
use std::io;
use std::os::unix::fs::MetadataExt;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::path::PathBuf;

use induckgo_core::launch::appimage_menu_entry::AppImageMenuEntry;

use crate::installation_directories::InstallationDirectories;

pub struct AppImageIntegration {
    applications: PathBuf,
    /// The owner of the person's data directory, who is trusted to own the AppImage and its folders (root is too).
    person: u32,
    icons: PathBuf,
    system_data_directories: Vec<PathBuf>,
}

impl AppImageIntegration {
    /// The `.desktop` file name the window's `app_id` selects.
    pub const ENTRY_FILE: &'static str = "io.github.induckgo.InDuckGo.desktop";
    const ICON_FILE: &'static str = "io.github.induckgo.InDuckGo.svg";
    const PACKAGED_ENTRY: &'static str = include_str!("../../../packaging/linux/io.github.induckgo.InDuckGo.desktop");
    const ICON: &'static str = include_str!("../../../assets/icons/io.github.induckgo.InDuckGo.svg");
    /// The group-write and other-write permission bits.
    const WRITABLE_BY_OTHERS: u32 = 0o022;
    const STICKY: u32 = 0o1000;
    const ROOT: u32 = 0;
    const DEFAULT_SYSTEM_DATA_DIRECTORIES: &'static str = "/usr/local/share:/usr/share";

    #[must_use]
    pub fn new(directories: &InstallationDirectories, system_data_directories: Vec<PathBuf>) -> Self {
        let applications = directories.applications_directory().to_path_buf();
        // The data directory may not exist yet: its first existing parent belongs to the same person.
        let person =
            applications.ancestors().find_map(|folder| fs::metadata(folder).ok()).map_or(0, |folder| folder.uid());
        Self { applications, person, icons: directories.application_icons_directory(), system_data_directories }
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
        let entry_file = self.applications.join(Self::ENTRY_FILE);
        let Some(appimage) = self.trusted_file(appimage) else { return Ok(()) };
        if self.is_installed_system_wide() || Self::is_written_by_hand(&entry_file) {
            return Ok(());
        }
        let icon = self.icons.join(Self::ICON_FILE);
        Self::write_when_changed(&icon, Self::ICON)?;
        let entry = AppImageMenuEntry::new(Self::PACKAGED_ENTRY, &appimage, &icon).render();
        Self::write_when_changed(&entry_file, &entry)
    }

    /// An existing file that does not carry the marker is the person's own entry.
    fn is_written_by_hand(entry_file: &Path) -> bool {
        fs::read_to_string(entry_file).is_ok_and(|text| !text.lines().any(|line| line == AppImageMenuEntry::MARKER))
    }

    /// The real path of `appimage` when nobody but root and the person could replace it, else nothing.
    fn trusted_file(&self, appimage: &Path) -> Option<PathBuf> {
        let real = fs::canonicalize(appimage).ok()?;
        let file = fs::metadata(&real).ok().filter(fs::Metadata::is_file)?;
        let folders_are_closed = real.ancestors().skip(1).enumerate().all(|(level, folder)| {
            let is_own_folder = level == 0;
            fs::metadata(folder)
                .is_ok_and(|metadata| self.is_closed(&metadata) || (!is_own_folder && Self::is_root_sticky(&metadata)))
        });
        (self.is_closed(&file) && folders_are_closed).then_some(real)
    }

    /// Owned by root or the person, and not writable by the group or by others.
    fn is_closed(&self, metadata: &fs::Metadata) -> bool {
        let is_trusted_owner = metadata.uid() == Self::ROOT || metadata.uid() == self.person;
        is_trusted_owner && metadata.permissions().mode() & Self::WRITABLE_BY_OTHERS == 0
    }

    /// A shared folder like `/tmp`: others may create files in it, but cannot rename or delete the ones of others.
    fn is_root_sticky(metadata: &fs::Metadata) -> bool {
        metadata.uid() == Self::ROOT && metadata.permissions().mode() & Self::STICKY != 0
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
