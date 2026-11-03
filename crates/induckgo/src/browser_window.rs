//! A browser window: tabs, address bar, navigation, per-site protection switch, Fire button and menu.
//!
//! In the Tor window's process the same window covers its tabs with the Tor connection page until Tor is
//! connected, and its Fire button gives a New Identity: data burned, new circuits, new SOCKS credentials. It
//! offers no web apps, profiles or search engines, which would take browsing out of Tor.

use std::cell::Cell;
use std::cell::RefCell;
use std::rc::Rc;
use std::rc::Weak;

use adw::prelude::AdwApplicationWindowExt;
use adw::prelude::AdwDialogExt;
use adw::prelude::AlertDialogExt;
use adw::prelude::AlertDialogExtManual;
use gio::prelude::ActionMapExt;
use gio::prelude::ActionMapExtManual;
use gtk::prelude::BoxExt;
use gtk::prelude::ButtonExt;
use gtk::prelude::Cast;
use gtk::prelude::EditableExt;
use gtk::prelude::EntryExt;
use gtk::prelude::GtkApplicationExt;
use gtk::prelude::GtkWindowExt;
use gtk::prelude::ToVariant;
use gtk::prelude::ToggleButtonExt;
use gtk::prelude::WidgetExt;
use induckgo_core::navigation::address_bar_input::AddressBarInput;
use induckgo_core::navigation::search_engine::SearchEngine;
use induckgo_core::page_zoom::zoom_level::ZoomLevel;
use induckgo_core::protection::domain_name::DomainName;
use induckgo_core::web_app::installed_web_app::InstalledWebApp;
use url::Url;
use webkit6::WebView;
use webkit6::prelude::WebViewExt;

use crate::app_context::AppContext;
use crate::app_icons::AppIcons;
use crate::find_bar::FindBar;
use crate::management_dialogs::ProfilesDialog;
use crate::management_dialogs::WebAppsDialog;
use crate::page_view::PageView;
use crate::tor_connection::TorConnection;
use crate::tor_connection::TorState;
use crate::tor_connection_page::TorConnectionPage;
use crate::tor_settings_dialog::TorSettingsDialog;
use crate::web_app_installer::WebAppInstaller;
use crate::zoom_controls::ZoomControls;

