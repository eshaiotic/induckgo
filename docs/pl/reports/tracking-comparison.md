# Porównanie śledzenia: InDuckGo, Google Chrome i Brave

Pomiar wykonany 2 listopada 2026 (21:53–22:10 CET) na jednym komputerze z Linuksem w Polsce. Fakty o
ustawieniach domyślnych i politykach producentów sprawdzono tego samego dnia w ich źródłach. Każdy fakt ma
link.

## 1. Na jakie pytanie odpowiada raport

Przeglądarka może wystawiać użytkownika na śledzenie na trzy sposoby. Raport mierzy każdy z nich we
wszystkich trzech przeglądarkach.

1. **Sama przeglądarka.** Program może wysyłać dane do producenta (telemetria), choć użytkownik nic nie
   robi.
2. **Strony, które ładuje.** Serwis informacyjny osadza dziesiątki cudzych skryptów. To przeglądarka
   decyduje, czy się załadują.
3. **To, co strony mogą odczytać o urządzeniu.** Fingerprinting rozpoznaje przeglądarkę bez ciasteczek, po
   wartościach takich jak sposób rysowania obrazka albo liczba zgłaszanych rdzeni procesora.

Krótka odpowiedź jest w §6. Rozdziały 3–5 pokazują, skąd się wzięła każda liczba, żeby dało się ją
sprawdzić i powtórzyć.

## 2. Słownik

| Pojęcie | Znaczenie |
|---|---|
| Tracker | Firma lub usługa, która śledzi ludzi między stronami: reklama, analityka, widżety społecznościowe, fingerprinting. Tutaj host jest trackerem, jeśli tak mówi lista Disconnect (§3). |
| Host strony trzeciej | Host należący do innej witryny (domeny rejestrowalnej) niż odwiedzana strona, np. `doubleclick.net` na `onet.pl`. |
| Telemetria | Dane, które przeglądarka wysyła producentowi o tym, jak jest używana: statystyki użycia, raporty awarii, przydział do eksperymentów. |
| Fingerprinting (odcisk palca) | Rozpoznawanie przeglądarki po wartościach, które ujawnia (rysowanie canvas, przetwarzanie dźwięku, liczba rdzeni, ekran), zamiast po ciasteczku. |
| Farbling / szum | Nazwa używana przez Brave, a w Guardzie praktyka: dodawanie do tych wartości drobnych losowych zmian, żeby odcisk różnił się między witrynami. |
| GPC | Global Privacy Control: nagłówek `Sec-GPC: 1` i wartość `navigator.globalPrivacyControl`, które znaczą „nie sprzedawaj i nie udostępniaj moich danych”. Prawo części stanów USA nakazuje stronom go respektować. |
| Safe Browsing | Usługa Google, która ostrzega przed stronami phishingowymi i ze złośliwym oprogramowaniem. Przeglądarka sprawdza w niej odwiedzane adresy. |
| P3A | Privacy-Preserving Product Analytics w Brave: odpowiedzi na stałe pytania o użycie, wysyłane przez STAR. To schemat, który ujawnia odpowiedź dopiero wtedy, gdy podało ją dość wielu użytkowników. |
| Strona startowa | Strona, którą przeglądarka pokazuje po uruchomieniu i w nowej karcie. |

## 3. Jak mierzono

**Wersje.**

| Przeglądarka | Wersja | Sposób uruchomienia |
|---|---|---|
| Google Chrome | 156.0.8078.79 | Oficjalny `.deb` Google, rozpakowany do `/tmp` (bez instalacji) |
| Brave | 1.98.63 (Chromium 156) | Pakiet Arch `brave-bin` |
| InDuckGo | 1.0.0 z Guardem (commit `0934be1`) | Wydany AppImage; WebKitGTK 2.54.2 z systemu |

**Warunki.**

- Każda przeglądarka startowała ze **świeżym profilem**, własnym `HOME` i katalogami XDG oraz prywatną sesją
  D-Bus. Nic nie było współdzielone z prawdziwymi profilami użytkownika.
- Każda przeglądarka działała **z oknem**, na własnym wirtualnym ekranie (Xvfb). Nie użyto trybu headless,
  bo strony go rozpoznają i serwują mniej reklam.
