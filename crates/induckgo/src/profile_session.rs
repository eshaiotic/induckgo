//! One isolated browsing profile, live in this process.
//!
//! Each InDuckGo process serves exactly one profile (or one web app's private profile), so a session owns:
//! a WebKit `NetworkSession` with its own cookie jar, storage, cache and HSTS list; the page protections;
//! and the state that feeds them (privacy configuration, Tracker Data Set, InDuckGo Guard's filter lists, sites
//! the user unprotected); and the zoom the user gave each site and the search engine the user chose.
//!
//! The Tor window's process serves a session that keeps nothing on disk ([`ProfileSession::start_tor`]): an
//! ephemeral `NetworkSession`, settings held in memory, compiled rules in the runtime directory (deleted at
//! start and on [`ProfileSession::close`]), WebRTC off (it would reach the network around the proxy), and
//! every request sent through Tor's SOCKS port once Tor is connected.
//!
//! Pages must not load before tracker blocking is active, nor in the Tor window before Tor is connected:
//! [`ProfileSession::when_protected`] holds a load back until rule lists are active and the route is set. Building the tracker list from the Tracker Data Set costs a few tenths of
//! a second of processor time, so it runs on a worker thread, and a start does not wait for it: the list
//! compiled last time is activated at once (as DuckDuckGo's macOS app does), then replaced if the new one
//! differs. Only the very first start, with nothing compiled yet, waits for the full compilation.
//!
//! Guard's lists take seconds to compile, so pages never wait for them: until a changed list is compiled, its
//! previous copy goes on protecting, except right after the user switched protection off for a site, when the
//! previous copy would keep blocking that very site and is left out instead. Switching protection back on is
//! the one change whose reload waits for the compiled lists, since the previous copy still exempts the site.

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::HashMap;
use std::io;
use std::path::PathBuf;
use std::rc::Rc;
use std::rc::Weak;
use std::sync::Arc;
use std::time::Instant;

use induckgo_core::guard::filter_list_source::FilterListSource;
use induckgo_core::guard::fingerprint_hardening::FingerprintHardening;
use induckgo_core::guard::guard_rule_list::GuardRuleList;
use induckgo_core::guard::pending_rule_changes::PendingRuleChanges;
use induckgo_core::guard::pending_rule_changes::RuleChange;
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
use webkit6::NetworkProxyMode;
use webkit6::NetworkProxySettings;
use webkit6::NetworkSession;
use webkit6::Settings;
use webkit6::WebContext;
use webkit6::WebView;
use webkit6::WebsiteDataTypes;

use crate::download_tracker::DownloadTracker;
use crate::filter_list_feed::FilterListFeed;
use crate::http_client::HttpClient;
use crate::installation_paths::InstallationPaths;
use crate::page_protections::PageProtections;
use crate::page_protections::PreparedRuleList;
use crate::page_protections::RuleList;
use crate::protection_feed::ProtectionDocuments;
use crate::protection_feed::ProtectionFeed;
use crate::site_zoom::SiteZoom;
use crate::stored_file::StoredFile;

/// Whether pages may load yet, and the loads waiting until they may.
struct Readiness {
    is_protected: bool,
    /// Always set for a profile; set for the Tor window once Tor is connected.
    is_routed: bool,
    waiting: Vec<Box<dyn FnOnce()>>,
}

impl Readiness {
    fn is_ready(&self) -> bool {
        self.is_protected && self.is_routed
    }

    /// The waiting loads, once pages may load.
    fn released(&mut self) -> Vec<Box<dyn FnOnce()>> {
        if self.is_ready() { std::mem::take(&mut self.waiting) } else { Vec::new() }
    }
}

/// What differs between a profile's session and the Tor window's.
struct SessionStorage {
    profile: Option<ProfileId>,
    network_session: NetworkSession,
    settings: Settings,
    content_filters: PathBuf,
    unprotected_sites: StoredFile,
    search_engine: StoredFile,
    site_zoom: StoredFile,
    default_search_engine: SearchEngine,
    burned: WebsiteDataTypes,
    /// Deleted at start and on close.
    scratch: Option<PathBuf>,
    is_routed: bool,
}

/// What the rule lists and the content scope script are built from.
struct ProtectionInputs {
    tracker_data: Option<Arc<TrackerDataSet>>,
    configuration: Option<PrivacyConfiguration>,
    unprotected_sites: UnprotectedSites,
    https_upgrades: HttpsUpgradePolicy,
    filter_lists: Arc<HashMap<FilterListSource, String>>,
}

