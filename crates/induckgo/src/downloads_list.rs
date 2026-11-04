//! The list the downloads button and the main menu's "Downloads" entry open: the latest downloads, each with
//! a button that shows its folder (files can be in different folders), and a last row to the download folder.
//!
//! Clicking a finished download opens the file; the list follows the [`DownloadCenter`] while it is visible.

use std::path::Path;
use std::rc::Rc;

use glib::prelude::ObjectExt;
use gtk::prelude::BoxExt;
use gtk::prelude::ButtonExt;
use gtk::prelude::Cast;
use gtk::prelude::WidgetExt;
use induckgo_core::download::recent_downloads::DownloadState;
use induckgo_core::download::recent_downloads::RecentDownload;

use crate::download_center::DownloadCenter;

pub struct DownloadsList {
    root: gtk::Box,
    center: Rc<DownloadCenter>,
    /// `false` in the Tor window: a file opened in another application would connect outside Tor.
    opens_files: bool,
}

impl DownloadsList {
    const WIDTH: i32 = 360;

    pub fn new(center: &Rc<DownloadCenter>, opens_files: bool) -> Rc<Self> {
        let list = Rc::new(Self {
            root: gtk::Box::builder()
                .orientation(gtk::Orientation::Vertical)
                .spacing(2)
                .width_request(Self::WIDTH)
                .build(),
            center: Rc::clone(center),
            opens_files,
        });
        list.rebuild();
        let followed = list.root.downgrade();
        let owner = Rc::downgrade(&list);
        center.observe(move || match (followed.upgrade(), owner.upgrade()) {
            (Some(root), Some(list)) => {
                if root.is_mapped() {
                    list.rebuild();
                }
                glib::ControlFlow::Continue
            }
            _ => glib::ControlFlow::Break,
        });
        let owner = Rc::clone(&list);
        list.root.connect_map(move |_| owner.rebuild());
        list
    }

    pub fn widget(&self) -> &gtk::Box {
        &self.root
    }

    fn rebuild(&self) {
        while let Some(child) = self.root.first_child() {
            self.root.remove(&child);
        }
        let entries = self.center.entries();
        if entries.is_empty() {
            let empty = gtk::Label::builder()
                .label(tr!("No downloads yet"))
                .css_classes(["dim-label"])
                .margin_top(12)
                .margin_bottom(12)
                .build();
            self.root.append(&empty);
        }
        for entry in &entries {
            self.root.append(&self.row(entry));
        }
        self.root.append(&gtk::Separator::new(gtk::Orientation::Horizontal));
        let folder = self.center.folder();
        let go_label = gtk::Label::builder().label(tr!("Go to Download Folder")).css_classes(["body"]).build();
        let go = gtk::Button::builder().child(&go_label).css_classes(["flat"]).build();
        go.connect_clicked(move |button| {
            gtk::FileLauncher::new(Some(&gio::File::for_path(&folder))).launch(
                Self::window_of(button).as_ref(),
                None::<&gio::Cancellable>,
                |_| {},
            );
        });
        self.root.append(&go);
    }

    fn row(&self, entry: &RecentDownload) -> gtk::Box {
        let row = gtk::Box::builder().spacing(4).build();
        let name = gtk::Label::builder()
            .label(entry.name())
            .xalign(0.0)
            .ellipsize(gtk::pango::EllipsizeMode::Middle)
            .max_width_chars(34)
            .css_classes(["body"])
            .build();
        let status = gtk::Label::builder()
            .label(Self::status_of(entry))
            .xalign(0.0)
            .ellipsize(gtk::pango::EllipsizeMode::Start)
            .max_width_chars(40)
            .css_classes(["caption", "dim-label"])
            .build();
        let text = gtk::Box::builder().orientation(gtk::Orientation::Vertical).build();
        text.append(&name);
        text.append(&status);
        let open = gtk::Button::builder().child(&text).css_classes(["flat"]).hexpand(true).build();
        let path = entry.path().to_owned();
        let finished = *entry.state() == DownloadState::Finished;
        open.set_sensitive(self.opens_files && finished && path.exists());
        if !self.opens_files {
            open.set_tooltip_text(Some(tr!(
                "Other applications do not open from the Tor window: they would connect outside Tor."
            )));
        }
        open.connect_clicked(move |button| {
            let launcher = gtk::FileLauncher::new(Some(&gio::File::for_path(&path)));
            // A program, script or launcher the site sent is never started without the person choosing so.
            let (content_type, _) = gio::content_type_guess(Some(&path), None::<&[u8]>);
            launcher.set_always_ask(gio::content_type_can_be_executable(&content_type));
            launcher.launch(Self::window_of(button).as_ref(), None::<&gio::Cancellable>, |_| {});
        });
        let folder = gtk::Button::builder()
            .icon_name("folder-open-symbolic")
            .tooltip_text(tr!("Show in Folder"))
            .valign(gtk::Align::Center)
            .css_classes(["flat", "circular"])
            .build();
        let path = entry.path().to_owned();
        folder.connect_clicked(move |button| {
            gtk::FileLauncher::new(Some(&gio::File::for_path(&path))).open_containing_folder(
                Self::window_of(button).as_ref(),
                None::<&gio::Cancellable>,
                |_| {},
            );
        });
        row.append(&open);
        row.append(&folder);
        row
    }

    /// Where the file is, or how far the download got.
    fn status_of(entry: &RecentDownload) -> String {
        match entry.state() {
            DownloadState::Finished => {
                entry.path().parent().map(Path::display).map(|folder| folder.to_string()).unwrap_or_default()
            }
            DownloadState::Failed => tr!("Download failed").to_owned(),
            DownloadState::InProgress { received, total: Some(total) } if *total > 0 => {
                tr!("Downloading… {percent}%", percent = received.saturating_mul(100) / total)
            }
            DownloadState::InProgress { .. } => tr!("Downloading…").to_owned(),
        }
    }

    fn window_of(button: &gtk::Button) -> Option<gtk::Window> {
        button.root().and_then(|root| root.downcast::<gtk::Window>().ok())
    }
}
