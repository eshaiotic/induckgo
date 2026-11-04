//! Preferences: the interface language (shared by every profile) and, for the profile the window belongs to,
//! which buttons the toolbar shows around the address bar.
//!
//! Every toolbar switch takes effect at once in all of the profile's open windows and is saved with the profile.
//! The language is saved at once too, and applies the next time InDuckGo starts: every window, menu and dialog
//! is built in one language, so none ever mixes two.

use std::rc::Rc;

use adw::prelude::AdwDialogExt;
use adw::prelude::ComboRowExt;
use adw::prelude::ObjectExt;
use adw::prelude::PreferencesDialogExt;
use adw::prelude::PreferencesGroupExt;
use adw::prelude::PreferencesPageExt;
use induckgo_core::language::interface_language::InterfaceLanguage;
use induckgo_core::language::language_choice::LanguageChoice;
use induckgo_core::profile::toolbar_layout::ToolbarButton;

use crate::app_context::AppContext;
use crate::interface_language_setting::InterfaceLanguageSetting;

pub struct PreferencesDialog;

impl PreferencesDialog {
    pub fn present(context: &Rc<AppContext>, parent: &adw::ApplicationWindow) {
        let dialog = adw::PreferencesDialog::builder().title(tr!("Preferences")).build();
        dialog.add(&Self::interface_page(context, &dialog));
        dialog.add(&Self::appearance_page(context));
        dialog.present(Some(parent));
    }

    /// The Interface page: one row choosing the language, with the system's language first.
    fn interface_page(context: &Rc<AppContext>, dialog: &adw::PreferencesDialog) -> adw::PreferencesPage {
        let system_entry =
            tr!("System default ({language})", language = InterfaceLanguageSetting::system_language().endonym());
        let names = gtk::StringList::new(&[system_entry.as_str()]);
        for language in InterfaceLanguage::ALL {
            names.append(language.endonym());
        }
        let row = adw::ComboRow::builder()
            .title(tr!("Language"))
            .subtitle(tr!("Applies the next time InDuckGo starts"))
            .model(&names)
            .enable_search(true)
            .selected(Self::position_of(context.interface_language.choice()))
            .build();
        let setting = Rc::downgrade(context);
        let dialog = dialog.downgrade();
        row.connect_selected_notify(move |row| {
            let (Some(context), Some(dialog)) = (setting.upgrade(), dialog.upgrade()) else { return };
            let choice = Self::choice_at(row.selected());
            if choice != context.interface_language.choice() {
                context.interface_language.choose(choice);
                dialog.add_toast(adw::Toast::new(tr!("The language changes when InDuckGo is restarted")));
            }
        });
        let group = adw::PreferencesGroup::builder()
            .title(tr!("Language"))
            .description(tr!("The language of menus, dialogs and messages, in every profile and web app."))
            .build();
        group.add(&row);
        let page = adw::PreferencesPage::builder()
            .title(tr!("Interface"))
            .icon_name("preferences-desktop-locale-symbolic")
            .build();
        page.add(&group);
        page
    }

    /// Row 0 is "System default"; row `n` is the `n`-th language of the list.
    fn position_of(choice: LanguageChoice) -> u32 {
        match choice {
            LanguageChoice::System => 0,
            LanguageChoice::Fixed(language) => InterfaceLanguage::ALL
                .iter()
                .position(|candidate| *candidate == language)
                .and_then(|index| u32::try_from(index + 1).ok())
                .unwrap_or(0),
        }
    }

    fn choice_at(position: u32) -> LanguageChoice {
        usize::try_from(position)
            .ok()
            .and_then(|row| row.checked_sub(1))
            .and_then(|index| InterfaceLanguage::ALL.get(index).copied())
            .map_or(LanguageChoice::System, LanguageChoice::Fixed)
    }

    fn appearance_page(context: &Rc<AppContext>) -> adw::PreferencesPage {
        let group = adw::PreferencesGroup::builder()
            .title(tr!("Toolbar Buttons"))
            .description(tr!("Buttons shown around the address bar in this profile's windows."))
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
            .title(tr!("Appearance"))
            .icon_name("preferences-desktop-appearance-symbolic")
            .build();
        page.add(&group);
        page
    }

    fn describe(button: ToolbarButton) -> (&'static str, &'static str) {
        match button {
            ToolbarButton::BackAndForward => (tr!("Back and Forward"), tr!("Go to the previous or next page")),
            ToolbarButton::Reload => (tr!("Reload"), tr!("Load the page again, or stop loading it")),
            ToolbarButton::TorWindow => (tr!("Tor Window"), tr!("Open a window whose traffic goes only through Tor")),
            ToolbarButton::Protections => {
                (tr!("Protection Shield"), tr!("Switch tracker protection off or on for the site"))
            }
            ToolbarButton::BurnData => {
                (tr!("Burn Browsing Data"), tr!("Delete this profile's cookies, site data and cache"))
            }
            ToolbarButton::NewTab => (tr!("New Tab"), tr!("Open a new tab")),
            ToolbarButton::InstallApp => (tr!("Install Site as App"), tr!("The icon at the end of the address bar")),
        }
    }
}
