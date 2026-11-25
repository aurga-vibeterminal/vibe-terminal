Sekcja `LLM` Web UI przypisuje dostawcę LLM każdej funkcji AI — klasyfikacji terminala, komunikatom commitów Git, rozwiązywaniu konfliktów, szlifowi tekstu głosowego i tłumaczeniom. Sami dostawcy to dodatki typu LLM zarządzane w sekcji `Addons`.

## Dostawcy to dodatki

Dostawcy LLM mieszkają w grupie `LLM` sekcji `Addons`:

- Dostawca **OpenAI Compatible LLM** jest wbudowany; własnych można wgrać jako zip (przycisk `Upload`). Manifest dodatku deklaruje pola konfiguracji (klucz API, bazowy URL, model, …), które Web UI zamienia w formularz;
- dostawca pojawia się w selektorach funkcji tylko, póki jego dodatek jest **włączony** — najpierw włącz go w sekcji Addons;
- przycisk ołówka na karcie dodatku edytuje jego źródło; zmodyfikowany dodatek wbudowany można wycofać przez `Reset`.

![Sekcja Addons — dostawcy LLM to dodatki, które można włączać, edytować i wgrywać](images/ai-llm-addons.jpg)

## Konfiguracja per funkcja

Każda karta funkcji w sekcji `LLM` ma własny przełącznik włączenia i własny wybór dostawcy, więc różne prace mogą używać różnych modeli:

- **Terminal Classification** — klasyfikuje stan terminala, gdy wbudowane wzorce nie wystarczają. `Max Consecutive Failures` wyłącza LLM dla sesji po tylu porażkach z rzędu (minimum 3);
- **Git Commit Messages** — generuje komunikaty commitów z ostagedowanych zmian;
- **Git Conflict Resolve** — rozwiązuje konflikty rebase/scalania hunk po hunku;
- **Voice Text Polish** — szlifuje transkrypcje wprowadzania głosowego przed wysłaniem;
- **Translation** — tłumaczy tekst na wybrany język docelowy.

Rozwiń kartę, by wybrać jej `Provider` i wypełnić pola dostawcy. Konfiguracja zapisuje się nawet przy wyłączonej funkcji i zaczyna działać po włączeniu. Gdy kilka funkcji korzysta z tego samego dodatku dostawcy, przyciski kopiuj/wklej obok `Provider Configuration` pozwalają sklonować jego wartości pól między nimi (patrz niżej).

Funkcje Terminala i Gita oferują język wyjściowy z opcją `Auto` (model decyduje); tłumaczenie używa jawnej listy celów English/简体中文.

![Sekcja LLM — każda funkcja wybiera własnego dostawcę i dane](images/ai-llm-features.jpg)

## Kopiuj/wklej między funkcjami

Każda funkcja przechowuje własną kopię pól dostawcy, więc użycie jednego dostawcy w kilku funkcjach oznacza zwykle ponowne wpisywanie tego samego bazowego URL, klucza API i modelu w każdej karcie. Nagłówek `Provider Configuration` ma dwa małe przyciski, które eliminują tę powtórkę:

- **Kopiuj** — kopiuje wartości pól obecnie widoczne w tej karcie wraz z dodatkiem dostawcy, do którego należą;
- **Wklej** — pojawia się na karcie tylko wtedy, gdy wybrany tam dostawca to ten sam dodatek, co sklonowana konfiguracja; kliknięcie wypełnia pola karty sklonowanymi wartościami.

Aby współdzielić jeden punkt końcowy, powiedzmy, między komunikatami commitów Git a tłumaczeniami: w pierwszej karcie wybierz dostawcę i wypełnij jego pola, naciśnij **Kopiuj**, otwórz drugą kartę (z wybranym tym samym dostawcą) i naciśnij **Wklej**. Wklejone wartości są zapisywane we własnej konfiguracji tej funkcji jak każda inna edycja pól i są zapisywane razem z nią. Schowek działa tylko w obrębie strony — czyści się przy wyjściu z sekcji LLM.

![Przyciski Kopiuj i Wklej w nagłówku Provider Configuration — duplikuj pola dostawcy między funkcjami używającymi tego samego dodatku](images/ai-llm-duplicate-config.jpg)

## Dostawca zgodny z OpenAI

Wbudowany dostawca działa z każdym API czatu zgodnym z OpenAI. Kluczowe pola:

- `Base URL` / `API Key` / `Model` — punkt, dane, nazwa modelu;
- `Extra Body` — obiekt JSON wpleciony w każde ciało żądania, dla parametrów specyficznych dla modelu;
- `Max Input Tokens` / `Max Output Tokens` — budżet promptu i pułap uzupełnienia;
- `Timeout` — limit na żądanie w sekundach; podnieś dla wolnych modeli lokalnych.

## Polecane modele

Te zadania AI to małe, czułe na opóźnienie prompty — szybki model „w sam raz" wygrywa z wielkim modelem rozumującym:

- **Lokalnie: Qwen3.5 9B** — złoty środek dla komunikatów commitów Git, klasyfikacji terminala, tłumaczeń i szlifu głosu. Podaj go punktem zgodnym z OpenAI (Ollama, vLLM, llama.cpp) i skieruj tam `Base URL`;
- **API: DeepSeek V4 Flash (`deepseek-v4-flash`)** — wystarczająco szybki do pracy interaktywnej i zdolny udźwignąć wszystkie powyższe funkcje.

> Tryb myślenia dodaje sekundy opóźnienia i rzadko pomaga tym krótkim zadaniom — wyłącz go w `Extra Body` dla lepszych wrażeń, np. `{"enable_thinking": false}` dla Qwen albo `{"thinking": {"type": "disabled"}}` dla DeepSeek (dokładny klucz zależy od stacku serwowania; JSON jest wplatany w ciało żądania dosłownie).
