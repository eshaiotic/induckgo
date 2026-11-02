//! One web page on screen, with the behaviour every InDuckGo page shares, in a tab or in a web app window.
//!
//! * Tracking parameters: the `links` rule list stops a top-frame navigation that carries one before its
//!   request leaves the machine; WebKit reports it as a failed load, and the page opens the cleaned address.
//! * HTTPS upgrades: when an upgraded load fails, a banner explains that the site has no secure version and
//!   offers to load it insecurely (never silently, so an attacker cannot force a downgrade).
//! * Links to other applications (`mailto:`, `tel:`, ...) open outside, and only after a real click.
//! * Permission requests are asked, never granted silently.

use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;
use std::rc::Weak;

use adw::prelude::WidgetExt;
use gtk::prelude::BoxExt;
use gtk::prelude::Cast;
use gtk::prelude::IsA;
use url::Url;
use webkit6::LoadEvent;
use webkit6::NavigationPolicyDecision;
use webkit6::NetworkError;
use webkit6::PolicyDecision;
use webkit6::PolicyDecisionType;
use webkit6::PolicyError;
use webkit6::WebView;
use webkit6::prelude::PolicyDecisionExt;
use webkit6::prelude::WebViewExt;

use crate::external_launcher::ExternalLauncher;
use crate::permission_prompt::PermissionPrompt;
use crate::profile_session::ProfileSession;

pub struct PageView {
    session: Rc<ProfileSession>,
    web_view: WebView,
    root: gtk::Box,
    insecure_banner: adw::Banner,
    /// Recent plain-HTTP navigation targets, to recognise the failure of a load the HTTPS rules upgraded.
    recent_plain_http: RefCell<VecDeque<String>>,
    /// `(https, http)` of the failed upgrade the banner offers to load insecurely.
    insecure_fallback: RefCell<Option<(String, String)>>,
    /// The last address this page cleaned, so a rule/cleaner mismatch can never loop.
    last_cleaned: RefCell<Option<String>>,
}

