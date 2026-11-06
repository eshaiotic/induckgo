//! What protection data the browser starts with: the cached copies at once, or a download on the first start.

use std::fs;
use std::sync::Arc;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;

use induckgo_core::protection::document_fetcher::DocumentFetcher;
use induckgo_core::protection::document_fetcher::FetchFailure;
use induckgo_core::protection::document_fetcher::FetchedDocument;
use induckgo_core::protection::protection_feed::ProtectionFeed;
use induckgo_qt::protection_updates::ProtectionUpdates;
use tempfile::TempDir;

use induckgo_qt as _;

/// A server that has the two protection documents (or nothing at all) and counts the requests it got.
struct CountingServer {
    has_documents: bool,
    requests: AtomicUsize,
}

impl CountingServer {
    fn serving_the_documents() -> Arc<Self> {
        Arc::new(Self { has_documents: true, requests: AtomicUsize::new(0) })
    }

    fn unreachable() -> Arc<Self> {
        Arc::new(Self { has_documents: false, requests: AtomicUsize::new(0) })
    }

    fn requests(&self) -> usize {
        self.requests.load(Ordering::SeqCst)
    }
}

impl DocumentFetcher for CountingServer {
    fn fetch(&self, url: &str, _etag: Option<&str>, _max_bytes: usize) -> Result<FetchedDocument, FetchFailure> {
        self.requests.fetch_add(1, Ordering::SeqCst);
        let body = match url {
            ProtectionFeed::TRACKER_DATA_URL if self.has_documents => r#"{"trackers":{}}"#,
            ProtectionFeed::CONFIGURATION_URL if self.has_documents => r#"{"features":{}}"#,
            _ => return Err(FetchFailure::new("no route to host")),
        };
        Ok(FetchedDocument::Body { bytes: body.as_bytes().to_vec(), etag: None })
    }
}

fn cache_holding_both_documents() -> TempDir {
    let cache = TempDir::new().expect("cache");
    fs::write(cache.path().join("tracker-data.json"), r#"{"trackers":{}}"#).expect("tracker data");
    fs::write(cache.path().join("configuration.json"), r#"{"features":{}}"#).expect("configuration");
    cache
}

#[test]
fn should_start_with_the_cached_documents_without_downloading() {
    let cache = cache_holding_both_documents();
    let server = CountingServer::serving_the_documents();
    let updates = ProtectionUpdates::new(cache.path().to_path_buf(), Arc::clone(&server) as Arc<dyn DocumentFetcher>);

    let starting = updates.starting_protection();

    assert!(starting.documents.is_some());
    assert!(!starting.is_first_start);
    assert_eq!(server.requests(), 0);
}

#[test]
fn should_download_the_documents_first_when_nothing_is_cached() {
    let cache = TempDir::new().expect("cache");
    let server = CountingServer::serving_the_documents();
    let updates = ProtectionUpdates::new(cache.path().to_path_buf(), Arc::clone(&server) as Arc<dyn DocumentFetcher>);

    let starting = updates.starting_protection();

    assert!(starting.documents.is_some());
    assert!(starting.is_first_start);
}

#[test]
fn should_start_without_documents_when_the_first_download_fails() {
    let cache = TempDir::new().expect("cache");
    let server = CountingServer::unreachable();
    let updates = ProtectionUpdates::new(cache.path().to_path_buf(), server as Arc<dyn DocumentFetcher>);

    let starting = updates.starting_protection();

    assert!(starting.documents.is_none());
    assert!(starting.is_first_start);
}
