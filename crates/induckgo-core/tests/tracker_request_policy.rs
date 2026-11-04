//! Blocking third-party requests to known trackers, except on the tracker company's own sites.

use std::sync::Arc;

use induckgo_core::network::network_request::NetworkRequest;
use induckgo_core::network::request_decision::RequestDecision;
use induckgo_core::network::request_policy::RequestPolicy;
use induckgo_core::network::resource_type::ResourceType;
use induckgo_core::network::tracker_request_policy::TrackerRequestPolicy;
use induckgo_core::protection::tracker_data_set::TrackerDataSet;
use induckgo_core::web_app::registrable_domain_lookup::RegistrableDomainLookup;
use serde_json::json;

struct KnownSuffixes;

impl RegistrableDomainLookup for KnownSuffixes {
    fn is_public_suffix(&self, domain: &str) -> bool {
        matches!(domain, "com" | "net" | "org")
    }
}

/// Given: `tracker.com` (company Tracker Inc, also owning `tracker-sites.net`) is blocked by default and
/// `harmless.com` has the default action `ignore`.
struct TrackerBlockingScenario {
    policy: TrackerRequestPolicy,
}

impl TrackerBlockingScenario {
    fn given_published_tracker_data_set() -> Self {
        let data_set = TrackerDataSet::from_json(&json!({
            "trackers": {
                "tracker.com": {"domain": "tracker.com", "owner": {"name": "Tracker Inc"}, "default": "block", "rules": []},
                "harmless.com": {"domain": "harmless.com", "owner": {"name": "Harmless"}, "default": "ignore", "rules": []}
            },
            "entities": {"Tracker Inc": {"domains": ["tracker.com", "tracker-sites.net"]}},
            "cnames": {"metrics.example.org": "collector.tracker.com"}
        }));
        Self { policy: TrackerRequestPolicy::new(Arc::new(data_set), Arc::new(KnownSuffixes)) }
    }

    fn when_page_requests(&self, page: Option<&str>, address: &str) -> RequestDecision {
        self.policy.decide(&NetworkRequest::new(address, page, ResourceType::Image, 1))
    }

    fn when_navigating_to(&self, address: &str) -> RequestDecision {
        self.policy.decide(&NetworkRequest::new(address, None, ResourceType::MainFrame, 1))
    }
}

const PAGE: &str = "https://news.example.org/story";

#[test]
fn should_block_a_third_party_request_to_a_tracker() {
    let decision = TrackerBlockingScenario::given_published_tracker_data_set()
        .when_page_requests(Some(PAGE), "https://tracker.com/pixel.gif");
    assert_eq!(decision, RequestDecision::Block);
}

#[test]
fn should_block_a_request_to_a_subdomain_of_a_tracker() {
    let decision = TrackerBlockingScenario::given_published_tracker_data_set()
        .when_page_requests(Some(PAGE), "https://cdn.eu.tracker.com/pixel.gif");
    assert_eq!(decision, RequestDecision::Block);
}

#[test]
fn should_pass_a_request_to_a_tracker_whose_default_action_is_ignore() {
    let decision = TrackerBlockingScenario::given_published_tracker_data_set()
        .when_page_requests(Some(PAGE), "https://harmless.com/a.js");
    assert_eq!(decision, RequestDecision::Pass);
}

#[test]
fn should_pass_a_request_to_a_tracker_from_a_site_of_the_same_company() {
    let decision = TrackerBlockingScenario::given_published_tracker_data_set()
        .when_page_requests(Some("https://www.tracker-sites.net/"), "https://tracker.com/pixel.gif");
    assert_eq!(decision, RequestDecision::Pass);
}

#[test]
fn should_pass_a_first_party_request_even_when_the_site_is_a_tracker() {
    let decision = TrackerBlockingScenario::given_published_tracker_data_set()
        .when_page_requests(Some("https://www.tracker.com/"), "https://cdn.tracker.com/a.js");
    assert_eq!(decision, RequestDecision::Pass);
}

#[test]
fn should_pass_a_request_to_an_unknown_site() {
    let decision = TrackerBlockingScenario::given_published_tracker_data_set()
        .when_page_requests(Some(PAGE), "https://unknown.org/a.js");
    assert_eq!(decision, RequestDecision::Pass);
}

#[test]
fn should_pass_a_navigation_to_a_tracker_domain_typed_in_the_address_bar() {
    let decision =
        TrackerBlockingScenario::given_published_tracker_data_set().when_navigating_to("https://tracker.com/");
    assert_eq!(decision, RequestDecision::Pass);
}

#[test]
fn should_block_a_tracker_requested_by_a_page_without_a_host() {
    let decision = TrackerBlockingScenario::given_published_tracker_data_set()
        .when_page_requests(Some("data:text/html,x"), "https://tracker.com/pixel.gif");
    assert_eq!(decision, RequestDecision::Block);
}

#[test]
fn should_block_a_tracker_requested_by_an_unknown_page() {
    let decision = TrackerBlockingScenario::given_published_tracker_data_set()
        .when_page_requests(None, "https://tracker.com/pixel.gif");
    assert_eq!(decision, RequestDecision::Block);
}

#[test]
fn should_block_a_cname_cloaked_tracker_on_the_same_site() {
    let decision = TrackerBlockingScenario::given_published_tracker_data_set()
        .when_page_requests(Some(PAGE), "https://metrics.example.org/collect");
    assert_eq!(decision, RequestDecision::Block);
}

#[test]
fn should_block_a_subdomain_of_a_cloaked_name() {
    let decision = TrackerBlockingScenario::given_published_tracker_data_set()
        .when_page_requests(Some(PAGE), "https://eu.metrics.example.org/collect");
    assert_eq!(decision, RequestDecision::Block);
}

#[test]
fn should_pass_a_cloaked_tracker_on_a_site_of_the_trackers_own_company() {
    let decision = TrackerBlockingScenario::given_published_tracker_data_set()
        .when_page_requests(Some("https://www.tracker-sites.net/"), "https://metrics.example.org/collect");
    assert_eq!(decision, RequestDecision::Pass);
}
