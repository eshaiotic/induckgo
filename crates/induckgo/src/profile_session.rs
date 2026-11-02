//! One isolated browsing profile, live in this process.
//!
//! Each InDuckGo process serves exactly one profile (or one web app's private profile), so a session owns:
//! a WebKit `NetworkSession` with its own cookie jar, storage, cache and HSTS list; the page protections;
//! and the state that feeds them (privacy configuration, Tracker Data Set, sites the user unprotected); and the
//! zoom the user gave each site and the search engine the user chose.
//!
//! Pages must not load before tracker blocking is active: [`ProfileSession::when_protected`] holds a load
//! back until rule lists are active. Building the tracker list from the Tracker Data Set costs a few tenths of
//! a second of processor time, so it runs on a worker thread, and a start does not wait for it: the list
//! compiled last time is activated at once (as DuckDuckGo's macOS app does), then replaced if the new one
//! differs. Only the very first start, with nothing compiled yet, waits for the full compilation.

use std::cell::Cell;
use std::cell::RefCell;
use std::rc::Rc;
use std::rc::Weak;
use std::sync::Arc;
use std::time::Instant;

use induckgo_core::navigation::https_upgrade_policy::HttpsUpgradePolicy;
use induckgo_core::navigation::search_engine::SearchEngine;
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
use crate::page_protections::PreparedRuleList;
use crate::page_protections::RuleList;
use crate::protection_feed::ProtectionDocuments;
use crate::protection_feed::ProtectionFeed;
use crate::site_zoom::SiteZoom;
use crate::stored_file::StoredFile;

enum Readiness {
    Waiting(Vec<Box<dyn FnOnce()>>),
    Protected,
}

/// What the rule lists and the content scope script are built from.
struct ProtectionInputs {
    tracker_data: Option<Arc<TrackerDataSet>>,
    configuration: Option<PrivacyConfiguration>,
    unprotected_sites: UnprotectedSites,
    https_upgrades: HttpsUpgradePolicy,
}

/// What the tracker list is built from, owned so that a worker thread can build it.
struct TrackerListInputs {
    tracker_data: Arc<TrackerDataSet>,
    exceptions: RuleExceptions,
    is_enabled: bool,
}

impl TrackerListInputs {
    const NAME: &'static str = "trackers";

    fn prepared(&self) -> PreparedRuleList {
        let rules = if self.is_enabled {
            ContentBlockerRulesBuilder::new(&self.tracker_data).build_rules(&self.exceptions)
        } else {
            Vec::new()
        };
        RuleList { name: Self::NAME, rules }.prepared()
    }
}

pub struct ProfileSession {
    profile: ProfileId,
    network_session: NetworkSession,
    settings: Settings,
    protections: PageProtections,
    feed: ProtectionFeed,
    http: Rc<HttpClient>,
    unprotected_sites_file: StoredFile,
    zoom: SiteZoom,
    search_engine_file: StoredFile,
    search_engine: Cell<SearchEngine>,
    inputs: RefCell<ProtectionInputs>,
    readiness: RefCell<Readiness>,
    /// Bumped by every rebuild; a slower, older rebuild never overwrites a newer one.
    generation: Cell<u64>,
    started: Instant,
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
        let search_engine_file = StoredFile::new(location.data.join("search-engine.json"));
        let search_engine = search_engine_file.read_json().map(|document| SearchEngine::from_json(&document));
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
            zoom: SiteZoom::load(StoredFile::new(location.data.join("site-zoom-levels.json"))),
            search_engine_file,
            search_engine: Cell::new(search_engine.unwrap_or_default()),
            inputs: RefCell::new(ProtectionInputs {
                tracker_data: None,
                configuration: None,
                unprotected_sites,
                https_upgrades: HttpsUpgradePolicy::default(),
            }),
            readiness: RefCell::new(Readiness::Waiting(Vec::new())),
            generation: Cell::new(0),
            started: Instant::now(),
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

    pub fn zoom(&self) -> &SiteZoom {
        &self.zoom
    }

    /// What the address bar searches with and where new tabs open.
    pub fn search_engine(&self) -> SearchEngine {
        self.search_engine.get()
    }

