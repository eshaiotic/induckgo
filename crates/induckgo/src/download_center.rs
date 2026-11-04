//! What a profile's windows share about downloads: the list of recent ones, the settings, and who to tell
//! when either changes.
//!
//! The list (`downloads.json`) and the settings (`download-settings.json`) belong to the profile; the Tor
//! window keeps both in memory only, like its other preferences.

use std::cell::RefCell;
use std::path::Path;
use std::path::PathBuf;
use std::rc::Rc;

use induckgo_core::download::download_settings::DownloadSettings;
use induckgo_core::download::recent_downloads::DownloadId;
use induckgo_core::download::recent_downloads::RecentDownload;
use induckgo_core::download::recent_downloads::RecentDownloads;

use crate::stored_file::StoredFile;

type Observer = Box<dyn Fn() -> glib::ControlFlow>;

pub struct DownloadCenter {
    history_file: StoredFile,
    settings_file: StoredFile,
    recent: RefCell<RecentDownloads>,
    settings: RefCell<DownloadSettings>,
    system_folder: PathBuf,
    observers: RefCell<Vec<Observer>>,
}

impl DownloadCenter {
    pub const HISTORY_FILE_NAME: &'static str = "downloads.json";
    pub const SETTINGS_FILE_NAME: &'static str = "download-settings.json";
    const LOG_DOMAIN: &'static str = "induckgo";

    /// `system_folder` is the system's Downloads folder, used until the person picks another.
    pub fn load(history_file: StoredFile, settings_file: StoredFile, system_folder: &Path) -> Rc<Self> {
        let recent = history_file.read_json().map(|document| RecentDownloads::from_json(&document));
        let settings = settings_file.read_json().map(|document| DownloadSettings::from_json(&document));
        Rc::new(Self {
            history_file,
            settings_file,
            recent: RefCell::new(recent.unwrap_or_default()),
            settings: RefCell::new(settings.unwrap_or_default()),
            system_folder: system_folder.to_owned(),
            observers: RefCell::new(Vec::new()),
        })
    }

    /// The recent downloads, newest first.
    pub fn entries(&self) -> Vec<RecentDownload> {
        self.recent.borrow().entries().to_vec()
    }

    pub fn begin(&self, path: PathBuf) -> DownloadId {
        let id = self.recent.borrow_mut().begin(path);
        self.notify();
        id
    }

    pub fn progress(&self, id: DownloadId, received: u64, total: Option<u64>) {
        self.recent.borrow_mut().progress(id, received, total);
        self.notify();
    }

    pub fn finish(&self, id: DownloadId) {
        self.recent.borrow_mut().finish(id);
        self.save_history();
        self.notify();
    }

    pub fn fail(&self, id: DownloadId) {
        self.recent.borrow_mut().fail(id);
        self.notify();
    }

    pub fn discard(&self, id: DownloadId) {
        self.recent.borrow_mut().discard(id);
        self.notify();
    }

    /// Forgets the list of finished and failed downloads, for a profile whose browsing data was burned.
    pub fn forget_all(&self) {
        self.recent.borrow_mut().forget_finished();
        self.save_history();
        self.notify();
    }

    pub fn asks_where_to_save(&self) -> bool {
        self.settings.borrow().asks_where_to_save()
    }

    /// The folder downloads go to (or the save dialog starts in).
    pub fn folder(&self) -> PathBuf {
        self.settings.borrow().folder_or(&self.system_folder).to_owned()
    }

    pub fn set_asking(&self, asks_where_to_save: bool) {
        let settings = self.settings.take().with_asking(asks_where_to_save);
        self.store_settings(settings);
    }

    pub fn set_folder(&self, folder: PathBuf) {
        let settings = self.settings.take().with_folder(folder);
        self.store_settings(settings);
    }

    /// Calls `observer` after every change to the list or the settings, until it returns `Break`.
    pub fn observe(&self, observer: impl Fn() -> glib::ControlFlow + 'static) {
        self.observers.borrow_mut().push(Box::new(observer));
    }

    fn store_settings(&self, settings: DownloadSettings) {
        if let Err(error) = self.settings_file.write_json(&settings.to_json()) {
            glib::g_warning!(Self::LOG_DOMAIN, "cannot save the download settings: {error}");
        }
        self.settings.replace(settings);
        self.notify();
    }

    fn save_history(&self) {
        let document = self.recent.borrow().to_json();
        if let Err(error) = self.history_file.write_json(&document) {
            glib::g_warning!(Self::LOG_DOMAIN, "cannot save the downloads list: {error}");
        }
    }

    fn notify(&self) {
        let mut observers = self.observers.take();
        observers.retain(|observer| observer() == glib::ControlFlow::Continue);
        let added_meanwhile = self.observers.replace(observers);
        self.observers.borrow_mut().extend(added_meanwhile);
    }
}
