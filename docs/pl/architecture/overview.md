# Architektura InDuckGo

Ten dokument opisuje, jak zbudowane jest InDuckGo i dlaczego właśnie tak. Jest pisany dla osoby, która
programuje, ale nigdy nie pracowała nad przeglądarką. Każde pojęcie spoza wiedzy ogólnej jest wyjaśnione w
[Słowniku](#2-słownik).

## 1. Problem

Większość stron wczytuje kod firm, które zarabiają na śledzeniu ludzi między witrynami: analitykę, reklamy
i „społecznościowe” widżety. Na Linuksie przeglądarki, które dobrze to blokują, bazują na Chromium (Brave)
albo Firefoksie. Obie przyjmują rozszerzenia, a rozszerzenie samo w sobie może czytać każdą odwiedzaną
stronę.

DuckDuckGo publikuje jako open source ochronę przed śledzeniem ze swojej przeglądarki dla macOS: listę
trackerów, reguły decydujące, co blokować, wyjątki per witryna, dzięki którym strony działają, oraz skrypty
przeciw fingerprintingowi. Tamta przeglądarka używa silnika **WebKit**, a Linux ma ten sam silnik jako
**WebKitGTK**. InDuckGo przenosi tę ochronę do przeglądarki WebKitGTK na Linuksa, **całkowicie bez obsługi
rozszerzeń**.

Projekt kształtują jeszcze dwa cele:

- **Izolowane profile.** „Praca” i „Prywatny” nigdy nie dzielą cookies, logowań ani cache.
- **Każda strona jako aplikacja.** mBank, Gmail czy dowolną inną stronę można zainstalować jako osobną
  aplikację pulpitu, z ikoną w menu aplikacji. Każda aplikacja ma własną sesję, więc link phishingowy kliknięty
  w przeglądarce nigdy nie dosięgnie logowania do banku. Aplikacje webowe (PWA) w Brave dzielą cookies z
  profilem przeglądarki, co daje słabszą izolację.

## 2. Słownik

| Pojęcie | Znaczenie |
|---|---|
| WebKitGTK | Linuksowe wydanie WebKit, silnika, który pobiera, renderuje i uruchamia strony. InDuckGo używa jego API dla GTK 4, `webkitgtk-6.0`. |
| Reguła blokowania treści | Reguła JSON, którą WebKit kompiluje i sprawdza, zanim żądanie opuści komputer: „jeśli URL pasuje do wzorca, zablokuj go” (albo podnieś do HTTPS). Reguły są grupowane w *listy reguł*. |
| Tracker Data Set (TDS) | Lista domen śledzących DuckDuckGo: kto jest ich właścicielem i które ich żądania są dozwolone. Licencja CC BY-NC-SA 4.0, więc InDuckGo pobiera ją w trakcie działania i nigdy jej nie dystrybuuje. |
| Konfiguracja prywatności | „Rozdzielnia” DuckDuckGo: która ochrona jest włączona i na których witrynach musi być wyłączona, bo je psuje (*wyjątki*). |
| Content Scope Scripts (C-S-S) | JavaScript DuckDuckGo działający w każdej stronie i ramce: blokuje fingerprinting i ogłasza GPC. Apache-2.0, wersja 20.0.0 leży w `assets/`. |
| GPC (Global Privacy Control) | Sygnał „nie sprzedawaj i nie udostępniaj moich danych”; w niektórych stanach USA wiążący prawnie. |
| ITP (Intelligent Tracking Prevention) | Wbudowane w WebKit ograniczenia cookies i magazynu witryn uznanych za trackery. |
| Upgrade HTTPS | Wczytanie `https://`, gdy strona prosi o `http://`, żeby nikt w sieci nie mógł czytać ani zmieniać strony. |
| Parametr śledzący | Parametr zapytania dopisywany do linków, by śledzić kliknięcie, np. `utm_source`, `fbclid`, `gclid`. |
| Profil | Osobna tożsamość przeglądarki z własnymi cookies, magazynem, cache, historią i uprawnieniami witryn. |
| Aplikacja webowa (PWA) | Strona zainstalowana jako aplikacja pulpitu: własne okno, ikona i pozycja w menu. |
| Sufiks publiczny, domena rejestrowalna | `co.uk` i `github.io` to *sufiksy publiczne*: każdy może zarejestrować nazwę pod nimi. *Domena rejestrowalna* to sufiks plus jedna etykieta (`bbc.co.uk`). Dwa hosty z tą samą domeną rejestrowalną to *ta sama witryna*. |
| Identyfikator aplikacji | Nazwa D-Bus działającej aplikacji GTK, np. `io.github.induckgo.InDuckGo`. GTK uruchamia tylko jeden proces na identyfikator. |
| Plik desktop | Plik `.desktop` w `~/.local/share/applications`, który dodaje aplikację do menu pulpitu. |
| Ogień (Fire) | Przycisk, który jednym kliknięciem usuwa dane przeglądania profilu. |

## 3. Dwa crate'y

Kod jest podzielony na dwa crate'y Rust, a zależność biegnie w jedną stronę: `induckgo` → `induckgo-core`.

- **`crates/induckgo-core`** zawiera logikę domenową: budowanie list reguł z TDS, czytanie konfiguracji
  prywatności, decyzje o upgrade HTTPS, wykrywanie parametrów śledzących, parsowanie linii poleceń, katalogi
  profili i aplikacji webowych, wybór ikon, generowanie plików desktop i nazw pobranych plików. Nie wie nic o
  GTK ani WebKit, dlatego każdą regułę sprawdzają szybkie testy jednostkowe (`cargo test -p induckgo-core`).
- **`crates/induckgo`** to aplikacja GTK 4 + libadwaita. Jej typy są adapterami: biorą decyzję od rdzenia i
  wykonują ją przez WebKitGTK, libsoup (HTTP) albo GIO (pliki, procesy).

Zasady inżynierskie obu crate'ów (Object Oriented Programming First, obiektowe testy Given/When/Then, jeden
`use` na element) są w [engineering-rules.md](../engineering-rules.md).

