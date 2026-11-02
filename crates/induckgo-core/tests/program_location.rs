//! Which file starts InDuckGo again: the AppImage itself, never its temporary mount.

use std::ffi::OsStr;
use std::path::Path;
use std::path::PathBuf;

use induckgo_core::launch::program_location::ProgramLocation;

struct RunningProgramScenario {
    appimage: Option<&'static str>,
}

impl RunningProgramScenario {
    const MOUNTED_EXECUTABLE: &'static str = "/tmp/.mount_InDuckX1/usr/bin/induckgo";

    fn given_started_from_appimage(appimage: &'static str) -> Self {
        Self { appimage: Some(appimage) }
    }

    fn given_installed_program() -> Self {
        Self { appimage: None }
    }

    fn when_locating(&self) -> PathBuf {
        ProgramLocation::choose(self.appimage.map(OsStr::new), PathBuf::from(Self::MOUNTED_EXECUTABLE))
    }
}

#[test]
fn should_start_the_appimage_file_instead_of_its_temporary_mount() {
    let program = RunningProgramScenario::given_started_from_appimage("/tmp/InDuckGo-x86_64.AppImage").when_locating();

    assert_eq!(program, Path::new("/tmp/InDuckGo-x86_64.AppImage"));
}

#[test]
fn should_start_the_running_executable_when_installed_as_a_package() {
    let program = RunningProgramScenario::given_installed_program().when_locating();

    assert_eq!(program, Path::new(RunningProgramScenario::MOUNTED_EXECUTABLE));
}

#[test]
fn should_ignore_an_appimage_variable_that_is_not_an_absolute_path() {
    let program = RunningProgramScenario::given_started_from_appimage("InDuckGo.AppImage").when_locating();

    assert_eq!(program, Path::new(RunningProgramScenario::MOUNTED_EXECUTABLE));
}
