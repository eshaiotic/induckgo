//! What every window of this process shares: file locations, the HTTP client, the live profile and its
//! catalogs, and the list of open browser windows.

use std::rc::Rc;

use crate::app_icons::AppIcons;
use crate::browser_windows::BrowserWindows;
use crate::catalog_files::CatalogFiles;
use crate::http_client::HttpClient;
use crate::installation_paths::InstallationPaths;
use crate::profile_session::ProfileSession;

pub struct AppContext {
    pub paths: InstallationPaths,
    pub http: Rc<HttpClient>,
    pub catalogs: CatalogFiles,
    pub icons: AppIcons,
    pub session: Rc<ProfileSession>,
    pub browser_windows: BrowserWindows,
}