## 4. Procesy i izolacja

![Każdy profil i każda aplikacja webowa to osobny proces InDuckGo z własną sesją WebKit; wspólny, tylko do odczytu cache danych ochrony; drugie uruchomienie przekazywane przez D-Bus; link do innej witryny kliknięty w aplikacji webowej otwiera się w przeglądarce](../../assets/diagrams/architecture-process-isolation.pl.svg)

Każdy profil i każda zainstalowana aplikacja webowa działa jako **osobny proces z własnym identyfikatorem
aplikacji**:

| Co działa | Identyfikator aplikacji | Katalog profilu |
|---|---|---|
| Przeglądarka, profil domyślny | `io.github.induckgo.InDuckGo` | `profiles/default` |
| Przeglądarka, profil „work” | `io.github.induckgo.InDuckGo.Profile.work` | `profiles/work` |
| Aplikacja webowa „mBank” | `io.github.induckgo.InDuckGo.WebApp.mbank_pl` | `profiles/app_mbank_pl` |

`LaunchRequest` (rdzeń) zamienia linię poleceń `induckgo [--profile NAZWA | --web-app ID] [URL …]` na jeden z
tych identyfikatorów. GTK pyta następnie D-Bus, czy ten identyfikator już działa. Jeśli tak, nowa linia
poleceń trafia do działającego procesu (który otwiera URL w oknie), a nowy proces kończy pracę.

Każdy proces ma dokładnie jedną *sesję sieciową* WebKit, zapisaną w katalogach profilu
(`~/.local/share/induckgo/profiles/<id>` i `~/.cache/induckgo/profiles/<id>`). Są tam baza cookies, local
storage, IndexedDB, cache, favikony i wpisy HSTS. Cookies stron trzecich są odrzucane, a ITP jest włączone.
Nic z tej sesji nie jest widoczne dla innego profilu ani aplikacji.