- Chrome i Brave uruchomiono z `--no-first-run --no-default-browser-check`. Poza tym wszystkie ustawienia
  były domyślne.
- Wszystkie trzy przeglądarki działały **jednocześnie**, więc widziały te same wersje tych samych stron.
- Cały ruch szedł przez lokalne proxy, które zapisuje **tylko nazwę hosta** każdego połączenia. Nie
  odszyfrowuje TLS, więc widzi to, co dostawca internetu albo operator Wi-Fi, a nie treść.

**Przebieg.**

1. Każda przeglądarka przez **5 minut** stała bezczynnie na swojej stronie startowej. Wszystko, z czym się
   wtedy połączyła, zrobiła sama z siebie.
2. Potem otwierała **12 witryn**, każdą w nowej karcie na **20 sekund**:
   - polskie: onet.pl, wp.pl, interia.pl, gazeta.pl, o2.pl, allegro.pl, olx.pl;
   - zagraniczne: edition.cnn.com, theguardian.com, bbc.com, weather.com, forbes.com.
3. Każdy host, z którym przeglądarka połączyła się w oknie czasowym danej witryny, przypisano do tej
   witryny.

**Kto rozstrzyga, co jest trackerem.** Neutralna lista: lista trackerów Disconnect, ta sama, której używa
Firefox (kategorie Advertising, Analytics, Fingerprinting, Social, Cryptomining). To nie są listy InDuckGo
ani Brave. Trackera należącego do samej odwiedzanej witryny (np. trackera Google na stronie Google) się nie
liczy.

**Ograniczenia metody.**

- Jeden przebieg, z jednego kraju.
- 20 sekund na stronę, bez przewijania i bez klikania w banery zgody. Wiele stron ładuje więcej trackerów po
  „Akceptuję”, więc wszystkie liczby, a najbardziej liczby Chrome, zaniżają rzeczywiste użycie.
- Widać tylko nazwy hostów: dwa żądania do tego samego hosta liczą się raz, a treści wysyłanych danych nie
  widać.
- Lista Disconnect nie zna wszystkich trackerów.

## 4. Wyniki

### 4.1 Co każda przeglądarka wysyła sama z siebie

Hosty, z którymi przeglądarki połączyły się w ciągu 5 minut bezczynności, zanim otwarto jakąkolwiek
witrynę.

| | Chrome | Brave | InDuckGo |
|---|---|---|---|
| Połączenia | 46 | 43 | 20 |
| Różne hosty | 15 | 12 | 8 |
| Właściciel tych hostów | Google (wszystkie 15) | Brave (10), Google (2) | DuckDuckGo (3), serwery list filtrów (5) |

**Chrome** łączył się wyłącznie z Google. Hosty i to, do czego według dokumentacji Google służą:

- `accounts.google.com`: sprawdzenie konta. Nikt nie był zalogowany.
- `clientservices.googleapis.com`: ziarno eksperymentów („variations”).
- `update.googleapis.com`, `redirector.gvt1.com`, `edgedl.me.gvt1.com` i serwer pamięci podręcznej
  `*.gvt1.com`: aktualizacje komponentów.
- `safebrowsing.googleapis.com`: listy Safe Browsing.
- `optimizationguide-pa.googleapis.com`: podpowiedzi i modele przyspieszające ładowanie stron.
- `chromewebstore.googleapis.com`, `clients2.google.com`, `clients2.googleusercontent.com`: aktualizacje
  rozszerzeń.
- `www.google.com`, `www.gstatic.com`, `ogads-pa.clients6.google.com`, `play.google.com`: strona nowej
  karty, czyli pole wyszukiwania i pasek konta Google.

