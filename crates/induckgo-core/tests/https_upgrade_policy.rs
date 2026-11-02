//! Upgrading plain-HTTP navigations to HTTPS, falling back once a site proves it has no HTTPS.

use induckgo_core::navigation::https_upgrade_policy::HttpsUpgradePolicy;
use induckgo_core::protection::content_blocker_rule::ContentBlockerRule;

/// Given: the privacy configuration excludes `legacy-portal.com` from HTTPS upgrades.
struct HttpsUpgradeScenario {
    policy: HttpsUpgradePolicy,
}

impl HttpsUpgradeScenario {
    fn given_https_first_browsing() -> Self {
        Self { policy: HttpsUpgradePolicy::new(&["legacy-portal.com".to_owned()]) }
    }

    fn after_https_failed_for(mut self, url: &str) -> Self {
        self.policy.record_failed_upgrade(url);
        self
    }

    /// `(action, url-filter)` of every WebKit rule, in order.
    fn when_upgrade_rules_are_built(&self) -> Vec<(String, String)> {
        let rules = self.policy.content_rules();
        rules
            .iter()
            .map(ContentBlockerRule::to_json)
            .map(|rule| (rule["action"]["type"].to_string(), rule["trigger"]["url-filter"].to_string()))
            .collect()
    }

    fn when_navigating_to(&self, url: &str) -> NavigationOutcome {
        NavigationOutcome { url: url.to_owned(), upgraded: self.policy.upgraded(url) }
    }
}

struct NavigationOutcome {
    url: String,
    upgraded: Option<String>,
}

impl NavigationOutcome {
    fn then_loads(&self, expected: &str) {
        assert_eq!(self.upgraded.as_deref(), Some(expected));
    }

    fn then_proceeds_unchanged(&self) {
        assert_eq!(self.upgraded, None, "url: {}", self.url);
    }
}

#[test]
fn should_upgrade_plain_http_navigation() {
    HttpsUpgradeScenario::given_https_first_browsing()
        .when_navigating_to("http://news.com/article?id=1")
        .then_loads("https://news.com/article?id=1");
}

#[test]
fn should_not_change_navigation_that_is_already_https() {
    HttpsUpgradeScenario::given_https_first_browsing()
        .when_navigating_to("https://news.com/")
        .then_proceeds_unchanged();
}

#[test]
fn should_not_upgrade_local_network_addresses() {
    let scenario = HttpsUpgradeScenario::given_https_first_browsing();

    for url in
        ["http://localhost:3000/", "http://192.168.0.1/", "http://[::1]/", "http://printer.local/", "http://nas/"]
    {
        scenario.when_navigating_to(url).then_proceeds_unchanged();
    }
}

#[test]
fn should_not_upgrade_site_excluded_by_privacy_configuration() {
    HttpsUpgradeScenario::given_https_first_browsing()
        .when_navigating_to("http://www.legacy-portal.com/")
        .then_proceeds_unchanged();
}

#[test]
fn should_stop_upgrading_host_after_failed_upgrade() {
    HttpsUpgradeScenario::given_https_first_browsing()
        .after_https_failed_for("https://old-blog.com/post")
        .when_navigating_to("http://old-blog.com/other")
        .then_proceeds_unchanged();
}

#[test]
fn should_upgrade_every_plain_http_request_before_it_is_sent() {
    let rules = HttpsUpgradeScenario::given_https_first_browsing().when_upgrade_rules_are_built();

    assert_eq!(rules[0], (r#""make-https""#.to_owned(), r#""^http://""#.to_owned()));
}

#[test]
fn should_keep_local_network_and_excluded_hosts_on_plain_http() {
    let rules = HttpsUpgradeScenario::given_https_first_browsing()
        .after_https_failed_for("https://old-blog.com/post")
        .when_upgrade_rules_are_built();
    let exceptions: Vec<&str> = rules[1..].iter().map(|(_, filter)| filter.as_str()).collect();

    assert_eq!(
        exceptions,
        [
            r#""^http://[0-9.]+([:/?#].*)?$""#,
            r#""^http://\\[""#,
            r#""^http://[^/:?#.]+([:/?#].*)?$""#,
            r#""^[^:]+://+([^:/]+\\.)?localhost[:/]""#,
            r#""^[^:]+://+([^:/]+\\.)?local[:/]""#,
            r#""^[^:]+://+([^:/]+\\.)?lan[:/]""#,
            r#""^[^:]+://+([^:/]+\\.)?internal[:/]""#,
            r#""^[^:]+://+([^:/]+\\.)?home\\.arpa[:/]""#,
            r#""^[^:]+://+([^:/]+\\.)?legacy-portal\\.com[:/]""#,
            r#""^[^:]+://+old-blog\\.com[:/]""#,
        ]
    );
}

#[test]
fn should_express_every_exception_as_ignore_previous_rules() {
    let rules = HttpsUpgradeScenario::given_https_first_browsing().when_upgrade_rules_are_built();

    assert!(rules[1..].iter().all(|(action, _)| action == r#""ignore-previous-rules""#));
}

#[test]
fn should_remember_failed_hosts_when_configuration_is_refreshed() {
    let mut scenario =
        HttpsUpgradeScenario::given_https_first_browsing().after_https_failed_for("https://old-blog.com/");

    scenario.policy.set_excluded_domains(&[]);

    scenario.when_navigating_to("http://old-blog.com/").then_proceeds_unchanged();
}
