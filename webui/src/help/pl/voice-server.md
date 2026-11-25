Wprowadzanie głosowe zamienia mowę na wejście terminala: mów zamiast pisać, z opcjonalnym szlifem LLM transkrypcji przed wysłaniem. Rozpoznawanie działa jako lokalny serwer ASR z modelami na urządzeniu — audio i tekst zostają na Twojej maszynie. Ta strona obejmuje stronę serwera; doświadczenie desktopowe mieszka w Korzystaniu z klienta.

## Karta Local ASR Server

Otwórz sekcję `Voice` Web UI i włącz rozpoznawanie mowy. Karta `Local ASR Server` zarządza nadzorowanym procesem vterm-voice:

- `Load on Startup` — uruchomienie serwera ASR razem z backendem;
- `Bind Address` (`127.0.0.1` tylko localhost / `0.0.0.0` wszystkie interfejsy) i `Port` (domyślnie `8398`) — gdzie nasłuchuje punkt ASR;
- `Model Directory` + `Local Model` — wybór z wbudowanego katalogu: rodziny Qwen3-ASR, Fun-ASR i Parakeet, każda w wariantach full / 8-bit / 4-bit. `Download` sprowadza model z Hugging Face albo ModelScope do katalogu z paskiem postępu na żywo; ręczne komendy `git clone` są pokazane jako plan B, a modele już na dysku są oznaczone jako dostępne;
- `Device` — backend wnioskowania (`Auto`, `CPU`, `CUDA`, `Vulkan`, `Metal`), zgodnie z raportem serwera;
- `Access Token` — wspólny sekret punktu; przycisk kostki go generuje;
- `Save` utrwala kartę (edycje są do tego czasu lokalnym szkicem), a `Start Server` / `Stop Server` sterują procesem. Ikona statusu w tytule karty odzwierciedla osiągalność i stan modelu (wczytany / wczytuje / błąd).

![Sekcja Voice — karta Local ASR Server z modelem, urządzeniem i tokenem](images/voice-server-local-asr.jpg)

> **Kolizja CUDA**: gdy inny program wnioskowania CUDA już zajmuje GPU (np. llama.cpp z lokalnym modelem), pozwolenie vterm-voice też na `CUDA` może drastycznie spowolnić jedno i drugie. Wtedy ustaw `Device` na `CPU` — te kompaktowe modele ASR są na CPU wystarczająco szybkie.

## Dostawca i testy

Rozwijana lista dostawców wypisuje dodatki ASR — wbudowany dodatek `vibe-voice` wskazuje ten nadzorowany serwer, więc jego wybór to sposób, by panel używał Twoich lokalnych modeli.

Obszar `Test` próbuje wszystkiego bez klienta: `Record` (mikrofon → fala → transkrypcja) i `Live` (rozpoznawanie strumieniowe), każde z czasami i przyciskiem kopiowania.

![Obszar Test — karty Record i Live](images/voice-server-test.jpg)