**Dlaczego proces na profil, a nie kilka sesji w jednym procesie?** Granica procesu systemu operacyjnego to
najsilniejsza separacja, jaką Linux daje bez kontenerów: awaria, wyciek pamięci czy błąd WebKit w aplikacji
banku nie dotknie przeglądarki. Pulpit widzi też każdą aplikację jako osobny program (własna ikona w doku,
grupowanie okien przez `StartupWMClass`), a tego właśnie obiecuje „zainstaluj jako aplikację”.

Jedyne wspólne dane to **kanał ochrony** w `~/.cache/induckgo/protections`: TDS i konfiguracja prywatności.
Czyta je każdy proces. Pobranie podmienia plik atomowo (zapis `nazwa.part`, potem zmiana nazwy), więc proces
nigdy nie przeczyta pół pliku.

Przed usunięciem profilu albo odinstalowaniem aplikacji InDuckGo pyta D-Bus, czy jej identyfikator nadal
działa (`RunningInstances`), i odmawia usunięcia danych, których używa żywy proces.

## 5. Od danych DuckDuckGo do chronionych stron

![Tracker Data Set i konfiguracja prywatności są pobierane z CDN DuckDuckGo, walidowane i zapisywane w cache; rdzeń zamienia je na trzy listy reguł i skrypt content scope; aplikacja kompiluje listy w magazynie filtrów WebKit i instaluje je razem ze skryptem w każdej stronie](../../assets/diagrams/architecture-protection-pipeline.pl.svg)

### 5.1 Pobieranie danych

`ProtectionFeed` pobiera `macos-tds.json` i `macos-config.json` z `staticcdn.duckduckgo.com` przez `HttpClient`
(libsoup, biblioteka HTTP, której używa sam WebKitGTK). Sieć jest traktowana jako wroga:

- żądanie nie niesie cookies, ma limit czasu 30 sekund i przestaje czytać po 32 MiB;
- odpowiedź zastępuje kopię w cache tylko wtedy, gdy jest poprawnym JSON z oczekiwanym kluczem najwyższego
  poziomu (`trackers`, `features`);
- odświeżenia wysyłają zapisany `ETag`, więc niezmieniony plik kosztuje jedno małe żądanie. Odświeżanie
  działa co 12 godzin, a co 30 sekund, dopóki profil nie ma żadnych danych ochrony.

Strony nie wczytują się, dopóki ochrona nie jest aktywna (`ProfileSession::when_protected`), więc pierwsza
strona nowego profilu nigdy nie wczyta się bez ochrony.

### 5.2 Trzy listy reguł

Akcja `ignore-previous-rules` w WebKit (wyjątek) anuluje tylko reguły *z tej samej listy*. Dlatego InDuckGo
buduje trzy osobne listy, żeby wyjątki jednej funkcji nie wyłączyły innej:

1. **`trackers`**: `ContentBlockerRulesBuilder`, port `TrackerRadarKit` z DuckDuckGo, blokuje żądania stron
   trzecich do domen śledzących. Respektuje reguły zezwalające TDS, wyjątki `contentBlocking` z konfiguracji,
   allowlistę trackerów oraz witryny, dla których użytkownik wyłączył ochronę (przycisk tarczy).
2. **`https`**: `HttpsUpgradePolicy` tworzy jedną regułę `make-https` dla każdego URL `http://`, a po niej
   wyjątki dla adresów IP, nazw jednoczłonowych i lokalnych (`localhost`, `.local`, `.lan`, `.internal`,
   `.home.arpa`), wyjątków `https` z konfiguracji i hostów, dla których upgrade wcześniej zawiódł. Każdy
   wzorzec wyjątku kończy się na hoście i opcjonalnym porcie, więc dane logowania w URL nie mogą udawać
   zwolnionego hosta: `http://localhost:1@example.com/` prowadzi do `example.com` i jest podnoszony do HTTPS.
