//! The Fire button: what the next start deletes.

use std::fs;

use induckgo_core::profile::profile_directories::ProfileDirectories;
use induckgo_core::profile::profile_directories::ProfileLocation;
use induckgo_core::profile::profile_id::ProfileId;
use induckgo_qt::burn_restart::BurnRestart;
use tempfile::TempDir;

use induckgo_qt as _;

struct BurnScenario {
    roots: TempDir,
    location: ProfileLocation,
}

impl BurnScenario {
    fn given_a_profile_with_browsing_data_and_settings() -> Self {
        let roots = TempDir::new().expect("roots");
        let location = ProfileDirectories::new(&roots.path().join("data"), &roots.path().join("cache"))
            .for_profile(&ProfileId::default_profile());
        fs::create_dir_all(location.data.join("engine/Cookies")).expect("engine data");
        fs::create_dir_all(location.cache.join("engine")).expect("cache");
        fs::write(location.data.join("unprotected-sites.json"), b"{}").expect("settings");
        Self { roots, location }
    }

    fn burn(&self) -> BurnRestart {
        BurnRestart::new(self.location.clone())
    }
}

#[test]
fn should_keep_everything_until_a_burn_is_requested() {
    let scenario = BurnScenario::given_a_profile_with_browsing_data_and_settings();
    assert!(!scenario.burn().complete_pending().expect("complete"));
    assert!(scenario.location.data.join("engine").exists());
}

#[test]
fn should_delete_the_engine_directory_and_the_cache_after_a_request() {
    let scenario = BurnScenario::given_a_profile_with_browsing_data_and_settings();
    scenario.burn().request().expect("request");
    assert!(scenario.burn().complete_pending().expect("complete"));
    assert!(!scenario.location.data.join("engine").exists() && !scenario.location.cache.exists());
}

#[test]
fn should_keep_the_persons_settings() {
    let scenario = BurnScenario::given_a_profile_with_browsing_data_and_settings();
    scenario.burn().request().expect("request");
    scenario.burn().complete_pending().expect("complete");
    assert!(scenario.location.data.join("unprotected-sites.json").exists());
}

#[test]
fn should_delete_the_download_list_and_the_site_zoom_but_not_the_download_choices() {
    let scenario = BurnScenario::given_a_profile_with_browsing_data_and_settings();
    for file in ["downloads.json", "site-zoom-levels.json", "download-settings.json"] {
        fs::write(scenario.location.data.join(file), b"{}").expect("file");
    }
    scenario.burn().request().expect("request");
    scenario.burn().complete_pending().expect("complete");
    let remaining: Vec<bool> = ["downloads.json", "site-zoom-levels.json", "download-settings.json"]
        .iter()
        .map(|file| scenario.location.data.join(file).exists())
        .collect();
    assert_eq!(remaining, [false, false, true]);
}

#[test]
fn should_burn_only_once_per_request() {
    let scenario = BurnScenario::given_a_profile_with_browsing_data_and_settings();
    scenario.burn().request().expect("request");
    scenario.burn().complete_pending().expect("first");
    fs::create_dir_all(scenario.location.data.join("engine")).expect("new data");
    assert!(!scenario.burn().complete_pending().expect("second"));
    assert!(scenario.location.data.join("engine").exists());
    drop(scenario.roots);
}
