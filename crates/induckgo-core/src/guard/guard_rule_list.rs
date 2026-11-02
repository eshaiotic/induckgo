//! The WebKit rule lists InDuckGo Guard builds from community filter lists.
//!
//! Each Guard rule list is compiled as a WebKit list of its own, for two WebKit limits: a list holds at most
//! 150 000 rules, and an exception (`ignore-previous-rules`) only cancels rules of its own list. That is why
//! uBlock Origin's "Unbreak" exceptions are read into every list, and why every list ends with the sites the
//! user switched protection off for.
//!
//! Brave's `adblock` crate (MPL-2.0) converts Adblock Plus syntax into WebKit's JSON, as Brave does for its
//! iOS browser. Filters without `$third-party` apply to a site's own requests too, which is what Brave Shields
//! does only in their "aggressive" mode.

use std::collections::HashMap;
use std::collections::hash_map::DefaultHasher;
use std::hash::Hash;
use std::hash::Hasher;

use adblock::content_blocking::CbRule;
use adblock::content_blocking::CbType;
use adblock::content_blocking::ignore_previous_fp_documents;
use adblock::lists::FilterSet;
use adblock::lists::ParseOptions;

use crate::guard::filter_list_source::FilterListSource;
use crate::protection::content_blocker_rule::ContentBlockerRule;
use crate::protection::content_blocker_rule::ResourceType;
use crate::protection::content_blocker_rule::RuleAction;
use crate::protection::content_blocker_rule::Trigger;
use crate::protection::content_blocker_rule::UrlFilter;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GuardRuleList {
    name: &'static str,
    sources: &'static [FilterListSource],
    blocks_link_pings: bool,
}

impl GuardRuleList {
    /// Ads, malware and scam sites, and the Polish sites' own ad servers.
    pub const ADS: Self = Self {
        name: "guard-ads",
        sources: &[
            FilterListSource::EASYLIST,
            FilterListSource::UBLOCK_FILTERS,
            FilterListSource::UBLOCK_BADWARE,
            FilterListSource::PETER_LOWE,
            FilterListSource::POLISH_FILTERS,
            FilterListSource::UBLOCK_UNBREAK,
        ],
        blocks_link_pings: false,
    };
    /// Trackers, analytics and link-click pings.
    pub const PRIVACY: Self = Self {
        name: "guard-privacy",
        sources: &[FilterListSource::EASYPRIVACY, FilterListSource::UBLOCK_PRIVACY, FilterListSource::UBLOCK_UNBREAK],
        blocks_link_pings: true,
    };
    /// Cookie consent notices.
    pub const ANNOYANCES: Self = Self {
        name: "guard-annoyances",
        sources: &[FilterListSource::COOKIE_NOTICES, FilterListSource::UBLOCK_UNBREAK],
        blocks_link_pings: false,
    };
    pub const ALL: [Self; 3] = [Self::ADS, Self::PRIVACY, Self::ANNOYANCES];
    /// WebKit rejects a whole list above this many rules (`maxRuleCount` in its `ContentExtensionParser`).
    pub const MAX_RULES: usize = 150_000;
    /// Bump when the conversion changes (an `adblock` upgrade included), so lists compiled by an older
    /// InDuckGo are rebuilt.
    const CONVERSION_REVISION: u32 = 1;

