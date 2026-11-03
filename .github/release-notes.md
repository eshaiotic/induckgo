Packages for x86-64 Linux. WebKitGTK 6.0, GTK 4 and libadwaita come from your distribution, so the browser
engine keeps receiving its security updates.

| File | For |
|---|---|
| `induckgo_*_amd64.deb` | Debian 13, Ubuntu 24.04 LTS and newer |
| `induckgo-*.fc44.x86_64.rpm` | Fedora 44 |
| `induckgo-*-x86_64.pkg.tar.zst` | Arch Linux, CachyOS, EndeavourOS, Manjaro |
| `InDuckGo-*-x86_64.AppImage` | any other distribution with WebKitGTK 6.0, GTK ≥ 4.14 and libadwaita ≥ 1.5 |

Check a download before installing it:

```sh
sha256sum --check --ignore-missing SHA256SUMS
gh attestation verify induckgo_*_amd64.deb --repo eshaiotic/induckgo
```

InDuckGo is an independent project, not affiliated with or endorsed by Duck Duck Go, Inc.
