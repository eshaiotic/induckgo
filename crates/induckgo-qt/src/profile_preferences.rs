//! The profile's choices the preferences window edits: which toolbar buttons show and whether the mouse wheel
//! scrolls smoothly. Each is kept in its own file of the profile, as in the GTK edition.

use std::path::Path;

use induckgo_core::profile::toolbar_layout::ToolbarButton;
use induckgo_core::profile::toolbar_layout::ToolbarLayout;
use induckgo_core::scrolling::scrolling_preferences::ScrollingPreferences;
use induckgo_core::storage::owned_json_file::OwnedJsonFile;

pub struct ProfilePreferences {
    toolbar: ToolbarLayout,
    toolbar_file: OwnedJsonFile,
    scrolling: ScrollingPreferences,
    scrolling_file: OwnedJsonFile,
}

impl ProfilePreferences {
    #[must_use]
    pub fn open(directory: &Path) -> Self {
        let toolbar_file = OwnedJsonFile::new(directory.join("toolbar.json"));
        let scrolling_file = OwnedJsonFile::new(directory.join("scrolling.json"));
        Self {
            toolbar: toolbar_file.read().map(|document| ToolbarLayout::from_json(&document)).unwrap_or_default(),
            scrolling: scrolling_file
                .read()
                .map(|document| ScrollingPreferences::from_json(&document))
                .unwrap_or_default(),
            toolbar_file,
            scrolling_file,
        }
    }

    /// Whether the toolbar shows the button stored as `id` (`reload`); a name nobody offers counts as shown.
    #[must_use]
    pub fn shows_toolbar_button(&self, id: &str) -> bool {
        Self::button_named(id).is_none_or(|button| self.toolbar.shows(button))
    }

    pub fn set_toolbar_button_shown(&mut self, id: &str, is_shown: bool) {
        let Some(button) = Self::button_named(id) else { return };
        self.toolbar.set_shown(button, is_shown);
        // A choice that cannot be written still applies until the browser closes.
        let _ = self.toolbar_file.write(&self.toolbar.to_json());
    }

    #[must_use]
    pub fn smooth_wheel(&self) -> bool {
        self.scrolling.smooth_wheel()
    }

    pub fn set_smooth_wheel(&mut self, is_enabled: bool) {
        self.scrolling.set_smooth_wheel(is_enabled);
        let _ = self.scrolling_file.write(&self.scrolling.to_json());
    }

    fn button_named(id: &str) -> Option<ToolbarButton> {
        ToolbarButton::ALL.into_iter().find(|button| button.id() == id)
    }
}
