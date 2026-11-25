Panel Git wbudowuje kontrolę wersji w klienta: indeksowanie, commity, zarządzanie gałęziami, worktree, interaktywny rebase, rozwiązywanie konfliktów i wypychanie — bez opuszczania okna terminala. Wszystko operuje na repozytorium Git bieżącej sesji terminala: w jakimkolwiek katalogu sesja działa, to właśnie to repozytorium pokazuje panel.

## Otwarcie

Kliknij kartę `Git` w lewym pasku aktywności (dane repozytorium pojawiają się tylko, gdy aktywna sesja jest wewnątrz repozytorium Git). Repozytorium karty podąża za jej aktywną sesją; okruszki repozytorium pozwalają skakać między repozytoriami nadrzędnymi i podrzędnymi, a submoduły są wypisywane osobno.

![Panel Git — zmiany, historia i gałęzie](images/git-overview-panel.jpg)

## Co potrafi

- **Indeksowanie i commity** — stage/unstage per plik lub hurtowo, odrzucanie zmian, commit z komunikatem, komunikaty commitów generowane przez AI (zob. Wsparcie LLM);
- **Historia** — historia commitów i per plik, porównywanie i przywracanie między rewizjami;
- **Gałęzie** — checkout, tworzenie i usuwanie gałęzi lokalnych/zdalnych oraz tagów;
- **Worktree** — wyprowadzenie gałęzi do odizolowanego drzewa roboczego (nowa karta terminala otwiera się automatycznie), scalenie z powrotem po skończeniu (zob. Worktree);
- **Interaktywny rebase** — wizualne przestawianie/sciskanie/edycja kolejki commitów z rozwiązywaniem konfliktów (zob. Interaktywny rebase);
- **Rozwiązywanie konfliktów** — edytor scalania z czterema trybami, w tym agent AI rozwiązujący hunki po kolei (zob. Rozwiązywanie konfliktów);
- **Push i inne** — pełnoprawne okno wypychania, stash, obsługa odłączonego HEAD (zob. Push, checkout i więcej).

Kolejne strony omawiają każdy obszar w tej kolejności.
