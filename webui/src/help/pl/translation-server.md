Tłumaczenie to zadanie LLM per funkcja, konfigurowane raz na serwerze. Każde żądanie tłumaczenia od dowolnego klienta — wiadomości czatu, szkice commitów, zaznaczenia edytora i diffów — przechodzi przez te same ustawienia.

## Włączenie funkcji

Otwórz sekcję `LLM` interfejsu WWW i znajdź kartę `Tłumaczenie`:

1. Włącz `Włącz tłumaczenie LLM`;
2. Wybierz `Dostawca` — lista rozwijana pokazuje addony LLM **włączone** w sekcji `Dodatki` (wbudowany provider **OpenAI Compatible LLM** pokrywa większość przypadków);
3. Wypełnij pola providera pod `Konfiguracja dostawcy` (dla wbudowanego: `Base URL`, `API Key`, `Model`, opcjonalnie `Extra Body`, limity tokenów, `Timeout`);
4. Wybierz `Język docelowy`, w którym powstanie każde tłumaczenie.

Konfiguracja zapisuje się nawet przy wyłączonej funkcji, ale serwer stosuje ją dopiero po włączeniu przełącznika. Bez niego żądania klienta kończą się błędem *"translation addon is not configured or disabled"*.

![Karta Tłumaczenie w sekcji LLM — przełącznik włączenia, wybór providera i język docelowy](images/translation-server-card.jpg)

## Język docelowy

W przeciwieństwie do funkcji terminala i Gita tłumaczenie nie ma opcji `Automatycznie` — zawsze tłumaczy na jeden jawny język z listy (English, 简体中文, 繁體中文, Deutsch, Español, Français, 日本語, 한국어, Nederlands, Polski, Português, Русский, ไทย, Tiếng Việt, العربية, Bahasa Indonesia, Bahasa Melayu, Українська). Ustawienie działa dla całego serwera; zmiana wpływa tylko na nowe żądania. Nieustawiony język wraca do angielskiego.

![Lista rozwijana języka docelowego](images/translation-server-language.jpg)

## Ponowne użycie danych providera

Przyciski kopiuj/wklej obok nagłówka `Konfiguracja dostawcy` klonują wartości pól między kartami funkcji korzystającymi z tego samego addonu providera — wybierz i wypełnij providera raz (np. dla komunikatów commitów Gita), naciśnij **Kopiuj**, a potem **Wklej** w karcie Tłumaczenie.

## Wybór modelu

Tłumaczenia to krótkie, wrażliwe na opóźnienia prompty — szybki model wygrywa z dużym modelem rozumującym. **Qwen3.5 9B** za endpointem zgodnym z OpenAI (Ollama, vLLM, llama.cpp) albo **DeepSeek V4 Flash** przez API sprawdzają się świetnie. Serwer czeka do 60 sekund na żądanie; wolnym lokalnym modelom może być potrzebny wyższy `Timeout` w polach providera.

> Tryb myślenia dodaje sekundy opóźnienia bez zysku jakości — wyłącz go w `Extra Body`, np. `{"enable_thinking": false}` dla Qwen albo `{"thinking": {"type": "disabled"}}` dla DeepSeek.
