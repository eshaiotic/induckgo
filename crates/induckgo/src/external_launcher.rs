//! Hands work to other processes: another InDuckGo process (browser, profile, web app) or another program.
//!
//! InDuckGo processes are GTK applications with fixed ids, so starting `induckgo <address>` while the browser
//! runs only forwards the address to the running browser over D-Bus; the started process exits at once.

use std::ffi::OsStr;

use induckgo_core::launch::launch_request::LaunchRequest;
use induckgo_core::profile::profile_id::ProfileId;
use induckgo_core::web_app::desktop_entry::DesktopEntry;
use induckgo_core::web_app::web_app_id::WebAppId;
use url::Url;

pub struct ExternalLauncher;

impl ExternalLauncher {
    const LOG_DOMAIN: &'static str = "induckgo";

    /// Opens a web address in the default profile's browser.
    pub fn open_in_browser(address: &str) {
        if Url::parse(address).is_ok_and(|url| matches!(url.scheme(), "http" | "https")) {
            Self::start_induckgo(&[address]);
        }
    }

    pub fn start_profile(profile: &ProfileId) {
        if profile.is_default() {
            Self::start_induckgo(&[]);
        } else {
            Self::start_induckgo(&[LaunchRequest::PROFILE_SWITCH, profile.as_str()]);
        }
    }

    pub fn start_web_app(web_app: &WebAppId) {
        Self::start_induckgo(&[DesktopEntry::WEB_APP_SWITCH, web_app.as_str()]);
    }

    /// Opens a non-web link (`mailto:`, `tel:`, ...) with the program the desktop chooses for it.
    pub fn open_outside(address: &str, parent: Option<&gtk::Window>) {
        gtk::UriLauncher::new(address).launch(parent, None::<&gio::Cancellable>, |result| {
            if let Err(error) = result {
                glib::g_message!(Self::LOG_DOMAIN, "no application opened the link: {error}");
            }
        });
    }

    fn start_induckgo(arguments: &[&str]) {
        let executable = match std::env::current_exe() {
            Ok(executable) => executable,
            Err(error) => {
                glib::g_warning!(Self::LOG_DOMAIN, "cannot locate the InDuckGo executable: {error}");
                return;
            }
        };
        let mut command_line = vec![executable.as_os_str()];
        command_line.extend(arguments.iter().map(OsStr::new));
        match gio::Subprocess::newv(&command_line, gio::SubprocessFlags::NONE) {
            Ok(process) => process.wait_async(None::<&gio::Cancellable>, |_| {}),
            Err(error) => glib::g_warning!(Self::LOG_DOMAIN, "cannot start InDuckGo: {error}"),
        }
    }
}