3. **`links`**: `TrackingParameterRemover` tworzy jedną regułę na parametr śledzący. Reguła blokuje nawigację
   *dokumentu w ramce głównej*, która go niesie, poza witrynami zwolnionymi przez konfigurację.

`PageProtections` kompiluje każdą listę przez `UserContentFilterStore` WebKit. Kompilacja listy trackerów
trwa około sekundy, więc wynik jest trzymany na dysku obok odcisku (hasha) źródła. Niezmieniona lista jest
wczytywana zamiast kompilowana.

Nawet niezmieniona lista trackerów kosztuje: zbudowanie jej 7000 reguł i serializacja (4,7 MB JSON-a) tylko po
to, by policzyć odcisk, zajmuje 0,1–0,2 s procesora, a na obciążonej maszynie więcej. Ta praca idzie na wątek
roboczy (`gio::spawn_blocking`), a start na nią nie czeka. Tak jak aplikacja DuckDuckGo na macOS, sesja
najpierw włącza listę trackerów skompilowaną poprzednio, więc strony ładują się od razu, a nową listę podmienia,
gdy jest gotowa i się różni. Listy `https` i `links` są zawsze budowane od nowa przed pierwszą stroną. Zajmują
znacznie poniżej milisekundy, a zapisana lista `https` mogłaby wciąż zwalniać hosty, które użytkownik pozwolił
wczytać bez szyfrowania w poprzedniej sesji. Tylko pierwszy w historii start, bez niczego skompilowanego, czeka
na całą kompilację.

### 5.3 Reakcja na zablokowane i nieudane wczytania

Reguły działają, zanim żądanie opuści komputer. Dwie z nich potrzebują potem pomocy strony (`PageView`):

- **Linki śledzące.** Gdy lista `links` zablokuje nawigację, WebKit zgłasza nieudane wczytanie z błędem
  polityki. `PageView` wczytuje wtedy ten sam adres bez parametrów śledzących. Strażnik pamięta ostatni
  oczyszczony adres, więc niezgodność reguły i czyściciela nigdy nie spowoduje pętli.
- **Witryny bez HTTPS.** Gdy podniesione wczytanie zawiedzie, baner informuje, że „*host* nie oferuje
  bezpiecznego połączenia”, i pokazuje przycisk **Load Insecure Page**. Dopiero po tym kliknięciu host trafia
  do nieudanych hostów profilu, lista `https` jest przebudowana i wczytywany jest `http://`. Ciche cofnięcie do
  HTTP pozwoliłoby atakującemu w sieci zepsuć HTTPS dowolnej witryny i zmusić ją do zwykłego HTTP.

**Strony błędów.** Każda inna porażka dostaje stronę błędu InDuckGo zamiast wbudowanej strony WebKita, która
jest gołym nagłówkiem na białym tle. WebKit, libsoup i GLib zgłaszają porażki kodami błędów z kilku domen:
resolvera, wejścia-wyjścia, TLS oraz własnych błędów sieci i polityk WebKita. `ErrorPageView` (aplikacja)
zamienia te kody na kilka przyczyn, na które człowiek może zareagować (`FailureCause` w rdzeniu):

- brak sieci;
- nieznany host;
- połączenie odrzucone, przekroczony czas albo przerwane połączenie;
- zastrzeżony port;
- niezaufany certyfikat, z każdym powodem;
- awaria strony albo brak pamięci;
- status błędu HTTP z pustą treścią;
- brak pliku.

Strona nazywa problem jednym nagłówkiem, mówi, której witryny dotyczy, i podaje, co sprawdzić. Oferuje tylko
pomocne akcje: Try Again albo Reload, Search DuckDuckGo dla nieistniejącego hosta i Go to Home Page dla
pustego 404. Jest rysowana w bieżącym jasnym albo ciemnym stylu przeglądarki, w kolorze akcentu pulpitu.
Strona jest wczytywana jako *alternate HTML* WebKita dla nieudanego adresu, więc pasek adresu, Reload i Wstecz
działają jak dla strony, która zawiodła.

