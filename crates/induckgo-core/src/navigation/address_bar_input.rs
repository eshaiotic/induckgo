//! What the user typed into the address bar: either an address to open or a DuckDuckGo search.
//!
//! Only `http`, `https`, `file` and `about` URLs are opened directly. Anything else, `javascript:` and
//! `data:` included, becomes a search query, so pasting a malicious snippet into the address bar can never
//! execute it in the current page.

use std::net::Ipv4Addr;

use url::Url;

const SEARCH_URL: &str = "https://duckduckgo.com/?q=";
const DIRECT_SCHEMES: [&str; 4] = ["http://", "https://", "file://", "about:"];

pub struct AddressBarInput {
    text: String,
}

impl AddressBarInput {
    #[must_use]
    pub fn new(text: &str) -> Self {
        Self { text: text.trim().to_owned() }
    }

    /// The URL to open, or `None` for blank input.
    #[must_use]
    pub fn destination(&self) -> Option<String> {
        if self.text.is_empty() {
            return None;
        }
        Some(self.address().unwrap_or_else(|| self.search_url()))
    }

    fn address(&self) -> Option<String> {
        if self.text.chars().any(char::is_whitespace) {
            return None;
        }
        let lowercase = self.text.to_lowercase();
        if DIRECT_SCHEMES.iter().any(|scheme| lowercase.starts_with(scheme)) {
            return Url::parse(&self.text).ok().map(String::from);
        }
        let scheme = Self::implied_scheme(self.host_part())?;
        Url::parse(&format!("{scheme}://{}", self.text)).ok().map(String::from)
    }

    /// `host` of `host:port/path?query#fragment`.
    fn host_part(&self) -> &str {
        let authority = self.text.split(['/', '?', '#']).next().unwrap_or_default();
        authority.rsplit_once(':').map_or(authority, |(host, _port)| host)
    }

    /// Local machines are reached over plain HTTP; anything that looks like a public domain over HTTPS.
    fn implied_scheme(host: &str) -> Option<&'static str> {
        if host.eq_ignore_ascii_case("localhost") || host.parse::<Ipv4Addr>().is_ok() {
            return Some("http");
        }
        let labels: Vec<&str> = host.split('.').collect();
        let is_label = |label: &&str| !label.is_empty() && label.chars().all(|c| c.is_alphanumeric() || c == '-');
        let top_level = labels.last().copied().unwrap_or_default();
        let has_real_top_level = top_level.chars().count() >= 2 && top_level.chars().all(char::is_alphabetic)
            || top_level.to_ascii_lowercase().starts_with("xn--");
        (labels.len() >= 2 && labels.iter().all(is_label) && has_real_top_level).then_some("https")
    }

    fn search_url(&self) -> String {
        let query: String = form_urlencoded::byte_serialize(self.text.as_bytes()).collect();
        format!("{SEARCH_URL}{query}")
    }
}
