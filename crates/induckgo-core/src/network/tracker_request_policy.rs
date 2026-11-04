//! Blocks third-party requests to known trackers, like DuckDuckGo's tracker blocking.
//!
//! Only whole-domain rules are applied here (a tracker whose default action is `block`); the URL-pattern
//! rules of the Tracker Data Set are applied by the filter-list policy that is chained after this one.

use std::sync::Arc;

use crate::network::network_request::NetworkRequest;
use crate::network::request_decision::RequestDecision;
use crate::network::request_policy::RequestPolicy;
use crate::protection::domain_name::DomainName;
use crate::protection::tracker_data_set::KnownTracker;
use crate::protection::tracker_data_set::TrackerAction;
use crate::protection::tracker_data_set::TrackerDataSet;
use crate::web_app::registrable_domain_lookup::RegistrableDomainLookup;

pub struct TrackerRequestPolicy {
    data_set: Arc<TrackerDataSet>,
    sites: Arc<dyn RegistrableDomainLookup + Send + Sync>,
}

impl TrackerRequestPolicy {
    #[must_use]
    pub fn new(data_set: Arc<TrackerDataSet>, sites: Arc<dyn RegistrableDomainLookup + Send + Sync>) -> Self {
        Self { data_set, sites }
    }

    fn tracker_serving(&self, host: &str) -> Option<&KnownTracker> {
        DomainName::new(host).self_and_parent_domains().find_map(|candidate| self.data_set.tracker(candidate))
    }

    /// A tracker's own company is not a third party on its own sites (Google on google.com).
    fn is_owned_by_page_company(&self, tracker: &KnownTracker, page_host: &str) -> bool {
        let page_site = self.sites.site_of(page_host);
        self.data_set
            .related_domains(tracker.owner_name.as_deref())
            .is_some_and(|domains| domains.iter().any(|domain| self.sites.site_of(domain) == page_site))
    }
}

impl RequestPolicy for TrackerRequestPolicy {
    fn decide(&self, request: &NetworkRequest) -> RequestDecision {
        let Some(host) = request.host().filter(|_| request.goes_over_the_network() && !request.is_navigation()) else {
            return RequestDecision::Pass;
        };
        let tracker = if request.is_third_party(self.sites.as_ref()) {
            self.tracker_serving(host)
        } else {
            // A first-party name that points at a tracker is cloaking (the tracker pretends to be the site).
            self.data_set.tracker_cloaked_as(host)
        };
        match tracker {
            Some(tracker)
                if tracker.default_action == Some(TrackerAction::Block)
                    && !request.first_party_host().is_some_and(|page| self.is_owned_by_page_company(tracker, page)) =>
            {
                RequestDecision::Block
            }
            _ => RequestDecision::Pass,
        }
    }
}