Ważne są cztery szczegóły:

- Status błędu z `Content-Length: 0` jest zatrzymywany na etapie polityki odpowiedzi. Wczytanie wtedy się
  nie udaje, a jego strona błędu zajmuje wpis w historii jak przy każdym nieudanym wczytaniu. Pokazana dopiero
  po pustej stronie, dodałaby drugi wpis, a Wstecz wracałoby do pustej strony i objaśniało ją od nowa, bez
  końca.
- Porażki certyfikatu przychodzą przez `load-failed-with-tls-errors` ze wszystkimi powodami znalezionymi przez
  GLib. Strona wymienia je wszystkie. Nie ma „przejdź mimo to”: WebKit nigdy nie wczyta takiej strony, a
  obejście uczyłoby użytkowników przeklikiwać ostrzeżenie, na którym polega atakujący.
- Połączenie z serwerem, który nigdy nie odpowiada, libsoup anuluje (`G_IO_ERROR_CANCELLED`), zamiast zgłosić
  przekroczenie czasu. Dlatego jest objaśniane jako przekroczony czas, a wczytanie zatrzymane przez
  użytkownika (`WEBKIT_NETWORK_ERROR_CANCELLED`) nie pokazuje niczego.
- Nad stroną błędu pasek adresu nie pokazuje kłódki ani przycisku instalacji, bo strona niczego nie dowodzi
  o połączeniu. Zwykłe HTTP zachowuje ostrzeżenie: strona działa w pochodzeniu witryny, więc inne okna tej
  witryny nadal mogą ją przepisać.

Strona działa w pochodzeniu nieudanej witryny, więc nie może nikomu zaszkodzić. Wszystko, co pochodzi z
porażki, jest escapowane w HTML: adres, host i komunikat GLib. Szablon jest wypełniany w jednym przebiegu, więc
wartość zawierająca `{{…}}` nigdy nie zostanie rozwinięta. Content Security Policy (`default-src 'none'`)
zabrania skryptów, formularzy i wszelkich zdalnych wczytań. Linki ponowienia powstają tylko dla adresów
`http`, `https` i `file`.

### 5.4 Content Scope Scripts, GPC i cookies

`ContentScopeScript` wypełnia szablon C-S-S konfiguracją prywatności, niechronionymi witrynami użytkownika i
preferencjami (GPC włączone, losowy klucz sesji i sekret wiadomości na proces, język interfejsu). Skrypt
działa na starcie dokumentu w każdej ramce, w świecie JavaScript strony, tak jak w aplikacji DuckDuckGo na
macOS.

Każda ramka parsuje osadzoną konfigurację od nowa, dlatego `ContentScopeScript` osadza tylko funkcje, które
uruchamia dołączony build `apple` (`platformSupport.apple` i ich `featureConfigDependencies`). Bundle sam
odrzuca wszystkie pozostałe funkcje na samym początku (`pruneConfigFeatures`), więc wynik jest identyczny, ale
każda ramka czyta teraz ok. 165 KB konfiguracji zamiast 840 KB (sam `autoconsent` to 520 KB), a skrypt maleje z
1,36 MB do 0,67 MB. Test odczytuje ten zbiór funkcji z dołączonego bundla, więc aktualizacja C-S-S nie może po
cichu wyłączyć żadnej funkcji. Wiadomości skryptu trafiają do handlera `contentScopeScripts`. Handler odpowiada `{}` na powiadomienia i
obiektem błędu na żądania, bo InDuckGo nie implementuje natywnych funkcji, o które proszą.

### 5.5 Ogień

Przycisk Ognia (w nagłówku każdego okna) zamyka pozostałe okna profilu i usuwa wszystkie dane witryn profilu
(cookies, magazyn, cache, favikony, zoom nadany witrynom) z wyjątkiem wpisów **HSTS**. Te pamiętają, które witryny wymagają HTTPS,
a ich zapomnienie tylko osłabiłoby następną wizytę. Bieżące okno zaczyna od nowa z pustą kartą.