/// The rule lists of one rebuild, by how they are built.
struct RuleInputs {
    /// Built later, on a worker thread.
    trackers: TrackerListInputs,
    /// HTTPS upgrades and link cleaning: cheap, and the HTTPS list must not carry over exemptions the user
    /// allowed in an earlier session, so they are always built fresh.
    quick_lists: Vec<PreparedRuleList>,
    /// Built only when the stored compiled copy is out of date.
    guard_lists: Vec<PreparedRuleList>,
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
    profile: Option<ProfileId>,
    network_session: NetworkSession,
    settings: Settings,
    protections: PageProtections,
    feed: ProtectionFeed,
    filter_list_feed: FilterListFeed,
    http: Rc<HttpClient>,
    unprotected_sites_file: StoredFile,
    zoom: SiteZoom,
    search_engine_file: StoredFile,
    search_engine: Cell<SearchEngine>,
    burned: WebsiteDataTypes,
    scratch: Option<PathBuf>,
    inputs: RefCell<ProtectionInputs>,
    readiness: RefCell<Readiness>,
    /// Bumped by every rebuild; a slower, older rebuild never overwrites a newer one.
    generation: Cell<u64>,
    /// What rebuilds promised (reloads, Guard left out), kept for whichever rebuild activates rules.
    pending: RefCell<PendingRuleChanges<Box<dyn FnOnce()>>>,
    started: Instant,
    _downloads: DownloadTracker,
}

impl ProfileSession {
    const LOG_DOMAIN: &'static str = "induckgo";
    const HTTPS_FEATURE: &'static str = "https";
    const REFRESH_SECONDS: u32 = 12 * 60 * 60;
    const RETRY_SECONDS: u32 = 30;
    const TOR_LANGUAGE: &'static str = "en-US";

    /// The session of `profile`, kept in the profile's directories.
    pub fn start(profile: ProfileId, paths: &InstallationPaths, http: Rc<HttpClient>) -> Rc<Self> {
        let location = paths.profile(&profile);
        let storage = SessionStorage {
            profile: Some(profile),
            network_session: Self::isolated_network_session(&location),
            settings: Self::private_settings(),
            content_filters: location.cache.join("content-filters"),
            unprotected_sites: StoredFile::new(location.data.join("unprotected-sites.json")),
            search_engine: StoredFile::new(location.data.join("search-engine.json")),
            site_zoom: StoredFile::new(location.data.join("site-zoom-levels.json")),
            default_search_engine: SearchEngine::default(),
            // The HSTS list is kept: forgetting it would let a network attacker downgrade sites to plain HTTP.
            burned: WebsiteDataTypes::all() - WebsiteDataTypes::HSTS_CACHE,
            scratch: None,
            is_routed: true,
        };
        Self::assemble(storage, paths, http)
    }

    /// The Tor window's session: nothing on disk, no page before [`Self::route_through_tor`]; `http` must be
    /// a client awaiting Tor as well.
    pub fn start_tor(paths: &InstallationPaths, http: Rc<HttpClient>) -> Rc<Self> {
        let content_filters = paths.tor_content_filters();
        Self::delete_scratch(&content_filters);
        // Pages learn the preferred languages (`Accept-Language`, `navigator.languages`); the system's would set
        // Tor window users apart, so it asks for English, as Tor Browser does. This process serves only the
        // Tor window, so the default context is its own.
        if let Some(context) = WebContext::default() {
            context.set_preferred_languages(&[Self::TOR_LANGUAGE]);
        }
        let settings = Self::private_settings();
        settings.set_enable_webrtc(false);
        settings.set_enable_media_stream(false);
        let storage = SessionStorage {
            profile: None,
            network_session: Self::protected_network_session(NetworkSession::new_ephemeral()),
            settings,
            content_filters: content_filters.clone(),
            unprotected_sites: StoredFile::discarded(),
            search_engine: StoredFile::discarded(),
            site_zoom: StoredFile::discarded(),
            default_search_engine: SearchEngine::DUCKDUCKGO_ONION,
            burned: WebsiteDataTypes::all(),
            scratch: Some(content_filters),
            is_routed: false,
        };
        Self::assemble(storage, paths, http)
    }

