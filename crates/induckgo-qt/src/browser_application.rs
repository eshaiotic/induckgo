//! Starts the browser: language, engine, policies, then the QML window.

use std::fs;
use std::path::Path;
use std::process::ExitCode;
use std::sync::Arc;

use cxx_qt_lib::QFont;
use cxx_qt_lib::QGuiApplication;
use cxx_qt_lib::QQmlApplicationEngine;
use cxx_qt_lib::QString;
use cxx_qt_lib::QUrl;
use induckgo_core::engine::engine_verdict::EngineVerdict;
use induckgo_core::engine::minimum_engine_version::MinimumEngineVersion;
use induckgo_core::engine::window_kind::WindowKind;
use induckgo_core::guard::shared_guard_engine::SharedGuardEngine;
use induckgo_core::network::site_allow_list::SiteAllowList;
use induckgo_core::profile::profile_directories::ProfileLocation;
use induckgo_core::profile::profile_id::ProfileId;
use induckgo_engine_qt::qt_web_engine::QtWebEngine;
use serde_json::Value;

use crate::burn_restart::BurnRestart;
use crate::desktop_font::DesktopFont;
use crate::download_center::DownloadCenter;
use crate::guard_lists::GuardLists;
use crate::installation_directories::InstallationDirectories;
use crate::interface_catalogue::InterfaceCatalogue;
use crate::interface_language_setting::InterfaceLanguageSetting;
use crate::profile_policies::ProfilePolicies;
use crate::profile_policies::ProfileStrictness;
use crate::profile_preferences::ProfilePreferences;
use crate::profile_session::ProfileSession;
use crate::protection_documents::ProtectionDocuments;
use crate::site_protection::SiteProtection;
use crate::site_zoom_store::SiteZoomStore;
use crate::startup_notice::StartupNotice;

pub struct BrowserApplication;

impl BrowserApplication {
    const WINDOW: &'static str = "qrc:/qt/qml/io/induckgo/browser/qml/Main.qml";
    /// The name of the installed desktop entry; Wayland compositors take the window's icon and grouping from it.
    const DESKTOP_ENTRY: &'static str = "io.github.induckgo.InDuckGo";
    const WINDOW_ICON: &'static str = ":/icons/app.svg";

    #[must_use]
    pub fn run() -> ExitCode {
        let directories = InstallationDirectories::from_environment();
        InterfaceCatalogue::choose(&directories);
        QtWebEngine::initialize();

        let mut application = QGuiApplication::new();
        if let Some(mut application) = application.as_mut() {
            application.as_mut().set_application_name(&QString::from("InDuckGo"));
            Self::use_desktop_font(application);
        }
        QGuiApplication::set_desktop_file_name(&QString::from(Self::DESKTOP_ENTRY));
        QtWebEngine::set_window_icon(Self::WINDOW_ICON);
        Self::install_policies(&directories);
        Self::check_engine_version();

        let mut engine = QQmlApplicationEngine::new();
        if let Some(mut engine) = engine.as_mut() {
            engine.as_mut().load(&QUrl::from(Self::WINDOW));
        }
        match application.as_mut() {
            Some(application) => ExitCode::from(u8::try_from(application.exec()).unwrap_or(1)),
            None => ExitCode::FAILURE,
        }
    }

    /// Draws the interface in the font the desktop chose, as the GTK edition did.
    fn use_desktop_font(application: std::pin::Pin<&mut QGuiApplication>) {
        let mut font = QFont::default();
        let wanted = DesktopFont::from_desktop()
            .description(&font.family().map(|family| family.to_string()).unwrap_or_default());
        if font.from_string(&QString::from(wanted.as_str())) {
            application.set_application_font(&font);
        }
    }

    fn install_policies(directories: &InstallationDirectories) {
        let profile = directories.profile(&ProfileId::default_profile());
        let guard = Arc::new(SharedGuardEngine::default());
        let session = Self::open_session(directories, &profile, &guard);
        GuardLists::in_cache(&directories.protection_cache()).build_into(guard);
        let documents = ProtectionDocuments::from_cache(&directories.protection_cache());
        let allow_list = Self::read_json(&profile.data.join("site-allow-list.json"))
            .map(|document| SiteAllowList::from_json(&document))
            .unwrap_or_default();
        let policies = ProfilePolicies::assemble(
            documents.as_ref(),
            session.protection.shared_list(),
            Arc::clone(&session.guard),
            allow_list,
            ProfileStrictness::Standard,
        );
        session.install();
        let engine_directory = profile.data.join("engine");
        let engine_cache = profile.cache.join("engine");
        QtWebEngine::install_policies(policies.request, policies.cookies, &engine_directory, &engine_cache);
    }

    /// Finishes a burn the Fire button asked for before anything opens the profile's files, then reads the
    /// profile's stored choices.
    fn open_session(
        directories: &InstallationDirectories,
        profile: &ProfileLocation,
        guard: &Arc<SharedGuardEngine>,
    ) -> ProfileSession {
        let burned = BurnRestart::new(profile.clone()).complete_pending();
        let downloads = DownloadCenter::open(&profile.data, directories.system_downloads_folder());
        if let Err(error) = burned {
            StartupNotice::set(
                InterfaceCatalogue::current()
                    .text_with("Some data could not be deleted: {error}", &[("error", &error.to_string())]),
            );
        }
        ProfileSession::new(
            profile,
            SiteProtection::open(&profile.data),
            downloads,
            SiteZoomStore::open(&profile.data),
            ProfilePreferences::open(&profile.data),
            InterfaceLanguageSetting::open(&directories.interface_language_file()),
            Arc::clone(guard),
        )
    }

    fn check_engine_version() {
        let Some(installed) = QtWebEngine::chromium_version() else { return };
        if MinimumEngineVersion::current().verdict(&installed, WindowKind::Normal) == EngineVerdict::Warn {
            StartupNotice::set(format!(
                "The web engine ({installed}) is older than InDuckGo expects and may lack security fixes."
            ));
        }
    }

    fn read_json(path: &Path) -> Option<Value> {
        serde_json::from_slice(&fs::read(path).ok()?).ok()
    }
}
