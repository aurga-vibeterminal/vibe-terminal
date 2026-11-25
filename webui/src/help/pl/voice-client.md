Naciśnij `Ctrl` szybko dwa razy (w ~300 ms) gdziekolwiek w kliencie desktopowym, by otworzyć okno wprowadzania głosowego.

## Nagrywanie

Okno od razu zaczyna nagrywać: płynna fala podąża za głosem, górny selektor przełącza mikrofony (wybór jest pamiętany), a licznik pokazuje czas. `Stop` kończy klip i wysyła go do transkrypcji (`Reset` startuje od nowa).

![Faza nagrywania — fala, selektor mikrofonu i licznik](images/voice-client-record.jpg)

## Przetwarzanie

Gdy serwer transkrybuje audio, odgrywa się animacja strumienia cząstek.

## Edycja

Transkrypcja ląduje w edytowalnym polu tekstowym:

- pigułka trybu przełącza `original` i `polished`; `Polish` raz przegania LLM, a `Auto polish` robi to zaraz po każdej transkrypcji;
- `Re-record` porzuca tekst i zaczyna od zera; `Insert` nagrywa ponownie i wplata nową transkrypcję w istniejący tekst przy karetce (zastępując zaznaczenie), więc możesz rozszerzyć lub poprawić już dyktowane — karetka ląduje tuż za wstawionym fragmentem;
- najedź na pole tekstowe, by odsłoniąć przycisk kopiowania w jego prawym górnym rogu: kliknięcie kopiuje tekst i zmienia go w ptaszka; edycja tekstu albo zmiana trybu ponownie go uzbraja;
- `Submit` (albo `Ctrl+Enter`) wpisuje tekst do aktywnego terminala; `Esc` zamyka okno.

![Faza edycji — pigułka trybu, auto szlif, rząd akcji z Insert i przycisk kopiowania przy najechaniu](images/voice-client-editing.jpg)

## Konfiguracja szlifu

`Polish` potrzebuje głosowego LLM szlifującego: w sekcji `LLM` Web UI włącz `Enable voice text polish` na liście per funkcja (zwykły dostawca plus ten przełącznik to wszystko). Bez niego przycisk `Polish` pozostaje wyszarzony, a opcja automatycznego szlifu ukryta.