/// A window action: its name and what it does.
type BrowserCommand = (&'static str, fn(&Rc<BrowserWindow>));

pub struct BrowserWindow {
    context: Rc<AppContext>,
    window: adw::ApplicationWindow,
    tab_view: adw::TabView,
    address: gtk::Entry,
    reload: gtk::Button,
    shield: gtk::ToggleButton,
    find_bar: Rc<FindBar>,
    zoom: ZoomControls,
    toasts: adw::ToastOverlay,
    /// The tabs, or in the Tor window the connection page until Tor is connected.
    content: gtk::Stack,
    tor_page: Option<TorConnectionPage>,
    pages: RefCell<Vec<Rc<PageView>>>,
    /// Set while the header mirrors the page, so mirroring does not trigger the switch's own handler.
    updating_header: Cell<bool>,
}

impl BrowserWindow {
    const PRODUCT: &'static str = "InDuckGo";
    const TABS: &'static str = "tabs";
    const TOR_PAGE: &'static str = "tor";

    /// Opens a window with one tab per typed address (or the search engine's home page).
    pub fn open(application: &adw::Application, context: &Rc<AppContext>, addresses: &[String]) -> Rc<Self> {
        let tab_view = adw::TabView::new();
        let address = gtk::Entry::builder().hexpand(true).build();
        let toasts = adw::ToastOverlay::new();
        let find_bar = FindBar::new();
        let browser = Rc::new(Self {
            context: Rc::clone(context),
            window: adw::ApplicationWindow::builder()
                .application(application)
                .default_width(1280)
                .default_height(860)
                .build(),
            reload: gtk::Button::builder()
                .icon_name("view-refresh-symbolic")
                .action_name("win.reload")
                .tooltip_text("Reload")
                .build(),
            shield: gtk::ToggleButton::builder()
                .icon_name("security-high-symbolic")
                .tooltip_text("Tracker protection on this site")
                .build(),
            tab_view,
            address,
            find_bar,
            zoom: ZoomControls::new(),
            toasts,
            content: gtk::Stack::new(),
            tor_page: context.tor.as_ref().map(|_| TorConnectionPage::new()),
            pages: RefCell::new(Vec::new()),
            updating_header: Cell::new(false),
        });
        browser.build_layout();
        browser.install_actions();
        browser.connect_signals();
        browser.show_search_engine();
        browser.follow_tor();
        if !browser.open_typed(addresses) {
            browser.open_tab(browser.home_page());
        }
        let keep_alive = Rc::clone(&browser);
        browser.window.connect_destroy(move |_| keep_alive.pages.borrow_mut().clear());
        browser.window.present();
        browser
    }

    pub fn window(&self) -> &adw::ApplicationWindow {
        &self.window
    }

    /// Opens a tab for each typed address or search, as if typed into the address bar; whether any opened.
    pub fn open_typed(self: &Rc<Self>, typed: &[String]) -> bool {
        let engine = self.context.session.search_engine();
        let destinations: Vec<String> =
            typed.iter().filter_map(|text| AddressBarInput::new(text).destination(engine)).collect();
        for destination in &destinations {
            self.open_tab(destination);
        }
        !destinations.is_empty()
    }

    /// Names the profile's search engine in the empty address bar.
    pub fn show_search_engine(&self) {
        let engine = self.context.session.search_engine();
        self.address.set_placeholder_text(Some(&format!("Search {} or enter address", engine.name())));
    }

    /// Opens `address` in a new, selected tab.
    pub fn open_tab(self: &Rc<Self>, address: &str) {
        let page = PageView::new(&self.context.session, self.context.session.new_web_view());
        self.add_page(&page);
        page.open(address);
    }

    /// Closes the tabs that showed `app` — its own pages and the tab it was installed from — now that the
    /// app has its own window. A window left without tabs closes.
    pub fn close_tabs_of(&self, app: &InstalledWebApp, installed_from: &WebView) {
        let closing: Vec<adw::TabPage> = self
            .pages
            .borrow()
            .iter()
            .filter(|page| {
                page.web_view() == installed_from || page.web_view().uri().is_some_and(|uri| app.is_own_page(&uri))
            })
            .map(|page| self.tab_view.page(page.widget()))
            .collect();
        for tab in &closing {
            self.tab_view.close_page(tab);
        }
    }

    /// Closes every tab and opens a fresh one; used after the Fire button deleted the profile's data.
    pub fn start_over(self: &Rc<Self>) {
        let open_pages: Vec<adw::TabPage> =
            (0..self.tab_view.n_pages()).map(|index| self.tab_view.nth_page(index)).collect();
        self.pages.borrow_mut().clear();
        self.open_tab(self.home_page());
        for page in &open_pages {
            self.tab_view.close_page(page);
        }
    }

    fn build_layout(self: &Rc<Self>) {
        let header = adw::HeaderBar::new();
        for (icon, action, tooltip) in
            [("go-previous-symbolic", "win.back", "Back"), ("go-next-symbolic", "win.forward", "Forward")]
        {
            header
                .pack_start(&gtk::Button::builder().icon_name(icon).action_name(action).tooltip_text(tooltip).build());
        }
        header.pack_start(&self.reload);
        let address_row = gtk::Box::new(gtk::Orientation::Horizontal, 6);
        address_row.append(&self.address);
        address_row.append(self.zoom.badge());
        header.set_title_widget(Some(
            &adw::Clamp::builder().maximum_size(860).tightening_threshold(560).child(&address_row).build(),
        ));
        let menu = gtk::MenuButton::builder().icon_name("open-menu-symbolic").tooltip_text("Main Menu").build();
        let browser = Rc::downgrade(self);
        menu.set_create_popup_func(move |button| {
            if let Some(browser) = browser.upgrade() {
                let level = browser.selected_page().map_or(ZoomLevel::ACTUAL_SIZE, |page| page.zoom_level());
                button.set_menu_model(Some(&Self::menu_model(&browser.context, level)));
            }
        });
        header.pack_end(&menu);
        header.pack_end(
            &gtk::Button::builder()
                .icon_name("tab-new-symbolic")
                .action_name("win.new-tab")
                .tooltip_text("New Tab")
                .build(),
        );
        header.pack_end(
            &gtk::Button::builder()
                .icon_name(AppIcons::FIRE)
                .action_name("win.fire")
                .tooltip_text(if self.context.tor.is_some() { "New Identity" } else { "Burn Browsing Data" })
                .build(),
        );
        header.pack_end(&self.shield);
        let toolbar = adw::ToolbarView::new();
        toolbar.add_top_bar(&header);
        toolbar.add_top_bar(&adw::TabBar::builder().view(&self.tab_view).autohide(true).build());
        toolbar.add_top_bar(self.find_bar.widget());
        self.content.add_named(&self.tab_view, Some(Self::TABS));
        if let Some(tor_page) = &self.tor_page {
            self.content.add_named(tor_page.widget(), Some(Self::TOR_PAGE));
        }
        toolbar.set_content(Some(&self.content));
        self.toasts.set_child(Some(&toolbar));
        self.window.set_content(Some(&self.toasts));
        self.window.set_title(Some(&self.product_title()));
    }

    fn menu_model(context: &AppContext, zoom: ZoomLevel) -> gio::Menu {
        let menu = gio::Menu::new();
        let tabs = gio::Menu::new();
        tabs.append(Some("New Tab"), Some("win.new-tab"));
        if context.tor.is_none() {
            tabs.append(Some("New Window"), Some("app.new-window"));
        }
        tabs.append(Some("New Tor Window"), Some("app.new-tor-window"));
        menu.append_section(None, &tabs);
        let page = gio::Menu::new();
        page.append(Some("Find in Page…"), Some("win.find"));
        menu.append_section(None, &page);
        menu.append_item(&ZoomControls::menu_row(zoom));
        if context.tor.is_some() {
            let tor = gio::Menu::new();
            tor.append(Some("New Identity"), Some("win.fire"));
            tor.append(Some("Tor Connection…"), Some(TorConnectionPage::SETTINGS_ACTION));
            menu.append_section(None, &tor);
            let about = gio::Menu::new();
            about.append(Some("About InDuckGo"), Some("app.about"));
            menu.append_section(None, &about);
            return menu;
        }
        let apps = gio::Menu::new();
        apps.append(Some("Install Site as App…"), Some("win.install-web-app"));
        apps.append(Some("Installed Apps…"), Some("win.manage-web-apps"));
        menu.append_section(None, &apps);
        let profiles = gio::Menu::new();
        for profile in context.catalogs.profiles().profiles() {
            let item = gio::MenuItem::new(Some(&profile.name), None);
            item.set_action_and_target_value(Some("app.open-profile"), Some(&profile.id.as_str().to_variant()));
            profiles.append_item(&item);
        }
        profiles.append(Some("Manage Profiles…"), Some("win.manage-profiles"));
        menu.append_section(Some("Profiles"), &profiles);
        let search_engines = gio::Menu::new();
        for engine in SearchEngine::ALL {
            let item = gio::MenuItem::new(Some(engine.name()), None);
            item.set_action_and_target_value(Some("app.search-engine"), Some(&engine.id().to_variant()));
            search_engines.append_item(&item);
        }
        let about = gio::Menu::new();
        about.append_submenu(Some("Search Engine"), &search_engines);
        about.append(Some("About InDuckGo"), Some("app.about"));
        menu.append_section(None, &about);
        menu
    }

    fn install_actions(self: &Rc<Self>) {
        let actions: [BrowserCommand; 16] = [
            ("back", |browser| browser.with_web_view(WebViewExt::go_back)),
            ("forward", |browser| browser.with_web_view(WebViewExt::go_forward)),
            ("reload", |browser| {
                browser.with_web_view(|view| if view.is_loading() { view.stop_loading() } else { view.reload() });
            }),
            ("new-tab", |browser| {
                browser.open_tab(browser.home_page());
                browser.focus_address();
            }),
            ("close-tab", |browser| {
                if let Some(page) = browser.tab_view.selected_page() {
                    browser.tab_view.close_page(&page);
                }
            }),
            ("focus-address", Self::focus_address),
            ("find", |browser| {
                if let Some(page) = browser.selected_page() {
                    browser.find_bar.show_for(page.web_view());
                }
            }),
            ("zoom-in", |browser| browser.change_zoom(ZoomLevel::zoomed_in)),
            ("zoom-out", |browser| browser.change_zoom(ZoomLevel::zoomed_out)),
            ("zoom-reset", |browser| browser.change_zoom(|_| ZoomLevel::ACTUAL_SIZE)),
            ("fire", Self::confirm_fire),
            ("install-web-app", Self::install_web_app),
            ("manage-web-apps", |browser| WebAppsDialog::present(&browser.context, &browser.window)),
            ("manage-profiles", |browser| ProfilesDialog::present(&browser.context, &browser.window)),
            ("tor-retry", |browser| browser.context.tor.iter().for_each(TorConnection::start)),
            ("tor-settings", |browser| {
                browser.context.tor.iter().for_each(|tor| TorSettingsDialog::present(tor, &browser.window));
            }),
        ];
        for (name, activate) in actions {
            let browser = Rc::downgrade(self);
            let action = gio::ActionEntry::builder(name)
                .activate(move |_: &adw::ApplicationWindow, _, _| {
                    if let Some(browser) = browser.upgrade() {
                        activate(&browser);
                    }
                })
                .build();
            self.window.add_action_entries([action]);
        }
    }

    fn connect_signals(self: &Rc<Self>) {
        self.address.set_icon_tooltip_text(gtk::EntryIconPosition::Secondary, Some("Install Site as App"));
        self.address.connect_icon_press(|entry, position| {
            if position == gtk::EntryIconPosition::Secondary {
                let _ = entry.activate_action("win.install-web-app", None);
            }
        });
        let browser = Rc::downgrade(self);
        self.address.connect_activate(move |entry| {
            let Some(browser) = browser.upgrade() else { return };
            let engine = browser.context.session.search_engine();
            if let (Some(destination), Some(page)) =
                (AddressBarInput::new(&entry.text()).destination(engine), browser.selected_page())
            {
                page.open(&destination);
                page.web_view().grab_focus();
            }
        });
        let browser = Rc::downgrade(self);
        self.tab_view.connect_selected_page_notify(move |_| Self::refresh(&browser));
        let browser = Rc::downgrade(self);
        self.tab_view.connect_close_page(move |tab_view, tab_page| {
            if let Some(browser) = browser.upgrade() {
                browser.pages.borrow_mut().retain(|page| !page.shows(&tab_page.child()));
            }
            tab_view.close_page_finish(tab_page, true);
            if tab_view.n_pages() == 0
                && let Some(window) = tab_view.root().and_then(|root| root.downcast::<gtk::Window>().ok())
            {
                window.close();
            }
            glib::Propagation::Stop
        });
        let browser = Rc::downgrade(self);
        self.shield.connect_toggled(move |shield| {
            let Some(browser) = browser.upgrade() else { return };
            if browser.updating_header.get() {
                return;
            }
            if let Some(page) = browser.selected_page()
                && let Some(host) = page.host()
            {
                let web_view = page.web_view().clone();
                browser.context.session.set_site_protected(&host, shield.is_active(), move || web_view.reload());
            }
            Self::refresh(&Rc::downgrade(&browser));
        });
    }

    fn add_page(self: &Rc<Self>, page: &Rc<PageView>) {
        let tab_page = self.tab_view.append(page.widget());
        tab_page.set_title("New Tab");
        self.pages.borrow_mut().push(Rc::clone(page));
        let view = page.web_view();
        let browser = Rc::downgrade(self);
        let tab = tab_page.clone();
        view.connect_title_notify(move |view| {
            tab.set_title(&view.title().filter(|title| !title.is_empty()).or_else(|| view.uri()).unwrap_or_default());
            Self::refresh(&browser);
        });
        let tab = tab_page.clone();
        view.connect_favicon_notify(move |view| tab.set_icon(view.favicon().as_ref()));
        let browser = Rc::downgrade(self);
        let tab = tab_page.clone();
        view.connect_is_loading_notify(move |view| {
            tab.set_loading(view.is_loading());
            Self::refresh(&browser);
        });
        let browser = Rc::downgrade(self);
        view.connect_uri_notify(move |_| Self::refresh(&browser));
        let browser = Rc::downgrade(self);
        view.connect_estimated_load_progress_notify(move |_| Self::refresh(&browser));
        let browser = Rc::downgrade(self);
        view.connect_zoom_level_notify(move |_| Self::refresh(&browser));
        let browser = Rc::downgrade(self);
        view.connect_create(move |view, _| {
            let browser = browser.upgrade()?;
            let related = WebView::builder().related_view(view).build();
            browser.add_page(&PageView::new(&browser.context.session, related.clone()));
            Some(related.upcast())
        });
        let browser = Rc::downgrade(self);
        let tab = tab_page.clone();
        view.connect_close(move |_| {
            if let Some(browser) = browser.upgrade() {
                browser.tab_view.close_page(&tab);
            }
        });
        self.tab_view.set_selected_page(&tab_page);
    }

    /// Mirrors the selected page into the header: address, security, navigation and protection state.
    fn refresh(browser: &Weak<Self>) {
        let Some(browser) = browser.upgrade() else { return };
        let Some(page) = browser.selected_page() else { return };
        let view = page.web_view();
        browser.updating_header.set(true);
        let address = view.uri().unwrap_or_default();
        if !browser.address.has_focus() {
            browser.address.set_text(&address);
        }
        // An error page proves nothing about the connection, so it never gets the padlock; plain HTTP stays marked,
        // except on onion services, which Tor encrypts end to end.
        let is_onion =
            Url::parse(&address).ok().and_then(|url| url.host_str().map(|host| DomainName::new(host).is_onion()));
        let security_icon = match address.split_once(':').map(|(scheme, _)| scheme) {
            Some("https") if !page.shows_error_page() => Some("channel-secure-symbolic"),
            Some("http") if is_onion == Some(true) && !page.shows_error_page() => Some("channel-secure-symbolic"),
            Some("http") => Some("channel-insecure-symbolic"),
            _ => None,
        };
        browser.address.set_icon_from_icon_name(gtk::EntryIconPosition::Primary, security_icon);
        let is_installable = security_icon.is_some() && !page.shows_error_page() && browser.context.tor.is_none();
        browser.address.set_icon_from_icon_name(
            gtk::EntryIconPosition::Secondary,
            is_installable.then_some(AppIcons::INSTALL_APP),
        );
        browser.address.set_progress_fraction(if view.is_loading() { view.estimated_load_progress() } else { 0.0 });
        browser.set_action_enabled("back", view.can_go_back());
        browser.set_action_enabled("forward", view.can_go_forward());
        browser.reload.set_icon_name(if view.is_loading() { "process-stop-symbolic" } else { "view-refresh-symbolic" });
        browser.zoom.show(&browser.window, page.zoom_level());
        let host = page.host();
        let protected = host.as_deref().is_none_or(|host| browser.context.session.is_site_protected(host));
        browser.shield.set_sensitive(host.is_some());
        browser.shield.set_active(protected);
        browser.shield.set_icon_name(if protected { "security-high-symbolic" } else { "security-low-symbolic" });
        let title = view.title().filter(|title| !title.is_empty());
        let product = browser.product_title();
        browser.window.set_title(Some(&title.map_or_else(|| product.clone(), |title| format!("{title} — {product}"))));
        browser.updating_header.set(false);
    }

    fn confirm_fire(self: &Rc<Self>) {
        let dialog = if self.context.tor.is_some() {
            let dialog = adw::AlertDialog::new(
                Some("New Identity?"),
                Some(
                    "Closes all Tor tabs and windows, deletes their cookies, site data and cache, and switches to \
                     new Tor circuits, so sites cannot link what you do next to what you did before.",
                ),
            );
            dialog.add_responses(&[("cancel", "_Cancel"), ("burn", "_New Identity")]);
            dialog
        } else {
            let dialog = adw::AlertDialog::new(
                Some("Burn Browsing Data?"),
                Some(
                    "Closes all tabs and windows of this profile and deletes its cookies, site data, cache and history.",
                ),
            );
            dialog.add_responses(&[("cancel", "_Cancel"), ("burn", "_Burn")]);
            dialog
        };
        dialog.set_response_appearance("burn", adw::ResponseAppearance::Destructive);
        dialog.set_close_response("cancel");
        let browser = Rc::downgrade(self);
        dialog.connect_response(Some("burn"), move |_, _| {
            let Some(browser) = browser.upgrade() else { return };
            if let Some(application) = browser.window.application() {
                application
                    .windows()
                    .iter()
                    .filter(|window| *window != browser.window.upcast_ref::<gtk::Window>())
                    .for_each(GtkWindowExt::close);
            }
            glib::spawn_future_local(async move {
                let result = browser.context.session.burn_website_data().await;
                let identity = match &browser.context.tor {
                    Some(tor) => tor.new_identity().await,
                    None => Ok(()),
                };
                browser.start_over();
                let message = match (result, identity) {
                    (Ok(()), Ok(())) if browser.context.tor.is_some() => "New identity".to_owned(),
                    (Ok(()), Ok(())) => "Browsing data burned".to_owned(),
                    (Err(error), _) => format!("Some data could not be deleted: {error}"),
                    (_, Err(error)) => format!("Tor could not switch circuits: {error}"),
                };
                browser.toasts.add_toast(adw::Toast::new(&message));
            });
        });
        dialog.present(Some(&self.window));
    }

    /// Once the app is installed and started, the browser hands over: the app's tabs close in every window.
    fn install_web_app(self: &Rc<Self>) {
        let Some(page) = self.selected_page() else { return };
        let context = Rc::clone(&self.context);
        let installed_from = page.web_view().clone();
        WebAppInstaller::offer(&self.context, page.web_view(), &self.window, &self.toasts, move |app| {
            for browser in context.browser_windows.open() {
                browser.close_tabs_of(app, &installed_from);
            }
        });
    }

    fn home_page(&self) -> &'static str {
        self.context.session.search_engine().home_page()
    }

    fn focus_address(self: &Rc<Self>) {
        self.address.grab_focus();
        self.address.select_region(0, -1);
    }

    fn change_zoom(&self, change: fn(ZoomLevel) -> ZoomLevel) {
        if let Some(page) = self.selected_page() {
            page.change_zoom(change);
        }
    }

    fn with_web_view(&self, action: impl FnOnce(&WebView)) {
        if let Some(page) = self.selected_page() {
            action(page.web_view());
        }
    }

    fn selected_page(&self) -> Option<Rc<PageView>> {
        let selected = self.tab_view.selected_page()?.child();
        self.pages.borrow().iter().find(|page| page.shows(&selected)).cloned()
    }

    fn set_action_enabled(&self, name: &str, enabled: bool) {
        if let Some(action) =
            self.window.lookup_action(name).and_then(|action| action.downcast::<gio::SimpleAction>().ok())
        {
            action.set_enabled(enabled);
        }
    }

    /// Covers the tabs with the Tor connection page while Tor is not connected (Tor window only).
    fn follow_tor(self: &Rc<Self>) {
        let Some(tor) = &self.context.tor else { return };
        let browser = Rc::downgrade(self);
        tor.observe(move |state| {
            let Some(browser) = browser.upgrade() else { return glib::ControlFlow::Break };
            if let Some(tor_page) = &browser.tor_page {
                tor_page.show(state);
            }
            let shown = if matches!(state, TorState::Connected { .. }) { Self::TABS } else { Self::TOR_PAGE };
            browser.content.set_visible_child_name(shown);
            glib::ControlFlow::Continue
        });
    }

    fn product_title(&self) -> String {
        let Some(profile) = self.context.session.profile_id() else {
            return format!("{} (Tor)", Self::PRODUCT);
        };
        if profile.is_default() {
            return Self::PRODUCT.to_owned();
        }
        let name = self.context.catalogs.profiles().profile(profile).map(|found| found.name.clone());
        format!("{} ({})", Self::PRODUCT, name.unwrap_or_else(|| profile.as_str().to_owned()))
    }
}
