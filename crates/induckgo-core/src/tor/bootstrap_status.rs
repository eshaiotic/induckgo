//! How far Tor has got connecting to the Tor network ("bootstrapping"), from its `BOOTSTRAP` status reports.
//!
//! Tor reports each step twice over: as the answer to `GETINFO status/bootstrap-phase` and as asynchronous
//! `STATUS_CLIENT` events while it works. Both carry the same text:
//! `NOTICE BOOTSTRAP PROGRESS=50 TAG=loading_descriptors SUMMARY="Loading relay descriptors"`. When something
//! goes wrong the severity becomes `WARN` and the report adds `WARNING="Connection refused"` and a
//! `RECOMMENDATION`: `warn` when the user should be told, `ignore` when Tor expects to recover by itself.

use crate::tor::keyword_arguments::KeywordArguments;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BootstrapStatus {
    progress: u8,
    summary: String,
    problem: Option<String>,
}

impl BootstrapStatus {
    const BOOTSTRAP: &'static str = "BOOTSTRAP";
    const COMPLETE: u8 = 100;

    /// The report in `text` (the part after `STATUS_CLIENT ` or `status/bootstrap-phase=`); `None` for any other
    /// status report.
    #[must_use]
    pub fn from_report(text: &str) -> Option<Self> {
        let report = KeywordArguments::parse(text);
        let [severity, kind, ..] = report.words() else { return None };
        if kind != Self::BOOTSTRAP {
            return None;
        }
        let progress = report.value("PROGRESS")?.parse::<u8>().ok().filter(|progress| *progress <= Self::COMPLETE)?;
        let is_warning = matches!(severity.as_str(), "WARN" | "ERR");
        let wants_attention = report.value("RECOMMENDATION") != Some("ignore");
        let problem = (is_warning && wants_attention)
            .then(|| report.value("WARNING").unwrap_or("Tor reported a problem").to_owned());
        Some(Self { progress, summary: report.value("SUMMARY").unwrap_or_default().to_owned(), problem })
    }

    /// Per cent of the way, 0 to 100.
    #[must_use]
    pub fn progress(&self) -> u8 {
        self.progress
    }

    /// What Tor is doing, in its own words (`Loading relay descriptors`).
    #[must_use]
    pub fn summary(&self) -> &str {
        &self.summary
    }

    /// What went wrong, when Tor wants the user to know (`Connection refused`).
    #[must_use]
    pub fn problem(&self) -> Option<&str> {
        self.problem.as_deref()
    }

    /// Tor is connected: circuits can be built and the SOCKS port carries traffic.
    #[must_use]
    pub fn is_complete(&self) -> bool {
        self.progress == Self::COMPLETE
    }
}
