//! A safe file name for a download, derived from the name the website suggests.
//!
//! The suggestion comes from the `Content-Disposition` header or the URL, both controlled by the site. It
//! is reduced to a single visible path component: no directories (`../../.bashrc`), no hidden files, no
//! control characters, at most 255 bytes (the Linux file name limit). An existing file is never
//! overwritten; the name is numbered instead, as file managers do.

use std::path::Path;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DownloadFileName {
    stem: String,
    extension: String,
}

impl DownloadFileName {
    const FALLBACK: &'static str = "download";
    const MAX_BYTES: usize = 255;
    /// Room kept for the ` (n)` numbering.
    const NUMBERING_RESERVE: usize = 12;
    /// Longer "extensions" are just dots inside a name and stay part of the stem.
    const MAX_EXTENSION_BYTES: usize = 16;

    #[must_use]
    pub fn from_suggestion(suggested: &str) -> Self {
        let last_component = suggested.rsplit(['/', '\\']).next().unwrap_or_default();
        let visible: String = last_component.chars().filter(|character| !character.is_control()).collect();
        let name = visible.trim().trim_start_matches('.');
        let name = if name.is_empty() { Self::FALLBACK } else { name };
        let (stem, extension) = Self::split_extension(name);
        let stem_budget = Self::MAX_BYTES - Self::NUMBERING_RESERVE - extension.len();
        Self { stem: Self::truncated(stem, stem_budget).to_owned(), extension: extension.to_owned() }
    }

    /// `directory/name.ext`, or `name (2).ext`, `name (3).ext`... for the first one `exists` reports free.
    #[must_use]
    pub fn first_free_path_in(&self, directory: &Path, exists: impl Fn(&Path) -> bool) -> PathBuf {
        let mut candidate = directory.join(format!("{}{}", self.stem, self.extension));
        let mut number = 2_u32;
        while exists(&candidate) && number < u32::MAX {
            candidate = directory.join(format!("{} ({number}){}", self.stem, self.extension));
            number += 1;
        }
        candidate
    }

    /// Splits `report.tar.gz` into `report` and `.tar.gz`, so numbering lands before the whole extension.
    fn split_extension(name: &str) -> (&str, &str) {
        let Some(first_dot) = name.find('.') else { return (name, "") };
        let extension = &name[first_dot..];
        if first_dot == 0 || extension.len() > Self::MAX_EXTENSION_BYTES {
            return Self::split_last_extension(name);
        }
        (&name[..first_dot], extension)
    }

    fn split_last_extension(name: &str) -> (&str, &str) {
        match name.rfind('.') {
            Some(dot) if dot > 0 && name.len() - dot <= Self::MAX_EXTENSION_BYTES => (&name[..dot], &name[dot..]),
            _ => (name, ""),
        }
    }

    fn truncated(text: &str, max_bytes: usize) -> &str {
        if text.len() <= max_bytes {
            return text;
        }
        let mut end = max_bytes;
        while !text.is_char_boundary(end) {
            end -= 1;
        }
        &text[..end]
    }
}
