//! The window of an installed web app: one page, no tabs, no address bar, and only its own site.
//!
//! A link the user clicks to another site opens in the browser, so the app never becomes a general browser
//! window holding the app's cookies. Redirects and script navigations stay, since sign-in flows rely on
//! them (Gmail through `accounts.google.com`, banks through 3-D Secure pages). Pop-up windows the app opens
//! (OAuth, payment confirmation) open in a small window of the app and keep their link to the opener.

use std::rc::Rc;

use adw::prelude::AdwApplicationWindowExt;
use adw::prelude::AdwDialogExt;
use adw::prelude::AlertDialogExt;
use adw::prelude::AlertDialogExtManual;
use gio::prelude::ActionMapExtManual;
use gtk::prelude::Cast;
use gtk::prelude::GtkWindowExt;
use gtk::prelude::WidgetExt;
use induckgo_core::web_app::installed_web_app::InstalledWebApp;
use induckgo_core::web_app::web_app_navigation_policy::NavigationTrigger;
use induckgo_core::web_app::web_app_navigation_policy::WebAppNavigation;
use induckgo_core::web_app::web_app_navigation_policy::WebAppNavigationPolicy;
use webkit6::NavigationPolicyDecision;
use webkit6::NavigationType;
use webkit6::PolicyDecision;
use webkit6::PolicyDecisionType;
use webkit6::WebView;
use webkit6::prelude::PolicyDecisionExt;
use webkit6::prelude::WebViewExt;

use crate::app_context::AppContext;
use crate::external_launcher::ExternalLauncher;
use crate::page_view::PageView;
use crate::public_suffix_list::PublicSuffixList;

