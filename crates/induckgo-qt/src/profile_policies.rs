//! The request and cookie policies of one profile, put together from the core's building blocks.

use std::sync::Arc;
use std::sync::RwLock;

use induckgo_core::navigation::https_upgrade_policy::HttpsUpgradePolicy;
use induckgo_core::network::chained_request_policy::ChainedRequestPolicy;
use induckgo_core::network::https_upgrade_request_policy::HttpsUpgradeRequestPolicy;
use induckgo_core::network::link_cleaning_request_policy::LinkCleaningRequestPolicy;
use induckgo_core::network::public_suffix_lookup::PublicSuffixLookup;
use induckgo_core::network::request_policy::RequestPolicy;
use induckgo_core::network::site_allow_list::SiteAllowList;
use induckgo_core::network::third_party_request_policy::ThirdPartyRequestPolicy;
use induckgo_core::network::tracker_request_policy::TrackerRequestPolicy;
use induckgo_core::network::unprotected_site_request_policy::UnprotectedSiteRequestPolicy;
use induckgo_core::profile::unprotected_sites::UnprotectedSites;
use induckgo_core::protection::privacy_configuration::PrivacyConfiguration;
use induckgo_core::protection::tracker_data_set::TrackerDataSet;
use induckgo_core::storage::cookie_policy::CookiePolicy;
use induckgo_core::storage::strict_cookie_policy::StrictCookiePolicy;
use induckgo_core::storage::tracker_cookie_policy::TrackerCookiePolicy;

use crate::protection_documents::ProtectionDocuments;

/// How strictly a profile treats third parties.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProfileStrictness {
    Standard,
    /// The *Strict* profile: third-party requests and cookies are refused unless the person allowed them.
    Strict,
}

pub struct ProfilePolicies {
    pub request: Arc<dyn RequestPolicy>,
    pub cookies: Arc<dyn CookiePolicy>,
}

impl ProfilePolicies {
    const HTTPS_FEATURE: &'static str = "https";

    /// Without downloaded protection data the policies still upgrade to HTTPS and, in the strict profile,
    /// refuse third parties; tracker blocking and link cleaning start when the data exists.
    #[must_use]
    pub fn assemble(
        documents: Option<&ProtectionDocuments>,
        unprotected_sites: Arc<RwLock<UnprotectedSites>>,
        allow_list: SiteAllowList,
        strictness: ProfileStrictness,
    ) -> Self {
        let sites = Arc::new(PublicSuffixLookup);
        let allow_list = Arc::new(RwLock::new(allow_list));
        let is_strict = strictness == ProfileStrictness::Strict;

        let mut policies: Vec<Box<dyn RequestPolicy>> =
            vec![Box::new(ThirdPartyRequestPolicy::new(is_strict, Arc::clone(&allow_list), sites.clone()))];
        let mut cookies: Arc<dyn CookiePolicy> = Arc::new(StrictCookiePolicy::new(Arc::clone(&allow_list)));
        let mut excluded_from_https = vec![];
        if let Some(documents) = documents {
            let configuration =
                PrivacyConfiguration::new(documents.configuration.clone(), PrivacyConfiguration::COMPATIBILITY_VERSION);
            let data_set = Arc::new(TrackerDataSet::from_json(&documents.tracker_data));
            excluded_from_https = configuration.exception_domains(Self::HTTPS_FEATURE);
            policies.push(Box::new(TrackerRequestPolicy::new(Arc::clone(&data_set), sites)));
            policies.push(Box::new(LinkCleaningRequestPolicy::new(configuration, Arc::clone(&unprotected_sites))));
            if !is_strict {
                cookies = Arc::new(TrackerCookiePolicy::new(data_set));
            }
        }
        let upgrades = Arc::new(RwLock::new(HttpsUpgradePolicy::new(&excluded_from_https)));
        policies.push(Box::new(HttpsUpgradeRequestPolicy::new(upgrades)));

        let chain: Arc<dyn RequestPolicy> = Arc::new(ChainedRequestPolicy::new(policies));
        Self { request: Arc::new(UnprotectedSiteRequestPolicy::new(chain, unprotected_sites)), cookies }
    }
}
