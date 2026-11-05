[English](README.md) | **Polski**

<p align="center">
  <img src="assets/icons/io.github.induckgo.InDuckGo.svg" alt="InDuckGo: indygo tarcza z białą kaczką" width="160">
</p>

# InDuckGo

**Przeglądarka dla Linuksa, w której prywatność jest na pierwszym miejscu.** Odtwarza ochronę z otwartej
przeglądarki DuckDuckGo dla macOS na silniku **WebKitGTK 6.0** (linuksowa wersja tego samego WebKita, którego
używają Safari i DuckDuckGo na macOS), z interfejsem **GTK 4 + libadwaita** napisanym w **Rust**. Bez telemetrii,
bez rozszerzeń do pilnowania, każdy profil i każda zainstalowana aplikacja webowa są od siebie odizolowane, a
**okno Tor** przegląda sieć przez własnego Tora InDuckGo.

[![tests](https://github.com/eshaiotic/induckgo/actions/workflows/tests.yml/badge.svg)](https://github.com/eshaiotic/induckgo/actions/workflows/tests.yml)
[![release](https://img.shields.io/github/v/release/eshaiotic/induckgo?color=6A5ACD)](https://github.com/eshaiotic/induckgo/releases/latest)
![cargo nextest](https://img.shields.io/badge/cargo%20nextest-670%20test%C3%B3w-success)
![stack](https://img.shields.io/badge/stack-Rust%20%E2%80%A2%20GTK%204%20%E2%80%A2%20WebKitGTK%206.0-8E7FE8)
![telemetria](https://img.shields.io/badge/telemetria-brak-FFB300)
[![licencja](https://img.shields.io/badge/licencja-GPL--3.0--or--later-2E1A87)](LICENSE)

InDuckGo to niezależny projekt. Nie jest powiązany z Duck Duck Go, Inc. ani przez nią wspierany.

---

## Zmierzone wobec Chrome i Brave, porównane z DuckDuckGo

Świeże profile Chrome, Brave i InDuckGo działały jednocześnie 2026-11-02, za proxy, które zapisywało tylko
nazwy hostów. Trackery ocenia neutralna lista (Disconnect), każdy serwer liczony raz na serwis.

| | Chrome | Brave | DuckDuckGo¹ | InDuckGo |
|---|---|---|---|---|
| Trackery przepuszczone na 12 popularnych serwisach | 94 | 5 | niezmierzone | **2** |
| … w tym serwery reklam | 75 | 4 | niezmierzone | **1** |
| Połączenia przez 5 minut bezczynności | 46 do 15 hostów | 43 do 12 hostów | niezmierzone | **20 do 8 hostów** |
| Odcisk przeglądarki widziany przez serwisy | wszędzie ten sam | inny na każdym serwisie, ten sam po restarcie | niezmierzony; ochrona canvas i audio wyłączona na macOS | **inny na każdym serwisie i w każdej sesji** |

¹ DuckDuckGo nie wydaje przeglądarki na Linuksa, więc nie mogła działać na tej samej maszynie w tym samym
czasie. Jej funkcje porównuje, bez pomiaru, [następna tabela](#porównanie-z-przeglądarką-duckduckgo).

Połączenia InDuckGo w bezczynności pobierają dane ochrony (listę trackerów i konfigurację prywatności
DuckDuckGo, listy filtrów Guarda) i ładują stronę startową DuckDuckGo. Metoda, wszystkie hosty i miejsca, w
których InDuckGo jest słabszy (brak Safe Browsing, rzadki silnik, który wyróżnia użytkowników), są w
[porównaniu śledzenia](docs/pl/reports/tracking-comparison.md). Jednostronicowe podsumowanie dla osób spoza
branży to [tracking-summary.html](docs/pl/reports/tracking-summary.html) (pobierz i otwórz w przeglądarce).

### Porównanie z przeglądarką DuckDuckGo

DuckDuckGo nie wydaje przeglądarki na Linuksa (tylko na macOS, Windows, iOS i Androida), więc nie mogła wziąć
udziału w pomiarze powyżej. Tabela zestawia InDuckGo z przeglądarką DuckDuckGo na macOS, której ochrony InDuckGo
przenosi, tak jak opisuje ją DuckDuckGo (listopad 2026). Porównuje funkcje; nic tu nie było mierzone.

| | DuckDuckGo na macOS | InDuckGo |
|---|---|---|
| **Skrypty śledzące** | | |
| Trackery stron trzecich | blokowane według listy trackerów DuckDuckGo | blokowane według tej samej listy |
| Reklamy i trackery spoza tej listy | nieblokowane; brak list filtrów reklam | blokowane przez EasyList, EasyPrivacy, listy uBlock Origin i inne, także w żądaniach do domeny samego serwisu |
| Śledzenie reklam Microsoftu (`bat.bing.com`) | dozwolone na stronie reklamodawcy po kliknięciu reklamy w wyszukiwarce DuckDuckGo | blokowane; brak wyjątku dla kliknięć w reklamy |
| Parametry śledzące w linkach (`utm_*`, `fbclid`, ...) | usuwane tylko z pierwszej nawigacji | usuwane z każdej nawigacji najwyższego poziomu |
| Strony Google AMP | zastępowane oryginalną stroną wydawcy | zostają bez zmian |
| Komunikaty o cookies | odpowiadane „odrzuć” | ukrywane przez listę filtrów, bez odpowiedzi |
| **Anonimowość** | | |
| Ochrona przed odciskiem przeglądarki (canvas, audio, bateria, ekran, referrer) | wyłączona w konfiguracji DuckDuckGo dla macOS | włączona: odcisk zmienia się na każdym serwisie i w każdej sesji |
| Jak popularnie wygląda przeglądarka | WebKit na macOS, silnik każdego użytkownika Safari | WebKitGTK na Linuksie: mało użytkowników, więc InDuckGo się wyróżnia |
| Global Privacy Control | w JavaScripcie dla każdego serwisu; nagłówek `Sec-GPC` tylko do serwisów, o których wiadomo, że go respektują | w JavaScripcie i jako nagłówek `Sec-GPC: 1` do każdego serwisu |
| HTTPS | serwisy z listy DuckDuckGo | każda nawigacja; pyta przed wczytaniem serwisu bez HTTPS |
| Dane o użyciu wysyłane do twórcy | anonimowe metryki użycia do DuckDuckGo | brak |
| **Ukrywanie tożsamości** | | |
| Oddzielne tożsamości | Fire Windows: odizolowane okno, którego dane znikają po zamknięciu | profile i aplikacje webowe, każda w osobnym procesie z własnymi cookies i danymi; okno Tor nic nie zapisuje na dysku |
| Adres e-mail | Email Protection: darmowe adresy `@duck.com`, które przekazują pocztę z usuniętymi trackerami | brak |
| Adres IP | VPN w płatnej subskrypcji DuckDuckGo | okno Tor: za darmo, przez własnego Tora InDuckGo, z usługami onion i mostami; wolniejsze niż VPN |
| Strony oszustów i phishing | Scam Blocker: lista od Netcraft, sprawdzana na urządzeniu | brak |

W skrócie: InDuckGo blokuje więcej i daje serwisom mniej, po czym mogłyby go rozpoznać, ale rzadki silnik go
wyróżnia. Obie ukrywają adres IP: DuckDuckGo z płatnym VPN, InDuckGo darmowym oknem Tor. DuckDuckGo ukrywa też
adres e-mail, ostrzega przed stronami oszustów i ginie w tłumie użytkowników Safari. Źródła: strony pomocy DuckDuckGo o
[ochronie przed śledzeniem](https://duckduckgo.com/duckduckgo-help-pages/privacy/web-tracking-protections),
[Fire Windows](https://duckduckgo.com/duckduckgo-help-pages/privacy/fire-tabs),
[Email Protection](https://duckduckgo.com/duckduckgo-help-pages/email-protection/what-is-duckduckgo-email-protection),
[Scam Blocker](https://duckduckgo.com/duckduckgo-help-pages/threat-protection/scam-blocker) i
[anonimowych metrykach](https://duckduckgo.com/duckduckgo-help-pages/privacy/atb) oraz konfiguracja prywatności
DuckDuckGo dla macOS, którą InDuckGo pobiera.

---

## Przed czym chroni

| Ochrona | Jak działa |
|---|---|
| Trackery stron trzecich | Tracker Data Set DuckDuckGo jest tłumaczony na reguły blokowania treści WebKita (port `TrackerRadarKit`). |
| Psucie stron | Konfiguracja prywatności DuckDuckGo dostarcza wyjątki dla serwisów, żeby działały; przycisk tarczy (w aplikacji webowej: *Chroń tę witrynę* w menu) wyłącza ochronę dla jednego serwisu. |
| Fingerprinting, GPC | Content Scope Scripts DuckDuckGo działają w każdej ramce; InDuckGo Guard włącza obronę canvas, audio, ekranu i referera, którą DuckDuckGo zostawia wyłączoną, i wysyła nagłówek `Sec-GPC: 1`. |
| Reklamy, trackery, komunikaty o cookies | InDuckGo Guard pobiera listy EasyList, EasyPrivacy, uBlock Origin, Petera Lowe'a, polskie i listy komunikatów o cookies, i stosuje je także do własnych zapytań serwisu. |
| Ciasteczka między serwisami | Ciasteczka stron trzecich są odrzucane; włączone jest Intelligent Tracking Prevention WebKita. |
| Linki śledzące | Nawigacja z `utm_*`, `fbclid`, `gclid`, ... jest zatrzymywana, zanim opuści komputer, i ładowana od nowa bez nich. |
| Zwykłe HTTP | Nawigacje są podnoszone do HTTPS; jeśli serwis nie ma HTTPS, baner pyta przed niezabezpieczonym załadowaniem. |
| Pozostawione dane | Przycisk Fire jednym kliknięciem kasuje ciasteczka, magazyn, pamięć podręczną i favikony profilu. |
| Mieszanie tożsamości | Każdy profil i każda zainstalowana aplikacja webowa działa jako osobny proces z własnymi ciasteczkami i danymi. |
| Twój adres IP | Okno Tor wysyła wszystko przez własnego Tora InDuckGo; jego sesja żyje tylko w pamięci. |

Rozszerzeń nie ma — celowo, i dlatego, że WebKitGTK nie ma API rozszerzeń.

**Aplikacje webowe.** *Zainstaluj witrynę jako aplikację* (przycisk na prawym końcu paska adresu albo menu okna) zamienia
dowolny serwis — bank, pocztę — w aplikację na pulpicie z ikoną serwisu. Każda aplikacja ma własną,
odizolowaną sesję; link do innego serwisu kliknięty w aplikacji otwiera się w przeglądarce, więc logowanie
aplikacji nigdy nie spotyka obcych stron.

**Okno Tor.** Przycisk z cebulą na pasku narzędzi, *Nowe okno Tor* w menu (`Alt+Shift+N`) albo
`induckgo --tor` otwiera okno, którego każde żądanie
idzie przez Tora, więc serwisy widzą przekaźnik wyjściowy Tora zamiast Twojego adresu. InDuckGo uruchamia dla
niego własny `tor`, nigdy ogólnosystemową usługę Tor, i pokazuje postęp łączenia, aż Tor będzie gotowy; do
tego czasu żadna strona się nie wczytuje i nic nigdy nie przechodzi na połączenie bezpośrednie. Ciasteczka,
pamięć podręczna i ustawienia okna żyją tylko w pamięci, WebRTC jest wyłączone, wyszukiwanie idzie przez usługę
onion DuckDuckGo, a strony `.onion` się otwierają. Tam, gdzie Tor jest blokowany, *Połączenie z Tor…* oferuje
Snowflake, obfs4 (z wbudowanymi mostami Tor Browsera) albo własne mosty. Przycisk Fire staje się *Nową
tożsamością*: dane usunięte, nowe obwody Tora. Okno ukrywa Twój adres, ale nie to, że używasz InDuckGo: gdy liczy
się nieodróżnialność wśród użytkowników Tora, używaj Tor Browsera.

**Pasek narzędzi.** *Preferencje…* w menu (`Ctrl+,`) wybiera, które przyciski pasek pokazuje wokół paska
adresu (wstecz i dalej, odświeżanie, cebula okna Tor, tarcza ochrony, Fire, nowa karta, ikona instalacji),
osobno w każdym profilu.

---

## Jak wycinamy śledzenie i co to potwierdza

Strona, reklamodawca albo sam producent przeglądarki mogą Cię śledzić na siedem sposobów. Tabela nazywa każdy
z nich, mechanizm użyty przeciwko niemu przez InDuckGo, miejsce tego mechanizmu w kodzie i test lub pomiar,
który go sprawdza. Mechanizmy dotyczą edycji na WebKitGTK (1.1.x); edycja 1.2 na QtWebEngine jest odtwarzana
tak, by je powtórzyć (zob. [plan migracji](docs/pl/plans/2026-11-04-qtwebengine-migration-plan.md)),
i jeszcze nie ma ich wszystkich.

| Sposób śledzenia | Co robi InDuckGo | Kod (`crates/induckgo-core/src`) | Sprawdza to |
|---|---|---|---|
| Przeglądarka raportuje do producenta | W ogóle nie ma kodu telemetrii: żadnego pingu użycia, raportów awarii, ziarna eksperymentów ani konta. Jedyne połączenia, jakie program sam nawiązuje, to zwykłe `GET` po dane ochronne, bez ciasteczek i bez identyfikatora. | `protection/`, `guard/` (tylko pobieranie) | Ruch bezczynności zmierzony na poziomie nazw hostów: 8 hostów, żaden nie należy do InDuckGo ([raport §4.1](docs/pl/reports/tracking-comparison.md)) |
| Trackery stron trzecich | Tracker Data Set DuckDuckGo zamieniany jest na reguły blokowania treści WebKit, razem z wyjątkami DuckDuckGo dla poszczególnych witryn, by strony działały. | `protection/tracker_data_set.rs`, `protection/content_blocker_rules_builder.rs`, `network/tracker_request_policy.rs` | Testy scenariuszowe `tracker_data_set`, `content_blocker_rules_builder`, `rule_exceptions` |
| Reklamy i trackery, których pierwsza lista nie zna | Guard pobiera dziewięć list społeczności (EasyList, EasyPrivacy, uBlock Origin, Petera Lowe'a, polskie i listy komunikatów o ciasteczkach), kompiluje je i stosuje także do żądań własnej witryny. Komunikaty o ciasteczkach są ukrywane. | `guard/guard_engine.rs`, `guard/guard_rule_list.rs`, `network/guard_request_policy.rs` | Testy `guard_engine`, `guard_rule_list`, `guard_request_policy`; 94 → 2 trackery na 12 stronach (niżej) |
| Ciasteczka i zapisane identyfikatory | Ciasteczka stron trzecich są odrzucane, włączona jest Intelligent Tracking Prevention z WebKit, każdy profil i aplikacja internetowa to osobny proces z własnym magazynem, a Ogień kasuje profil jednym kliknięciem. | `storage/cookie_policy.rs`, `storage/tracker_cookie_policy.rs`, `storage/burn_planner.rs` | Testy `cookie_policies`, `burn_planner` |
| Identyfikatory w linkach | `utm_*`, `fbclid`, `gclid` i inne parametry z listy DuckDuckGo są usuwane z każdej nawigacji najwyższego poziomu, zanim żądanie opuści komputer. | `protection/tracking_parameter_remover.rs`, `network/link_cleaning_request_policy.rs` | Testy `tracking_parameter_remover`, `link_cleaning_request_policy` |
| Odciski przeglądarki | Content Scope Scripts DuckDuckGo działają w każdej ramce, a Guard włącza zabezpieczenia canvas, dźwięku, ekranu i referera, które konfiguracja DuckDuckGo na macOS zostawia wyłączone. Szum zmienia się z witryną i z sesją. | `guard/fingerprint_hardening.rs`, `protection/content_scope_script.rs` | Testy `fingerprint_hardening`, `content_scope_script`; strona próbkująca w raporcie (§4.3) |
| Co widzi sieć i serwer strony | Nawigacje są podnoszone do HTTPS, a baner pyta, zanim wczyta się strona bez HTTPS; `Sec-GPC: 1` trafia do każdej witryny. Dla adresu IP okno Tor kieruje wszystko przez własny Tor InDuckGo, nigdy nie wraca do połączenia bezpośredniego, nic nie zapisuje na dysku, wyłącza WebRTC i prosi o angielski jak Tor Browser. | `navigation/https_upgrade_policy.rs`, `network/https_upgrade_request_policy.rs`, `tor/` | Testy `https_upgrade_policy`, `https_upgrade_request_policy`, `tor_configuration`, `tor_endpoint`; [architektura §4.1](docs/pl/architecture/overview.md) |

Każda lista i zestaw danych jest pobierany, nigdy dołączany, a przed użyciem ograniczany rozmiarem i
walidowany. Lista filtrów może tylko blokować albo ukrywać: konwersja nie zachowuje skryptletów, więc złośliwa
lista mogłaby zepsuć stronę, ale nigdy nie uruchomi w niej kodu. Cały zestaw (`./run-tests`, 670 testów oraz
clippy i `cargo audit` w CI) uruchamia się przy każdym wypchnięciu.

### Dowody

Pomiar stojący za [pierwszą tabelą](#zmierzone-wobec-chrome-i-brave-porównane-z-duckduckgo) uruchomił
2026-11-02 obok siebie Chrome 156, Brave 1.98 i InDuckGo, na świeżych profilach, każdy we własnym oknie, przez
proxy zapisujące wyłącznie nazwy hostów. Liczby, powtórzone tu obok mechanizmów:

- **Trackery osiągnięte na 12 stronach:** Chrome 94, Brave 5, InDuckGo 2, według listy Disconnect, której
  żadna z trzech przeglądarek nie używa.
- **Wysłane do producenta w 5 minut bezczynności:** Chrome łączył się wyłącznie z Google (15 hostów), Brave
  głównie z własnymi serwerami (dzienny ping użycia, P3A), InDuckGo z niczym własnym.
- **Odcisk przeglądarki:** te same wartości canvas i dźwięku na każdej stronie w Chrome; różne dla witryn, ale
  takie same po restarcie w Brave; różne dla witryny *i* sesji w InDuckGo.
- **Metoda, ograniczenia i przepis na powtórzenie pomiaru:** [porównanie śledzenia](docs/pl/reports/tracking-comparison.md), §3 i §8.

### Czego to nie dowodzi

Pomiar potwierdza, że „InDuckGo daje trackerom i własnemu producentowi mniej materiału”. Nie potwierdza
bez zastrzeżeń, że „InDuckGo jest bardziej anonimowy niż Brave lub DuckDuckGo”, i projekt tego nie twierdzi.

- **2 trackery wobec 5 w Brave to jedno uruchomienie na 12 stronach.** Sam raport uznaje różnicę jednego–dwóch
  hostów za mieszczącą się w zmianach, jakie strony wprowadzają między wczytaniami. Czytaj to jako „co najmniej
  tak dobrze jak Brave”, a nie jako ranking.
- **Przeglądarki DuckDuckGo nie zmierzono**, bo nie istnieje na Linuksa.
  [Tabela funkcji](#porównanie-z-przeglądarką-duckduckgo) porównuje udokumentowane zachowanie, nie ruch.
- **Blokowanie to nie anonimowość.** Silnik InDuckGo, WebKitGTK na Linuksie, ma niewielu użytkowników, więc
  łatwiej go wyróżnić niż Brave (wygląda jak Chrome) albo DuckDuckGo na macOS (wygląda jak Safari). Szum zmienia
  wartości, które widzi strona, ale nie silnik. Zainstalowane czcionki pozostają widoczne, tak jak w Brave i
  Chrome.
- **Brak ostrzeżeń przed phishingiem i złośliwym oprogramowaniem** (brak Safe Browsing), brak aliasów
  e-mail i VPN. Okno Tor ukrywa Twój adres, nie fakt używania InDuckGo; gdy liczy się nieodróżnialność, użyj
  Tor Browser.
- **Brakuje niezależnej weryfikacji:** [privacytests.org](https://privacytests.org/) nie testuje InDuckGo.

---

## Stan

Projekt w toku, ale używalny: działają karty, wszystkie powyższe zabezpieczenia, profile, aplikacje webowe,
okno Tor, pobieranie plików, pytania o uprawnienia i wyszukiwanie na stronie. Jeszcze nie ma: przywracania sesji,
zakładek, okna historii, obsługi zgód na cookies. Pełna lista z uzasadnieniem jest w dokumencie
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
bezpieczeństwa Twojej dystrybucji. Na potrzeby okna Tor paczki dla Arch i Fedory dociągają `tor` z
dystrybucji (żadna usługa Tor nie jest uruchamiana); `.deb` niesie Debianowy `tor` sam, bo paczka Debiana
uruchamiałaby ogólnosystemową usługę Tor, i poleca `obfs4proxy` oraz `snowflake-client` dla mostów; AppImage
niesie wszystkie trzy. AppImage nie wymaga instalacji:
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

Wypchnięcie taga wersji publikuje wydanie: `git tag vX.Y.Z && git push origin vX.Y.Z`.
[Workflow wydania](.github/workflows/release.yml) uruchamia te same skrypty (paczkę Archa w kontenerze Arch
Linux), odrzuca zależności ze znanymi podatnościami (`cargo audit`) i wysyła cztery paczki z `SHA256SUMS` oraz
atestacjami. Tag musi zgadzać się z wersją w `Cargo.toml`.

---

## Uruchamianie ze źródeł

```sh
sudo pacman -S --needed rust webkitgtk-6.0 gtk4 libadwaita tor
cargo run -p induckgo                          # przeglądarka, profil domyślny
cargo run -p induckgo -- --profile work        # przeglądarka, profil "work"
cargo run -p induckgo -- https://example.com/  # otwórz stronę
cargo run -p induckgo -- --tor                 # okno Tor
```

Zainstalowane aplikacje webowe uruchamia się z menu aplikacji pulpitu (`induckgo --web-app <id>`).

---

## Testy

Stos: **cargo-nextest** — 670 obiektowych scenariuszy Given/When/Then: 600 dla domeny w
`crates/induckgo-core/tests/` (72 pliki, jeden na typ domenowy), 61 dla narzędzia raportu w
`crates/induckgo-test-report/tests/` i 9 dla narzędzia benchmarków w `crates/induckgo-bench/tests/`.

```sh
./run-tests                     # wszystkie testy + raport HTML w tests/reports/ (otwiera się w przeglądarce)
cargo test -p induckgo-core     # tylko testy domeny (szybkie, bez GTK i WebKita)
cargo clippy --all-targets      # linty (pedantic, unsafe zabroniony)
cargo fmt --check               # formatowanie
cargo audit                     # podatności zależności (polityka zero podatności)
```

**Raport testów:** `./run-tests` zawsze zapisuje `tests/reports/test-report-YYYYMMDD-HHmm.html` — dashboard
historii przebiegów z ostatnim przebiegiem, ostatnimi 20 przebiegami, rankingami błędów, funkcjonalnościami i
najwolniejszymi testami, a gdy zainstalowany jest `cargo-llvm-cov`, także z pokryciem `induckgo-core`. Zobacz
[Raport testów HTML](docs/pl/tests/test-report.md).

**Architektura testów:**
- Domena (`induckgo-core`) nie zależy od GTK ani WebKita, więc każda decyzja ochrony — budowanie reguł,
  konwersja list filtrów, podnoszenie do HTTPS, czyszczenie linków, profile, aplikacje webowe, zoom — jest
  testowana bez ekranu.
- Każdy scenariusz to obiekt: `given_…` go buduje, `when_…` wykonuje akcję i zwraca wynik, `then_…` sprawdza
  jedną rzecz, zgodnie z [Object Oriented Programming First](docs/pl/engineering-rules.md):
  `LinkCleaningScenario::given_tracking_parameters_protection().when_link_is_opened(url).then_opens_unchanged()`.
- Nieudany test jest powtarzany raz: jeśli wtedy przejdzie, raport oznacza go jako FLAKY — to błąd do
  naprawy, nigdy do zignorowania.
- [Workflow testów](.github/workflows/tests.yml) uruchamia fmt, clippy, `cargo audit` i `./run-tests` przy
  każdym pushu do `main` i przy każdym pull requeście, i zachowuje raport każdego przebiegu jako artefakt.

---

## Struktura projektu

```
crates/induckgo-core/    logika domeny bez GTK/WebKita: reguły ochrony, listy filtrów Guarda,
                         profile, aplikacje webowe, zoom, uruchamianie
crates/induckgo-core/tests/
                         jeden plik scenariuszy OO na typ domenowy
crates/induckgo/         aplikacja GTK 4 + libadwaita: adaptery wykonujące decyzje domeny w
                         WebKitGTK, libsoup i GIO
crates/induckgo-test-report/
                         narzędzie deweloperskie za ./run-tests: raport testów HTML (nie trafia do paczek)
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