## 6. Aplikacje webowe

**Instalacja.** „Install Site as App” (przycisk na prawym końcu paska adresu na każdej stronie WWW albo menu
okna) czyta tytuł strony, link do manifestu i linki do ikon skryptem
działającym w *izolowanym świecie JavaScript*, którego sama strona nie widzi i nie może zmienić. Manifest i
ikony są pobierane bez cookies, z limitami rozmiaru (manifest 512 KiB, ikona 2 MiB). Dekodowane są tylko ikony
PNG i tylko po tym, jak `PngHeader` potwierdzi rozsądne wymiary (najwyżej 2048 px na bok), co zatrzymuje
„bomby dekompresyjne”. Wybrana ikona jest ponownie kodowana do PNG i zapisywana. Gdy żadna ikona się nie
pobierze, używana jest favikona, którą WebKit już zdekodował. Wynikiem jest wpis w katalogu, ikona i plik
desktop, którego linia `Exec=` uruchamia `induckgo --web-app <id>`.

Po instalacji aplikacja się uruchamia, a przeglądarka przekazuje jej witrynę: każde okno przeglądarki tego
profilu zamyka kartę, z której zaczęła się instalacja, i każdą kartę ze stroną aplikacji
(`InstalledWebApp::is_own_page`: to samo pochodzenie, czyli schemat, host i port, co adres startowy aplikacji).
Okno, w którym nie została żadna karta, się zamyka. Test jest celowo węższy niż witryna aplikacji: aplikacja Gmail
posiada `mail.google.com`, więc karta Dokumentów czy Kalendarza Google z tej samej witryny zostaje w przeglądarce.
Gdy aplikacji nie udało się uruchomić, nic się nie zamyka, a komunikat proponuje jej otwarcie.

**Nawigacja.** `WebAppNavigationPolicy` (rdzeń) decyduje, dokąd może prowadzić nawigacja w aplikacji:

- **link kliknięty przez użytkownika** do innej *witryny* (domeny rejestrowalnej) otwiera się w zwykłej
  przeglądarce, więc linki z maili i reklamy nigdy nie działają w sesji banku;
- **przekierowania i nawigacje skryptów** zostają, bo logowanie (Gmail przez `accounts.google.com`) i
  płatności kartą (strony 3-D Secure) przeskakują między witrynami;
- **wyskakujące okna** otwierane przez aplikację (OAuth, potwierdzenie płatności) otwierają się w małym oknie
  tej samej aplikacji. Dzielą jej sesję, więc mają tę samą bramkę, mierzoną względem witryny, dla której okno
  otwarto (linki na stronie 3-D Secure działają, kliknięty link do trzeciej witryny otwiera się w
  przeglądarce), i ten sam wskaźnik witryny w nagłówku.

Przekazanie strony do przeglądarki zamyka okno, *z którego strona się przeniosła*, i tylko je. **Open Page in
Browser** przenosi bieżącą stronę aplikacji, więc okno aplikacji się zamyka. Wyskakujące okno, którego własna
strona trafia do przeglądarki, też się zamyka, bo pokazywałoby już tylko stronę, którą opuściło. Okno aplikacji
zostaje, gdy link w nim otwiera się w przeglądarce, bo aplikacja nadal jest na swojej stronie (skrzynka pocztowa
musi przetrwać kliknięcie linku w wiadomości). Zostaje też wyskakujące okno, którego link prosił o *nowe* okno
(`target=_blank`), bo jego strona, np. potwierdzenie płatności, jest wciąż potrzebna. Okno zamyka się dopiero
po uruchomieniu przeglądarki.

Przekierowanie może wyprowadzić aplikację poza jej witrynę, dlatego podtytuł okna zawsze pokazuje host
widocznej strony. Gdy strona należy do innej witryny, podtytuł zmienia kolor na ostrzegawczy i dostaje
wyjaśniający tooltip. Aplikacja banku nie może więc niepostrzeżenie pokazać obcej strony pod nazwą banku.

