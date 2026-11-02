//! One isolated browsing profile, live in this process.
//!
//! Each InDuckGo process serves exactly one profile (or one web app's private profile), so a session owns:
//! a WebKit `NetworkSession` with its own cookie jar, storage, cache and HSTS list; the page protections;
//! and the state that feeds them (privacy configuration, Tracker Data Set, sites the user unprotected).
//!
//! Pages must not load before tracker blocking is active: [`ProfileSession::when_protected`] holds a load
//! back until the first rule lists are compiled, which takes a moment only on the very first start.

use std::cell::Cell;
use std::cell::RefCell;
use std::rc::Rc;
use std::rc::Weak;

use induckgo_core::navigation::https_upgrade_policy::HttpsUpgradePolicy;
use induckgo_core::profile::profile_directories::ProfileLocation;
use induckgo_core::profile::profile_id::ProfileId;
use induckgo_core::profile::unprotected_sites::UnprotectedSites;
use induckgo_core::protection::content_blocker_rules_builder::ContentBlockerRulesBuilder;
use induckgo_core::protection::content_blocker_rules_builder::RuleExceptions;
use induckgo_core::protection::privacy_configuration::PrivacyConfiguration;
use induckgo_core::protection::tracker_data_set::TrackerDataSet;
use induckgo_core::protection::tracking_parameter_remover::TrackingParameterRemover;
use webkit6::CookieAcceptPolicy;
use webkit6::CookiePersistentStorage;
use webkit6::NetworkSession;
use webkit6::Settings;
use webkit6::WebView;
use webkit6::WebsiteDataTypes;

use crate::download_tracker::DownloadTracker;
use crate::http_client::HttpClient;
use crate::installation_paths::InstallationPaths;
use crate::page_protections::PageProtections;
use crate::page_protections::RuleList;
use crate::protection_feed::ProtectionDocuments;
use crate::protection_feed::ProtectionFeed;
use crate::stored_file::StoredFile;

enum Readiness {
    Waiting(Vec<Box<dyn FnOnce()>>),
    Protected,
}

/// What the rule lists and the content scope script are built from.
struct ProtectionInputs {
    tracker_data: Option<TrackerDataSet>,
    configuration: Option<PrivacyConfiguration>,
    unprotected_sites: UnprotectedSites,
    https_upgrades: HttpsUpgradePolicy,
}

pub struct ProfileSession {
    profile: ProfileId,
    network_session: NetworkSession,
    settings: Settings,
    protections: PageProtections,
    feed: ProtectionFeed,
    http: Rc<HttpClient>,
    unprotected_sites_file: StoredFile,
    inputs: RefCell<ProtectionInputs>,
    readiness: RefCell<Readiness>,
    /// Bumped by every rebuild; a slower, older rebuild never overwrites a newer one.
    generation: Cell<u64>,
    _downloads: DownloadTracker,
}

impl ProfileSession {
    const LOG_DOMAIN: &'static str = "induckgo";
    const HTTPS_FEATURE: &'static str = "https";
    const REFRESH_SECONDS: u32 = 12 * 60 * 60;
    const RETRY_SECONDS: u32 = 30;

    pub fn start(profile: ProfileId, paths: &InstallationPaths, http: Rc<HttpClient>) -> Rc<Self> {
        let location = paths.profile(&profile);
        let network_session = Self::isolated_network_session(&location);
        let unprotected_sites_file = StoredFile::new(location.data.join("unprotected-sites.json"));
        let unprotected_sites = unprotected_sites_file
            .read_json()
            .map(|document| UnprotectedSites::from_json(&document))
            .unwrap_or_default();
        let session = Rc::new(Self {
            profile,
            _downloads: DownloadTracker::watch(&network_session, paths.downloads()),
            network_session,
            settings: Self::private_settings(),
            protections: PageProtections::new(&location.cache.join("content-filters")),
            feed: ProtectionFeed::new(&paths.protection_feed()),
            http,
            unprotected_sites_file,
            inputs: RefCell::new(ProtectionInputs {
                tracker_data: None,
                configuration: None,
                unprotected_sites,
                https_upgrades: HttpsUpgradePolicy::default(),
            }),
            readiness: RefCell::new(Readiness::Waiting(Vec::new())),
            generation: Cell::new(0),
        });
        if let Some(documents) = session.feed.cached() {
            session.adopt(documents);
        }
        Self::keep_refreshing(Rc::downgrade(&session));
        session
    }

