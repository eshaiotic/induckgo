//! The policies Chromium consults, and how their answers become the bridge's plain values.

use std::sync::Arc;
use std::sync::OnceLock;

use induckgo_core::network::network_request::NetworkRequest;
use induckgo_core::network::request_decision::RequestDecision;
use induckgo_core::network::request_policy::RequestPolicy;
use induckgo_core::storage::cookie_access::CookieAccess;
use induckgo_core::storage::cookie_policy::CookiePolicy;

use crate::engine_bridge::ffi::HookDecision;
use crate::resource_type_mapping::ResourceTypeMapping;

/// Chromium has one request pipeline per process, so there is one set of policies per process.
static INSTALLED: OnceLock<EnginePolicies> = OnceLock::new();

pub struct EnginePolicies {
    request: Arc<dyn RequestPolicy>,
    cookies: Arc<dyn CookiePolicy>,
}

impl EnginePolicies {
    #[must_use]
    pub fn new(request: Arc<dyn RequestPolicy>, cookies: Arc<dyn CookiePolicy>) -> Self {
        Self { request, cookies }
    }

    /// Makes these the process's policies. Only the first call has an effect; returns whether it was the first.
    pub fn install(self) -> bool {
        INSTALLED.set(self).is_ok()
    }

    /// Until policies are installed nothing passes: a request must never skip protection because it was
    /// made before start-up finished.
    pub(crate) fn installed() -> Option<&'static Self> {
        INSTALLED.get()
    }
}

/// The two questions Chromium asks, answered from a set of policies.
pub struct PolicyDecisions;

impl PolicyDecisions {
    const ACTION_PASS: u8 = 0;
    const ACTION_BLOCK: u8 = 1;
    const ACTION_REDIRECT: u8 = 2;

    #[must_use]
    pub fn for_request(
        policies: Option<&EnginePolicies>,
        url: &str,
        first_party: &str,
        resource_type: i32,
        request_id: u64,
    ) -> HookDecision {
        let Some(policies) = policies else { return Self::decision(Self::ACTION_BLOCK, String::new()) };
        let page = (!first_party.is_empty()).then_some(first_party);
        let request = NetworkRequest::new(url, page, ResourceTypeMapping::from_engine_code(resource_type), request_id);
        match policies.request.decide(&request) {
            RequestDecision::Pass => Self::decision(Self::ACTION_PASS, String::new()),
            RequestDecision::Block => Self::decision(Self::ACTION_BLOCK, String::new()),
            RequestDecision::Redirect(address) => Self::decision(Self::ACTION_REDIRECT, address),
        }
    }

    #[must_use]
    pub fn for_cookie(
        policies: Option<&EnginePolicies>,
        domain: &str,
        first_party: &str,
        is_third_party: bool,
    ) -> bool {
        let Some(policies) = policies else { return false };
        let page = (!first_party.is_empty()).then_some(first_party);
        policies.cookies.allows(&CookieAccess::new(domain, page, is_third_party))
    }

    fn decision(action: u8, redirect_to: String) -> HookDecision {
        HookDecision { action, redirect_to }
    }
}
