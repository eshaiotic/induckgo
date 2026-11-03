//! Preferences of the profile the window belongs to: which buttons the toolbar shows around the address bar.
//!
//! Every switch takes effect at once in all of the profile's open windows and is saved with the profile.

use std::rc::Rc;

use adw::prelude::AdwDialogExt;
use adw::prelude::PreferencesDialogExt;
use adw::prelude::PreferencesGroupExt;
use adw::prelude::PreferencesPageExt;
use induckgo_core::profile::toolbar_layout::ToolbarButton;

use crate::app_context::AppContext;

pub struct PreferencesDialog;

impl PreferencesDialog {
    pub fn present(context: &Rc<AppContext>, parent: &adw::ApplicationWindow) {
        let group = adw::PreferencesGroup::builder()
            .title("Toolbar Buttons")
            .description("Buttons shown around the address bar in this profile's windows.")
            .build();
        for button in ToolbarButton::ALL {
            let (title, subtitle) = Self::describe(button);
            let row =
                adw::SwitchRow::builder().title(title).subtitle(subtitle).active(context.toolbar.shows(button)).build();
            let context = Rc::downgrade(context);
            row.connect_active_notify(move |row| {
                let Some(context) = context.upgrade() else { return };
                context.toolbar.set_shown(button, row.is_active());
                for browser in context.browser_windows.open() {
                    browser.show_toolbar_buttons();
                }
            });
            group.add(&row);
        }
        let page = adw::PreferencesPage::builder()
            .title("Appearance")
            .icon_name("preferences-desktop-appearance-symbolic")
            .build();
        page.add(&group);
        let dialog = adw::PreferencesDialog::builder().title("Preferences").build();
        dialog.add(&page);
        dialog.present(Some(parent));
    }

    fn describe(button: ToolbarButton) -> (&'static str, &'static str) {
        match button {
            ToolbarButton::BackAndForward => ("Back and Forward", "Go to the previous or next page"),
            ToolbarButton::Reload => ("Reload", "Load the page again, or stop loading it"),
            ToolbarButton::TorWindow => ("Tor Window", "Open a window whose traffic goes only through Tor"),
            ToolbarButton::Protections => ("Protection Shield", "Switch tracker protection off or on for the site"),
            ToolbarButton::BurnData => ("Burn Browsing Data", "Delete this profile's cookies, site data and cache"),
            ToolbarButton::NewTab => ("New Tab", "Open a new tab"),
            ToolbarButton::InstallApp => ("Install Site as App", "The icon at the end of the address bar"),
        }
    }
}
