Worktree wyprowadzają wiele gałęzi jednego repozytorium do osobnych katalogów: rozwijaj równolegle, nie dotykając głównego drzewa roboczego, potem scal gałąź z powrotem i posprzątaj. Vibe Terminal zamienia cały cykl w klikanie.

## Tworzenie worktree

1. Na liście gałęzi panelu Git kliknij prawym gałąź docelową (lokalną lub zdalną) → `New Worktree`;
2. okno nazywa nowy worktree od gałęzi — edytuj, jeśli chcesz; potwierdź, a serwer utworzy katalog worktree i wyprowadzi tam gałąź;
3. **nowa karta terminala otwiera się automatycznie** z katalogiem roboczym ustawionym na nowy worktree — zacznij rozwijać w izacji od razu, a główne drzewo robocze zostaje na swojej gałęzi.

![Okno New Worktree — nazwa domyślnie to gałąź](images/git-worktree-create.jpg)

![Nowa karta otwarta dla świeżo utworzonego worktree](images/git-worktree-terminal.jpg)

## Praca równoległa

Każdy worktree to w pełni niezależny katalog roboczy: buildy i testy gałęzi głównej oraz feature'owej działają ramię w ramię w różnych kartach bez kolizji. Gałęzie wyprowadzone w worktree niosą na liście gałęzi znacznik worktree z ich ścieżką, a commitowanie i wypychanie działa prosto z karty worktree jak z każdej gałęzi.

Prawy przycisk na gałęzi worktree dodaje dwie pozycje specyficzne: `Reveal in File Explorer` (sesje lokalne) otwiera jego katalog w menedżerze plików systemu, a `Open in Terminal` otwiera tam kolejną kartę.

## Scalanie z powrotem i sprzątanie

1. Commituj i wypychaj w karcie worktree jak zwykle;
2. przełącz kartę główną na gałąź docelową (np. `main`), kliknij prawym feature'ową → `Merge into Current Branch`, by przenieść pracę;

   ![Scalanie gałęzi worktree z powrotem do gałęzi głównej](images/git-worktree-merge.jpg)

3. po scaleniu kliknij prawym gałąź → `Delete Worktree`: usuwa katalog worktree i jego gałąź razem. Upewnij się wcześniej, że wszystko jest scalone lub wypchnięte — okno ostrzega ponownie, a akcja jest nieodwracalna.

   ![Potwierdzenie Delete Worktree — katalog i gałąź są usuwane razem](images/git-worktree-delete.jpg)

