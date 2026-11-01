//! Stripping tracking parameters (fbclid, gclid, utm_*, ...) from links, like `DuckDuckGo`'s `LinkCleaner`.

use induckgo_core::protection::privacy_configuration::PrivacyConfiguration;
use induckgo_core::protection::tracking_parameter_remover::TrackingParameterRemover;
use serde_json::json;

/// Given: `DuckDuckGo`'s configuration strips `fbclid` and `utm_source`, except on `needs-utm.com`.
struct LinkCleaningScenario {
    configuration: PrivacyConfiguration,
    user_unprotected_domains: Vec<String>,
    clicked_on_page: Option<String>,
}

impl LinkCleaningScenario {
    fn given_tracking_parameters_protection() -> Self {
        let configuration = PrivacyConfiguration::new(
            json!({"features": {"trackingParameters": {
                "state": "enabled",
                "exceptions": [{"domain": "needs-utm.com"}],
                "settings": {"parameters": ["fbclid", "utm_source"]}
            }}}),
            "1.214.0",
        );
        Self { configuration, user_unprotected_domains: vec![], clicked_on_page: None }
    }

    fn with_protection_switched_off_by_user_on(mut self, domain: &str) -> Self {
        self.user_unprotected_domains.push(domain.to_owned());
        self
    }

    fn with_link_clicked_on_page(mut self, page: &str) -> Self {
        self.clicked_on_page = Some(page.to_owned());
        self
    }

    fn when_link_is_opened(&self, url: &str) -> OpenedLink {
        let remover = TrackingParameterRemover::new(&self.configuration, &self.user_unprotected_domains);
        OpenedLink { requested: url.to_owned(), opened: remover.cleaned(url, self.clicked_on_page.as_deref()) }
    }
}

struct OpenedLink {
    requested: String,
    opened: String,
}

impl OpenedLink {
    fn then_opens(&self, expected: &str) {
        assert_eq!(self.opened, expected);
    }

    fn then_opens_unchanged(&self) {
        assert_eq!(self.opened, self.requested);
    }
}

#[test]
fn should_remove_tracking_parameter_and_keep_others_in_order() {
    LinkCleaningScenario::given_tracking_parameters_protection()
        .when_link_is_opened("https://shop.com/item?id=7&fbclid=abc&color=red")
        .then_opens("https://shop.com/item?id=7&color=red");
}

#[test]
fn should_drop_question_mark_when_only_tracking_parameters_were_present() {
    LinkCleaningScenario::given_tracking_parameters_protection()
        .when_link_is_opened("https://shop.com/?utm_source=mail&fbclid=1")
        .then_opens("https://shop.com/");
}

#[test]
fn should_keep_fragment_after_cleaning() {
    LinkCleaningScenario::given_tracking_parameters_protection()
        .when_link_is_opened("https://shop.com/a?fbclid=1#reviews")
        .then_opens("https://shop.com/a#reviews");
}

#[test]
fn should_match_parameter_names_exactly() {
    LinkCleaningScenario::given_tracking_parameters_protection()
        .when_link_is_opened("https://shop.com/?fbclid_note=1&FBCLID=2")
        .then_opens_unchanged();
}

#[test]
fn should_leave_url_untouched_on_exception_site() {
    LinkCleaningScenario::given_tracking_parameters_protection()
        .when_link_is_opened("https://www.needs-utm.com/?utm_source=x")
        .then_opens_unchanged();
}

#[test]
fn should_leave_url_untouched_when_link_comes_from_exception_site() {
    LinkCleaningScenario::given_tracking_parameters_protection()
        .with_link_clicked_on_page("https://needs-utm.com/page")
        .when_link_is_opened("https://shop.com/?fbclid=1")
        .then_opens_unchanged();
}

#[test]
fn should_leave_url_untouched_on_site_user_unprotected() {
    LinkCleaningScenario::given_tracking_parameters_protection()
        .with_protection_switched_off_by_user_on("shop.com")
        .when_link_is_opened("https://shop.com/?fbclid=1")
        .then_opens_unchanged();
}

#[test]
fn should_ignore_non_web_urls() {
    LinkCleaningScenario::given_tracking_parameters_protection()
        .when_link_is_opened("file:///home/me/page.html?fbclid=1")
        .then_opens_unchanged();
}
