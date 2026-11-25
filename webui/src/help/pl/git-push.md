Menu prawego przycisku listy gałęzi skupia wypychanie, checkout i codzienne operacje gałęzi.

## Push

Prawy przycisk na gałęzi → `Push '{branch}'...` otwiera okno wypychania:

![Prawy przycisk na gałęzi — pozycja Push](images/git-push-contextmenu.jpg)

- Pokazuje mapowanie celów `local → remote`; obie nazwy można zmienić (wypchnięcie na istniejącą zdalną gałąź pod inną nazwą pyta o potwierdzenie);
- Opcje: `Push all branches`, `Set upstream` (zwiąż upstream przy pierwszym pushu), `Push tags`, `Dry run`;
- Przepisujący historię `Force` i bezpieczniejszy `Force with lease` (odmawia, gdy ktoś inny wypchnął — preferowany). Operacje przepisujące historię, jak push po rebase, wymagają jednego z nich;
- W oknie pokazana jest wskazówka ryzyka force pushy.

![Okno wypychania — cele local → remote i pola opcji](images/git-push-dialog.jpg)

## Checkout

Prawy przycisk na gałęzi → `Checkout`, by przełączyć gałąź — lokalne gałęzie są checkoutowane od razu, a zdalna gałąź albo tag otwiera okno checkoutu:

![Prawy przycisk na gałęzi — pozycja Checkout](images/git-checkout-contextmenu.jpg)

- `New branch name` — lokalna gałąź do utworzenia, wstępnie wypełniona z refa;
- `Create new local branch` (`-B`), `Track remote as upstream` (`--track`, tylko gałęzie zdalne), `Force` (`-f`), `Merge` (`--merge`).

![Okno checkoutu — nazwa gałęzi i opcje](images/git-checkout-dialog.jpg)

Katalog roboczy sesji podąża za przełączeniem; terminal i panel Git odzwierciedlają nową gałąź natychmiast.

## Odłączony HEAD

Checkout gołego commita (np. checkout commita podczas przeglądania historii) odłącza HEAD. Kolejny commit otwiera wtedy okno `Detached HEAD`: wybierz istniejącą gałąź do commitowania, `+ Create new branch…` na miejscu albo na razie `Skip`.

## Więcej operacji

- **Stash** — zstashuj wszystkie zmiany na stos (komunikat opcjonalny) z nagłówka zmian; lista `STASHES` obsługuje `Pop` (zastosuj i usuń) i `Drop` (porzuć, nieodwracalne);
- **Usuwanie gałęzi/worktree** — przez menu prawego przycisku na pozycji, obie z potwierdzeniami nieodwracalnych akcji;
- **Historia i przywracanie** — `HISTORY` wypisuje commity; prawy przycisk dla historii pliku, diffa z drzewem roboczym albo przywrócenia do rewizji;
- **Kopiowanie** — nazwy gałęzi i hashe commitów kopiują się jednym kliknięciem, poręcznie do wklejenia agentowi lub koledze.
