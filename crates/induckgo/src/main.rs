//! InDuckGo: a privacy-first WebKitGTK browser for Linux built on DuckDuckGo's open-source protections.

mod app_context;
mod app_icons;
mod browser_window;
mod browser_windows;
mod catalog_files;
mod download_tracker;
mod error_page_view;
mod external_launcher;
mod find_bar;
mod http_client;
mod induckgo_application;
mod installation_paths;
mod management_dialogs;
mod page_protections;
mod page_view;
mod permission_prompt;
mod profile_session;
mod protection_feed;
mod public_suffix_list;
mod running_instances;
mod stored_file;
mod web_app_installer;
mod web_app_window;

use crate::induckgo_application::InDuckGoApplication;

fn main() -> glib::ExitCode {
    InDuckGoApplication::run()
}