    /// The name the compiled list is stored under.
    #[must_use]
    pub fn name(self) -> &'static str {
        self.name
    }

    #[must_use]
    pub fn sources(self) -> &'static [FilterListSource] {
        self.sources
    }

    /// Whether the list would hold any rule; WebKit rejects an empty list.
    #[must_use]
    pub fn has_rules(self, downloaded: &HashMap<FilterListSource, String>) -> bool {
        self.blocks_link_pings || self.sources.iter().any(|source| downloaded.contains_key(source))
    }

    /// Identifies what the list is built from: its filter lists, the user's unprotected sites and the
    /// conversion itself. An unchanged fingerprint means the list compiled last time is still right, so the
    /// conversion (a few tenths of a second and tens of megabytes) is skipped.
    #[must_use]
    pub fn fingerprint(
        self,
        downloaded: &HashMap<FilterListSource, String>,
        user_unprotected_domains: &[String],
    ) -> String {
        let mut hasher = DefaultHasher::new();
        (self.name, Self::CONVERSION_REVISION, user_unprotected_domains).hash(&mut hasher);
        for source in self.sources {
            (source.id(), downloaded.get(source)).hash(&mut hasher);
        }
        format!("{:016x}", hasher.finish())
    }

    /// The list as WebKit reads it (a JSON array): link-click pings blocked (privacy list), the downloaded filters
    /// converted, then one rule standing everything down on the sites the user switched protection off for.
    /// Filter lists not downloaded yet are left out.
    #[must_use]
    pub fn content_rules(
        self,
        downloaded: &HashMap<FilterListSource, String>,
        user_unprotected_domains: &[String],
    ) -> String {
        let mut leading = Vec::new();
        if self.blocks_link_pings {
            let pings = Trigger { resource_type: Some(vec![ResourceType::Ping]), ..Trigger::new(UrlFilter::any_url()) };
            leading.push(ContentBlockerRule::new(pings, RuleAction::Block).to_json().to_string());
        }
        let mut closing = Vec::new();
        if !user_unprotected_domains.is_empty() {
            let unprotected =
                Trigger { if_domain: Some(user_unprotected_domains.to_vec()), ..Trigger::new(UrlFilter::any_url()) };
            closing.push(ContentBlockerRule::new(unprotected, RuleAction::IgnorePreviousRules).to_json().to_string());
        }
        let converted = self.converted(downloaded, Self::MAX_RULES - leading.len() - closing.len());
        format!("[{}]", leading.into_iter().chain(converted).chain(closing).collect::<Vec<String>>().join(","))
    }

    /// The filters as WebKit rules, at most `room` of them, each serialised on its own: a whole list as
    /// `serde_json::Value`s would take ten times the memory.
    fn converted(self, downloaded: &HashMap<FilterListSource, String>, room: usize) -> Vec<String> {
        let mut filters = FilterSet::new(true);
        for text in self.sources.iter().filter_map(|source| downloaded.get(source)) {
            filters.add_filter_list(text.clone(), ParseOptions::default());
        }
        let Ok((mut rules, _)) = filters.into_content_blocking() else { return Vec::new() };
        Self::move_page_exception_ahead_of_hiding(&mut rules);
        Self::fit(&mut rules, room);
        rules.iter().filter_map(|rule| serde_json::to_string(rule).ok()).collect()
    }

    /// `adblock` closes its output with an exception for first-party documents, so that no filter blocks the
    /// page the user asked for. WebKit picks a page's hidden elements on that same load, though, so at the end
    /// the exception would cancel every element-hiding rule: it moves ahead of them.
    fn move_page_exception_ahead_of_hiding(rules: &mut Vec<CbRule>) {
        if rules.last() != Some(&ignore_previous_fp_documents()) {
            return;
        }
        let Some(exception) = rules.pop() else { return };
        let first_hiding =
            rules.iter().position(|rule| rule.action.typ == CbType::CssDisplayNone).unwrap_or(rules.len());
        rules.insert(first_hiding, exception);
    }

    /// Drops the blocking rules just before the first exception until the rules fit: losing a few filters
    /// beats WebKit rejecting the whole list, and every exception is kept so that nothing they repair breaks
    /// again.
    fn fit(rules: &mut Vec<CbRule>, room: usize) {
        let excess = rules.len().saturating_sub(room);
        if excess > 0 {
            let first_exception =
                rules.iter().position(|rule| rule.action.typ == CbType::IgnorePreviousRules).unwrap_or(rules.len());
            rules.drain(first_exception.saturating_sub(excess)..first_exception);
        }
    }
}
