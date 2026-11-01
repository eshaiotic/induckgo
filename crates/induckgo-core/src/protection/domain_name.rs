//! A host name in the form `WebKit` rules and URL hosts use.

use std::borrow::Borrow;
use std::fmt;

/// Lower-case ASCII (punycode), without a trailing dot: `Żółw.PL.` becomes `xn--w-uga1v8h.pl`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct DomainName {
    ascii: String,
}

impl DomainName {
    /// Name suffixes that only resolve inside a home or office network.
    const LOCAL_NETWORK_SUFFIXES: [&'static str; 5] = [".local", ".localhost", ".lan", ".internal", ".home.arpa"];

    #[must_use]
    pub fn new(raw: &str) -> Self {
        let cleaned = raw.trim().trim_end_matches('.');
        let ascii = idna::domain_to_ascii(cleaned).unwrap_or_else(|_| cleaned.to_lowercase());
        Self { ascii }
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.ascii
    }

    /// `www.shop.com` yields `www.shop.com`, then `shop.com`; the bare top-level label is never yielded,
    /// so listing `shop.com` never accidentally covers every `.com` site.
    pub fn self_and_parent_domains(&self) -> impl Iterator<Item = &str> {
        let mut next = Some(self.ascii.as_str());
        std::iter::from_fn(move || {
            let current = next.filter(|candidate| candidate.contains('.'))?;
            next = current.split_once('.').map(|(_, parent)| parent);
            Some(current)
        })
    }

    /// Single-label names (`nas`), `localhost` and local suffixes (`printer.local`).
    #[must_use]
    pub fn is_local_network(&self) -> bool {
        !self.ascii.contains('.')
            || self.ascii == "localhost"
            || Self::LOCAL_NETWORK_SUFFIXES.iter().any(|suffix| self.ascii.ends_with(suffix))
    }
}

impl Borrow<str> for DomainName {
    fn borrow(&self) -> &str {
        &self.ascii
    }
}

impl fmt::Display for DomainName {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.ascii)
    }
}
