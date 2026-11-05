//! Keeps DuckDuckGo's protection data and Guard's filter lists up to date, and puts what arrives to use.
//!
//! The first start has nothing cached, so the data is downloaded before the window opens: a page must never
//! load unprotected because the rules were still on their way. Later starts use the cached copies at once and
//! check for newer ones in the background, again every few hours; whatever changed replaces the rules in
//! force without a restart.

use std::path::PathBuf;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use induckgo_core::guard::shared_guard_engine::SharedGuardEngine;
use induckgo_core::protection::document_fetcher::DocumentFetcher;
use induckgo_core::protection::protection_feed::FeedRefresh;
use induckgo_core::protection::protection_feed::ProtectionFeed;

use crate::guard_lists::GuardLists;
use crate::profile_policies::ProfilePolicies;
use crate::protection_documents::ProtectionDocuments;

pub struct ProtectionUpdates {
    feed: ProtectionFeed,
    fetcher: Arc<dyn DocumentFetcher>,
    cache: PathBuf,
}

impl ProtectionUpdates {
    /// How long the browser runs between two checks for newer data.
    const CHECK_INTERVAL: Duration = Duration::from_hours(6);

    #[must_use]
    pub fn new(cache: PathBuf, fetcher: Arc<dyn DocumentFetcher>) -> Self {
        Self { feed: ProtectionFeed::new(&cache), fetcher, cache }
    }

    /// Whether the protection data was ever downloaded.
    #[must_use]
    pub fn has_cached_data(&self) -> bool {
        ProtectionDocuments::from_cache(&self.cache).is_some()
    }

    /// Downloads what is missing or newer. Failures are reported on the error stream; the copies already held
    /// stay in use.
    pub fn refresh(&self) -> FeedRefresh {
        let outcome = self.feed.refresh(self.fetcher.as_ref());
        for failure in outcome.failures() {
            eprintln!("induckgo: protection data not refreshed: {failure}");
        }
        outcome
    }

    /// Checks for newer data now and then for as long as the browser runs, applying what changed to `policies`
    /// and `guard`. With `check_now` false the first check waits one interval (the data was just downloaded).
    pub fn keep_current(
        self: Arc<Self>,
        policies: Arc<ProfilePolicies>,
        guard: Arc<SharedGuardEngine>,
        check_now: bool,
    ) {
        thread::spawn(move || {
            let mut pause = !check_now;
            loop {
                if pause {
                    thread::sleep(Self::CHECK_INTERVAL);
                }
                pause = true;
                if self.refresh().changed() {
                    if let Some(documents) = ProtectionDocuments::from_cache(&self.cache) {
                        policies.use_documents(&documents);
                    }
                    GuardLists::in_cache(&self.cache).install_into(&guard);
                }
            }
        });
    }
}
