//! Everything WebKit injects into the pages of one profile: compiled rule lists and the content scope script.
//!
//! Three rule lists, each compiled separately because `ignore-previous-rules` only reaches back within its
//! own list: tracker blocking (the Tracker Data Set), HTTPS upgrades and tracking-parameter detection.
//! Compiling the tracker list takes about a second, so every compiled list is kept in a per-profile
//! `UserContentFilterStore` together with a fingerprint of its source; an unchanged list is loaded instead.

use std::collections::hash_map::DefaultHasher;
use std::hash::Hash;
use std::hash::Hasher;
use std::path::Path;
use std::path::PathBuf;

use induckgo_core::protection::content_blocker_rule::ContentBlockerRule;
use induckgo_core::protection::content_scope_message::ContentScopeMessage;
use induckgo_core::protection::content_scope_preferences::ContentScopePreferences;
use induckgo_core::protection::content_scope_script::ContentScopeScript;
use induckgo_core::protection::privacy_configuration::PrivacyConfiguration;
use serde_json::Value;
use webkit6::UserContentFilter;
use webkit6::UserContentFilterStore;
use webkit6::UserContentInjectedFrames;
use webkit6::UserContentManager;
use webkit6::UserScript;
use webkit6::UserScriptInjectionTime;

use crate::stored_file::StoredFile;

/// One named rule list, e.g. `trackers`.
pub struct RuleList {
    pub name: &'static str,
    pub rules: Vec<ContentBlockerRule>,
}

impl RuleList {
    /// The list as WebKit reads it. For the tracker list this takes a tenth of a second or more, which is why
    /// `ProfileSession` prepares that list on a worker thread.
    pub fn prepared(&self) -> PreparedRuleList {
        let source = Value::Array(self.rules.iter().map(ContentBlockerRule::to_json).collect()).to_string();
        PreparedRuleList {
            name: self.name,
            fingerprint: Self::fingerprint(&source),
            source,
            is_empty: self.rules.is_empty(),
        }
    }

    fn fingerprint(source: &str) -> String {
        let mut hasher = DefaultHasher::new();
        source.hash(&mut hasher);
        format!("{:016x}", hasher.finish())
    }
}

/// A rule list serialized for WebKit, with a fingerprint telling whether the stored compiled copy is current.
pub struct PreparedRuleList {
    name: &'static str,
    source: String,
    fingerprint: String,
    is_empty: bool,
}

pub struct PageProtections {
    content_manager: UserContentManager,
    filter_store: UserContentFilterStore,
    filter_store_path: PathBuf,
    preferences: ContentScopePreferences,
}

impl PageProtections {
    /// DuckDuckGo Content Scope Scripts 20.0.0 (Apache-2.0), vendored and pinned by SHA-256 in the repository.
    const SCRIPT_TEMPLATE: &'static str = include_str!("../../../assets/content-scope-scripts/contentScope.js");
    /// The macOS app version that bundles this script; the privacy configuration gates features on it.
    pub const COMPATIBILITY_VERSION: &'static str = "1.214.0";
    const LOG_DOMAIN: &'static str = "induckgo";

    pub fn new(filter_store_path: &Path) -> Self {
        let content_manager = UserContentManager::new();
        let handler = ContentScopePreferences::MESSAGING_CONTEXT_NAME;
        content_manager.register_script_message_handler_with_reply(handler, None);
        content_manager.connect_script_message_with_reply_received(Some(handler), |_, message, reply| {
            let parsed = message.to_json(0).and_then(|text| serde_json::from_str(&text).ok()).unwrap_or(Value::Null);
            if let Some(context) = message.context() {
                let answer = ContentScopeMessage::new(&parsed).reply();
                reply.return_value(&javascriptcore::Value::new_string(&context, Some(&answer)));
            }
            true
        });
        Self {
            content_manager,
            filter_store: UserContentFilterStore::new(&filter_store_path.to_string_lossy()),
            filter_store_path: filter_store_path.to_owned(),
            preferences: ContentScopePreferences {
                global_privacy_control: true,
                session_key: glib::uuid_string_random().to_string(),
                message_secret: glib::uuid_string_random().to_string(),
                language: Self::user_language(),
                compatibility_version: Self::COMPATIBILITY_VERSION.to_owned(),
            },
        }
    }

    pub fn content_manager(&self) -> &UserContentManager {
        &self.content_manager
    }

    /// Replaces the content scope script; it runs at document start in every frame, in the page's world.
    pub fn install_script(&self, configuration: &PrivacyConfiguration, user_unprotected_hosts: &[String]) {
        let source = ContentScopeScript::new(Self::SCRIPT_TEMPLATE).render(
            configuration,
            user_unprotected_hosts,
            &self.preferences,
        );
        let script =
            UserScript::new(&source, UserContentInjectedFrames::AllFrames, UserScriptInjectionTime::Start, &[], &[]);
        self.content_manager.remove_all_scripts();
        self.content_manager.add_script(&script);
    }

    /// The list compiled last time, if WebKit still has it. The store belongs to this profile alone, so this
    /// is the profile's own last list: it can protect pages while the current one is being built.
    pub async fn last_compiled(&self, name: &str) -> Option<UserContentFilter> {
        if !self.marker(name).exists() {
            return None;
        }
        self.filter_store.load_future(name).await.ok()
    }

    /// Compiles (or loads) every non-empty list; a list WebKit rejects is logged and left out.
    pub async fn compile(&self, lists: &[PreparedRuleList]) -> Vec<UserContentFilter> {
        let mut filters = Vec::new();
        for list in lists.iter().filter(|list| !list.is_empty) {
            match self.compiled(list).await {
                Ok(filter) => filters.push(filter),
                Err(error) => glib::g_critical!(Self::LOG_DOMAIN, "rule list {} rejected: {error}", list.name),
            }
        }
        filters
    }

    /// Swaps in a new set of compiled lists; pages pick them up on their next load.
    pub fn activate(&self, filters: &[UserContentFilter]) {
        self.content_manager.remove_all_filters();
        for filter in filters {
            self.content_manager.add_filter(filter);
        }
    }

    async fn compiled(&self, list: &PreparedRuleList) -> Result<UserContentFilter, glib::Error> {
        let marker = self.marker(list.name);
        if marker.read_text().as_deref() == Some(list.fingerprint.as_str())
            && let Ok(filter) = self.filter_store.load_future(list.name).await
        {
            return Ok(filter);
        }
        let filter = self.filter_store.save_future(list.name, &glib::Bytes::from(list.source.as_bytes())).await?;
        if let Err(error) = marker.write(list.fingerprint.as_bytes()) {
            glib::g_warning!(Self::LOG_DOMAIN, "cannot record rule list fingerprint: {error}");
        }
        Ok(filter)
    }

    /// Fingerprint of the source the stored compiled list `name` was made from.
    fn marker(&self, name: &str) -> StoredFile {
        StoredFile::new(self.filter_store_path.join(format!("{name}.fingerprint")))
    }

    /// Two-letter language of the user interface (`pl` from `pl_PL.UTF-8`).
    fn user_language() -> String {
        glib::language_names()
            .iter()
            .map(|name| name.as_str().chars().take_while(char::is_ascii_alphabetic).collect::<String>())
            .find(|language| language.len() == 2)
            .unwrap_or_else(|| "en".to_owned())
    }
}
