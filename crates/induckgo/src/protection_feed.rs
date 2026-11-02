//! DuckDuckGo's Tracker Data Set and privacy configuration, downloaded at run time.
//!
//! The Tracker Data Set is licensed CC BY-NC-SA 4.0, so it is never bundled: InDuckGo downloads the macOS
//! edition from DuckDuckGo's CDN and keeps the last good copy in the cache directory. Refreshes are
//! conditional (`ETag`), so an unchanged list costs one small request. A download replaces the cached copy
//! only after it parsed as the expected kind of document.

use std::error::Error;
use std::path::Path;

use serde_json::Value;

use crate::http_client::Fetched;
use crate::http_client::HttpClient;
use crate::stored_file::StoredFile;

pub struct ProtectionDocuments {
    pub tracker_data: Value,
    pub configuration: Value,
}

struct FeedDocument {
    url: &'static str,
    /// A top-level key every genuine document of this kind has.
    required_key: &'static str,
    content: StoredFile,
    etag: StoredFile,
}

impl FeedDocument {
    const MAX_BYTES: usize = 32 * 1024 * 1024;

    fn new(directory: &Path, name: &str, url: &'static str, required_key: &'static str) -> Self {
        Self {
            url,
            required_key,
            content: StoredFile::new(directory.join(format!("{name}.json"))),
            etag: StoredFile::new(directory.join(format!("{name}.etag"))),
        }
    }

    fn cached(&self) -> Option<Value> {
        self.content.read_json().filter(|document| self.is_genuine(document))
    }

    /// Downloads a newer copy; `Ok(false)` when the cached copy is still current.
    async fn refresh(&self, http: &HttpClient) -> Result<bool, Box<dyn Error>> {
        let etag = self.etag.read_text().filter(|_| self.content.exists());
        match http.get(self.url, Self::MAX_BYTES, etag.as_deref()).await? {
            Fetched::NotModified => Ok(false),
            Fetched::Body { bytes, etag } => {
                let document: Value = serde_json::from_slice(&bytes)?;
                if !self.is_genuine(&document) {
                    return Err(format!("{} did not return a {} document", self.url, self.required_key).into());
                }
                self.content.write(&bytes)?;
                match etag {
                    Some(etag) => self.etag.write(etag.as_bytes())?,
                    None => self.etag.remove()?,
                }
                Ok(true)
            }
        }
    }

    fn is_genuine(&self, document: &Value) -> bool {
        document.get(self.required_key).is_some_and(Value::is_object)
    }
}

pub struct ProtectionFeed {
    tracker_data: FeedDocument,
    configuration: FeedDocument,
}

impl ProtectionFeed {
    const TRACKER_DATA_URL: &'static str = "https://staticcdn.duckduckgo.com/trackerblocking/v6/current/macos-tds.json";
    const CONFIGURATION_URL: &'static str =
        "https://staticcdn.duckduckgo.com/trackerblocking/config/v4/macos-config.json";

    pub fn new(cache_directory: &Path) -> Self {
        Self {
            tracker_data: FeedDocument::new(cache_directory, "tracker-data", Self::TRACKER_DATA_URL, "trackers"),
            configuration: FeedDocument::new(cache_directory, "configuration", Self::CONFIGURATION_URL, "features"),
        }
    }

    /// The last good copies, if both were ever downloaded.
    pub fn cached(&self) -> Option<ProtectionDocuments> {
        Some(ProtectionDocuments {
            tracker_data: self.tracker_data.cached()?,
            configuration: self.configuration.cached()?,
        })
    }

    /// Fresh documents when either changed, `None` when both cached copies are current.
    ///
    /// # Errors
    /// Network, HTTP or parse failures; the cached copies are left untouched.
    pub async fn refreshed(&self, http: &HttpClient) -> Result<Option<ProtectionDocuments>, Box<dyn Error>> {
        let tracker_data_changed = self.tracker_data.refresh(http).await?;
        let configuration_changed = self.configuration.refresh(http).await?;
        if !(tracker_data_changed || configuration_changed) {
            return Ok(None);
        }
        Ok(self.cached())
    }
}
