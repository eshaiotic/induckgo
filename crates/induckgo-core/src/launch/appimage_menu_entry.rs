//! The application-menu entry an AppImage writes for itself.
//!
//! A package installs `io.github.induckgo.InDuckGo.desktop`; an AppImage installs nothing. A Wayland desktop takes a
//! window's icon from the `.desktop` file named like the window's `app_id`, so without an entry the window shows a
//! generic icon. The entry is the packaged one with its program and icon pointed at the files of this machine.

use std::path::Path;

use crate::web_app::desktop_entry::DesktopEntry;

pub struct AppImageMenuEntry<'entry> {
    packaged_entry: &'entry str,
    appimage: &'entry Path,
    icon: &'entry Path,
}

impl<'entry> AppImageMenuEntry<'entry> {
    /// The name of the installed program in the packaged entry's `Exec` lines.
    const PACKAGED_PROGRAM: &'static str = "induckgo";

    #[must_use]
    pub fn new(packaged_entry: &'entry str, appimage: &'entry Path, icon: &'entry Path) -> Self {
        Self { packaged_entry, appimage, icon }
    }

    /// The packaged entry with `Exec` starting the AppImage file and `Icon` naming the icon file.
    #[must_use]
    pub fn render(&self) -> String {
        let mut text: String =
            self.packaged_entry.lines().map(|line| self.adapted(line)).collect::<Vec<_>>().join("\n");
        text.push('\n');
        text
    }

    fn adapted(&self, line: &str) -> String {
        if let Some(arguments) = line.strip_prefix("Exec=").and_then(|exec| exec.strip_prefix(Self::PACKAGED_PROGRAM)) {
            let program = DesktopEntry::exec_argument(&self.appimage.to_string_lossy());
            return format!("Exec={program}{arguments}");
        }
        if line.starts_with("Icon=") {
            return format!("Icon={}", DesktopEntry::escaped_value(&self.icon.to_string_lossy()));
        }
        line.to_owned()
    }
}
