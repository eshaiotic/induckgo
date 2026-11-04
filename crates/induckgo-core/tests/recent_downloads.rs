//! The most recent downloads a profile shows in its downloads list.

use std::path::Path;

use induckgo_core::download::recent_downloads::DownloadId;
use induckgo_core::download::recent_downloads::DownloadState;
use induckgo_core::download::recent_downloads::RecentDownload;
use induckgo_core::download::recent_downloads::RecentDownloads;
use serde_json::Value;
use serde_json::json;

struct RecentDownloadsScenario {
    downloads: RecentDownloads,
    last: Option<DownloadId>,
}

impl RecentDownloadsScenario {
    fn given_nothing_downloaded() -> Self {
        Self { downloads: RecentDownloads::new(), last: None }
    }

    fn given_stored(document: &Value) -> Self {
        Self { downloads: RecentDownloads::from_json(document), last: None }
    }

    fn when_started(mut self, path: &str) -> Self {
        self.last = Some(self.downloads.begin(Path::new(path).to_path_buf()));
        self
    }

    fn when_finished(mut self) -> Self {
        self.downloads.finish(self.last.expect("a download was started"));
        self
    }

    fn when_failed(mut self) -> Self {
        self.downloads.fail(self.last.expect("a download was started"));
        self
    }

    fn when_cancelled(mut self) -> Self {
        self.downloads.discard(self.last.expect("a download was started"));
        self
    }

    fn when_received(mut self, received: u64, total: Option<u64>) -> Self {
        self.downloads.progress(self.last.expect("a download was started"), received, total);
        self
    }

    fn after_saving_and_loading(self) -> Self {
        Self { downloads: RecentDownloads::from_json(&self.downloads.to_json()), last: None }
    }

    fn names(&self) -> Vec<String> {
        self.downloads.entries().iter().map(|entry| entry.name().to_owned()).collect()
    }

    fn then_lists(&self, expected: &[&str]) {
        assert_eq!(self.names(), expected);
    }

    fn then_newest_is(&self, expected: &DownloadState) {
        assert_eq!(self.downloads.entries().first().map(RecentDownload::state), Some(expected));
    }
}

#[test]
fn should_list_the_newest_download_first() {
    RecentDownloadsScenario::given_nothing_downloaded()
        .when_started("/d/a.zip")
        .when_started("/d/b.zip")
        .then_lists(&["b.zip", "a.zip"]);
}

#[test]
fn should_keep_only_the_five_most_recent_downloads() {
    let mut scenario = RecentDownloadsScenario::given_nothing_downloaded();
    for name in ["1", "2", "3", "4", "5", "6", "7"] {
        scenario = scenario.when_started(&format!("/d/{name}.bin"));
    }
    scenario.then_lists(&["7.bin", "6.bin", "5.bin", "4.bin", "3.bin"]);
}

#[test]
fn should_track_how_much_of_a_download_has_arrived() {
    RecentDownloadsScenario::given_nothing_downloaded()
        .when_started("/d/a.iso")
        .when_received(512, Some(2048))
        .then_newest_is(&DownloadState::InProgress { received: 512, total: Some(2048) });
}

#[test]
fn should_mark_a_download_finished() {
    RecentDownloadsScenario::given_nothing_downloaded()
        .when_started("/d/a.iso")
        .when_finished()
        .then_newest_is(&DownloadState::Finished);
}

#[test]
fn should_mark_a_download_failed() {
    RecentDownloadsScenario::given_nothing_downloaded()
        .when_started("/d/a.iso")
        .when_failed()
        .then_newest_is(&DownloadState::Failed);
}

#[test]
fn should_drop_a_download_the_person_cancelled() {
    RecentDownloadsScenario::given_nothing_downloaded().when_started("/d/a.iso").when_cancelled().then_lists(&[]);
}

#[test]
fn should_keep_only_finished_downloads_across_restarts() {
    RecentDownloadsScenario::given_nothing_downloaded()
        .when_started("/d/done.pdf")
        .when_finished()
        .when_started("/d/broken.pdf")
        .when_failed()
        .when_started("/d/halfway.pdf")
        .after_saving_and_loading()
        .then_lists(&["done.pdf"]);
}

#[test]
fn should_restore_stored_downloads_as_finished_and_in_the_same_order() {
    RecentDownloadsScenario::given_stored(&json!({"downloads": [{"path": "/d/new.txt"}, {"path": "/d/old.txt"}]}))
        .then_lists(&["new.txt", "old.txt"]);
}

#[test]
fn should_ignore_stored_entries_without_an_absolute_path() {
    RecentDownloadsScenario::given_stored(
        &json!({"downloads": [{"path": "rel/a.txt"}, {"name": "x"}, 5, {"path": "/d/ok.txt"}]}),
    )
    .then_lists(&["ok.txt"]);
}
