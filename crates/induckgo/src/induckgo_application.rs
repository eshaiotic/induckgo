//! The GTK application of one InDuckGo process.
//!
//! The command line decides the application id ([`LaunchRequest`]): the default profile, another profile or
//! one web app. GTK makes the first process with an id the primary one; later launches with the same id
//! pass their command line to it over D-Bus and exit. So `induckgo https://example.com` opens a tab in the
//! running browser, and clicking a web app's icon twice shows its existing window.

use std::cell::OnceCell;
use std::cell::RefCell;
use std::rc::Rc;
use std::rc::Weak;

use adw::prelude::AdwDialogExt;
use gio::prelude::ActionMapExtManual;
use gio::prelude::ApplicationCommandLineExt;
use gio::prelude::ApplicationExt;
use gio::prelude::ApplicationExtManual;
use gtk::prelude::Cast;
use gtk::prelude::GtkApplicationExt;
use gtk::prelude::GtkWindowExt;
use induckgo_core::launch::launch_request::LaunchRequest;
use induckgo_core::profile::profile_id::ProfileId;

use crate::app_context::AppContext;
use crate::app_icons::AppIcons;
use crate::browser_window::BrowserWindow;
use crate::catalog_files::CatalogFiles;
use crate::external_launcher::ExternalLauncher;
use crate::http_client::HttpClient;
use crate::installation_paths::InstallationPaths;
use crate::profile_session::ProfileSession;
use crate::web_app_window::WebAppWindow;

pub struct InDuckGoApplication {
    profile: ProfileId,
    context: OnceCell<Rc<AppContext>>,
    browser_windows: RefCell<Vec<Weak<BrowserWindow>>>,
}

