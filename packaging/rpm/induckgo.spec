# Built by packaging/release.sh inside a Fedora container (packaging/rpm/build.sh). Source0 is a tarball of
# the working tree. Library dependencies (libwebkitgtk-6.0.so.4, ...) are found by rpm in the ELF binary.

# The release profile strips symbols, so there is nothing for a debuginfo package.
%global debug_package %{nil}

Name:           induckgo
Version:        1.2.1
Release:        1%{?dist}
Summary:        Privacy-first WebKitGTK browser built on DuckDuckGo's open-source tracker protections
License:        GPL-3.0-or-later
Source0:        %{name}-%{version}.tar.gz

BuildRequires:  cargo
BuildRequires:  rust
BuildRequires:  pkgconfig(gtk4)
BuildRequires:  pkgconfig(libadwaita-1)
BuildRequires:  pkgconfig(libsoup-3.0)
BuildRequires:  pkgconfig(webkitgtk-6.0)
Requires:       hicolor-icon-theme
# The Tor window starts its own tor; Fedora's tor package leaves its system service off.
Requires:       tor
Recommends:     obfs4

%description
InDuckGo blocks third-party trackers with DuckDuckGo's open-source protections, upgrades connections to
HTTPS, removes tracking parameters from links and announces Global Privacy Control. Every profile and every
site installed as an app runs in its own process with its own cookies and storage. The Tor window browses
through InDuckGo's own Tor, with a session that lives in memory only.

%prep
%autosetup
if ! grep -qx 'version = "%{version}"' Cargo.toml; then
    echo "Version %{version} differs from the version in Cargo.toml" >&2
    exit 1
fi

%build
cargo build --release --locked -p induckgo

%check
cargo test --locked -p induckgo-core

%install
install -Dm0755 target/release/induckgo %{buildroot}%{_bindir}/induckgo
install -Dm0644 packaging/linux/io.github.induckgo.InDuckGo.desktop -t %{buildroot}%{_datadir}/applications
install -Dm0644 packaging/linux/io.github.induckgo.InDuckGo.metainfo.xml -t %{buildroot}%{_metainfodir}
install -Dm0644 assets/icons/io.github.induckgo.InDuckGo.svg -t %{buildroot}%{_datadir}/icons/hicolor/scalable/apps

%files
%license LICENSE NOTICE
%doc README.md
%{_bindir}/induckgo
%{_datadir}/applications/io.github.induckgo.InDuckGo.desktop
%{_metainfodir}/io.github.induckgo.InDuckGo.metainfo.xml
%{_datadir}/icons/hicolor/scalable/apps/io.github.induckgo.InDuckGo.svg

%changelog
* Thu Nov 05 2026 Michał Ligus <eshaiotic@eshaiotic.io> - 1.2.1-1
- The WebKitGTK browser is unchanged in behaviour; the shared core gained tab placement, window button style and profile tile preferences for the Qt edition (QtWebEngine), which is not packaged yet.
- Interface texts for the new features were added to all 66 translations.

* Thu Nov 05 2026 Michał Ligus <eshaiotic@eshaiotic.io> - 1.2.0-1
- The WebKitGTK browser is unchanged from 1.1.2 in behaviour; the shared core gained the groundwork of the Qt edition (QtWebEngine), which is not packaged yet.
- Interface texts for the new Qt features were added to all 66 translations.

* Wed Nov 04 2026 Michał Ligus <eshaiotic@eshaiotic.io> - 1.1.2-1
- Smooth mouse-wheel scrolling: every wheel step is animated like in Chromium (switch in Preferences → Appearance).
- Faster scrolling on pages with scroll animations: Guard's page script no longer retests the same elements.

* Wed Nov 04 2026 Michał Ligus <eshaiotic@eshaiotic.io> - 1.1.1-1
- Interface language: choose among 68 languages in the preferences.
- Downloads: toolbar list, save-folder choice, clear list, and a manager with the whole history.
- Faster pages: drawing with GTK's OpenGL renderer instead of copying every frame through memory.

* Tue Nov 03 2026 Michał Ligus <eshaiotic@eshaiotic.io> - 1.1.0-1
- Tor window: browsing through InDuckGo's own Tor, with bridges and New Identity.
- Toolbar: an onion button for the Tor window, and preferences to choose the toolbar's buttons.

* Tue Nov 03 2026 Michał Ligus <eshaiotic@eshaiotic.io> - 1.0.1-1
- New application icon.

* Mon Nov 02 2026 Michał Ligus <eshaiotic@eshaiotic.io> - 1.0.0-1
- First release.
