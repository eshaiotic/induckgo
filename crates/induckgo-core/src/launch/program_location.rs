//! The file that starts InDuckGo again: for web app desktop entries and for opening the browser from a web app.
//!
//! Inside an AppImage the running executable lives in a temporary mount (`/tmp/.mount_XXXX/usr/bin/induckgo`)
//! that disappears when the program quits, so a desktop entry pointing there breaks, and a process started
//! from there crashes once the mount is gone. The AppImage runtime names the real file in `APPIMAGE`.

use std::ffi::OsStr;
use std::path::Path;
use std::path::PathBuf;

pub struct ProgramLocation;

impl ProgramLocation {
    /// The environment variable the AppImage runtime sets to the AppImage file.
    pub const APPIMAGE_VARIABLE: &'static str = "APPIMAGE";

    /// The AppImage file when `appimage` (the variable's value) is an absolute path, else the running executable.
    #[must_use]
    pub fn choose(appimage: Option<&OsStr>, running_executable: PathBuf) -> PathBuf {
        appimage.map(Path::new).filter(|path| path.is_absolute()).map_or(running_executable, Path::to_path_buf)
    }
}
