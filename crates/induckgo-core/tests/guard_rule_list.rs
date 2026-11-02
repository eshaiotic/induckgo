//! InDuckGo Guard: community filter lists (EasyList, uBlock Origin, ...) turned into WebKit rule lists that
//! block ads, trackers, link-click pings and cookie notices on top of DuckDuckGo's tracker blocking.

use std::collections::HashMap;

use induckgo_core::guard::filter_list_source::FilterListSource;
use induckgo_core::guard::guard_rule_list::GuardRuleList;
use regex::RegexBuilder;
use serde_json::Value;

struct GuardScenario {
    downloaded: HashMap<FilterListSource, String>,
    user_unprotected_domains: Vec<String>,
}

impl GuardScenario {
    fn given_nothing_downloaded() -> Self {
        Self { downloaded: HashMap::new(), user_unprotected_domains: Vec::new() }
    }

    fn given_downloaded(source: FilterListSource, text: &str) -> Self {
        Self::given_nothing_downloaded().and_downloaded(source, text)
    }

    fn and_downloaded(mut self, source: FilterListSource, text: &str) -> Self {
        self.downloaded.insert(source, text.to_owned());
        self
    }

    fn with_protection_switched_off_by_user_on(mut self, domain: &str) -> Self {
        self.user_unprotected_domains.push(domain.to_owned());
        self
    }

    fn when_built(&self, list: GuardRuleList) -> WebKitRuleList {
        let source = list.content_rules(&self.downloaded, &self.user_unprotected_domains);
        WebKitRuleList { rules: serde_json::from_str(&source).expect("a JSON array of rules") }
    }

    fn when_fingerprinted(&self, list: GuardRuleList) -> String {
        list.fingerprint(&self.downloaded, &self.user_unprotected_domains)
    }
}

/// One load as WebKit sees it when it consults a rule list.
struct Load<'a> {
    url: &'a str,
    resource_type: &'a str,
    page_host: &'a str,
}

impl Load<'_> {
    fn request_host(&self) -> &str {
        let authority = self.url.split_once("://").map_or(self.url, |(_, rest)| rest);
        authority.split(['/', ':', '?']).next().unwrap_or_default()
    }

    fn is_third_party(&self) -> bool {
        let (request, page) = (self.request_host(), self.page_host);
        !(request == page || request.ends_with(&format!(".{page}")) || page.ends_with(&format!(".{request}")))
    }
}

/// A rule list evaluated the way WebKit does: in order, `ignore-previous-rules` cancelling every action that
/// matched before it.
struct WebKitRuleList {
    rules: Vec<Value>,
}

impl WebKitRuleList {
    fn then_blocks(&self, url: &str, page_host: &str) {
        assert!(self.blocks(&Load { url, resource_type: "script", page_host }), "{url} on {page_host} loads");
    }

    fn then_allows(&self, url: &str, page_host: &str) {
        assert!(!self.blocks(&Load { url, resource_type: "script", page_host }), "{url} on {page_host} blocked");
    }

    fn then_allows_page(&self, url: &str, page_host: &str) {
        assert!(!self.blocks(&Load { url, resource_type: "document", page_host }), "{url} itself blocked");
    }

    fn then_blocks_link_ping(&self, url: &str, page_host: &str) {
        assert!(self.blocks(&Load { url, resource_type: "ping", page_host }), "ping to {url} sent");
    }

    fn then_allows_link_ping(&self, url: &str, page_host: &str) {
        assert!(!self.blocks(&Load { url, resource_type: "ping", page_host }), "ping to {url} blocked");
    }

    fn then_hides(&self, selector: &str, page_host: &str) {
        assert!(self.hidden_selectors(page_host).contains(&selector.to_owned()), "{selector} shown on {page_host}");
    }

    fn then_shows(&self, selector: &str, page_host: &str) {
        assert!(!self.hidden_selectors(page_host).contains(&selector.to_owned()), "{selector} hidden on {page_host}");
    }

    /// Counted, not evaluated: matching 150 000 regular expressions would take minutes.
    fn then_fits_in_webkit_with_exceptions(&self, expected: usize) {
        assert_eq!((self.rules.len() <= GuardRuleList::MAX_RULES, self.exception_count()), (true, expected));
    }

    fn exception_count(&self) -> usize {
        self.rules.iter().filter(|rule| rule["action"]["type"] == "ignore-previous-rules").count()
    }

    fn blocks(&self, load: &Load) -> bool {
        self.surviving_actions(load).iter().any(|action| action["type"] == "block")
    }

