Packages for x86-64 Linux. InDuckGo runs on QtWebEngine (Chromium) with a Qt Quick interface. The deb, rpm and
pacman packages use your distribution's Qt and QtWebEngine, so the browser engine keeps receiving its security
updates; the AppImage carries its own Qt and QtWebEngine and gets engine updates with each InDuckGo release.

| File | For |
|---|---|
| `induckgo_*_amd64.deb` | Debian 13 and newer |
| `induckgo-*.fc44.x86_64.rpm` | Fedora 44 |
| `induckgo-*-x86_64.pkg.tar.zst` | Arch Linux, CachyOS, EndeavourOS, Manjaro |
| `InDuckGo-*-x86_64.AppImage` | any other distribution with glibc 2.41 or newer (large: it contains Chromium) |

Check a download before installing it:

```sh
sha256sum --check --ignore-missing SHA256SUMS
gh attestation verify induckgo_*_amd64.deb --repo eshaiotic/induckgo
```

InDuckGo is an independent project, not affiliated with or endorsed by Duck Duck Go, Inc.
