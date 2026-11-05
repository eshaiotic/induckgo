//! An AppImage puts itself into the application menu, so the desktop can show its icon.
//!
//! A packaged InDuckGo already has its entry in `/usr/share/applications`; the AppImage must not shadow it.

use std::fs;
use std::path::Path;
use std::path::PathBuf;

use induckgo_qt::appimage_integration::AppImageIntegration;
use induckgo_qt::installation_directories::InstallationDirectories;
use tempfile::TempDir;

use induckgo_qt as _;

struct IntegrationScenario {
    roots: TempDir,
    appimage: PathBuf,
}

impl IntegrationScenario {
    const ENTRY: &'static str = "applications/io.github.induckgo.InDuckGo.desktop";
    const ICON: &'static str = "icons/hicolor/scalable/apps/io.github.induckgo.InDuckGo.svg";

    fn given_an_appimage_on_a_machine_without_an_installed_package() -> Self {
        Self { roots: TempDir::new().expect("roots"), appimage: PathBuf::from("/home/ola/Apps/InDuckGo.AppImage") }
    }

    fn with_the_package_installed_system_wide(self) -> Self {
        let entry = self.system_data().join(Self::ENTRY);
        fs::create_dir_all(entry.parent().expect("parent")).expect("system applications");
        fs::write(entry, "[Desktop Entry]\n").expect("system entry");
        self
    }

    fn with_the_appimage_moved_to(mut self, path: &str) -> Self {
        self.integrate();
        self.appimage = PathBuf::from(path);
        self
    }

    fn data_home(&self) -> PathBuf {
        self.roots.path().join("data")
    }

    fn system_data(&self) -> PathBuf {
        self.roots.path().join("usr-share")
    }

    fn integrate(&self) {
        let directories = InstallationDirectories::under(&self.data_home(), &self.roots.path().join("cache"));
        AppImageIntegration::new(&directories, vec![self.system_data()]).install(&self.appimage).expect("install");
    }

    fn when_started(self) -> Self {
        self.integrate();
        self
    }

    fn user_file(&self, relative: &str) -> PathBuf {
        self.data_home().join(relative)
    }
}

#[test]
fn should_write_a_menu_entry_that_starts_the_appimage() {
    let scenario = IntegrationScenario::given_an_appimage_on_a_machine_without_an_installed_package().when_started();

    let entry = fs::read_to_string(scenario.user_file(IntegrationScenario::ENTRY)).expect("entry");
    assert!(entry.contains("Exec=/home/ola/Apps/InDuckGo.AppImage %U"));
}

#[test]
fn should_write_the_icon_the_entry_names() {
    let scenario = IntegrationScenario::given_an_appimage_on_a_machine_without_an_installed_package().when_started();

    let entry = fs::read_to_string(scenario.user_file(IntegrationScenario::ENTRY)).expect("entry");
    let icon = scenario.user_file(IntegrationScenario::ICON);
    assert!(entry.contains(&format!("Icon={}", icon.display())));
    assert!(fs::read_to_string(icon).expect("icon").contains("<svg"));
}

#[test]
fn should_follow_the_appimage_when_it_is_moved() {
    let scenario = IntegrationScenario::given_an_appimage_on_a_machine_without_an_installed_package()
        .with_the_appimage_moved_to("/home/ola/Downloads/InDuckGo.AppImage")
        .when_started();

    let entry = fs::read_to_string(scenario.user_file(IntegrationScenario::ENTRY)).expect("entry");
    assert!(entry.contains("Exec=/home/ola/Downloads/InDuckGo.AppImage %U"));
}

#[test]
fn should_leave_the_installed_package_alone() {
    let scenario = IntegrationScenario::given_an_appimage_on_a_machine_without_an_installed_package()
        .with_the_package_installed_system_wide()
        .when_started();

    assert!(!Path::new(&scenario.user_file(IntegrationScenario::ENTRY)).exists());
}
