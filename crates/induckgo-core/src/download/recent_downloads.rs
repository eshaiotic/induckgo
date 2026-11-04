//! The few most recent downloads of a profile, newest first, as its downloads list shows them.
//!
//! Only finished downloads outlive the process (`downloads.json`): a download that was running or failed
//! when the browser closed has no file worth offering.

use std::path::Path;
use std::path::PathBuf;

use serde_json::Value;
use serde_json::json;

/// Names one download while it runs; never reused within a process.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DownloadId(u64);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DownloadState {
    /// `total` is `None` when the server did not say how large the file is.
    InProgress {
        received: u64,
        total: Option<u64>,
    },
    Finished,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecentDownload {
    id: DownloadId,
    path: PathBuf,
    state: DownloadState,
}

impl RecentDownload {
    #[must_use]
    pub fn id(&self) -> DownloadId {
        self.id
    }

    /// The file's full path.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// The file's name, as the list shows it.
    #[must_use]
    pub fn name(&self) -> &str {
        self.path.file_name().and_then(|name| name.to_str()).unwrap_or_default()
    }

    #[must_use]
    pub fn state(&self) -> &DownloadState {
        &self.state
    }
}

#[derive(Debug, Clone, Default)]
pub struct RecentDownloads {
    entries: Vec<RecentDownload>,
    next_id: u64,
}

impl RecentDownloads {
    /// How many downloads the list keeps.
    pub const CAPACITY: usize = 5;

    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Reads the stored document: every entry with an absolute path is a finished download.
    #[must_use]
    pub fn from_json(document: &Value) -> Self {
        let mut downloads = Self::new();
        let stored = document.get("downloads").and_then(Value::as_array).map(Vec::as_slice).unwrap_or_default();
        for entry in stored.iter().rev() {
            let path = entry.get("path").and_then(Value::as_str).map(PathBuf::from).filter(|path| path.is_absolute());
            if let Some(path) = path {
                let id = downloads.begin(path);
                downloads.finish(id);
            }
        }
        downloads
    }

    /// The finished downloads only, newest first.
    #[must_use]
    pub fn to_json(&self) -> Value {
        let finished: Vec<Value> = self
            .entries
            .iter()
            .filter(|entry| entry.state == DownloadState::Finished)
            .map(|entry| json!({"path": entry.path}))
            .collect();
        json!({"downloads": finished})
    }

    /// Newest first.
    #[must_use]
    pub fn entries(&self) -> &[RecentDownload] {
        &self.entries
    }

    /// Adds a download that has just started as the newest, dropping the oldest beyond [`Self::CAPACITY`].
    pub fn begin(&mut self, path: PathBuf) -> DownloadId {
        let id = DownloadId(self.next_id);
        self.next_id += 1;
        self.entries
            .insert(0, RecentDownload { id, path, state: DownloadState::InProgress { received: 0, total: None } });
        self.entries.truncate(Self::CAPACITY);
        id
    }

    pub fn progress(&mut self, id: DownloadId, received: u64, total: Option<u64>) {
        self.set_state(id, DownloadState::InProgress { received, total });
    }

    pub fn finish(&mut self, id: DownloadId) {
        self.set_state(id, DownloadState::Finished);
    }

    pub fn fail(&mut self, id: DownloadId) {
        self.set_state(id, DownloadState::Failed);
    }

    /// Forgets a download, for one the person cancelled.
    pub fn discard(&mut self, id: DownloadId) {
        self.entries.retain(|entry| entry.id != id);
    }

    fn set_state(&mut self, id: DownloadId, state: DownloadState) {
        if let Some(entry) = self.entries.iter_mut().find(|entry| entry.id == id) {
            entry.state = state;
        }
    }
}
