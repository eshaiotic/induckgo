//! The search engine a profile sends searches to: what the address bar searches with, where a new tab opens,
//! and what an error page offers for a mistyped name.
//!
//! DuckDuckGo is the default; Brave Search and Google can be chosen instead. A search is an ordinary
//! top-level navigation to the engine, so InDuckGo's protections apply to the engine's pages as to any site.

use serde_json::Value;
use serde_json::json;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SearchEngine {
    id: &'static str,
    name: &'static str,
    home_page: &'static str,
    /// The search address, to which the encoded query is appended.
    query_address: &'static str,
}

impl SearchEngine {
    pub const DUCKDUCKGO: Self = Self {
        id: "duckduckgo",
        name: "DuckDuckGo",
        home_page: "https://duckduckgo.com/",
        query_address: "https://duckduckgo.com/?q=",
    };
    pub const BRAVE_SEARCH: Self = Self {
        id: "brave",
        name: "Brave Search",
        home_page: "https://search.brave.com/",
        query_address: "https://search.brave.com/search?q=",
    };
    pub const GOOGLE: Self = Self {
        id: "google",
        name: "Google",
        home_page: "https://www.google.com/",
        query_address: "https://www.google.com/search?q=",
    };
    /// Every engine the user can choose, in menu order.
    pub const ALL: [Self; 3] = [Self::DUCKDUCKGO, Self::BRAVE_SEARCH, Self::GOOGLE];
    const STORED_KEY: &'static str = "searchEngine";

    /// The engine with this identifier, when InDuckGo offers it.
    #[must_use]
    pub fn from_id(id: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|engine| engine.id == id)
    }

    /// The stored choice; DuckDuckGo when nothing (or an engine InDuckGo does not offer) was stored.
    #[must_use]
    pub fn from_json(document: &Value) -> Self {
        document.get(Self::STORED_KEY).and_then(Value::as_str).and_then(Self::from_id).unwrap_or_default()
    }

    #[must_use]
    pub fn to_json(self) -> Value {
        json!({ Self::STORED_KEY: self.id })
    }

    /// A stable identifier (`brave`), for storage and menu actions.
    #[must_use]
    pub fn id(self) -> &'static str {
        self.id
    }

    #[must_use]
    pub fn name(self) -> &'static str {
        self.name
    }

    /// Where a new tab opens.
    #[must_use]
    pub fn home_page(self) -> &'static str {
        self.home_page
    }

    /// The search for `text`, whatever it looks like.
    #[must_use]
    pub fn search_url(self, text: &str) -> String {
        let query: String = form_urlencoded::byte_serialize(text.as_bytes()).collect();
        format!("{}{query}", self.query_address)
    }
}

impl Default for SearchEngine {
    fn default() -> Self {
        Self::DUCKDUCKGO
    }
}
