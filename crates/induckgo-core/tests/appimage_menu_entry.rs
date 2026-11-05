//! The application-menu entry an AppImage writes for itself.
//!
//! A package installs `io.github.induckgo.InDuckGo.desktop`; an AppImage installs nothing, and a Wayland desktop
//! (KDE, GNOME) shows a generic icon for a window whose `app_id` has no `.desktop` file. The entry is the packaged
//! one with the program and the icon pointed at the files that exist on this machine.

use std::path::Path;

use induckgo_core::launch::appimage_menu_entry::AppImageMenuEntry;

const PACKAGED_ENTRY: &str = "[Desktop Entry]\n\
Type=Application\n\
Name=InDuckGo\n\
Exec=induckgo %U\n\
Icon=io.github.induckgo.InDuckGo\n\
StartupWMClass=InDuckGo\n\
Actions=new-window;\n\
\n\
[Desktop Action new-window]\n\
Name=New Window\n\
Exec=induckgo\n";

struct MenuEntryScenario {
    appimage: &'static str,
    icon: &'static str,
}

impl MenuEntryScenario {
    fn given_appimage_at(appimage: &'static str) -> Self {
        Self { appimage, icon: "/home/ola/.local/share/icons/hicolor/scalable/apps/io.github.induckgo.InDuckGo.svg" }
    }

    fn when_rendered(&self) -> Vec<String> {
        AppImageMenuEntry::new(PACKAGED_ENTRY, Path::new(self.appimage), Path::new(self.icon))
            .render()
            .lines()
            .map(str::to_owned)
            .collect()
    }
}

#[test]
fn should_start_the_appimage_file_from_the_menu_entry_and_its_new_window_action() {
    let lines = MenuEntryScenario::given_appimage_at("/home/ola/Apps/InDuckGo-1.2.3-x86_64.AppImage").when_rendered();

    let exec_lines: Vec<&String> = lines.iter().filter(|line| line.starts_with("Exec=")).collect();
    assert_eq!(
        exec_lines,
        ["Exec=/home/ola/Apps/InDuckGo-1.2.3-x86_64.AppImage %U", "Exec=/home/ola/Apps/InDuckGo-1.2.3-x86_64.AppImage"]
    );
}

#[test]
fn should_point_the_menu_entry_at_the_icon_file_written_next_to_it() {
    let lines = MenuEntryScenario::given_appimage_at("/home/ola/Apps/InDuckGo.AppImage").when_rendered();

    assert!(lines.contains(
        &"Icon=/home/ola/.local/share/icons/hicolor/scalable/apps/io.github.induckgo.InDuckGo.svg".to_owned()
    ));
}

#[test]
fn should_quote_an_appimage_path_that_contains_a_space() {
    let lines = MenuEntryScenario::given_appimage_at("/home/ola/My Apps/InDuckGo.AppImage").when_rendered();

    assert!(lines.contains(&"Exec=\"/home/ola/My Apps/InDuckGo.AppImage\" %U".to_owned()));
}

#[test]
fn should_keep_every_other_line_of_the_packaged_entry() {
    let lines = MenuEntryScenario::given_appimage_at("/home/ola/Apps/InDuckGo.AppImage").when_rendered();

    assert!(lines.contains(&"StartupWMClass=InDuckGo".to_owned()));
    assert!(lines.contains(&"Name=New Window".to_owned()));
}

#[test]
fn should_mark_the_entry_as_written_by_the_appimage_so_that_a_hand_written_one_is_never_replaced() {
    let lines = MenuEntryScenario::given_appimage_at("/home/ola/Apps/InDuckGo.AppImage").when_rendered();

    assert_eq!(lines[..2], ["[Desktop Entry]", AppImageMenuEntry::MARKER]);
}