impl InDuckGoApplication {
    const USAGE_ERROR: u8 = 2;
    const NOT_FOUND: u8 = 1;
    const ACCELERATORS: [(&'static str, &'static [&'static str]); 13] = [
        ("win.new-tab", &["<Control>t"]),
        ("win.close-tab", &["<Control>w"]),
        ("app.new-window", &["<Control>n"]),
        ("win.focus-address", &["<Control>l", "F6"]),
        ("win.reload", &["<Control>r", "F5"]),
        ("win.back", &["<Alt>Left"]),
        ("win.forward", &["<Alt>Right"]),
        ("win.find", &["<Control>f"]),
        ("win.zoom-in", &["<Control>plus", "<Control>equal"]),
        ("win.zoom-out", &["<Control>minus"]),
        ("win.zoom-reset", &["<Control>0"]),
        ("win.fire", &["<Control><Shift>Delete"]),
        ("app.quit", &["<Control>q"]),
    ];

    /// Runs the process; the exit code is 2 for a malformed command line.
    pub fn run() -> glib::ExitCode {
        let arguments: Vec<String> =
            std::env::args_os().map(|argument| argument.to_string_lossy().into_owned()).collect();
        let request = match LaunchRequest::parse(arguments.get(1..).unwrap_or_default()) {
            Ok(request) => request,
            Err(error) => {
                eprintln!("induckgo: {error}");
                return glib::ExitCode::from(Self::USAGE_ERROR);
            }
        };
        let application = adw::Application::builder()
            .application_id(request.application_id().as_str())
            .flags(gio::ApplicationFlags::HANDLES_COMMAND_LINE)
            .build();
        let process = Rc::new(Self {
            profile: request.profile_id(),
            context: OnceCell::new(),
            browser_windows: RefCell::new(Vec::new()),
        });
        let starting = Rc::clone(&process);
        application.connect_startup(move |application| starting.start(application));
        application.connect_command_line(move |application, command_line| process.handle(application, command_line));
        application.run_with_args(&arguments)
    }

    fn start(self: &Rc<Self>, application: &adw::Application) {
        Self::switch_off_animations();
        let paths = InstallationPaths::from_environment();
        let http = Rc::new(HttpClient::new());
        let icons = AppIcons::new(&paths);
        icons.install();
        let session = ProfileSession::start(self.profile.clone(), &paths, Rc::clone(&http));
        let catalogs = CatalogFiles::new(&paths);
        let _ = self.context.set(Rc::new(AppContext { paths, http, catalogs, icons, session }));
        for (action, accelerators) in Self::ACCELERATORS {
            application.set_accels_for_action(action, accelerators);
        }
        self.install_actions(application);
    }

    /// Tabs, dialogs, banners, toasts and the find bar appear at once instead of sliding and fading: the
    /// interface reacts immediately, and on a busy machine animations are the first thing to stutter. Set by
    /// the application, this outranks the desktop's own animation setting for InDuckGo's processes.
    fn switch_off_animations() {
        if let Some(settings) = gtk::Settings::default() {
            settings.set_gtk_enable_animations(false);
        }
    }

    fn handle(
        self: &Rc<Self>,
        application: &adw::Application,
        command_line: &gio::ApplicationCommandLine,
    ) -> glib::ExitCode {
        let arguments: Vec<String> =
            command_line.arguments().iter().skip(1).map(|argument| argument.to_string_lossy().into_owned()).collect();
        let request = match LaunchRequest::parse(&arguments) {
            Ok(request) => request,
            Err(error) => {
                command_line.printerr_literal(&format!("induckgo: {error}\n"));
                return glib::ExitCode::from(Self::USAGE_ERROR);
            }
        };
        let Some(context) = self.context.get() else { return glib::ExitCode::FAILURE };
        if let Some(web_app_id) = request.web_app_id() {
            if let Some(window) = application.active_window() {
                window.present();
                return glib::ExitCode::SUCCESS;
            }
            let Some(installed) = context.catalogs.web_apps().web_app(web_app_id).cloned() else {
                command_line.printerr_literal(&format!("induckgo: web app {web_app_id} is not installed\n"));
                return glib::ExitCode::from(Self::NOT_FOUND);
            };
            WebAppWindow::open(application, context, installed);
            return glib::ExitCode::SUCCESS;
        }
        if !self.profile.is_default() && context.catalogs.profiles().profile(&self.profile).is_none() {
            command_line.printerr_literal(&format!("induckgo: there is no profile {}\n", self.profile));
            return glib::ExitCode::from(Self::NOT_FOUND);
        }
        match self.active_browser_window(application).filter(|_| !request.addresses().is_empty()) {
            Some(browser) => {
                request.addresses().iter().for_each(|address| browser.open_tab(address));
                browser.window().present();
            }
            None => self.open_browser_window(application, context, request.addresses()),
        }
        glib::ExitCode::SUCCESS
    }

    fn install_actions(self: &Rc<Self>, application: &adw::Application) {
        let process = Rc::downgrade(self);
        let new_window = gio::ActionEntry::builder("new-window")
            .activate(move |application: &adw::Application, _, _| {
                if let Some(process) = process.upgrade()
                    && let Some(context) = process.context.get()
                {
                    process.open_browser_window(application, context, &[]);
                }
            })
            .build();
        let process = Rc::downgrade(self);
        let open_profile = gio::ActionEntry::builder("open-profile")
            .parameter_type(Some(glib::VariantTy::STRING))
            .activate(move |application: &adw::Application, _, parameter| {
                let Some(process) = process.upgrade() else { return };
                let Some(profile) = parameter.and_then(|value| value.str()).and_then(|id| ProfileId::parse(id).ok())
                else {
                    return;
                };
                match process.context.get() {
                    Some(context) if profile == process.profile => {
                        process.open_browser_window(application, context, &[]);
                    }
                    _ => ExternalLauncher::start_profile(&profile),
                }
            })
            .build();
        let about = gio::ActionEntry::builder("about")
            .activate(|application: &adw::Application, _, _| Self::present_about(application))
            .build();
        let quit = gio::ActionEntry::builder("quit")
            .activate(|application: &adw::Application, _, _| application.quit())
            .build();
        application.add_action_entries([new_window, open_profile, about, quit]);
    }

    fn open_browser_window(&self, application: &adw::Application, context: &Rc<AppContext>, addresses: &[String]) {
        let browser = BrowserWindow::open(application, context, addresses);
        let mut windows = self.browser_windows.borrow_mut();
        windows.retain(|window| window.strong_count() > 0);
        windows.push(Rc::downgrade(&browser));
    }

    fn active_browser_window(&self, application: &adw::Application) -> Option<Rc<BrowserWindow>> {
        let active = application.active_window()?;
        self.browser_windows
            .borrow()
            .iter()
            .filter_map(Weak::upgrade)
            .find(|browser| browser.window().upcast_ref::<gtk::Window>() == &active)
    }

    fn present_about(application: &adw::Application) {
        let about = adw::AboutDialog::builder()
            .application_name("InDuckGo")
            .application_icon(AppIcons::APPLICATION)
            .version(env!("CARGO_PKG_VERSION"))
            .license_type(gtk::License::Apache20)
            .comments(
                "A privacy-first browser for Linux built on DuckDuckGo's open-source protections. \
                 Not affiliated with or endorsed by DuckDuckGo.",
            )
            .build();
        about.add_legal_section(
            "DuckDuckGo protections",
            Some("© DuckDuckGo, Inc."),
            gtk::License::Apache20,
            Some("Content Scope Scripts, the privacy configuration and the TrackerRadarKit rule algorithm."),
        );
        about.add_legal_section(
            "Tracker Data Set",
            Some("© DuckDuckGo, Inc."),
            gtk::License::Custom,
            Some("Downloaded at run time from DuckDuckGo; licensed CC BY-NC-SA 4.0."),
        );
        about.present(application.active_window().as_ref());
    }
}
