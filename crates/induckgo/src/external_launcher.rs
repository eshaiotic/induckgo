//! Hands work to other processes: another InDuckGo process (browser, profile, web app) or another program.
//!
//! InDuckGo processes are GTK applications with fixed ids, so starting `induckgo <address>` while the browser
//! runs only forwards the address to the running browser over D-Bus; the started process exits at once.

use std::ffi::OsStr;
use std::path::PathBuf;

use induckgo_core::launch::launch_request::LaunchRequest;
use induckgo_core::launch::program_location::ProgramLocation;
use induckgo_core::profile::profile_id::ProfileId;
use induckgo_core::web_app::desktop_entry::DesktopEntry;
use induckgo_core::web_app::web_app_id::WebAppId;
use url::Url;

pub struct ExternalLauncher;

impl ExternalLauncher {
    const LOG_DOMAIN: &'static str = "induckgo";

    /// Opens a web address in the default profile's browser; `false` when nothing was started (not a web
    /// address, or InDuckGo could not be started, which is logged).
    pub fn open_in_browser(address: &str) -> bool {
        Url::parse(address).is_ok_and(|url| matches!(url.scheme(), "http" | "https"))
            && Self::start_induckgo(&[address])
    }

    pub fn start_profile(profile: &ProfileId) {
        if profile.is_default() {
            Self::start_induckgo(&[]);
        } else {
            Self::start_induckgo(&[LaunchRequest::PROFILE_SWITCH, profile.as_str()]);
        }
    }

    /// Opens a Tor window, in the Tor window's own process.
    pub fn start_tor_window() {
        Self::start_induckgo(&[LaunchRequest::TOR_SWITCH]);
    }

    /// `false` when the app's process could not be started (logged).
    pub fn start_web_app(web_app: &WebAppId) -> bool {
        Self::start_induckgo(&[DesktopEntry::WEB_APP_SWITCH, web_app.as_str()])
    }

    /// Opens a non-web link (`mailto:`, `tel:`, ...) with the program the desktop chooses for it.
    pub fn open_outside(address: &str, parent: Option<&gtk::Window>) {
        gtk::UriLauncher::new(address).launch(parent, None::<&gio::Cancellable>, |result| {
            if let Err(error) = result {
                glib::g_message!(Self::LOG_DOMAIN, "no application opened the link: {error}");
            }
        });
    }

    /// The file that starts InDuckGo: the AppImage when running from one, else the running executable.
    ///
    /// # Errors
    /// The running executable cannot be determined.
    pub fn program() -> std::io::Result<PathBuf> {
        let appimage = std::env::var_os(ProgramLocation::APPIMAGE_VARIABLE);
        Ok(ProgramLocation::choose(appimage.as_deref(), std::env::current_exe()?))
    }

    /// `false` when the process could not be started (logged).
    fn start_induckgo(arguments: &[&str]) -> bool {
        let executable = match Self::program() {
            Ok(executable) => executable,
            Err(error) => {
                glib::g_warning!(Self::LOG_DOMAIN, "cannot locate the InDuckGo executable: {error}");
                return false;
            }
        };
        let mut command_line = vec![executable.as_os_str()];
        command_line.extend(arguments.iter().map(OsStr::new));
        match gio::Subprocess::newv(&command_line, gio::SubprocessFlags::NONE) {
            Ok(process) => {
                process.wait_async(None::<&gio::Cancellable>, |_| {});
                true
            }
            Err(error) => {
                glib::g_warning!(Self::LOG_DOMAIN, "cannot start InDuckGo: {error}");
                false
            }
        }
    }
}
