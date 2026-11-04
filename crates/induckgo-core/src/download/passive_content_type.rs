//! Which downloaded files may be opened straight from the downloads list.
//!
//! A file from a website can be a program, a script or a launcher dressed as a document. Only types that
//! merely display something are opened with one click; everything else is left to the file manager, which
//! asks the person what to do. The decision fails closed: an unknown or missing type is not passive.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PassiveContentType(String);

impl PassiveContentType {
    #[must_use]
    pub fn new(content_type: &str) -> Self {
        Self(content_type.trim().to_ascii_lowercase())
    }

    /// PDF documents, plain text, images (not SVG, which can carry scripts), audio and video.
    #[must_use]
    pub fn is_passive(&self) -> bool {
        let content_type = self.0.as_str();
        if content_type == "image/svg+xml" {
            return false;
        }
        content_type == "application/pdf"
            || content_type == "text/plain"
            || ["image/", "audio/", "video/"].iter().any(|prefix| content_type.starts_with(prefix))
    }
}