Test „ta sama witryna” używa listy sufiksów publicznych z libsoup, tej samej, której WebKit używa dla cookies,
więc aplikacja zgadza się z WebKit co do tego, czym jest witryna. Wywoływana jest tylko
`soup::tld_domain_is_public_suffix`. Wiązanie `soup::tld_get_base_domain` z crate'a `soup3` (0.9) czyta wynik
przez wskaźnik do tymczasowego, już zwolnionego napisu i wywracało aplikacje webowe przy pierwszej nawigacji.

## 7. Pobieranie plików, uprawnienia i inne aplikacje

- **Pobrane pliki** trafiają do katalogu XDG Downloads pod oczyszczoną nazwą (`DownloadFileName`: bez
  separatorów ścieżek, znaków sterujących i kropek na początku, najwyżej 255 bajtów, `nazwa (2).ext` zamiast
  nadpisania). Nigdy nie są otwierane automatycznie; komunikat proponuje „Show in Folder”.
- **Uprawnienia** (lokalizacja, powiadomienia, schowek, kamera, mikrofon, udostępnianie ekranu, dostęp do
  magazynu) są za każdym razem pytane w oknie dialogowym i nigdy nie są przyznawane po cichu. Blokada kursora
  jest dozwolona, wszystko inne odrzucane.
- **Inne schematy** (`mailto:`, `tel:`, `zoommtg:` …) są przekazywane do pulpitu tylko po prawdziwym geście
  użytkownika, więc strona nie może sama otwierać aplikacji.
- **Zoom strony** przechodzi przez rozmiary DuckDuckGo (`ZoomLevel`: 50, 75, 85, 100, 115, 125, 150, 175, 200, 250
  i 300 %) skrótami Ctrl `+`/`-`/`0` (także na klawiaturze numerycznej), Ctrl z kółkiem myszy albo rzędem zoomu
  w menu okna. Rozmiar inny niż 100 % widać jako znaczek obok paska adresu (w aplikacji webowej w nagłówku);
  kliknięcie wraca do 100 %. Jak w przeglądarce DuckDuckGo na macOS rozmiar należy do *witryny*
  (`SiteZoomLevels`, klucz to domena rejestrowalna, zapis per profil w `site-zoom-levels.json`): każda strona
  witryny otwiera się w nim, a jego zmiana od razu przeskalowuje pozostałe otwarte karty tej witryny. Bez tego
  WebKit przenosiłby zoom jednej karty na każdą witrynę, którą karta odwiedzi później. Strony spoza sieci
  (`file:`, `about:`) otwierają się w 100 % i nie są zapamiętywane. Ctrl z kółkiem jest przechwytywany, zanim
  zobaczy go strona, więc strona (np. mapa) nie może użyć tego gestu dla siebie; rozsunięcie palców na touchpadzie
  nadal powiększa stronę tak, jak robi to WebKit.

## 8. Co WebKitGTK potrafi, a czego nie (zmierzone na 2.54)

Powyższy projekt wynika z tych pomiarów. Każdy sprawdzono na działającym WebKitGTK 2.54, zanim kod zaczął na
nim polegać.

