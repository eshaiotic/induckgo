//! The messages of the interface in the language this process speaks.
//!
//! One language is chosen when the process starts (see `InterfaceLanguageSetting`) and installed here before
//! any window exists, so every part of the interface (menus, dialogs, toasts, the error page) draws from the
//! same catalogue and no screen mixes two languages. A message is written in English at its place in the code
//! and acts as its own key: `tr!("Back")`, `tr!("Close {name} first", name = profile)`.

use std::sync::OnceLock;

use induckgo_core::language::message_catalogue::MessageCatalogue;

static CATALOGUE: OnceLock<MessageCatalogue> = OnceLock::new();

/// Looks a message up in `tr!`'s catalogue; nothing else should call it.
pub struct InterfaceMessages;

impl InterfaceMessages {
    /// Installs the process's catalogue. Only the first call has an effect.
    pub fn install(catalogue: MessageCatalogue) {
        let _ = CATALOGUE.set(catalogue);
    }

    /// The catalogue in use; English as written until one is installed (tests, tools).
    pub fn catalogue() -> &'static MessageCatalogue {
        CATALOGUE.get_or_init(MessageCatalogue::untranslated)
    }

    pub fn text(message: &'static str) -> &'static str {
        Self::catalogue().text(message)
    }

    pub fn text_with(message: &'static str, values: &[(&str, &str)]) -> String {
        Self::catalogue().text_with(message, values)
    }
}

/// A message of the interface in the process's language: `tr!("Back")`, or with named placeholders
/// `tr!("Close {name} first", name = profile)` (every value is shown with `Display`).
macro_rules! tr {
    ($message:literal) => {
        $crate::interface_messages::InterfaceMessages::text($message)
    };
    ($message:literal, $($name:ident = $value:expr),+ $(,)?) => {
        $crate::interface_messages::InterfaceMessages::text_with(
            $message,
            &[$((stringify!($name), ($value).to_string().as_str())),+],
        )
    };
}
