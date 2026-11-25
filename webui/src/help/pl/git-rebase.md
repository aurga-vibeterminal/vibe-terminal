Interaktywny rebase zamienia `git rebase -i` w wizualny przepływ: przeciąganiem przestawiaj kolejkę commitów, stosuj commity po kolei, rozwiązuj konflikty wizualnie — bez wiersza poleceń.

## Start

Otwórz panel gałęzi i wpisz w pole `Filter branches...`, by znaleźć gałąź docelową, potem prawy przycisk → `Rebase Current Branch onto '{branch}'`, by rebasować bieżącą gałąź na cel. Panel przechodzi do widoku rebase, który biegnie przez cztery fazy:

![Szukanie gałęzi w panelu, potem start rebase na cel](images/git-rebase-search.jpg)

## 1. Edycja kolejki commitów

Commity Twojej gałęzi, których brakuje celowi, tworzą kolejkę:

- **Przeciągaj**, by je przestawiać;
- każdy commit ma rozwijane akcje: `pick` (zostaw) / `squash` (zwiń w poprzedni, scal komunikaty) / `fixup` (zwiń, porzuć komunikat) / `edit` (zatrzymaj się do poprawki) / `drop` (porzuć);
- sciskane i porzucane commity zwijają się wizualnie.

![Kolejka commitów rebase — przestawianie przeciąganiem, akcje per commit](images/git-rebase-queue.jpg)

## 2. Stosowanie

`Start Rebase` odtwarza kolejkę na celu commit po commicie z paskiem postępu na żywo; bez konfliktów przelatuje prosto do końca. `Abort` wycofuje w każdej chwili.

## 3. Konflikty (jeśli są)

Konflikt podczas stosowania przełącza do widoku konfliktów — lista plików plus edytor scalania z jego czterema trybami (`Merge` / `Split` / `Raw` / `AI`), gdzie każdy skonfliktowany hunk dostaje własną decyzję (`Ours` / `Theirs` / `Both`…; zob. Rozwiązywanie konfliktów). Gdy każdy plik jest rozwiązany, `Continue` wznawia rebase.

![Widok konfliktów — decyzje scalania per hunk](images/git-rebase-resolve-conflict.jpg)

## 4. Gotowe

Po zakończeniu panel potwierdza `Branch '{branch}' was rebased onto '{target}'` i pokazuje kartę wypchnięcia — rebase przepisał historię, więc gałąź zdalna musi być zaktualizowana siłą:

- `Force Push with Lease` — zalecane: nadpisuje tylko, gdy nikt inny nie wypchnął, i nigdy nie używa gołego `--force`;
- `Not now` — pomiń wypchnięcie; lokalny rebase już jest kompletny.

![Strona końcowa — Force Push with Lease albo pominięcie wypchnięcia](images/git-rebase-done.jpg)

> Sesja pozostaje użyteczna w trakcie rebase; zamknięcie panelu nie przerywa trwającego.