    /// Makes `engine` the profile's search engine, and saves the choice.
    pub fn choose_search_engine(&self, engine: SearchEngine) {
        self.search_engine.set(engine);
        if let Err(error) = self.search_engine_file.write_json(&engine.to_json()) {
            glib::g_warning!(Self::LOG_DOMAIN, "cannot save the search engine: {error}");
        }
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

    /// The Fire button: deletes every cookie, storage, cache, favicon and site zoom of this profile. The HSTS
    /// list is kept, because forgetting it would let a network attacker downgrade sites to plain HTTP again.
    pub async fn burn_website_data(&self) -> Result<(), glib::Error> {
        self.zoom.forget_all();
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
            inputs.tracker_data = Some(Arc::new(TrackerDataSet::from_json(&documents.tracker_data)));
            inputs.configuration = Some(configuration);
        }
        self.rebuild(|| {});
    }

    /// Rebuilds rule lists and script from the current inputs, then activates them and runs `then`.
    fn rebuild(self: &Rc<Self>, then: impl FnOnce() + 'static) {
        self.install_content_scope_script();
        let Some((trackers, other_lists)) = self.rule_inputs() else {
            then();
            return;
        };
        let generation = self.generation.get() + 1;
        self.generation.set(generation);
        let session = Rc::downgrade(self);
        glib::spawn_future_local(async move {
            let Some(live) = session.upgrade() else { return };
            if trackers.is_enabled && live.is_waiting() {
                live.protect_with_last_tracker_list(&other_lists).await;
            }
            let Ok(trackers) = gio::spawn_blocking(move || trackers.prepared()).await else {
                glib::g_critical!(Self::LOG_DOMAIN, "building the tracker rule list failed");
                return;
            };
            let lists: Vec<PreparedRuleList> = std::iter::once(trackers).chain(other_lists).collect();
            let filters = live.protections.compile(&lists).await;
            if live.generation.get() != generation {
                return;
            }
            live.protections.activate(&filters);
            live.mark_protected();
            then();
        });
    }

    /// Lets pages load under the tracker list compiled last time, with fresh HTTPS and link lists (cheap to
    /// build, and the HTTPS one must not carry over exemptions the user allowed in an earlier session).
    async fn protect_with_last_tracker_list(&self, other_lists: &[PreparedRuleList]) {
        let Some(last_trackers) = self.protections.last_compiled(TrackerListInputs::NAME).await else { return };
        let mut filters = vec![last_trackers];
        filters.extend(self.protections.compile(other_lists).await);
        if self.is_waiting() {
            self.protections.activate(&filters);
            self.mark_protected();
        }
    }

    fn is_waiting(&self) -> bool {
        matches!(*self.readiness.borrow(), Readiness::Waiting(_))
    }

    fn install_content_scope_script(&self) {
        let inputs = self.inputs.borrow();
        if let Some(configuration) = &inputs.configuration {
            self.protections.install_script(configuration, &inputs.unprotected_sites.hosts_to_match());
        }
    }

    /// The tracker list's inputs (built later, on a worker thread) and the other lists, ready for WebKit.
    fn rule_inputs(&self) -> Option<(TrackerListInputs, Vec<PreparedRuleList>)> {
        let inputs = self.inputs.borrow();
        let (Some(tracker_data), Some(configuration)) = (&inputs.tracker_data, &inputs.configuration) else {
            return None;
        };
        let user_hosts = inputs.unprotected_sites.hosts_to_match();
        let trackers = TrackerListInputs {
            tracker_data: Arc::clone(tracker_data),
            exceptions: RuleExceptions::from_configuration(configuration, user_hosts.clone()),
            is_enabled: configuration.is_enabled(RuleExceptions::CONTENT_BLOCKING),
        };
        let https = if configuration.is_enabled(Self::HTTPS_FEATURE) {
            inputs.https_upgrades.content_rules()
        } else {
            Vec::new()
        };
        let links = TrackingParameterRemover::new(configuration, &user_hosts).content_rules();
        let other_lists = vec![
            RuleList { name: "https", rules: https }.prepared(),
            RuleList { name: "links", rules: links }.prepared(),
        ];
        Some((trackers, other_lists))
    }

    fn mark_protected(&self) {
        let previous = self.readiness.replace(Readiness::Protected);
        if let Readiness::Waiting(pending) = previous {
            glib::g_debug!(
                Self::LOG_DOMAIN,
                "pages protected {} ms after the session started",
                self.started.elapsed().as_millis()
            );
            for action in pending {
                action();
            }
        }
    }
}
