//! InDuckGo's own icons, embedded in the binary and placed in a private icon theme directory at start-up.

use std::path::PathBuf;

use crate::installation_paths::InstallationPaths;
use crate::stored_file::StoredFile;

pub struct AppIcons {
    theme_directory: PathBuf,
}

impl AppIcons {
    pub const APPLICATION: &'static str = "io.github.induckgo.InDuckGo";
    pub const FIRE: &'static str = "induckgo-fire-symbolic";
    pub const INSTALL_APP: &'static str = "induckgo-install-app-symbolic";
    const APPLICATION_SVG: &'static str = include_str!("../../../assets/icons/io.github.induckgo.InDuckGo.svg");
    const FIRE_SVG: &'static str = include_str!("../../../assets/icons/induckgo-fire-symbolic.svg");
    const INSTALL_APP_SVG: &'static str = include_str!("../../../assets/icons/induckgo-install-app-symbolic.svg");
    const LOG_DOMAIN: &'static str = "induckgo";

    pub fn new(paths: &InstallationPaths) -> Self {
        Self { theme_directory: paths.icon_theme() }
    }

    /// Writes the icons (when changed) and makes GTK look them up; needs a display.
    pub fn install(&self) {
        let icons = [
            (self.application_icon(), Self::APPLICATION_SVG),
            (self.symbolic_icon(Self::FIRE), Self::FIRE_SVG),
            (self.symbolic_icon(Self::INSTALL_APP), Self::INSTALL_APP_SVG),
        ];
        for (path, svg) in icons {
            let file = StoredFile::new(path);
            if file.read_text().as_deref() != Some(svg)
                && let Err(error) = file.write(svg.as_bytes())
            {
                glib::g_warning!(Self::LOG_DOMAIN, "cannot install icon {}: {error}", file.path().display());
            }
        }
        if let Some(display) = gdk::Display::default() {
            gtk::IconTheme::for_display(&display).add_search_path(&self.theme_directory);
        }
        gtk::Window::set_default_icon_name(Self::APPLICATION);
    }

    /// Absolute path of the application icon; web apps without an icon of their own use it.
    pub fn application_icon(&self) -> PathBuf {
        self.theme_directory.join("hicolor/scalable/apps").join(format!("{}.svg", Self::APPLICATION))
    }

    fn symbolic_icon(&self, name: &str) -> PathBuf {
        self.theme_directory.join("hicolor/symbolic/apps").join(format!("{name}.svg"))
    }
}