    pub fn profile_id(&self) -> &ProfileId {
        &self.profile
    }

    /// A new top-level web view in this profile.
    pub fn new_web_view(&self) -> WebView {
        WebView::builder()
            .network_session(&self.network_session)
            .user_content_manager(self.protections.content_manager())
            .settings(&self.settings)
            .build()
    }

    /// Runs `action` once tracker blocking is active (immediately when it already is).
    pub fn when_protected(&self, action: impl FnOnce() + 'static) {
        match &mut *self.readiness.borrow_mut() {
            Readiness::Waiting(pending) => pending.push(Box::new(action)),
            Readiness::Protected => action(),
        }
    }

    /// The address without tracking parameters, when it differs from `address`.
    pub fn cleaned_address(&self, address: &str) -> Option<String> {
        let inputs = self.inputs.borrow();
        let configuration = inputs.configuration.as_ref()?;
        let user_hosts = inputs.unprotected_sites.hosts_to_match();
        let cleaned = TrackingParameterRemover::new(configuration, &user_hosts).cleaned(address, None);
        (cleaned != address).then_some(cleaned)
    }

    /// Whether the HTTPS rules turned `plain_http` into `https`.
    pub fn was_upgraded(&self, plain_http: &str, https: &str) -> bool {
        self.inputs.borrow().https_upgrades.upgraded(plain_http).as_deref() == Some(https)
    }

    /// Stops upgrading the host of `https`, then runs `then` once the new rules are active.
    pub fn stop_upgrading(self: &Rc<Self>, https: &str, then: impl FnOnce() + 'static) {
        self.inputs.borrow_mut().https_upgrades.record_failed_upgrade(https);
        self.rebuild(then);
    }

    pub fn is_site_protected(&self, host: &str) -> bool {
        !self.inputs.borrow().unprotected_sites.is_unprotected(host)
    }

    /// Switches protections for `host` and saves the choice; `then` runs once the new rules are active.
    pub fn set_site_protected(self: &Rc<Self>, host: &str, protected: bool, then: impl FnOnce() + 'static) {
        {
            let mut inputs = self.inputs.borrow_mut();
            if protected {
                inputs.unprotected_sites.switch_protection_on(host);
            } else {
                inputs.unprotected_sites.switch_protection_off(host);
            }
            if let Err(error) = self.unprotected_sites_file.write_json(&inputs.unprotected_sites.to_json()) {
                glib::g_warning!(Self::LOG_DOMAIN, "cannot save unprotected sites: {error}");
            }
        }
        self.rebuild(then);
    }

    /// The Fire button: deletes every cookie, storage, cache and favicon of this profile. The HSTS list is
    /// kept, because forgetting it would let a network attacker downgrade sites to plain HTTP again.
    pub async fn burn_website_data(&self) -> Result<(), glib::Error> {
        let Some(data_manager) = self.network_session.website_data_manager() else { return Ok(()) };
        if let Some(favicons) = data_manager.favicon_database() {
            favicons.clear();
        }
        let everything_but_hsts = WebsiteDataTypes::all() - WebsiteDataTypes::HSTS_CACHE;
        let (finished, cleared) = futures_channel::oneshot::channel();
        data_manager.clear(everything_but_hsts, glib::TimeSpan::from_seconds(0), None::<&gio::Cancellable>, |result| {
            let _ = finished.send(result);
        });
        cleared.await.unwrap_or(Ok(()))
    }

