//! Strict mode: block requests to other sites unless the person allowed third parties on the page's site.

use std::sync::Arc;
use std::sync::RwLock;

use induckgo_core::network::network_request::NetworkRequest;
use induckgo_core::network::request_decision::RequestDecision;
use induckgo_core::network::request_policy::RequestPolicy;
use induckgo_core::network::resource_type::ResourceType;
use induckgo_core::network::site_allow_list::SiteAllowList;
use induckgo_core::network::third_party_request_policy::ThirdPartyRequestPolicy;
use induckgo_core::web_app::registrable_domain_lookup::RegistrableDomainLookup;

struct KnownSuffixes;

impl RegistrableDomainLookup for KnownSuffixes {
    fn is_public_suffix(&self, domain: &str) -> bool {
        matches!(domain, "com" | "org" | "uk" | "co.uk")
    }
}

struct StrictBrowsingScenario {
    allow_list: Arc<RwLock<SiteAllowList>>,
    policy: ThirdPartyRequestPolicy,
}

impl StrictBrowsingScenario {
    fn given_strict_profile() -> Self {
        Self::given_profile(true)
    }

    fn given_default_profile() -> Self {
        Self::given_profile(false)
    }

    fn given_profile(is_strict: bool) -> Self {
        let allow_list = Arc::new(RwLock::new(SiteAllowList::default()));
        let policy = ThirdPartyRequestPolicy::new(is_strict, Arc::clone(&allow_list), Arc::new(KnownSuffixes));
        Self { allow_list, policy }
    }

    fn with_third_parties_allowed_on(self, site: &str) -> Self {
        self.allow_list.write().expect("lock").allow_third_parties_on(site);
        self
    }

    fn when_page_requests(&self, page: Option<&str>, address: &str) -> RequestDecision {
        self.policy.decide(&NetworkRequest::new(address, page, ResourceType::Script, 1))
    }
}

const PAGE: &str = "https://news.example.org/story";

#[test]
fn should_block_a_third_party_script_in_the_strict_profile() {
    let decision =
        StrictBrowsingScenario::given_strict_profile().when_page_requests(Some(PAGE), "https://cdn.other.com/a.js");
    assert_eq!(decision, RequestDecision::Block);
}

#[test]
fn should_pass_a_request_to_another_subdomain_of_the_same_site() {
    let decision = StrictBrowsingScenario::given_strict_profile()
        .when_page_requests(Some(PAGE), "https://static.example.org/a.js");
    assert_eq!(decision, RequestDecision::Pass);
}

#[test]
fn should_pass_a_third_party_request_on_a_site_the_person_allowed() {
    let scenario = StrictBrowsingScenario::given_strict_profile().with_third_parties_allowed_on("example.org");
    assert_eq!(scenario.when_page_requests(Some(PAGE), "https://cdn.other.com/a.js"), RequestDecision::Pass);
}

#[test]
fn should_pass_third_party_requests_in_the_default_profile() {
    let decision =
        StrictBrowsingScenario::given_default_profile().when_page_requests(Some(PAGE), "https://cdn.other.com/a.js");
    assert_eq!(decision, RequestDecision::Pass);
}

#[test]
fn should_treat_a_request_without_a_page_as_first_party() {
    let decision = StrictBrowsingScenario::given_strict_profile().when_page_requests(None, "https://cdn.other.com/");
    assert_eq!(decision, RequestDecision::Pass);
}

#[test]
fn should_pass_requests_that_do_not_use_the_network() {
    let decision = StrictBrowsingScenario::given_strict_profile().when_page_requests(Some(PAGE), "data:text/plain,hi");
    assert_eq!(decision, RequestDecision::Pass);
}
