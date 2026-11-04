# Pobieranie plików

InDuckGo zapisuje pobrany plik tam, gdzie chce użytkownik, nigdy nie nadpisuje istniejącego pliku i trzyma pięć
ostatnich pobrań pod ręką: przycisk na pasku narzędzi i pozycja w menu głównym otwierają tę samą listę.

## 1. Problem

Wcześniej każdy plik trafiał po cichu do systemowego folderu Pobrane, a jedynym śladem był komunikat z
„Pokaż w folderze”. Nie dało się wybrać folderu dla pojedynczego pliku, zobaczyć wcześniejszych pobrań ani
sprawdzić, jak daleko zaszło duże pobieranie.

## 2. Słownik

| Pojęcie | Znaczenie |
|---|---|
| **Folder pobierania** | Folder, do którego trafiają pliki (albo w którym otwiera się okno zapisu). Dopóki użytkownik nie wybierze innego, jest to systemowy folder Pobrane. |
| **Ostatnie pobrania** | Pięć ostatnich pobrań profilu, od najnowszego. Na dysk trafiają tylko ukończone. |
| **Mnemonik** | Podkreślona litera etykiety przycisku (`_Zapisz`), która uruchamia go z klawiszem Alt. |

## 3. Jak to jest połączone

![Pobranie przechodzi z WebKit przez tracker i centrum do listy](../../assets/diagrams/architecture-downloads.pl.svg)

- **`induckgo-core`** zawiera decyzje, bez GTK: `DownloadFileName` czyści nazwę podaną przez stronę i szuka
  wolnej ścieżki, `DownloadSettings` to dwa zapisywane wybory (pytać, gdzie zapisać; folder pobierania), a
  `RecentDownloads` to lista pięciu pozycji ze stanem każdej (w toku, ukończone, nieudane).
- **`DownloadTracker`** (aplikacja) nasłuchuje WebKit. Dla każdego pobrania proponuje ścieżkę w folderze
  pobierania. Gdy użytkownik chce być pytany, otwiera okno zapisu (zamknięcie go anuluje pobranie), w przeciwnym
  razie zapisuje od razu. Jeśli okna wyboru pliku nie da się pokazać wcale, plik zostaje w proponowanej ścieżce, a
  problem trafia do logu: pobranie, którego użytkownik chciał, nigdy nie przepada.
- **`DownloadCenter`** (aplikacja) przechowuje listę i ustawienia profilu, zapisuje `downloads.json` i
  `download-settings.json` oraz po każdej zmianie powiadamia obserwatorów. Okno Tor trzyma oba tylko w pamięci.
- **`DownloadsList`** (aplikacja) to widżet w okienku na pasku i w pozycji „Pobrane” menu głównego: wiersz na
  pobranie (kliknięcie otwiera plik, gdy jest ukończony) z przyciskiem folderu pokazującym, gdzie plik leży
  (pozycje mogą być w różnych folderach), oraz ostatni wiersz „Przejdź do folderu pobierania”.

## 4. Preferencje

*Preferencje → Pobrane* ma opcje „Pytaj, gdzie zapisać każdy plik” (domyślnie włączona) i „Folder pobierania”.
Przycisk na pasku to jeden z ukrywalnych przycisków (*Preferencje → Wygląd*); pozycja w menu głównym jest zawsze.

## 5. Dlaczego tak

Decyzje leżą w crate domenowym, więc testuje się je bez ekranu (`tests/download_settings.rs`,
`tests/recent_downloads.rs`). Postęp jest zgłaszany najwyżej cztery razy na sekundę, bo przy dużym pliku lista
byłaby inaczej przebudowywana tysiące razy. Zapisywane są tylko ukończone pobrania: pobranie przerwane
zamknięciem przeglądarki nie ma pliku wartego pokazania.

## 6. Ograniczenia

Okno zapisu przechodzi przez portal wyboru plików pulpitu. W środowisku testowym bez ekranu (brak magistrali
sesji) nie dało się go uruchomić; opisany wyżej plan awaryjny obejmuje jego brak.