/// A window action: its name and what it does.
type WebAppCommand = (&'static str, fn(&Rc<WebAppWindow>));

pub struct WebAppWindow {
    context: Rc<AppContext>,
    app: InstalledWebApp,
    window: adw::ApplicationWindow,
    page: Rc<PageView>,
    toasts: adw::ToastOverlay,
}

impl WebAppWindow {
    pub fn open(application: &adw::Application, context: &Rc<AppContext>, app: InstalledWebApp) -> Rc<Self> {
        let page = PageView::new(&context.session, context.session.new_web_view());
        let web_app = Rc::new(Self {
            context: Rc::clone(context),
            window: adw::ApplicationWindow::builder()
                .application(application)
                .title(&app.name)
                .default_width(1100)
                .default_height(800)
                .build(),
            app,
            page,
            toasts: adw::ToastOverlay::new(),
        });
        web_app.build_layout();
        web_app.install_actions();
        Self::keep_to_site(web_app.page.web_view(), web_app.app.start_url.clone());
        web_app.open_pop_ups_in_app();
        web_app.page.open(&web_app.app.start_url);
        let keep_alive = Rc::clone(&web_app);
        web_app.window.connect_destroy(move |_| {
            let _ = &keep_alive;
        });
        web_app.window.present();
        web_app
    }

    fn build_layout(&self) {
        let title = adw::WindowTitle::new(&self.app.name, &self.app.host());
        let header = adw::HeaderBar::builder().title_widget(&title).build();
        header.pack_start(
            &gtk::Button::builder()
                .icon_name("go-previous-symbolic")
                .action_name("win.back")
                .tooltip_text("Back")
                .build(),
        );
        header.pack_start(
            &gtk::Button::builder()
                .icon_name("view-refresh-symbolic")
                .action_name("win.reload")
                .tooltip_text("Reload")
                .build(),
        );
        let menu = gio::Menu::new();
        menu.append(Some("Open Page in Browser"), Some("win.open-in-browser"));
        menu.append(Some("Clear App Data…"), Some("win.fire"));
        menu.append(Some("About InDuckGo"), Some("app.about"));
        header.pack_end(&gtk::MenuButton::builder().icon_name("open-menu-symbolic").menu_model(&menu).build());
        Self::show_current_site(&self.page, &title, &self.app);
        let name = self.app.name.clone();
        self.page.web_view().connect_title_notify(move |view| {
            title.set_title(
                &view.title().filter(|page_title| !page_title.is_empty()).map_or_else(|| name.clone(), String::from),
            );
        });
        let toolbar = adw::ToolbarView::new();
        toolbar.add_top_bar(&header);
        toolbar.set_content(Some(self.page.widget()));
        self.toasts.set_child(Some(&toolbar));
        self.window.set_content(Some(&self.toasts));
    }

    /// The subtitle names the host of the page on screen, flagged when it is not on the app's own site.
    fn show_current_site(page: &Rc<PageView>, title: &adw::WindowTitle, app: &InstalledWebApp) {
        let shown = Rc::downgrade(page);
        let (title, start_url, name) = (title.clone(), app.start_url.clone(), app.name.clone());
        page.web_view().connect_uri_notify(move |view| {
            let (Some(page), Some(address)) = (shown.upgrade(), view.uri()) else { return };
            let Some(host) = page.host() else { return };
            title.set_subtitle(&host);
            if WebAppNavigationPolicy::new(&start_url, &PublicSuffixList).is_app_page(&address) {
                title.remove_css_class("warning");
                title.set_tooltip_text(None);
            } else {
                title.add_css_class("warning");
                title.set_tooltip_text(Some(&format!("This page is on another site, not part of {name}")));
            }
        });
    }

    fn install_actions(self: &Rc<Self>) {
        let actions: [WebAppCommand; 4] = [
            ("back", |web_app| web_app.page.web_view().go_back()),
            ("reload", |web_app| web_app.page.web_view().reload()),
            ("open-in-browser", |web_app| {
                if let Some(address) = web_app.page.web_view().uri() {
                    ExternalLauncher::open_in_browser(&address);
                }
            }),
            ("fire", Self::confirm_clear),
        ];
        for (name, activate) in actions {
            let web_app = Rc::downgrade(self);
            let action = gio::ActionEntry::builder(name)
                .activate(move |_: &adw::ApplicationWindow, _, _| {
                    if let Some(web_app) = web_app.upgrade() {
                        activate(&web_app);
                    }
                })
                .build();
            self.window.add_action_entries([action]);
        }
    }

    /// A clicked link to a site other than `site_url`'s opens in the browser; see [`WebAppNavigationPolicy`].
    fn keep_to_site(web_view: &WebView, site_url: String) {
        web_view.connect_decide_policy(move |_, decision, kind| Self::decide(&site_url, decision, kind));
    }

    fn decide(site_url: &str, decision: &PolicyDecision, kind: PolicyDecisionType) -> bool {
        if !matches!(kind, PolicyDecisionType::NavigationAction | PolicyDecisionType::NewWindowAction) {
            return false;
        }
        let Some(navigation) = decision.downcast_ref::<NavigationPolicyDecision>() else { return false };
        let Some(action) = navigation.navigation_action() else { return false };
        let Some(target) = action.request().and_then(|request| request.uri()) else { return false };
        let trigger = if action.navigation_type() == NavigationType::LinkClicked {
            NavigationTrigger::LinkClicked
        } else {
            NavigationTrigger::RedirectOrScript
        };
        match WebAppNavigationPolicy::new(site_url, &PublicSuffixList).decide(&target, trigger) {
            WebAppNavigation::StayInWebApp => false,
            WebAppNavigation::OpenInBrowser => {
                decision.ignore();
                ExternalLauncher::open_in_browser(&target);
                true
            }
        }
    }

    /// Pop-ups share the app's session, so they get the same gate and site indicator as the main window. A
    /// pop-up keeps to the site it was opened for (a sign-in or 3-D Secure page), whose own links must work.
    fn open_pop_ups_in_app(self: &Rc<Self>) {
        let web_app = Rc::downgrade(self);
        self.page.web_view().connect_create(move |opener, action| {
            let web_app = web_app.upgrade()?;
            let pop_up = WebView::builder().related_view(opener).build();
            let page = PageView::new(&web_app.context.session, pop_up.clone());
            let opened_for = action
                .request()
                .and_then(|request| request.uri())
                .map(String::from)
                .filter(|address| address.starts_with("http://") || address.starts_with("https://"))
                .unwrap_or_else(|| web_app.app.start_url.clone());
            Self::keep_to_site(&pop_up, opened_for);
            let title = adw::WindowTitle::new("", "");
            Self::show_current_site(&page, &title, &web_app.app);
            let header = adw::HeaderBar::builder().title_widget(&title).build();
            let toolbar = adw::ToolbarView::new();
            toolbar.add_top_bar(&header);
            toolbar.set_content(Some(page.widget()));
            let window = adw::Window::builder()
                .transient_for(&web_app.window)
                .default_width(560)
                .default_height(720)
                .content(&toolbar)
                .build();
            pop_up.connect_title_notify(glib::clone!(
                #[weak]
                window,
                #[weak]
                title,
                move |view| {
                    window.set_title(view.title().as_deref());
                    title.set_title(view.title().as_deref().unwrap_or_default());
                }
            ));
            pop_up.connect_close(glib::clone!(
                #[weak]
                window,
                move |_| window.close()
            ));
            window.connect_destroy(move |_| {
                let _ = &page;
            });
            window.present();
            Some(pop_up.upcast())
        });
    }

    fn confirm_clear(self: &Rc<Self>) {
        let dialog = adw::AlertDialog::new(
            Some(&format!("Clear Data of {}?", self.app.name)),
            Some("Signs you out and deletes this app's cookies, site data and cache. Nothing else is affected."),
        );
        dialog.add_responses(&[("cancel", "_Cancel"), ("clear", "C_lear")]);
        dialog.set_response_appearance("clear", adw::ResponseAppearance::Destructive);
        dialog.set_close_response("cancel");
        let web_app = Rc::downgrade(self);
        dialog.connect_response(Some("clear"), move |_, _| {
            let Some(web_app) = web_app.upgrade() else { return };
            glib::spawn_future_local(async move {
                let message = match web_app.context.session.burn_website_data().await {
                    Ok(()) => "App data cleared".to_owned(),
                    Err(error) => format!("Some data could not be deleted: {error}"),
                };
                web_app.page.web_view().load_uri(&web_app.app.start_url);
                web_app.toasts.add_toast(adw::Toast::new(&message));
            });
        });
        dialog.present(Some(&self.window));
    }
}
