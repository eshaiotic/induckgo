//! Starts InDuckGo's Qt window; see the library for the parts.

use std::process::ExitCode;

use induckgo_qt::browser_application::BrowserApplication;

fn main() -> ExitCode {
    BrowserApplication::run()
}