| Ustalenie | Konsekwencja |
|---|---|
| Sygnał `decide-policy` nie odróżnia nawigacji ramki głównej od iframe i nie pokazuje nagłówków `Sec-Fetch-*`. | Upgrade HTTPS i czyszczenie linków działają przez reguły treści, nie w haku nawigacji. |
| `make-https` działa przed wysłaniem żądania, we wszystkich ramkach. | To jest mechanizm upgrade HTTPS. |
| `redirect` (z `remove-params`) i `modify-headers` kompilują się, ale nic nie robią. | Parametry śledzące są blokowane, a strona wczytuje oczyszczony URL; nie da się wysłać nagłówka `Sec-GPC`. |
| `block` z `resource-type: document` i `load-context: top-frame` blokuje tylko nawigacje ramki głównej, przed żądaniem, i zgłasza błąd polityki 104. | Tak działają reguły `links`. |
| Dla wczytań ramki głównej `unless-domain` jest porównywane z celem. | Wyjątki reguł `links` wskazują witryny docelowe. |
| `ignore-previous-rules` sięga wstecz tylko w obrębie własnej listy. | Trzy osobne listy. |
| Wyłączenie `gtk-enable-animations` wewnątrz aplikacji nie zmienia stronom `prefers-reduced-motion`. | Animacje interfejsu (karty, dialogi, banery, toasty, pasek wyszukiwania) są wyłączone w każdym procesie InDuckGo, więc interfejs reaguje od razu, a witryny nie dowiadują się z tego niczego nowego o użytkowniku. |

## 9. Znane ograniczenia

| Ograniczenie | Dlaczego | Wrócić do tego, gdy |
|---|---|---|
| Brak nagłówka HTTP `Sec-GPC`; GPC jest ogłaszane tylko przez `navigator.globalPrivacyControl`. | WebKitGTK ignoruje reguły `modify-headers` i nie daje haka na nagłówki żądań w procesie UI. | WebKitGTK zaimplementuje `modify-headers`. |
| Parametry śledzące nie są usuwane, gdy strona, która zaczęła nawigację, jest na liście wyjątków (aplikacja macOS sprawdza inicjatora). | Reguły treści widzą tylko cel. | WebKitGTK udostępni inicjatora w regułach lub w haku nawigacji. |
| Formularz `POST`, którego URL akcji ma parametr śledzący, jest ponawiany jako `GET` oczyszczonego URL. | Zablokowana nawigacja jest wczytywana ponownie przez `load_uri`. | Pojawi się zgłoszenie realnej awarii. |
| Odpowiedzi na prośby o uprawnienia nie są zapamiętywane. | Pytanie za każdym razem to bezpieczna wartość domyślna. | Powstanie projekt przechowywania uprawnień per profil. |
| Brak przywracania sesji, zakładek i okna historii; brak automatycznej obsługi zgód na cookies (autoconsent DuckDuckGo). | Jeszcze niezbudowane. | Następny etap. |
| Interfejs jest tylko po angielsku. | Brak katalogu tłumaczeń. | Wejdzie gettext. |
| Każdy profil jest w doku osobną aplikacją. | Jeden identyfikator aplikacji na proces profilu. | Użytkownicy będą woleli grupowanie. |
| Logowanie, które w aplikacji webowej zaczyna się od *klikniętego linku* do innej witryny, otwiera się w przeglądarce. | Kliknięte linki poza witrynę celowo opuszczają aplikację (phishing). Większość logowań używa przekierowań lub okien i działa. | Realna aplikacja będzie tego potrzebować; wtedy allowlista per aplikacja. |

## 10. Granice zaufania

- **Dane z sieci** (TDS, konfiguracja, manifesty, ikony) mają limity rozmiaru, są parsowane defensywnie i
  walidowane przed użyciem. TDS i konfiguracja są pobierane wyłącznie przez HTTPS z CDN DuckDuckGo.
- **Strony** nigdy nie widzą skryptu instalatora (izolowany świat). Wiadomości C-S-S dostają tylko stałe
  odpowiedzi, a strony nie mogą otwierać innych aplikacji bez gestu użytkownika.
- **Pliki** zapisywane przez InDuckGo powstają z uprawnieniami `0600` w katalogach `0700`. Najpierw trafiają do
  pliku `.part`, a potem dostają docelową nazwę, więc awaria nie zostawi w połowie zapisanego katalogu.
- **Pliki desktop** generuje `DesktopEntry`, który escapuje nazwy pochodzące ze stron. Tytuł strony nie może
  więc wstrzyknąć do pliku linii takiej jak `Exec=`.
