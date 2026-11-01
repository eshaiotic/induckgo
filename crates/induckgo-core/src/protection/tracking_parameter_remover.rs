//! Strips tracking parameters (`fbclid`, `gclid`, `utm_*`, ...) from a link before the browser opens it.
//!
//! Port of `DuckDuckGo`'s `LinkCleaner.cleanTrackingParameters` and its byte-preserving filter: the query is
//! split on every `&`, a parameter's name ends at the first `=`, names match exactly (case-sensitive), and
//! everything else in the URL is kept byte for byte so signed or encoded links survive.

use std::collections::HashSet;

use url::Url;

use crate::protection::privacy_configuration::PrivacyConfiguration;

const FEATURE: &str = "trackingParameters";

pub struct TrackingParameterRemover<'config> {
    configuration: &'config PrivacyConfiguration,
    user_unprotected_domains: &'config [String],
    parameters: HashSet<String>,
}

impl<'config> TrackingParameterRemover<'config> {
    #[must_use]
    pub fn new(configuration: &'config PrivacyConfiguration, user_unprotected_domains: &'config [String]) -> Self {
        Self { configuration, user_unprotected_domains, parameters: configuration.tracking_parameters() }
    }

    /// `url` without tracking parameters. `initiator` is the page the link was clicked on, if any.
    #[must_use]
    pub fn cleaned(&self, url: &str, initiator: Option<&str>) -> String {
        if self.parameters.is_empty() || self.is_excluded(url) || initiator.is_some_and(|page| self.is_excluded(page)) {
            return url.to_owned();
        }
        self.without_tracking_parameters(url).unwrap_or_else(|| url.to_owned())
    }

    /// Non-web URLs count as excluded, exactly as upstream.
    fn is_excluded(&self, url: &str) -> bool {
        let Ok(parsed) = Url::parse(url) else { return true };
        let Some(host) = parsed.host_str().filter(|_| matches!(parsed.scheme(), "http" | "https")) else { return true };
        !self.configuration.is_enabled_for_site(FEATURE, host, self.user_unprotected_domains)
    }

    fn without_tracking_parameters(&self, url: &str) -> Option<String> {
        let (before_fragment, fragment) = match url.split_once('#') {
            Some((head, fragment)) => (head, Some(fragment)),
            None => (url, None),
        };
        let (path, query) = before_fragment.split_once('?')?;
        let items: Vec<&str> = query.split('&').collect();
        let kept: Vec<&str> = items.iter().copied().filter(|item| !self.is_tracking_item(item)).collect();
        if kept.len() == items.len() {
            return None;
        }
        let mut cleaned = path.to_owned();
        if !kept.is_empty() {
            cleaned.push('?');
            cleaned.push_str(&kept.join("&"));
        }
        if let Some(fragment) = fragment {
            cleaned.push('#');
            cleaned.push_str(fragment);
        }
        Some(cleaned)
    }

    fn is_tracking_item(&self, item: &str) -> bool {
        let name = item.split_once('=').map_or(item, |(name, _)| name);
        self.parameters.contains(name)
    }
}