    /// Selectors hidden on a page: WebKit applies `css-display-none` rules that match the page's own load.
    fn hidden_selectors(&self, page_host: &str) -> Vec<String> {
        let url = format!("https://{page_host}/");
        let load = Load { url: &url, resource_type: "document", page_host };
        self.surviving_actions(&load)
            .iter()
            .filter(|action| action["type"] == "css-display-none")
            .flat_map(|action| action["selector"].as_str().unwrap_or_default().split(", ").map(str::to_owned))
            .collect()
    }

    fn surviving_actions(&self, load: &Load) -> Vec<&Value> {
        let mut actions = Vec::new();
        for rule in self.rules.iter().filter(|rule| Self::applies(&rule["trigger"], load)) {
            if rule["action"]["type"] == "ignore-previous-rules" {
                actions.clear();
            } else {
                actions.push(&rule["action"]);
            }
        }
        actions
    }

    fn applies(trigger: &Value, load: &Load) -> bool {
        let case_sensitive = trigger["url-filter-is-case-sensitive"].as_bool().unwrap_or(false);
        let filter = RegexBuilder::new(trigger["url-filter"].as_str().expect("url-filter"))
            .case_insensitive(!case_sensitive)
            .build()
            .expect("url-filter WebKit can compile");
        let listed =
            |key: &str, wanted: &str| trigger[key].as_array().is_none_or(|kinds| kinds.iter().any(|k| k == wanted));
        let party = if load.is_third_party() { "third-party" } else { "first-party" };
        filter.is_match(load.url)
            && listed("resource-type", load.resource_type)
            && listed("load-type", party)
            && trigger["if-domain"].as_array().is_none_or(|domains| Self::covers(domains, load.page_host))
            && trigger["unless-domain"].as_array().is_none_or(|domains| !Self::covers(domains, load.page_host))
    }

    /// `*example.com` covers the domain and its subdomains, `example.com` the domain alone.
    fn covers(domains: &[Value], host: &str) -> bool {
        domains.iter().filter_map(Value::as_str).any(|domain| match domain.strip_prefix('*') {
            Some(parent) => host == parent || host.ends_with(&format!(".{parent}")),
            None => host == domain,
        })
    }
}

#[test]
fn should_block_third_party_ads_a_downloaded_list_names() {
    GuardScenario::given_downloaded(FilterListSource::EASYLIST, "[Adblock Plus 2.0]\n||ads.example^\n")
        .when_built(GuardRuleList::ADS)
        .then_blocks("https://ads.example/banner.js", "news.pl");
}

#[test]
fn should_block_tracking_scripts_a_site_serves_from_its_own_domain() {
    GuardScenario::given_downloaded(FilterListSource::EASYPRIVACY, "[Adblock Plus 1.1]\n/pixel/track.js\n")
        .when_built(GuardRuleList::PRIVACY)
        .then_blocks("https://news.pl/pixel/track.js", "news.pl");
}

#[test]
fn should_let_an_unbreak_exception_override_a_block() {
    GuardScenario::given_downloaded(FilterListSource::EASYLIST, "||cdn.example^\n")
        .and_downloaded(FilterListSource::UBLOCK_UNBREAK, "@@||cdn.example/player.js\n")
        .when_built(GuardRuleList::ADS)
        .then_allows("https://cdn.example/player.js", "news.pl");
}

#[test]
fn should_keep_blocking_what_an_unbreak_exception_does_not_cover() {
    GuardScenario::given_downloaded(FilterListSource::EASYLIST, "||cdn.example^\n")
        .and_downloaded(FilterListSource::UBLOCK_UNBREAK, "@@||cdn.example/player.js\n")
        .when_built(GuardRuleList::ADS)
        .then_blocks("https://cdn.example/ad.js", "news.pl");
}

#[test]
fn should_hide_page_elements_a_list_hides() {
    GuardScenario::given_downloaded(FilterListSource::POLISH_FILTERS, "news.pl##.ad-banner\n")
        .when_built(GuardRuleList::ADS)
        .then_hides(".ad-banner", "news.pl");
}

#[test]
fn should_hide_page_elements_of_a_list_that_also_blocks_requests() {
    GuardScenario::given_downloaded(FilterListSource::EASYLIST, "||ads.example^\nnews.pl##.ad-banner\n")
        .when_built(GuardRuleList::ADS)
        .then_hides(".ad-banner", "news.pl");
}

#[test]
fn should_never_block_the_page_the_user_asked_for() {
    GuardScenario::given_downloaded(FilterListSource::EASYLIST, "/ads/*\n")
        .when_built(GuardRuleList::ADS)
        .then_allows_page("https://news.pl/ads/how-to-buy-an-ad", "news.pl");
}

#[test]
fn should_hide_cookie_notices() {
    GuardScenario::given_downloaded(FilterListSource::COOKIE_NOTICES, "##.cookie-consent-bar\n")
        .when_built(GuardRuleList::ANNOYANCES)
        .then_hides(".cookie-consent-bar", "news.pl");
}

