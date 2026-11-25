Obszar `CHANGES` panelu Git obejmuje codzienną pętlę commitów: przegląd zmian, indeksowanie, odrzucanie, stash i commit z komunikatem.

## Stage i unstage

- Nieostagedowane zmiany lądują pod `CHANGES`, każdy plik z plakietką statusu — `M` zmieniony, `A` dodany, `D` usunięty, `?` nieśledzony, `!` konflikt (wiersze submodułów niosą dodatkowy znacznik `SUB`);
- Najechanie na wiersz odsłania przyciski akcji w wierszu: `+` ostage'owuje nieostagedowany plik, `−` unstage'uje ostagedowany, a `×` na nieostagedowanym wierszu go odrzuca; przyciski `Stage All Changes` / `Unstage All Changes` w nagłówkach sekcji załatwiają wszystko naraz;
- `Ctrl`-klik przełącza pliki do zaznaczenia wielokrotnego, a `Shift`-klik wybiera zakres — przycisk akcji stosuje się wtedy do każdego zaznaczonego pliku;
- Kliknięcie nazwy pliku otwiera jego diff, by przejrzeć zmianę przed decyzją;
- Prawy przycisk na pliku oferuje `View File History`, `Stash This File`, `Copy Full Path` / `Copy Relative Path`, `Reveal in File Explorer` i skrót `cd` skaczący terminalem do folderu pliku.

![Sekcja STAGED CHANGES — ostagedowane pliki z przyciskami Unstage w każdym wierszu](images/git-staging-staged.jpg)

## Commitowanie

1. Ostage'uj to, co ma wejść;
2. wpisz komunikat w polu commita (`Ctrl+Enter` commituje) albo kliknij przycisk AI, by wygenerować (zob. Wsparcie LLM);
3. kliknij `Commit`.

![Pole commita — wpisywanie komunikatu z przyciskiem generowania AI](images/git-staging-commit.jpg)

Po commicie najedź na nagłówek historii — pasek z nazwą gałęzi nad historią commitów — i przyciski `Fetch`, `Pull` i `Push` wyłaniają się po jego prawej. `Push` od razu wysyła bieżącą gałąź do jej upstream; dla opcji takich jak `Force with lease` użyj okna wypychania listy gałęzi (zob. Push, checkout i więcej).

![Najechanie na nagłówek historii odsłania przyciski Fetch, Pull i Push](images/git-staging-push.jpg)

## Stash

- Przycisk archiwum w nagłówku `CHANGES` otwiera okno `Stash All Changes`, w którym opcjonalny komunikat opisuje stash;
- prawy przycisk na zaznaczonych nieostagedowanych plikach oferuje `Stash This File` (albo `Stash <n> Selected Files` dla zaznaczenia wielokrotnego), by stashować tylko te ścieżki — wiersze submodułów są wykluczone;
- stashe zbierają się pod `STASHES` jako wpisy `WIP on <gałąź>`; rozwinięcie jednego wypisuje przechwycone pliki, a kliknięcie pliku pokazuje jego diff;
- najechanie na wiersz stasha pokazuje `Apply` (przywróć i zostaw), `Pop` (zastosuj i usuń) i `Drop` (usuń bez stosowania) — `Pop` i `Drop` pytają najpierw o potwierdzenie, a menu kontekstowe dodaje `View Diff`;
- sekcja `STASHES` pozostaje ukryta, póki lista stashy jest pusta.

![Okno Stash All Changes z opcjonalnym komunikatem](images/git-staging-stash-all.jpg)

![Sekcja STASHES — stash rozwinięty na swoje pliki, z przyciskami Apply, Pop i Drop](images/git-staging-stashes.jpg)

![Pop stosuje stash i usuwa go z listy po potwierdzeniu](images/git-staging-stash-pop.jpg)

## Odrzucanie

`Discard Changes` na nieostagedowanym pliku wyrzuca wszystkie jego niecommitowane zmiany — jest okno potwierdzenia, a akcja **nie może być cofnięta**.

> Commit obejmuje tylko ostagedowaną treść; nieostagedowane zmiany zostają w drzewie roboczym.
