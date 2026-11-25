Analizator terminala po stronie serwera obserwuje jego wyjście: wykrywa zachęty poleceń, rozpoznaje ukończenie zadań, a z pomocą LLM klasyfikuje to, co terminal pokazuje. `Auto Confirm` to najczęściej używana funkcja — gdy terminal zatrzyma się na zwyczajnym pytaniu potwierdzającym (jak `Proceed? [Y/n]`), Enter zostaje wysłany za Ciebie, więc długie zadania nie utykają już na pytaniach.

## Konfiguracja

Sekcja `Terminal Analysis` w Web UI:

- `Terminal observer` — główny przełącznik analizy po stronie serwera;
- `Auto confirm` — wysyła Enter tylko dla zachęt zaklasyfikowanych jako **zwykłe potwierdzenia**; inne pytania nigdy nie są odpowiadane w ciemno;
- `Analysis delay (seconds)` — jak długo czekać po przejściu agenta w stan spoczynku przed analizą, co unika zbędnych wywołań LLM między zadaniami;
- `Confirmation grace (seconds)` — okno, w którym możesz przejąć kontrolę po wykryciu potwierdzenia;
- `Notify on completion` — emituje zdarzenie, gdy wyjście zostanie zaklasyfikowane jako zadanie ukończone (podepnij pod powiadomienia).

> Notka na stronie: gdy tryb automatyczny Twojego agenta jest włączony, lepiej wyłączyć `Auto Confirm` w kliencie, aby dwie automatyzacje nie odpowiadały jedna drugiej.

## Zależność od LLM

Klasyfikację wykonuje LLM: wybierz `Provider` i wypełnij jego dane w sekcji `LLM` (współdzielonej przez klasyfikację terminala, komunikaty commitów Git, asystenta czatu i inne funkcje AI). Bez LLM analizator działa dalej z obniżoną dokładnością — w tym razie trzymaj auto potwierdzanie wyłączone.

![Strona ustawień Terminal Analysis](images/terminal-auto-confirm.jpg)

![Strona ustawień LLM — wybierz dostawcę i wypełnij dane](images/terminal-auto-confirm-llm.jpg)
