//! Saves downloads into the user's Downloads folder under a safe, never-overwriting name.
//!
//! The file name comes from the website and is cleaned by [`DownloadFileName`]. Downloaded files are never
//! opened automatically; the toast only offers to show the containing folder.

use std::cell::Cell;
use std::path::Path;
use std::path::PathBuf;
use std::rc::Rc;

use adw::prelude::WidgetExt;
use gtk::prelude::Cast;
use gtk::prelude::StaticType;
use induckgo_core::download::download_file_name::DownloadFileName;
use webkit6::Download;
use webkit6::NetworkSession;

pub struct DownloadTracker;

impl DownloadTracker {
    pub fn watch(session: &NetworkSession, downloads: &Path) -> Self {
        let downloads = downloads.to_owned();
        session.connect_download_started(move |_, download| Self::follow(download, downloads.clone()));
        Self
    }

    fn follow(download: &Download, downloads: PathBuf) {
        download.connect_decide_destination(move |download, suggested| {
            let destination = DownloadFileName::from_suggestion(suggested).first_free_path_in(&downloads, Path::exists);
            if let Some(directory) = destination.parent()
                && let Err(error) = std::fs::create_dir_all(directory)
            {
                Self::announce(download, &format!("Cannot save download: {error}"), None);
                download.cancel();
                return true;
            }
            download.set_destination(&destination.to_string_lossy());
            true
        });
        let failed = Rc::new(Cell::new(false));
        let failed_flag = Rc::clone(&failed);
        download.connect_failed(move |download, error| {
            failed_flag.set(true);
            if !error.matches(webkit6::DownloadError::CancelledByUser) {
                Self::announce(download, &format!("Download failed: {error}"), None);
            }
        });
        download.connect_finished(move |download| {
            if failed.get() {
                return;
            }
            let destination = download.destination().map(|path| PathBuf::from(path.as_str()));
            let name = destination.as_deref().and_then(Path::file_name).map(|name| name.to_string_lossy().into_owned());
            Self::announce(download, &format!("Downloaded {}", name.unwrap_or_default()), destination);
        });
    }

    /// A toast in the window the download came from, with "Show in Folder" when a file was saved.
    fn announce(download: &Download, text: &str, saved: Option<PathBuf>) {
        let Some(overlay) = download
            .web_view()
            .and_then(|view| view.ancestor(adw::ToastOverlay::static_type()))
            .and_then(|widget| widget.downcast::<adw::ToastOverlay>().ok())
        else {
            return;
        };
        let toast = adw::Toast::new(text);
        if let Some(saved) = saved {
            toast.set_button_label(Some("Show in Folder"));
            let window = overlay.root().and_then(|root| root.downcast::<gtk::Window>().ok());
            toast.connect_button_clicked(move |_| {
                gtk::FileLauncher::new(Some(&gio::File::for_path(&saved))).open_containing_folder(
                    window.as_ref(),
                    None::<&gio::Cancellable>,
                    |_| {},
                );
            });
        }
        overlay.add_toast(toast);
    }
}
