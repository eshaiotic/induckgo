//! InDuckGo: a privacy-first WebKitGTK browser for Linux built on DuckDuckGo's open-source protections.

#[macro_use]
mod interface_messages;
mod app_context;
mod app_icons;
mod browser_window;
mod browser_windows;
mod catalog_files;
mod closed_proxy;
mod download_tracker;
mod error_page_view;
mod external_launcher;
mod feed_document;
mod filter_list_feed;
mod find_bar;
mod http_client;
mod induckgo_application;
mod installation_paths;
mod management_dialogs;
mod page_protections;
mod page_view;
mod permission_prompt;
mod preferences_dialog;
mod profile_session;
mod protection_feed;
mod public_suffix_list;
mod running_instances;
mod site_zoom;
mod stored_file;
mod toolbar_preferences;
mod tor_connection;
mod tor_connection_page;
mod tor_control_port;
mod tor_settings_dialog;
mod web_app_installer;
mod web_app_window;
mod zoom_controls;

use crate::induckgo_application::InDuckGoApplication;

fn main() -> glib::ExitCode {
    InDuckGoApplication::run()
}
