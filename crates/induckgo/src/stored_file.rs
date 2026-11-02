//! A file InDuckGo owns: written atomically and readable only by the user.
//!
//! Writes go to `<name>.part` and are renamed over the target, so a crash never leaves half a catalog
//! behind. Directories are created `0700` and files `0600`: profile data includes session cookies.

use std::fs;
use std::fs::DirBuilder;
use std::fs::OpenOptions;
use std::io;
use std::io::Write;
use std::os::unix::fs::DirBuilderExt;
use std::os::unix::fs::OpenOptionsExt;
use std::path::Path;
use std::path::PathBuf;

use serde_json::Value;

pub struct StoredFile {
    path: PathBuf,
}

impl StoredFile {
    const LOG_DOMAIN: &'static str = "induckgo";

    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn exists(&self) -> bool {
        self.path.exists()
    }

    /// The parsed document, or `None` when missing or corrupt (corruption is logged, then treated as empty).
    pub fn read_json(&self) -> Option<Value> {
        let bytes = fs::read(&self.path).ok()?;
        serde_json::from_slice(&bytes)
            .inspect_err(|error| glib::g_warning!(Self::LOG_DOMAIN, "ignoring {}: {error}", self.path.display()))
            .ok()
    }

    pub fn read_text(&self) -> Option<String> {
        fs::read_to_string(&self.path).ok()
    }

    pub fn write_json(&self, document: &Value) -> io::Result<()> {
        self.write(&serde_json::to_vec_pretty(document)?)
    }

    pub fn write(&self, bytes: &[u8]) -> io::Result<()> {
        if let Some(directory) = self.path.parent() {
            DirBuilder::new().recursive(true).mode(0o700).create(directory)?;
        }
        let mut partial_name = self.path.file_name().unwrap_or_default().to_owned();
        partial_name.push(".part");
        let partial = self.path.with_file_name(partial_name);
        let mut file = OpenOptions::new().write(true).create(true).truncate(true).mode(0o600).open(&partial)?;
        file.write_all(bytes)?;
        file.sync_all()?;
        fs::rename(&partial, &self.path)
    }

    pub fn remove(&self) -> io::Result<()> {
        match fs::remove_file(&self.path) {
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
            result => result,
        }
    }
}
