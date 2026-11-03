[English](README.md) | **Polski**

<p align="center">
  <img src="assets/icons/io.github.induckgo.InDuckGo.svg" alt="InDuckGo: indygo tarcza z białą kaczką" width="160">
</p>

# InDuckGo

**Przeglądarka dla Linuksa, w której prywatność jest na pierwszym miejscu.** Odtwarza ochronę z otwartej
przeglądarki DuckDuckGo dla macOS na silniku **WebKitGTK 6.0** (linuksowa wersja tego samego WebKita, którego
używają Safari i DuckDuckGo na macOS), z interfejsem **GTK 4 + libadwaita** napisanym w **Rust**. Bez telemetrii,
bez rozszerzeń do pilnowania, a każdy profil i każda zainstalowana aplikacja webowa są od siebie odizolowane.

[![tests](https://github.com/eshaiotic/induckgo/actions/workflows/tests.yml/badge.svg)](https://github.com/eshaiotic/induckgo/actions/workflows/tests.yml)
[![release](https://img.shields.io/github/v/release/eshaiotic/induckgo?color=6A5ACD)](https://github.com/eshaiotic/induckgo/releases/latest)
![cargo test](https://img.shields.io/badge/cargo%20test-304%20testy-success)
![stack](https://img.shields.io/badge/stack-Rust%20%E2%80%A2%20GTK%204%20%E2%80%A2%20WebKitGTK%206.0-8E7FE8)
![telemetria](https://img.shields.io/badge/telemetria-brak-FFB300)
[![licencja](https://img.shields.io/badge/licencja-GPL--3.0--or--later-2E1A87)](LICENSE)

InDuckGo to niezależny projekt. Nie jest powiązany z Duck Duck Go, Inc. ani przez nią wspierany.

---

## Zmierzone wobec Chrome i Brave

Świeże profile trzech przeglądarek działały jednocześnie 2026-11-02, za proxy, które zapisywało tylko nazwy
hostów. Trackery ocenia neutralna lista (Disconnect), każdy serwer liczony raz na serwis.

| | Chrome | Brave | InDuckGo |
|---|---|---|---|
| Trackery przepuszczone na 12 popularnych serwisach | 94 | 5 | **2** |
| … w tym serwery reklam | 75 | 4 | **1** |
| Połączenia przez 5 minut bezczynności | 46 do 15 hostów | 43 do 12 hostów | **20 do 8 hostów** |
| Odcisk przeglądarki widziany przez serwisy | wszędzie ten sam | inny na każdym serwisie, ten sam po restarcie | **inny na każdym serwisie i w każdej sesji** |

Połączenia InDuckGo w bezczynności pobierają dane ochrony (listę trackerów i konfigurację prywatności
DuckDuckGo, listy filtrów Guarda) i ładują stronę startową DuckDuckGo. Metoda, wszystkie hosty i miejsca, w
których InDuckGo jest słabszy (brak Safe Browsing, rzadki silnik, który wyróżnia użytkowników), są w
[porównaniu śledzenia](docs/pl/reports/tracking-comparison.md). Jednostronicowe podsumowanie dla osób spoza
branży to [tracking-summary.html](docs/pl/reports/tracking-summary.html) (pobierz i otwórz w przeglądarce).

---

## Przed czym chroni

| Ochrona | Jak działa |
|---|---|
| Trackery stron trzecich | Tracker Data Set DuckDuckGo jest tłumaczony na reguły blokowania treści WebKita (port `TrackerRadarKit`). |
| Psucie stron | Konfiguracja prywatności DuckDuckGo dostarcza wyjątki dla serwisów, żeby działały; przycisk tarczy (w aplikacji webowej: *Protect This Site* w menu) wyłącza ochronę dla jednego serwisu. |
| Fingerprinting, GPC | Content Scope Scripts DuckDuckGo działają w każdej ramce; InDuckGo Guard włącza obronę canvas, audio, ekranu i referera, którą DuckDuckGo zostawia wyłączoną, i wysyła nagłówek `Sec-GPC: 1`. |
| Reklamy, trackery, komunikaty o cookies | InDuckGo Guard pobiera listy EasyList, EasyPrivacy, uBlock Origin, Petera Lowe'a, polskie i listy komunikatów o cookies, i stosuje je także do własnych zapytań serwisu. |
| Ciasteczka między serwisami | Ciasteczka stron trzecich są odrzucane; włączone jest Intelligent Tracking Prevention WebKita. |
| Linki śledzące | Nawigacja z `utm_*`, `fbclid`, `gclid`, ... jest zatrzymywana, zanim opuści komputer, i ładowana od nowa bez nich. |
| Zwykłe HTTP | Nawigacje są podnoszone do HTTPS; jeśli serwis nie ma HTTPS, baner pyta przed niezabezpieczonym załadowaniem. |
| Pozostawione dane | Przycisk Fire jednym kliknięciem kasuje ciasteczka, magazyn, pamięć podręczną i favikony profilu. |
| Mieszanie tożsamości | Każdy profil i każda zainstalowana aplikacja webowa działa jako osobny proces z własnymi ciasteczkami i danymi. |

Rozszerzeń nie ma — celowo, i dlatego, że WebKitGTK nie ma API rozszerzeń.

**Aplikacje webowe.** *Install Site as App* (przycisk na prawym końcu paska adresu albo menu okna) zamienia
dowolny serwis — bank, pocztę — w aplikację na pulpicie z ikoną serwisu. Każda aplikacja ma własną,
odizolowaną sesję; link do innego serwisu kliknięty w aplikacji otwiera się w przeglądarce, więc logowanie
aplikacji nigdy nie spotyka obcych stron.

---

## Stan

Projekt w toku, ale używalny: działają karty, wszystkie powyższe zabezpieczenia, profile, aplikacje webowe,
pobieranie plików, pytania o uprawnienia i wyszukiwanie na stronie. Jeszcze nie ma: przywracania sesji,
zakładek, okna historii, obsługi zgód na cookies, tłumaczeń. Pełna lista z uzasadnieniem jest w dokumencie
architektury.

W przyszłości InDuckGo będzie częścią nowej dystrybucji Linuksa. Pozostanie samodzielnym projektem i zachowa
paczki dla innych dystrybucji.

---

## Instalacja

Pobierz paczkę z [najnowszego wydania](https://github.com/eshaiotic/induckgo/releases/latest):

| Plik | Dla |
|---|---|
| `induckgo_*_amd64.deb` | Debian 13, Ubuntu 24.04 LTS i nowsze |
| `induckgo-*.fc44.x86_64.rpm` | Fedora 44 |
| `induckgo-*-x86_64.pkg.tar.zst` | Arch Linux, CachyOS, EndeavourOS, Manjaro |
| `InDuckGo-*-x86_64.AppImage` | każda inna dystrybucja z WebKitGTK 6.0, GTK ≥ 4.14 i libadwaita ≥ 1.5 |

Każde wydanie ma `SHA256SUMS` i atestację pochodzenia (build provenance), która dowodzi, że plik zbudował
GitHub Actions tego repozytorium:

```sh
sha256sum --check --ignore-missing SHA256SUMS
gh attestation verify induckgo_*_amd64.deb --repo eshaiotic/induckgo
```

Paczki korzystają z systemowych WebKitGTK, GTK i libadwaita, więc silnik przeglądarki dostaje aktualizacje
bezpieczeństwa Twojej dystrybucji. AppImage nie wymaga instalacji:
`chmod +x InDuckGo-*-x86_64.AppImage && ./InDuckGo-*-x86_64.AppImage`.

Z kopii tego repozytorium:

```sh
./install                 # Arch: buduje paczkę pacmana i ją instaluje; Debian/Ubuntu/Fedora: instaluje dist/
./uninstall               # usuwa paczkę; profile i aplikacje webowe zostają
./uninstall --purge       # kasuje też wszystkie dane InDuckGo i skróty aplikacji webowych
```

---

## Budowanie paczek wydania

```sh
packaging/release.sh                # wszystkie formaty do dist/, z SHA256SUMS
packaging/release.sh deb appimage   # albo tylko wybrane: arch, deb, appimage, rpm
```

`.deb` i AppImage powstają w kontenerze Debiana 13, a `.rpm` w kontenerze Fedory 44 (rootless podman), więc
linkują się z bibliotekami systemów, na które są przeznaczone. Paczkę dla Archa buduje `makepkg` na hoście,
z flagami jego `makepkg.conf`. Runtime AppImage jest przypięty do wydania i sprawdzany po SHA-256.

Wypchnięcie taga wersji publikuje wydanie: `git tag v1.0.0 && git push origin v1.0.0`.
[Workflow wydania](.github/workflows/release.yml) uruchamia te same skrypty (paczkę Archa w kontenerze Arch
Linux), odrzuca zależności ze znanymi podatnościami (`cargo audit`) i wysyła cztery paczki z `SHA256SUMS` oraz
atestacjami. Tag musi zgadzać się z wersją w `Cargo.toml`.

---

## Uruchamianie ze źródeł

```sh
sudo pacman -S --needed rust webkitgtk-6.0 gtk4 libadwaita
cargo run -p induckgo                          # przeglądarka, profil domyślny
cargo run -p induckgo -- --profile work        # przeglądarka, profil "work"
cargo run -p induckgo -- https://example.com/  # otwórz stronę
```

Zainstalowane aplikacje webowe uruchamia się z menu aplikacji pulpitu (`induckgo --web-app <id>`).

---

## Testy

Stos: **`cargo test`** — 304 obiektowe scenariusze Given/When/Then w `crates/induckgo-core/tests/`
(33 pliki, jeden na typ domenowy).

```sh
cargo test -p induckgo-core     # testy domeny (szybkie, bez GTK i WebKita)
cargo clippy --all-targets      # linty (pedantic, unsafe zabroniony)
cargo fmt --check               # formatowanie
cargo audit                     # podatności zależności (polityka zero podatności)
```

**Architektura testów:**
- Domena (`induckgo-core`) nie zależy od GTK ani WebKita, więc każda decyzja ochrony — budowanie reguł,
  konwersja list filtrów, podnoszenie do HTTPS, czyszczenie linków, profile, aplikacje webowe, zoom — jest
  testowana bez ekranu.
- Każdy scenariusz to obiekt: `given_…` go buduje, `when_…` wykonuje akcję i zwraca wynik, `then_…` sprawdza
  jedną rzecz, zgodnie z [Object Oriented Programming First](docs/pl/engineering-rules.md):
  `LinkCleaningScenario::given_tracking_parameters_protection().when_link_is_opened(url).then_opens_unchanged()`.
- [Workflow testów](.github/workflows/tests.yml) uruchamia wszystkie cztery polecenia przy każdym pushu do
  `main` i przy każdym pull requeście.

---

## Struktura projektu

```
crates/induckgo-core/    logika domeny bez GTK/WebKita: reguły ochrony, listy filtrów Guarda,
                         profile, aplikacje webowe, zoom, uruchamianie
crates/induckgo-core/tests/
                         jeden plik scenariuszy OO na typ domenowy
crates/induckgo/         aplikacja GTK 4 + libadwaita: adaptery wykonujące decyzje domeny w
                         WebKitGTK, libsoup i GIO
assets/                  dołączone Content Scope Scripts (Apache-2.0), skrypt strony Guarda, ikony,
                         szablon strony błędu; nigdy Tracker Data Set ani listy filtrów
packaging/               PKGBUILD, spec rpm, składanie deb/AppImage, plik .desktop, metadane AppStream
docs/en, docs/pl         architektura, zasady inżynierskie i raporty (kanoniczny angielski, polskie lustro)
```

- [Architektura](docs/pl/architecture/overview.md) — procesy, izolacja, potok ochrony, aplikacje webowe, ograniczenia.
- [Zasady inżynierskie](docs/pl/engineering-rules.md) — Object Oriented Programming First, testy OO Given/When/Then.

---

## Licencja

InDuckGo to wolne oprogramowanie na licencji [GNU General Public License w wersji 3 lub późniejszej](LICENSE)
(`GPL-3.0-or-later`). Atrybucje są w [NOTICE](NOTICE).

### Licencje danych stron trzecich

- Content Scope Scripts 20.0.0 — Apache-2.0, dołączone w `assets/content-scope-scripts` razem z licencją.
- Konfiguracja prywatności — Apache-2.0, pobierana w czasie działania.
- Tracker Data Set — **CC BY-NC-SA 4.0** (niekomercyjna), pobierany w czasie działania, nigdy nie commitowany.
- Listy filtrów InDuckGo Guard — GPL-3.0, CC BY-SA 3.0, CC BY 3.0, CC BY-NC-SA 4.0 lub warunki wydawcy,
  pobierane w czasie działania, nigdy nie commitowane.
- adblock-rust 0.13.5 — MPL-2.0, wkompilowany bez zmian (źródło: <https://github.com/brave/adblock-rust>).