#[test]
fn should_leave_out_lists_that_belong_to_another_rule_list() {
    GuardScenario::given_downloaded(FilterListSource::EASYPRIVACY, "||tracker.example^\n")
        .when_built(GuardRuleList::ADS)
        .then_allows("https://tracker.example/t.js", "news.pl");
}

#[test]
fn should_stand_down_on_sites_the_user_switched_protection_off_for() {
    GuardScenario::given_downloaded(FilterListSource::EASYLIST, "||ads.example^\n")
        .with_protection_switched_off_by_user_on("bank.pl")
        .when_built(GuardRuleList::ADS)
        .then_allows("https://ads.example/banner.js", "bank.pl");
}

#[test]
fn should_keep_page_elements_visible_on_sites_the_user_switched_protection_off_for() {
    GuardScenario::given_downloaded(FilterListSource::COOKIE_NOTICES, "##.cookie-consent-bar\n")
        .with_protection_switched_off_by_user_on("news.pl")
        .when_built(GuardRuleList::ANNOYANCES)
        .then_shows(".cookie-consent-bar", "news.pl");
}

#[test]
fn should_block_link_pings_before_any_list_is_downloaded() {
    GuardScenario::given_nothing_downloaded()
        .when_built(GuardRuleList::PRIVACY)
        .then_blocks_link_ping("https://news.pl/collect?event=click", "news.pl");
}

#[test]
fn should_send_link_pings_on_sites_the_user_switched_protection_off_for() {
    GuardScenario::given_nothing_downloaded()
        .with_protection_switched_off_by_user_on("bank.pl")
        .when_built(GuardRuleList::PRIVACY)
        .then_allows_link_ping("https://bank.pl/clicked?link=offers", "bank.pl");
}

#[test]
fn should_fit_in_webkit_and_keep_every_exception_when_lists_outgrow_the_limit() {
    let built_from = |easylist: &str| {
        GuardScenario::given_downloaded(FilterListSource::EASYLIST, easylist)
            .and_downloaded(FilterListSource::UBLOCK_UNBREAK, "@@||ads0.example/player.js\n")
            .with_protection_switched_off_by_user_on("bank.pl")
            .when_built(GuardRuleList::ADS)
    };
    let flood: String =
        (0..=GuardRuleList::MAX_RULES).map(|number| format!("||ads{number}.example^")).collect::<Vec<_>>().join("\n");

    built_from(&flood).then_fits_in_webkit_with_exceptions(built_from("||ads0.example^\n").exception_count());
}

#[test]
fn should_rebuild_a_rule_list_when_one_of_its_filter_lists_changes() {
    let before = GuardScenario::given_downloaded(FilterListSource::EASYLIST, "||ads.example^\n");
    let after = GuardScenario::given_downloaded(FilterListSource::EASYLIST, "||ads.example^\n||more-ads.example^\n");

    assert_ne!(before.when_fingerprinted(GuardRuleList::ADS), after.when_fingerprinted(GuardRuleList::ADS));
}

#[test]
fn should_rebuild_a_rule_list_when_the_user_switches_protection_off_for_a_site() {
    let before = GuardScenario::given_downloaded(FilterListSource::EASYLIST, "||ads.example^\n");
    let after = GuardScenario::given_downloaded(FilterListSource::EASYLIST, "||ads.example^\n")
        .with_protection_switched_off_by_user_on("bank.pl");

    assert_ne!(before.when_fingerprinted(GuardRuleList::ADS), after.when_fingerprinted(GuardRuleList::ADS));
}

#[test]
fn should_keep_a_compiled_rule_list_when_only_filter_lists_it_does_not_read_change() {
    let before = GuardScenario::given_downloaded(FilterListSource::EASYPRIVACY, "||tracker.example^\n");
    let after = GuardScenario::given_downloaded(FilterListSource::EASYPRIVACY, "||other-tracker.example^\n");

    assert_eq!(before.when_fingerprinted(GuardRuleList::ADS), after.when_fingerprinted(GuardRuleList::ADS));
}

#[test]
fn should_have_nothing_to_compile_before_its_filter_lists_are_downloaded() {
    assert!(!GuardRuleList::ADS.has_rules(&HashMap::new()));
}

#[test]
fn should_have_the_link_ping_rule_to_compile_before_any_filter_list_is_downloaded() {
    assert!(GuardRuleList::PRIVACY.has_rules(&HashMap::new()));
}

#[test]
fn should_have_rules_to_compile_once_one_of_its_filter_lists_is_downloaded() {
    let downloaded = HashMap::from([(FilterListSource::COOKIE_NOTICES, "##.cookie-bar\n".to_owned())]);

    assert!(GuardRuleList::ANNOYANCES.has_rules(&downloaded));
}
