//! How Chromium's questions are answered from the core's policies, without starting Chromium.

use std::sync::Arc;

use induckgo_core::network::network_request::NetworkRequest;
use induckgo_core::network::request_decision::RequestDecision;
use induckgo_core::network::request_policy::RequestPolicy;
use induckgo_core::storage::cookie_access::CookieAccess;
use induckgo_core::storage::cookie_policy::CookiePolicy;
use induckgo_engine_qt::engine_bridge::ffi::HookDecision;
use induckgo_engine_qt::engine_policies::EnginePolicies;
use induckgo_engine_qt::engine_policies::PolicyDecisions;

/// Blocks scripts, redirects main-frame navigations to `https://safe.test/`, passes the rest.
struct ScriptBlockingPolicy;

impl RequestPolicy for ScriptBlockingPolicy {
    fn decide(&self, request: &NetworkRequest) -> RequestDecision {
        match request.resource_type() {
            induckgo_core::network::resource_type::ResourceType::Script => RequestDecision::Block,
            induckgo_core::network::resource_type::ResourceType::MainFrame => {
                RequestDecision::Redirect("https://safe.test/".to_owned())
            }
            _ => RequestDecision::Pass,
        }
    }
}

/// Refuses third-party cookies.
struct FirstPartyCookiesOnly;

impl CookiePolicy for FirstPartyCookiesOnly {
    fn allows(&self, access: &CookieAccess) -> bool {
        !access.is_third_party()
    }
}

struct DecisionScenario {
    policies: EnginePolicies,
}

impl DecisionScenario {
    fn given_installed_policies() -> Self {
        Self { policies: EnginePolicies::new(Arc::new(ScriptBlockingPolicy), Arc::new(FirstPartyCookiesOnly)) }
    }

    fn when_chromium_asks_about_request(&self, resource_type: i32) -> HookDecision {
        PolicyDecisions::for_request(
            Some(&self.policies),
            "https://cdn.example.org/a",
            "https://news.example.org/",
            resource_type,
            1,
        )
    }

    fn when_chromium_asks_about_cookie(&self, is_third_party: bool) -> bool {
        PolicyDecisions::for_cookie(Some(&self.policies), "ads.example.com", "news.example.org", is_third_party)
    }
}

#[test]
fn should_tell_chromium_to_block_a_script() {
    let decision = DecisionScenario::given_installed_policies().when_chromium_asks_about_request(3);
    assert_eq!(decision.action, 1);
}

#[test]
fn should_tell_chromium_to_redirect_a_navigation_to_the_policys_address() {
    let decision = DecisionScenario::given_installed_policies().when_chromium_asks_about_request(0);
    assert_eq!((decision.action, decision.redirect_to.as_str()), (2, "https://safe.test/"));
}

#[test]
fn should_let_an_image_pass() {
    let decision = DecisionScenario::given_installed_policies().when_chromium_asks_about_request(4);
    assert_eq!(decision.action, 0);
}

#[test]
fn should_treat_an_unknown_resource_type_as_other_and_not_as_a_script_or_navigation() {
    let decision = DecisionScenario::given_installed_policies().when_chromium_asks_about_request(255);
    assert_eq!(decision.action, 0);
}

#[test]
fn should_refuse_a_third_party_cookie_and_accept_a_first_party_one() {
    let scenario = DecisionScenario::given_installed_policies();
    assert!(!scenario.when_chromium_asks_about_cookie(true));
    assert!(scenario.when_chromium_asks_about_cookie(false));
}

#[test]
fn should_block_requests_made_before_policies_are_installed() {
    let decision = PolicyDecisions::for_request(None, "https://cdn.example.org/a", "", 4, 1);
    assert_eq!(decision.action, 1);
}

#[test]
fn should_refuse_cookies_before_policies_are_installed() {
    assert!(!PolicyDecisions::for_cookie(None, "example.org", "example.org", false));
}
