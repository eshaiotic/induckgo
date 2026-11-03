//! What every window of this process shares: file locations, the HTTP client, the live profile and its
//! catalogs, the list of open browser windows and, in the Tor window's process, its Tor.

use std::rc::Rc;

use crate::app_icons::AppIcons;
use crate::browser_windows::BrowserWindows;
use crate::catalog_files::CatalogFiles;
use crate::http_client::HttpClient;
use crate::installation_paths::InstallationPaths;
use crate::profile_session::ProfileSession;
use crate::tor_connection::TorConnection;

pub struct AppContext {
    pub paths: InstallationPaths,
    pub http: Rc<HttpClient>,
    pub catalogs: CatalogFiles,
    pub icons: AppIcons,
    pub session: Rc<ProfileSession>,
    pub browser_windows: BrowserWindows,
    /// `Some` only in the Tor window's process.
    pub tor: Option<Rc<TorConnection>>,
}
