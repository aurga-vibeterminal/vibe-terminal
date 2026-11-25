Agenci kodu (Claude Code, Codex, Kimi, Pi, OpenCode…) wiedzą tylko to, co trafia do ich wejścia. `Dodawanie kontekstu` to sposób wręczenia im ścieżek plików i fragmentów kodu bez przepisywania: przeciągnij pliki z `Explorer`, albo przeciągnij zaznaczony tekst z edytora kodu i widoku diffa Gita prosto do terminala agenta albo wejścia czatu Copilot.

## Przeciąganie plików

Naciśnij plik — lub wielokrotne zaznaczenie — na liście plików `Explorer` i przeciągnij go:

- **Na terminal**: pełne ścieżki są wpisywane w miejscu upuszczenia, oddzielone spacjami, gotowe stać się częścią Twojego następnego promptu do agenta;
- **Na wejście czatu Copilot**: ścieżki trafiają do szkicu wiadomości, po jednej na wiersz, byś złożył wieloplikowy prompt przed wysłaniem.

Ścieżki przeciągnięte z menedżera plików systemu i upuszczone na terminal są automatycznie cytowane, więc nazwy ze spacjami przeżywają nietknięte.

![Przeciągnięcie pliku z listy Explorer do terminala wpisuje jego pełną ścieżkę](images/terminal-context-drag-file.gif)

## Przeciąganie zaznaczonego kodu jako kontekstu

Zaznacz tekst w edytorze kodu lub dififie Gita, a następnie naciśnij samo zaznaczenie i przeciągnij je na terminal:

- gdy terminal **uruchamia agenta kodu**, upuszczenie wstawia *blok kontekstu* — ścieżkę pliku, zakres wierszy i fragment owinięty w płotek kodu:

````text
File: crates/vterm/src/ui/terminal_view.rs
Lines: 558-566
```
fn can_accept_native_drop(&self, info: &DndInfo) -> bool { … }
```
````

- gdy terminal jest na **zwykłym zachęcie powłoki**, upuszczenie wpisuje surowy zaznaczony tekst, więc ten sam gest działa i dla zwykłych poleceń;
- gdy diff oglądany jest przy konkretnym commicie, blok niesie dodatkowo wiersz `Commit:` przypinający dokładną rewizję.

![Przeciągnięcie zaznaczenia edytora na terminal agenta upuszcza blok kontekstu File/Lines](images/terminal-context-drag-selection.mp4)

## Kopiowanie kontekstu

Ten sam blok mieszka w menu kontekstowym: `Copy Context` w edytorze kodu i widoku diffa kopiuje go do schowka. Wklej do wejścia czatu Copilot — albo do innego agenta — gdy przeciąganie jest niewygodne. Animacja powyżej pokazuje powstały blok w wejściu agenta.

## Wskazówki

- Zaznacz wiele w `Explorer`, by upuścić kilka ścieżek naraz;
- wejście czatu Copilot wysyła do terminala jak wpisany tekst (zob. `Historia agenta kodu`), więc kontekst dodany tam dociera do agenta dokładnie tak, jakbyś go wpisał;
- połącz z `Automatycznym potwierdzaniem`, by agent działał na upuszczonym kontekście bez dalszych promptów.
