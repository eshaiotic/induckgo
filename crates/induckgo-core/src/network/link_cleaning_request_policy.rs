//! Removes tracking parameters (`fbclid`, `utm_source`, ...) from the address a page is navigating to.

use crate::network::network_request::NetworkRequest;
use crate::network::request_decision::RequestDecision;
use crate::network::request_policy::RequestPolicy;
use crate::protection::privacy_configuration::PrivacyConfiguration;
use crate::protection::tracking_parameter_remover::TrackingParameterRemover;

pub struct LinkCleaningRequestPolicy {
    configuration: PrivacyConfiguration,
    user_unprotected_domains: Vec<String>,
}

impl LinkCleaningRequestPolicy {
    #[must_use]
    pub fn new(configuration: PrivacyConfiguration, user_unprotected_domains: Vec<String>) -> Self {
        Self { configuration, user_unprotected_domains }
    }
}

impl RequestPolicy for LinkCleaningRequestPolicy {
    fn decide(&self, request: &NetworkRequest) -> RequestDecision {
        if !request.is_navigation() || !request.goes_over_the_network() {
            return RequestDecision::Pass;
        }
        let remover = TrackingParameterRemover::new(&self.configuration, &self.user_unprotected_domains);
        let cleaned = remover.cleaned(request.address(), None);
        if cleaned == request.address() { RequestDecision::Pass } else { RequestDecision::Redirect(cleaned) }
    }
}