    fn assemble(storage: SessionStorage, paths: &InstallationPaths, http: Rc<HttpClient>) -> Rc<Self> {
        let search_engine = storage.search_engine.read_json().map(|document| SearchEngine::from_json(&document));
        let unprotected_sites = storage
            .unprotected_sites
            .read_json()
            .map(|document| UnprotectedSites::from_json(&document))
            .unwrap_or_default();
        let filter_list_feed = FilterListFeed::new(&paths.protection_feed());
        let session = Rc::new(Self {
            profile: storage.profile,
            _downloads: DownloadTracker::watch(&storage.network_session, paths.downloads()),
            network_session: storage.network_session,
            settings: storage.settings,
            protections: PageProtections::new(&storage.content_filters),
            feed: ProtectionFeed::new(&paths.protection_feed()),
            http,
            unprotected_sites_file: storage.unprotected_sites,
            zoom: SiteZoom::load(storage.site_zoom),
            search_engine_file: storage.search_engine,
            search_engine: Cell::new(search_engine.unwrap_or(storage.default_search_engine)),
            burned: storage.burned,
            scratch: storage.scratch,
            inputs: RefCell::new(ProtectionInputs {
                tracker_data: None,
                configuration: None,
                unprotected_sites,
                https_upgrades: HttpsUpgradePolicy::default(),
                filter_lists: Arc::new(filter_list_feed.cached()),
            }),
            filter_list_feed,
            readiness: RefCell::new(Readiness {
                is_protected: false,
                is_routed: storage.is_routed,
                waiting: Vec::new(),
            }),
            generation: Cell::new(0),
            pending: RefCell::new(PendingRuleChanges::default()),
            started: Instant::now(),
        });
        if let Some(documents) = session.feed.cached() {
            session.adopt(documents);
        }
        Self::keep_refreshing(Rc::downgrade(&session));
        session
    }

    /// `None` for the Tor window's session, which belongs to no profile.
    pub fn profile_id(&self) -> Option<&ProfileId> {
        self.profile.as_ref()
    }

    /// Whether this is the Tor window's session, whose traffic must never leave Tor.
    pub fn is_tor_window(&self) -> bool {
        self.profile.is_none()
    }

    /// Sends every request of this session through `proxy_uri` (Tor's SOCKS port with the identity's
    /// credentials) and lets pages load. Called again for a new identity or after Tor restarted.
    pub fn route_through_tor(&self, proxy_uri: &str) {
        let proxy = NetworkProxySettings::new(Some(proxy_uri), &[]);
        self.network_session.set_proxy_settings(NetworkProxyMode::Custom, Some(&proxy));
        self.http.route_through_tor(proxy_uri);
        let released = {
            let mut readiness = self.readiness.borrow_mut();
            readiness.is_routed = true;
            readiness.released()
        };
        for action in released {
            action();
        }
    }