impl PageView {
    const RECENT_NAVIGATIONS: usize = 16;
    /// Schemes WebKit displays itself; every other scheme belongs to another application.
    const WEB_SCHEMES: [&'static str; 7] = ["http", "https", "about", "data", "blob", "file", "webkit-pdfjs-viewer"];

    pub fn new(session: &Rc<ProfileSession>, web_view: WebView) -> Rc<Self> {
        let insecure_banner = adw::Banner::builder().button_label("Load Insecure Page").revealed(false).build();
        let root = gtk::Box::new(gtk::Orientation::Vertical, 0);
        web_view.set_vexpand(true);
        root.append(&insecure_banner);
        root.append(&web_view);
        let page = Rc::new(Self {
            session: Rc::clone(session),
            web_view,
            root,
            insecure_banner,
            recent_plain_http: RefCell::new(VecDeque::new()),
            insecure_fallback: RefCell::new(None),
            last_cleaned: RefCell::new(None),
        });
        page.connect_signals();
        page
    }

    pub fn web_view(&self) -> &WebView {
        &self.web_view
    }

    pub fn widget(&self) -> &gtk::Widget {
        self.root.upcast_ref()
    }

    /// Loads `address` as soon as tracker blocking is active.
    pub fn open(&self, address: &str) {
        let web_view = self.web_view.clone();
        let address = address.to_owned();
        self.session.when_protected(move || web_view.load_uri(&address));
    }

    /// Host of the current page, if it is a web page.
    pub fn host(&self) -> Option<String> {
        let address = self.web_view.uri()?;
        let url = Url::parse(&address).ok().filter(|url| matches!(url.scheme(), "http" | "https"))?;
        url.host_str().map(str::to_owned)
    }

    fn connect_signals(self: &Rc<Self>) {
        let page = Rc::downgrade(self);
        self.web_view.connect_decide_policy(move |_, decision, kind| {
            page.upgrade().is_some_and(|page| page.decide_navigation(decision, kind))
        });
        let page = Rc::downgrade(self);
        self.web_view.connect_load_failed(move |_, _, failing_address, error| {
            page.upgrade().is_some_and(|page| page.recover_from_failed_load(failing_address, error))
        });
        let page = Rc::downgrade(self);
        self.web_view.connect_load_changed(move |_, event| {
            if event == LoadEvent::Started
                && let Some(page) = page.upgrade()
            {
                page.insecure_banner.set_revealed(false);
            }
        });
        let page = Rc::downgrade(self);
        self.insecure_banner.connect_button_clicked(move |_| Self::load_insecurely(&page));
        self.web_view.connect_permission_request(PermissionPrompt::ask);
    }

    /// Remembers plain-HTTP targets and sends links to other applications outside; WebKit decides the rest.
    fn decide_navigation(&self, decision: &PolicyDecision, kind: PolicyDecisionType) -> bool {
        if !matches!(kind, PolicyDecisionType::NavigationAction | PolicyDecisionType::NewWindowAction) {
            return false;
        }
        let Some(navigation) = decision.downcast_ref::<NavigationPolicyDecision>() else { return false };
        let Some(action) = navigation.navigation_action() else { return false };
        let Some(address) = action.request().and_then(|request| request.uri()) else { return false };
        let scheme = address.split_once(':').map(|(scheme, _)| scheme.to_ascii_lowercase()).unwrap_or_default();
        if scheme == "http" {
            let mut recent = self.recent_plain_http.borrow_mut();
            recent.push_back(address.to_string());
            if recent.len() > Self::RECENT_NAVIGATIONS {
                recent.pop_front();
            }
        }
        if Self::WEB_SCHEMES.contains(&scheme.as_str()) {
            return false;
        }
        decision.ignore();
        if action.is_user_gesture() {
            ExternalLauncher::open_outside(&address, self.window().as_ref());
        }
        true
    }

    fn recover_from_failed_load(&self, failing_address: &str, error: &glib::Error) -> bool {
        if error.matches(NetworkError::Cancelled) || error.matches(PolicyError::FrameLoadInterruptedByPolicyChange) {
            return false;
        }
        if error.kind::<PolicyError>().is_some()
            && self.last_cleaned.borrow().as_deref() != Some(failing_address)
            && let Some(cleaned) = self.session.cleaned_address(failing_address)
        {
            self.last_cleaned.replace(Some(cleaned.clone()));
            self.web_view.load_uri(&cleaned);
            return true;
        }
        let upgraded_from = self
            .recent_plain_http
            .borrow()
            .iter()
            .find(|plain| self.session.was_upgraded(plain, failing_address))
            .cloned();
        if let Some(plain_http) = upgraded_from {
            let host =
                Url::parse(&plain_http).ok().and_then(|url| url.host_str().map(str::to_owned)).unwrap_or_default();
            self.insecure_banner.set_title(&format!("{host} does not offer a secure connection"));
            self.insecure_banner.set_revealed(true);
            self.insecure_fallback.replace(Some((failing_address.to_owned(), plain_http)));
        }
        false
    }

    fn load_insecurely(page: &Weak<Self>) {
        let Some(page) = page.upgrade() else { return };
        let Some((https, http)) = page.insecure_fallback.take() else { return };
        page.insecure_banner.set_revealed(false);
        let web_view = page.web_view.clone();
        page.session.stop_upgrading(&https, move || web_view.load_uri(&http));
    }

    fn window(&self) -> Option<gtk::Window> {
        self.web_view.root().and_then(|root| root.downcast::<gtk::Window>().ok())
    }

    /// Whether `widget` is this page's content (tab pages hold [`PageView::widget`]).
    pub fn shows(&self, widget: &impl IsA<gtk::Widget>) -> bool {
        self.root.upcast_ref::<gtk::Widget>() == widget.upcast_ref::<gtk::Widget>()
    }
}
