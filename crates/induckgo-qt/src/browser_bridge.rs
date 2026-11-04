//! What the QML window asks Rust: where an address-bar entry leads, interface text, and page scripts.

#[cxx_qt::bridge]
pub mod ffi {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
    }

    extern "RustQt" {
        #[qobject]
        #[qml_element]
        type BrowserBridge = super::BrowserBridgeRust;

        /// The address to open for what was typed in the address bar: a web address, else a search. Empty
        /// for blank input.
        #[qinvokable]
        fn destination(self: &BrowserBridge, typed_text: &QString) -> QString;

        /// An interface message in the language this process speaks (the message itself when untranslated).
        #[qinvokable]
        fn message(self: &BrowserBridge, text: &QString) -> QString;

        /// The mouse-wheel smoothing script, or empty when the person turned smoothing off.
        #[qinvokable]
        fn smooth_wheel_script(self: &BrowserBridge) -> QString;

        /// A notice to show when the window opens, or empty.
        #[qinvokable]
        fn startup_notice(self: &BrowserBridge) -> QString;

        /// The home page of the chosen search engine.
        #[qinvokable]
        fn home_page(self: &BrowserBridge) -> QString;
    }
}

use cxx_qt_lib::QString;
use induckgo_core::engine::smooth_wheel_page_script::SmoothWheelPageScript;
use induckgo_core::language::message_catalogue::MessageCatalogue;
use induckgo_core::navigation::address_bar_input::AddressBarInput;
use induckgo_core::navigation::search_engine::SearchEngine;
use induckgo_core::scrolling::smooth_wheel_scroll::SmoothWheelScroll;

use crate::interface_catalogue::InterfaceCatalogue;
use crate::startup_notice::StartupNotice;

#[derive(Default)]
pub struct BrowserBridgeRust;

// QML invokes these on the object, so they take `&self` even where they do not read it.
#[allow(clippy::unused_self)]
impl ffi::BrowserBridge {
    fn destination(&self, typed_text: &QString) -> QString {
        let address = AddressBarInput::new(&typed_text.to_string()).destination(SearchEngine::default());
        QString::from(address.unwrap_or_default().as_str())
    }

    fn message(&self, text: &QString) -> QString {
        let catalogue: &MessageCatalogue = InterfaceCatalogue::current();
        QString::from(catalogue.text(&text.to_string()))
    }

    fn smooth_wheel_script(&self) -> QString {
        QString::from(SmoothWheelPageScript::source_for(&SmoothWheelScroll::chromium_like()).as_str())
    }

    fn startup_notice(&self) -> QString {
        QString::from(StartupNotice::text())
    }

    fn home_page(&self) -> QString {
        QString::from(SearchEngine::default().home_page())
    }
}
