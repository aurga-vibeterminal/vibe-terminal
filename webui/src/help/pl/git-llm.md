Inteligencja panelu Git napędzana jest LLM: komunikaty commitów jednym kliknięciem, czytanie diffów z pomocą AI i rozwiązywanie konfliktów przez agenta, hunk po hunku. Wszystko opiera się na dostawcy LLM skonfigurowanym w Web UI.

## Konfiguracja dostawcy

Otwórz sekcję `LLM` Web UI:

- Wybierz `Provider` i wypełnij jego dane (pola definiuje manifest dostawcy); konfiguracja jest globalna — klasyfikacja terminala, komunikaty commitów Git, asystent czatu i inne funkcje AI współdzielą tego samego dostawcę;
- Dostawcy to dodatki typu LLM: typowe są wbudowane, a własnych można wgrać jako zip w sekcji `Addons` (manifest deklaruje pola konfiguracji, które Web UI zamienia w formularz).

![Sekcja LLM Web UI — wybierz dostawcę i wypełnij jego dane](images/git-llm-provider-config.jpg)

## Funkcje LLM w Gicie

- **Generowanie komunikatu commita** — kliknij przycisk AI obok pola commita; LLM czyta ostaged diff i pisze komunikat, wpisywany do pola z animacją. Komunikat pozostaje edytowalny — commituj, dopiero gdy jesteś zadowolony;
- **Analiza diffa** — przycisk iskry w nagłówku widoku diffa analizuje bieżący diff;
- **Agent AI konfliktów** — tryb AI edytora scalania rebase rozwiązuje hunki po kolei z wyjaśnieniami i pewnością (zob. Rozwiązywanie konfliktów).

![Komunikat commita wygenerowany przez AI, wpisywany do pola commita](images/git-llm-commit-message.jpg)

> LLM czyta zawartość repozytorium wyłącznie, by wyprodukować tekst; commity i scalenia zdarzają się zawsze wtedy, gdy je potwierdzasz.