Statystyki użycia i raporty awarii są na Linuksie **domyślnie włączone** (trzeba je wyłączyć samemu).
([kod źródłowy](https://github.com/chromium/chromium/blob/main/chrome/browser/first_run/first_run_internal_posix.cc),
[pomoc Google](https://support.google.com/chrome/answer/14746339))

**Brave** łączył się głównie z własnymi serwerami:

- `usage-ping.brave.com`: codzienny ping użycia, domyślnie włączony. Jego pola obejmują m.in. datę
  instalacji, tydzień instalacji i kod polecenia. Od lutego 2026 zawiera też wczorajsze liczby wyszukiwań w
  Brave, Google i innych wyszukiwarkach.
  ([pola](https://github.com/brave/brave-core/blob/master/browser/brave_stats/brave_stats_updater_params.cc),
  [liczby wyszukiwań](https://github.com/brave/brave-core/pull/33474))
- `collector.bsg.brave.com`, `star-randsrv.bsg.brave.com`: P3A, domyślnie włączone (trzeba je wyłączyć
  samemu), wysyłane przez STAR ([wiki P3A](https://github.com/brave/brave-browser/wiki/P3A)).
- `variations.brave.com`: ziarno eksperymentów Brave.
- `go-updater.brave.com`, `componentupdater.brave.com`, `redirector.brave.com`, `brave-core-ext.s3.brave.com`:
  komponenty i listy filtrów, przez proxy Brave do aktualizatora Google.
- `safebrowsing.brave.com`: Safe Browsing przez proxy Brave, więc Google nie widzi adresu IP.
- `dict.brave.com`: słowniki sprawdzania pisowni.
- `clients2.google.com`, `clients2.googleusercontent.com`: bezpośrednio Google. Brave dokumentuje, że
  aktualizacje rozszerzeń nadal idą do Google
  ([Deviations from Chromium](https://github.com/brave/brave-browser/wiki/Deviations-from-Chromium-(features-we-disable-or-remove))).

**InDuckGo** nie wysyła żadnej telemetrii: nie ma pingu użycia, raportów awarii ani ziarna eksperymentów.
Jego 20 połączeń dzieli się na dwie grupy.

- **Dane ochrony, pobierane przez sam program (8 połączeń).**
  - `staticcdn.duckduckgo.com`: Tracker Data Set i konfiguracja prywatności DuckDuckGo.
  - `easylist.to`, `ublockorigin.github.io`, `pgl.yoyo.org`, `raw.githubusercontent.com`,
    `secure.fanboy.co.nz`: listy filtrów Guarda.
  - To zwykłe żądania `GET`, bez ciasteczek i bez identyfikatora, powtarzane co 12 godzin. Ujawniają tym
    serwerom adres IP, jak aktualizacje list w każdym blokerze.
  - Przy odświeżaniu przeglądarka odsyła `ETag` serwera (znacznik wersji), żeby nie pobierać ponownie
    niezmienionych plików. Dla tych statycznych plików ten znacznik oznacza wersję pliku.
- **Strona startowa (12 połączeń).**
  - Sześć połączeń poszło do `duckduckgo.com`. InDuckGo otwiera stronę główną wybranej wyszukiwarki, a
    domyślna jest DuckDuckGo.
  - Siedem poszło do `improving.duckduckgo.com`: to własne, anonimowe piksele statystyk DuckDuckGo, wysłane
    przez tę stronę. Poszły w pierwszych 3 sekundach świeżego profilu, zanim listy Guarda skończyły się
    pobierać.
  - To ruch witryny, nie przeglądarki, ale wciąż ruch, o który użytkownik nie prosił. Gdyby profil miał
    Google jako wyszukiwarkę, stroną startową byłoby google.com, a z nim pomiary Google. Zob. §7.

### 4.2 Trackery na 12 witrynach

Każda komórka to hosty trackerów / wszystkie hosty stron trzecich, z którymi przeglądarka połączyła się,
gdy witryna była otwarta.

| Witryna | Chrome | Brave | InDuckGo |
|---|---|---|---|
| onet.pl | 9 / 30 | 0 / 12 | 0 / 12 |
| wp.pl | 2 / 8 | 0 / 5 | 0 / 2 |
| interia.pl | 8 / 20 | 0 / 2 | 0 / 3 |
| gazeta.pl | 2 / 10 | 0 / 4 | 0 / 4 |
| o2.pl | 3 / 14 | 0 / 8 | 0 / 5 |
| allegro.pl | 0 / 6 | 0 / 2 | 0 / 2 |
| olx.pl | 17 / 35 | 1 / 11 | 0 / 10 |
| edition.cnn.com | 17 / 37 | 1 / 9 | 0 / 1 |
| theguardian.com | 0 / 9 | 0 / 5 | 0 / 6 |
| bbc.com | 10 / 20 | 1 / 6 | 1 / 8 |
| weather.com | 19 / 30 | 2 / 4 | 1 / 6 |
| forbes.com | 7 / 24 | 0 / 14 | 0 / 13 |
| **Razem** | **94 / 243** | **5 / 82** | **2 / 72** |

- **Chrome** nie blokuje trackerów. Filtruje tylko reklamy na stronach, które nie spełniają Better Ads
  Standards ([pomoc Google](https://support.google.com/chrome/answer/7632919)). Od lipca 2025 nie uruchamia
  już pełnego uBlock Origin, bo Manifest V2 zniknął
  ([harmonogram](https://developer.chrome.com/docs/extensions/develop/migrate/mv2-deprecation-timeline)).
- **Wśród 94 trackerów w Chrome** są:
  - sieć reklamowa i analityczna Google: `securepubads.g.doubleclick.net`, `pagead2.googlesyndication.com`,
    `google-analytics.com`;
  - Gemius, polski pomiar oglądalności (`*.hit.gemius.pl`);
  - reklamy Amazon, Taboola, Permutive i Quantcast.
- **5 trackerów w Brave:** `securepubads.g.doubleclick.net` (dwa razy), `pub.doubleverify.com`,
  `micro.rubiconproject.com` i `m.stripe.network`.
- **2 trackery w InDuckGo:** `securepubads.g.doubleclick.net` i `pub.doubleverify.com`.
- Wcześniejszy pomiar tym samym zestawem (przegląd architektury §5.6: Brave 6 / 82, Guard 3 / 70) daje tę
  samą kolejność. Różnice jednego czy dwóch hostów między przebiegami mieszczą się w tym, jak bardzo strony
  zmieniają się między kolejnymi wczytaniami.

### 4.3 Fingerprinting

Ta sama strona testowa odczytała wartości, których używają skrypty fingerprintingu.

- **Dwie witryny:** `127.0.0.1` i `localhost` liczą się jako różne witryny.
- **Dwie sesje:** między nimi przeglądarkę uruchomiono ponownie na tym samym profilu.

Odcisk, który jest taki sam na różnych witrynach, pozwala dwóm niezwiązanym stronom rozpoznać tę samą
osobę.

| Wartość | Chrome | Brave | InDuckGo |
|---|---|---|---|
| Rysowanie canvas | wszędzie takie samo | inne na każdej witrynie, takie samo po restarcie | inne na każdej witrynie i w każdej sesji |
| Przetwarzanie dźwięku | wszędzie takie samo | inne na każdej witrynie, takie samo po restarcie | inne na każdej witrynie i w każdej sesji |
| Rdzenie procesora | 16 (prawdziwa liczba) | 4 lub 5, zależnie od witryny | 8 (limit WebKit) |
| Pamięć urządzenia | 32 GB | 4 lub 8 GB, zależnie od witryny | nieujawniana |
| Języki | `pl-PL, pl, en-US, en` | `pl-PL` | `pl-PL` |
| Limit miejsca na dane | 10 GB | 2 GB | nieujawniany |
| GPC | nie wysyła | wysyła | wysyła |
| Zainstalowane czcionki (z 20 sprawdzanych) | widać 8 | widać 8 | widać 7 |

- **Chrome** nie zmienia niczego między witrynami. Od 16 lutego 2025 polityka platform reklamowych Google
  nie zakazuje już fingerprintingu, wymaga tylko jego ujawnienia
  ([polityka Google](https://support.google.com/platformspolicy/answer/15738904),
  [analiza](https://blog.lukaszolejnik.com/biggest-privacy-erosion-in-10-years-on-googles-policy-change-towards-fingerprinting/)).
- **Brave** losuje wartości osobno dla każdej witryny. Według dokumentacji ziarno losowania jest osobne dla
  każdej sesji i witryny ([wiki](https://github.com/brave/brave-browser/wiki/Fingerprinting-Protections)). W
  tym pomiarze ta sama witryna dostała jednak **te same wartości po restarcie** na tym samym profilu.
- **InDuckGo** zmieniało szum (Content Scope Scripts DuckDuckGo, włączone przez Guarda) zarówno między
  witrynami, jak i między sesjami.
- **Żadna przeglądarka nie ukryła zainstalowanych czcionek.**
- WebGL nie dało się porównać, bo Chromium blokuje go na wirtualnym ekranie bez GPU.

## 5. Ustawienia domyślne i polityki

| | Chrome | Brave | InDuckGo |
|---|---|---|---|
| Statystyki użycia do producenta | włączone (wyłącza się samemu) | codzienny ping i P3A włączone (wyłącza się samemu) | brak |
| To, co wpisujesz w pasek adresu | wysyłane do wyszukiwarki w trakcie pisania, z ciasteczkami ([pomoc](https://support.google.com/chrome/answer/13730681)) | w Polsce domyślnie wyłączone; włączane przy pierwszym uruchomieniu w 12 krajach ([kod](https://github.com/brave/brave-core/blob/master/browser/search_engines/search_engine_provider_util.cc)) | nic nie wychodzi przed Enterem |
| Ciasteczka stron trzecich | **dozwolone** w zwykłych oknach ([Google](https://privacysandbox.google.com/blog/privacy-sandbox-next-steps)) | blokowane; dane stron trzecich są tymczasowe i osobne dla każdej witryny | blokowane (`NoThirdParty`) plus Intelligent Tracking Prevention z WebKit |
| Blokowanie trackerów | brak | Shields (EasyList, EasyPrivacy, listy uBlock Origin i Brave) | Tracker Data Set DuckDuckGo plus Guard (EasyList, EasyPrivacy, uBlock Origin, listy polskie) |
| Parametry śledzące w linkach (`fbclid`, `gclid`…) | zostają | usuwane | usuwane (lista DuckDuckGo) |
| Fingerprinting | bez ochrony | losowanie osobne dla każdej witryny | szum osobny dla każdej witryny i sesji |
| GPC | brak w wersji stabilnej; tylko propozycja ([status](https://chromestatus.com/feature/5137324344213504)) | włączone | włączone (nagłówek i wartość w skryptach) |
| Safe Browsing (ostrzeżenia przed phishingiem) | Standard: 4-bajtowe prefiksy skrótów przez przekaźnik Oblivious HTTP | przez proxy Brave | **brak** (WebKitGTK nie ma takiej usługi) |
| Reklamowe API Privacy Sandbox | Topics, Protected Audience i inne zapowiedziane do wycofania w październiku 2025 ([Google](https://privacysandbox.google.com/blog/update-on-plans-for-privacy-sandbox-technologies)) | wyłączone | nie istnieją w WebKit |

**Niezależne porównanie.** privacytests.org, przebieg 105 (22 października 2026, desktop, ustawienia
domyślne):

- Brave 1.97 zaliczył **108 ze 146** testów.
- Chrome 156 zaliczył **31 ze 146**.
- Autor serwisu, jak sam ujawnia, pracował w Brave w latach 2022–2025.
- InDuckGo nie jest tam testowane.

([privacytests.org](https://privacytests.org/), [o autorze](https://privacytests.org/about.html))

## 6. Podsumowanie

| Pytanie | Chrome | Brave | InDuckGo |
|---|---|---|---|
| Czy sama przeglądarka raportuje producentowi? | tak, do Google, domyślnie | tak, do Brave, domyślnie, z założenia z ochroną prywatności | nie |
| Trackery osiągnięte na 12 witrynach | 94 | 5 | 2 |
| Ten sam odcisk palca na dwóch różnych witrynach? | tak | nie, ale jedna witryna zobaczyła te same wartości po restarcie | nie, a do tego zmienia się w każdej sesji; rzadki silnik nadal wyróżnia (§7) |
| Czy ostrzega przed phishingiem i złośliwym oprogramowaniem? | tak | tak | nie |

- **InDuckGo a Chrome.** Tu nie ma porównania. Chrome domyślnie raportuje do Google i przepuszcza każdy
  tracker: 94 hosty trackerów na 12 witrynach. Dopuszcza ciasteczka stron trzecich i nie broni przed
  fingerprintingiem, na który polityka reklamowa Google pozwala od 2025.
- **InDuckGo a Brave.** W tym pomiarze InDuckGo dotarło do mniejszej liczby trackerów (2 wobec 5) i nie
  wysłało niczego własnemu producentowi. Brave wysyła codzienny ping i P3A, z datą instalacji, kodem
  polecenia i liczbą wyszukiwań. InDuckGo zmieniało też odcisk palca między sesjami, a Brave zostawiał go
  bez zmian.
- **Gdzie Brave jest mocniejszy** (§7): ostrzeżenia przed phishingiem, dużo większy tłum, w którym można się
  ukryć, i kilka zabezpieczeń, których InDuckGo nie ma.

## 7. Gdzie InDuckGo jest słabsze i co by tę lukę zamknęło

| Luka | Dlaczego | Co by ją zamknęło |
|---|---|---|
| Brak ostrzeżeń przed phishingiem i złośliwym oprogramowaniem | WebKitGTK nie ma Safe Browsing. To luka bezpieczeństwa, nie śledzenia. | Sprawdzanie odwiedzanych hostów według listy, z wyszukiwaniem chroniącym prywatność. Potrzebne jest źródło danych z licencją, która na to pozwala. |
| Rzadka przeglądarka się wyróżnia | User-Agent WebKitGTK na Linuksie ma niewiele osób, więc sam silnik zawęża krąg podejrzanych. Brave wygląda jak Chrome, jedna z miliardów przeglądarek. Szum zmienia wartości, ale nie silnik. | W InDuckGo nie da się sprawić, żeby WebKitGTK był popularny. Udawanie User-Agenta Chrome psułoby strony, które rozpoznają silnik po innych cechach. |
| Widać zainstalowane czcionki | WebKitGTK nie daje sposobu na ograniczenie czcionek. Brave i Chrome w tym teście wypadły tak samo. | API w WebKitGTK, jeśli się pojawi (przegląd architektury §9). |
| Ruch strony startowej | Stroną startową jest strona główna wyszukiwarki: domyślnie anonimowe piksele DuckDuckGo, a przy wybranym Google pomiary Google. | Lokalna strona startowa z polem wyszukiwania, która nic nie wysyła, dopóki użytkownik nie zacznie szukać. |
| Brak odkrywania CNAME w czasie działania, debouncingu i przekierowania z AMP | Blokery treści WebKit nie widzą odpowiedzi DNS. Blokowane są tylko trackery ukryte za CNAME, które wymienia DuckDuckGo, także w żądaniach do własnej domeny strony. Część śledzenia przez przekierowania (bounce tracking) obsługuje ITP z WebKit. | WebKitGTK nie daje dostępu do DNS. Przekierowanie z AMP mogłoby korzystać z konfiguracji `ampLinks` DuckDuckGo. |
| Brak scriptletów uBlock Origin | Blokery treści WebKit nie potrafią ich wyrazić (przegląd architektury §9). | — |

## 8. Jak powtórzyć pomiar

Na potrzeby tego pomiaru zestaw narzędzi trzymano poza repozytorium (w `/tmp`).

- **Proxy rejestrujące.** Około 70 linii w asyncio; zapisuje nazwy hostów, przekazuje bajty bez zmian i
  nigdy nie przechwytuje TLS.
- **Skrypty uruchomieniowe dla każdej przeglądarki.** Każdy startuje przeglądarkę ze świeżym profilem pod
  `xvfb-run -a dbus-run-session`, czeka 5 minut bezczynności, a potem otwiera każdą witrynę na 20 sekund.
  Chrome i Brave są sterowane przez punkt DevTools; InDuckGo jest uruchamiane z każdym adresem.
- **Analiza.** Skrypt, który klasyfikuje hosty według `services.json` z Disconnect i Public Suffix List.
- **Sonda fingerprintingu.** Strona, która odczytuje wartości z §4.3 i wysyła je do lokalnego serwera.

Powtarzając pomiar, trzeba zadbać o trzy rzeczy: świeży profil dla każdej przeglądarki, to samo okno
czasowe dla wszystkich i listę trackerów, której nie używa żadna z badanych przeglądarek.
