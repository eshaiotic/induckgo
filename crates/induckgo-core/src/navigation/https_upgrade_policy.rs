//! Upgrades plain-HTTP navigations to HTTPS ("HTTPS-first").
//!
//! `DuckDuckGo` decides with its Smarter Encryption list; `InDuckGo` simply tries HTTPS for every public host and
//! falls back to HTTP for the rest of the session once a host fails over HTTPS. Hosts in the privacy
//! configuration's `https` exceptions and local-network hosts (which almost never have valid certificates)
//! are left alone.

use std::collections::HashSet;

use url::Host;
use url::Url;

use crate::protection::domain_name::DomainName;
use crate::protection::site_list::SiteList;

#[derive(Debug, Clone, Default)]
pub struct HttpsUpgradePolicy {
    excluded: SiteList,
    failed_hosts: HashSet<DomainName>,
}

impl HttpsUpgradePolicy {
    #[must_use]
    pub fn new(excluded_domains: &[String]) -> Self {
        Self { excluded: SiteList::new(excluded_domains.iter().map(String::as_str)), failed_hosts: HashSet::new() }
    }

    /// The HTTPS version of `url`, or `None` when the navigation should go ahead unchanged.
    #[must_use]
    pub fn upgraded(&self, url: &str) -> Option<String> {
        let mut parsed = Url::parse(url).ok().filter(|parsed| parsed.scheme() == "http")?;
        let Some(Host::Domain(host)) = parsed.host() else { return None };
        let host = DomainName::new(host);
        if host.is_local_network() || self.excluded.covers(host.as_str()) || self.failed_hosts.contains(&host) {
            return None;
        }
        parsed.set_scheme("https").ok()?;
        Some(parsed.into())
    }

    /// Remembers that `url`'s host could not be loaded over HTTPS, so it is not upgraded again.
    pub fn record_failed_upgrade(&mut self, url: &str) {
        if let Some(host) = Url::parse(url).ok().as_ref().and_then(Url::host_str) {
            self.failed_hosts.insert(DomainName::new(host));
        }
    }
}