    /// Deletes what the session kept only while it ran (the Tor window's compiled rules).
    pub fn close(&self) {
        if let Some(scratch) = &self.scratch {
            Self::delete_scratch(scratch);
        }
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

    /// Runs `action` once tracker blocking is active and the network route is set (immediately when both are).
    pub fn when_protected(&self, action: impl FnOnce() + 'static) {
        {
            let mut readiness = self.readiness.borrow_mut();
            if !readiness.is_ready() {
                readiness.waiting.push(Box::new(action));
                return;
            }
        }
        action();
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
        self.rebuild(RuleChange::Refresh, then);
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
        self.rebuild(if protected { RuleChange::SiteProtected } else { RuleChange::SiteUnprotected }, then);
    }

    /// The Fire button (New Identity in the Tor window): deletes every cookie, storage, cache, favicon and site
    /// zoom of this session; a profile keeps its HSTS list.
    pub async fn burn_website_data(&self) -> Result<(), glib::Error> {
        self.zoom.forget_all();
        let Some(data_manager) = self.network_session.website_data_manager() else { return Ok(()) };
        if let Some(favicons) = data_manager.favicon_database() {
            favicons.clear();
        }
        let (finished, cleared) = futures_channel::oneshot::channel();
        data_manager.clear(self.burned, glib::TimeSpan::from_seconds(0), None::<&gio::Cancellable>, |result| {
            let _ = finished.send(result);
        });
        cleared.await.unwrap_or(Ok(()))
    }

    /// A profile's session, with cookies kept in the profile's own directory.
    fn isolated_network_session(location: &ProfileLocation) -> NetworkSession {
        let session = Self::protected_network_session(NetworkSession::new(
            Some(&location.data.to_string_lossy()),
            Some(&location.cache.to_string_lossy()),
        ));
        if let Some(cookies) = session.cookie_manager() {
            let cookie_file = location.data.join("cookies.sqlite");
            cookies.set_persistent_storage(&cookie_file.to_string_lossy(), CookiePersistentStorage::Sqlite);
        }
        session
    }

    /// Third-party cookies blocked and Intelligent Tracking Prevention on, as in Safari.
    fn protected_network_session(session: NetworkSession) -> NetworkSession {
        session.set_itp_enabled(true);
        if let Some(cookies) = session.cookie_manager() {
            cookies.set_accept_policy(CookieAcceptPolicy::NoThirdParty);
        }
        if let Some(data_manager) = session.website_data_manager() {
            data_manager.set_favicons_enabled(true);
        }
        session
    }

    fn delete_scratch(directory: &std::path::Path) {
        match std::fs::remove_dir_all(directory) {
            Err(error) if error.kind() != io::ErrorKind::NotFound => {
                glib::g_warning!(Self::LOG_DOMAIN, "cannot delete {}: {error}", directory.display());
            }
            _ => {}
        }
    }

    /// Developer tools on, pop-ups only on a click, and the WebKit feature InDuckGo Guard switches on: the
    /// `Sec-GPC: 1` header (Global Privacy Control, a legally recognised "do not sell or share my data"
    /// signal) with every request, not only the `navigator.globalPrivacyControl` the page script reports.
    fn private_settings() -> Settings {
        const GUARD_FEATURES: [(&str, bool); 1] = [("GlobalPrivacyControl", true)];
        let settings = Settings::new();
        settings.set_enable_developer_extras(true);
        settings.set_javascript_can_open_windows_automatically(false);
        let features = Settings::all_features();
        for feature in features.iter().flat_map(|list| (0..list.length()).filter_map(|index| list.get(index))) {
            let identifier = feature.identifier();
            if let Some((_, enabled)) = GUARD_FEATURES.iter().find(|(name, _)| identifier.as_deref() == Some(*name)) {
                settings.set_feature_enabled(&feature, *enabled);
            }
        }
        settings
    }

    fn keep_refreshing(session: Weak<Self>) {
        glib::spawn_future_local(async move {
            loop {
                let Some(live) = session.upgrade() else { return };
                match live.feed.refreshed(&live.http).await {
                    Ok(documents) => {
                        if let Some(documents) = documents {
                            live.adopt(documents);
                        }
                        if let Some(lists) = live.filter_list_feed.refreshed(&live.http).await {
                            live.adopt_filter_lists(lists);
                        }
                    }
                    Err(error) => glib::g_warning!(Self::LOG_DOMAIN, "protection data refresh failed: {error}"),
                }
                let is_protected = live.readiness.borrow().is_protected;
                drop(live);
                glib::timeout_future_seconds(if is_protected { Self::REFRESH_SECONDS } else { Self::RETRY_SECONDS })
                    .await;
            }
        });
    }

    fn adopt(self: &Rc<Self>, documents: ProtectionDocuments) {
        {
            let mut inputs = self.inputs.borrow_mut();
            let configuration = PrivacyConfiguration::new(
                FingerprintHardening::applied_to(documents.configuration),
                PageProtections::COMPATIBILITY_VERSION,
            );
            inputs.https_upgrades.set_excluded_domains(&configuration.exception_domains(Self::HTTPS_FEATURE));
            inputs.tracker_data = Some(Arc::new(TrackerDataSet::from_json(&documents.tracker_data)));
            inputs.configuration = Some(configuration);
        }
        self.rebuild(RuleChange::Refresh, || {});
    }

    fn adopt_filter_lists(self: &Rc<Self>, lists: HashMap<FilterListSource, String>) {
        self.inputs.borrow_mut().filter_lists = Arc::new(lists);
        self.rebuild(RuleChange::Refresh, || {});
    }

    /// Rebuilds rule lists and script from the current inputs and activates them; Guard lists that changed
    /// follow once compiled. `then` runs once the rules reflect `change` (see [`RuleChange`]), even when a later
    /// rebuild takes over.
    fn rebuild(self: &Rc<Self>, change: RuleChange, then: impl FnOnce() + 'static) {
        self.install_content_scope_script();
        let Some(RuleInputs { trackers, quick_lists, guard_lists }) = self.rule_inputs() else {
            then();
            return;
        };
        self.pending.borrow_mut().record(change, Box::new(then));
        let generation = self.generation.get() + 1;
        self.generation.set(generation);
        let session = Rc::downgrade(self);
        glib::spawn_future_local(async move {
            let Some(live) = session.upgrade() else { return };
            if trackers.is_enabled && live.is_waiting() {
                live.protect_with_last_lists(&quick_lists).await;
            }
            let Ok(trackers) = gio::spawn_blocking(move || trackers.prepared()).await else {
                glib::g_critical!(Self::LOG_DOMAIN, "building the tracker rule list failed");
                return;
            };
            let lists: Vec<PreparedRuleList> = std::iter::once(trackers).chain(quick_lists).collect();
            let mut filters = live.protections.compile(&lists).await;
            let (current_guard, stale_guard) = live.protections.current_or_stale(guard_lists).await;
            filters.extend(current_guard);
            let mut interim = filters.clone();
            let leaves_guard_out = live.pending.borrow().leaves_guard_out();
            if !leaves_guard_out {
                for list in &stale_guard {
                    interim.extend(live.protections.last_compiled(list.name()).await);
                }
            }
            if live.generation.get() != generation {
                return;
            }
            live.protections.activate(&interim);
            live.mark_protected();
            live.keep_promises(stale_guard.is_empty());
            if stale_guard.is_empty() {
                return;
            }
            filters.extend(live.protections.compile(&stale_guard).await);
            if live.generation.get() == generation {
                live.protections.activate(&filters);
                live.keep_promises(true);
            }
        });
    }

    /// Runs what waited for the rules just activated; see [`PendingRuleChanges::settled`].
    fn keep_promises(&self, guard_compiled: bool) {
        let settled = self.pending.borrow_mut().settled(guard_compiled);
        for waiter in settled {
            waiter();
        }
    }

    /// Lets pages load under the tracker and Guard lists compiled last time, with fresh quick lists.
    async fn protect_with_last_lists(&self, quick_lists: &[PreparedRuleList]) {
        let Some(last_trackers) = self.protections.last_compiled(TrackerListInputs::NAME).await else { return };
        let mut filters = vec![last_trackers];
        for list in GuardRuleList::ALL {
            filters.extend(self.protections.last_compiled(list.name()).await);
        }
        filters.extend(self.protections.compile(quick_lists).await);
        if self.is_waiting() {
            self.protections.activate(&filters);
            self.mark_protected();
        }
    }

    fn is_waiting(&self) -> bool {
        !self.readiness.borrow().is_protected
    }

    fn install_content_scope_script(&self) {
        let inputs = self.inputs.borrow();
        if let Some(configuration) = &inputs.configuration {
            self.protections.install_script(configuration, &inputs.unprotected_sites.hosts_to_match());
        }
    }

    fn rule_inputs(&self) -> Option<RuleInputs> {
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
        let quick_lists = vec![
            RuleList { name: "https", rules: https }.prepared(),
            RuleList { name: "links", rules: links }.prepared(),
        ];
        let guard_lists = GuardRuleList::ALL
            .into_iter()
            .map(|list| Self::guard_list(list, &inputs.filter_lists, &user_hosts))
            .collect();
        Some(RuleInputs { trackers, quick_lists, guard_lists })
    }

    fn guard_list(
        list: GuardRuleList,
        downloaded: &Arc<HashMap<FilterListSource, String>>,
        user_hosts: &[String],
    ) -> PreparedRuleList {
        let (downloaded, user_hosts) = (Arc::clone(downloaded), user_hosts.to_vec());
        let fingerprint = list.fingerprint(&downloaded, &user_hosts);
        let is_empty = !list.has_rules(&downloaded);
        PreparedRuleList::built_on_demand(list.name(), fingerprint, is_empty, move || {
            list.converted(&downloaded, &user_hosts)
        })
    }

    fn mark_protected(&self) {
        let released = {
            let mut readiness = self.readiness.borrow_mut();
            if readiness.is_protected {
                return;
            }
            readiness.is_protected = true;
            readiness.released()
        };
        glib::g_debug!(
            Self::LOG_DOMAIN,
            "pages protected {} ms after the session started",
            self.started.elapsed().as_millis()
        );
        for action in released {
            action();
        }
    }
}
