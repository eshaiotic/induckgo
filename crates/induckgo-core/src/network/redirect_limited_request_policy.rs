//! Lets a request be redirected only once: a second redirect for the same request becomes a pass.

use std::sync::Arc;

use crate::network::network_request::NetworkRequest;
use crate::network::redirect_budget::RedirectBudget;
use crate::network::request_decision::RequestDecision;
use crate::network::request_policy::RequestPolicy;

pub struct RedirectLimitedRequestPolicy {
    inner: Arc<dyn RequestPolicy>,
    budget: RedirectBudget,
}

impl RedirectLimitedRequestPolicy {
    #[must_use]
    pub fn new(inner: Arc<dyn RequestPolicy>, budget: RedirectBudget) -> Self {
        Self { inner, budget }
    }
}

impl RequestPolicy for RedirectLimitedRequestPolicy {
    fn decide(&self, request: &NetworkRequest) -> RequestDecision {
        match self.inner.decide(request) {
            RequestDecision::Redirect(_) if !self.budget.allow(request.request_id()) => RequestDecision::Pass,
            decision => decision,
        }
    }
}
