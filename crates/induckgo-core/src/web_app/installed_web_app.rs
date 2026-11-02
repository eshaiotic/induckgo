//! A site the user installed as its own app.

use url::Url;

use crate::web_app::web_app_id::WebAppId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstalledWebApp {
    pub id: WebAppId,
    /// Shown in the application menu and the window title.
    pub name: String,
    /// Always an `http` or `https` URL (checked on install and on load).
    pub start_url: String,
}

impl InstalledWebApp {
    /// The start page's host, e.g. `online.mbank.pl`.
    #[must_use]
    pub fn host(&self) -> String {
        Url::parse(&self.start_url).ok().and_then(|url| url.host_str().map(str::to_owned)).unwrap_or_default()
    }
}
