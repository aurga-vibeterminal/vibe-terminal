Gdy rebase lub scalanie natrafi na konflikty, panel przechodzi do widoku konfliktów: lista plików skonfliktowanych po lewej, edytor scalania po prawej. Skonfliktowane pliki pojawiają się też na liście `CHANGES` z plakietką `!` — kliknięcie jednego wskakuje prosto w ten widok zamiast zwykłego diffa. Edytor oferuje cztery tryby rozwiązywania i można między nimi przełączać w dowolnej chwili.

![Widok konfliktów — lista plików po lewej, edytor scalania po prawej](images/git-conflicts-view.jpg)

## Lista plików skonfliktowanych

Pliki są grupowane według stanu: te scalone automatycznie z powodzeniem (automatycznie ostagedowane) i te wciąż wymagające ręcznej uwagi. Kliknij plik, by otworzyć go w edytorze; lista aktualizuje się na żywo, gdy rozwiązujesz, aż opustoszeje.

## Cztery tryby rozwiązywania

- **Merge** (domyślny) — konflikty są przedstawiane hunk po hunku; dla każdego hunka wybierz `take ours` / `take theirs` albo edytuj scalony wynik w miejscu;
- **Split** — diff jest łamany na drobniejsze segmenty poziomu wierszy, więc obie strony można przejmować kawałek po kawałku w jednym hunku;
- **Raw** — zwykła edycja ze znacznikami konfliktu `diff3` (`<<<<<<<` / `=======` / `>>>>>>>`) do ręcznego dostrajania;
- **AI** — agent rozwiązuje plik hunk po hunku: LLM analizuje każdy skonfliktowany hunk i tworzy karty rozwiązań z wyjaśnieniem i pewnością. Możesz wpisywać instrukcje, by kierować strategią (np. „preferuj theirs, zostaw logi"), pauzować, dostrajać w innym trybie i wznawiać — już rozwiązane hunki są pomijane.

W każdym trybie rozwiązanie jest zapisywane do pliku Twojej sesji i ostagedowane, dokładnie jak ręczna edycja.

## Kontynuacja rebase

Gdy każdy konflikt jest rozwiązany, rebase przechodzi do kolejnego commita; możesz też w każdym momencie przerwać, wracając do stanu sprzed rebase.