    /// Third-party cookies blocked and Intelligent Tracking Prevention on, as in Safari; cookies persist in
    /// the profile's own directory.
    fn isolated_network_session(location: &ProfileLocation) -> NetworkSession {
        let session =
            NetworkSession::new(Some(&location.data.to_string_lossy()), Some(&location.cache.to_string_lossy()));
        session.set_itp_enabled(true);
        if let Some(cookies) = session.cookie_manager() {
            cookies.set_accept_policy(CookieAcceptPolicy::NoThirdParty);
            let cookie_file = location.data.join("cookies.sqlite");
            cookies.set_persistent_storage(&cookie_file.to_string_lossy(), CookiePersistentStorage::Sqlite);
        }
        if let Some(data_manager) = session.website_data_manager() {
            data_manager.set_favicons_enabled(true);
        }
        session
    }

    fn private_settings() -> Settings {
        let settings = Settings::new();
        settings.set_enable_developer_extras(true);
        settings.set_javascript_can_open_windows_automatically(false);
        settings
    }

    fn keep_refreshing(session: Weak<Self>) {
        glib::spawn_future_local(async move {
            loop {
                let Some(live) = session.upgrade() else { return };
                match live.feed.refreshed(&live.http).await {
                    Ok(Some(documents)) => live.adopt(documents),
                    Ok(None) => {}
                    Err(error) => glib::g_warning!(Self::LOG_DOMAIN, "protection data refresh failed: {error}"),
                }
                let is_protected = matches!(*live.readiness.borrow(), Readiness::Protected);
                drop(live);
                glib::timeout_future_seconds(if is_protected { Self::REFRESH_SECONDS } else { Self::RETRY_SECONDS })
                    .await;
            }
        });
    }

    fn adopt(self: &Rc<Self>, documents: ProtectionDocuments) {
        {
            let mut inputs = self.inputs.borrow_mut();
            let configuration =
                PrivacyConfiguration::new(documents.configuration, PageProtections::COMPATIBILITY_VERSION);
            inputs.https_upgrades.set_excluded_domains(&configuration.exception_domains(Self::HTTPS_FEATURE));
            inputs.tracker_data = Some(TrackerDataSet::from_json(&documents.tracker_data));
            inputs.configuration = Some(configuration);
        }
        self.rebuild(|| {});
    }

    /// Rebuilds rule lists and script from the current inputs, then activates them and runs `then`.
    fn rebuild(self: &Rc<Self>, then: impl FnOnce() + 'static) {
        self.install_content_scope_script();
        let Some(lists) = self.rule_lists() else {
            then();
            return;
        };
        let generation = self.generation.get() + 1;
        self.generation.set(generation);
        let session = Rc::downgrade(self);
        glib::spawn_future_local(async move {
            let Some(live) = session.upgrade() else { return };
            let filters = live.protections.compile(lists).await;
            if live.generation.get() != generation {
                return;
            }
            live.protections.activate(&filters);
            live.mark_protected();
            then();
        });
    }

    fn install_content_scope_script(&self) {
        let inputs = self.inputs.borrow();
        if let Some(configuration) = &inputs.configuration {
            self.protections.install_script(configuration, &inputs.unprotected_sites.hosts_to_match());
        }
    }

    fn rule_lists(&self) -> Option<Vec<RuleList>> {
        let inputs = self.inputs.borrow();
        let (Some(tracker_data), Some(configuration)) = (&inputs.tracker_data, &inputs.configuration) else {
            return None;
        };
        let user_hosts = inputs.unprotected_sites.hosts_to_match();
        let trackers = if configuration.is_enabled(RuleExceptions::CONTENT_BLOCKING) {
            let exceptions = RuleExceptions::from_configuration(configuration, user_hosts.clone());
            ContentBlockerRulesBuilder::new(tracker_data).build_rules(&exceptions)
        } else {
            Vec::new()
        };
        let https = if configuration.is_enabled(Self::HTTPS_FEATURE) {
            inputs.https_upgrades.content_rules()
        } else {
            Vec::new()
        };
        let links = TrackingParameterRemover::new(configuration, &user_hosts).content_rules();
        Some(vec![
            RuleList { name: "trackers", rules: trackers },
            RuleList { name: "https", rules: https },
            RuleList { name: "links", rules: links },
        ])
    }

    fn mark_protected(&self) {
        let previous = self.readiness.replace(Readiness::Protected);
        if let Readiness::Waiting(pending) = previous {
            for action in pending {
                action();
            }
        }
    }
}
