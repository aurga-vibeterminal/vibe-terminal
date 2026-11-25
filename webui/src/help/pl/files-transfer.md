Transfery plików między Twoją maszyną a serwerem to przeciąganie i upuszczanie, a kierunek jest znaczeniem: **przeciągnij do środka = wyślij na serwer, przeciągnij na zewnątrz = pobierz na tę maszynę**. Transfery jadą szyfrowanym połączeniem klient-serwer; zdalne i udostępniane serwery zachowują się jak lokalne.

## Wysyłanie: przeciągnij do środka

Przeciągnij pliki (lub całe foldery) z menedżera plików systemu na listę plików i puść:

- w trybie drzewa **upuszczenie na wiersz folderu** wysyła do tego folderu; **upuszczenie w korzeniu** (linia wstawiania między wierszami) wysyła do bieżącego katalogu głównego;
- na dole listy pojawia się wiersz transferu z `Uploading`, nazwą pliku i paskiem postępu;
- ukończenie pokazuje `Uploaded n file(s)`; niepowodzenia zawierają powód (np. odmowa dostępu), a lista odświeża się, odsłaniając nowe pliki.

## Pobieranie: przeciągnij na zewnątrz

Naciśnij plik na liście, przeciągnij go poza okno klienta i upuść w menedżerze plików systemu (lub na pulpicie):

- plik spływa z serwera tam, gdzie go upuściłeś;
- wiersze transferu pokazują postęp tak samo; **ukończony wiersz pobierania jest klikalny** i otwiera lokalny folder pobrań;
- zaznacz wiele i przeciągnij kilka plików naraz.

![Postęp transferu — wiersze wysyłania/pobierania](images/files-transfer.jpg)

> Przeciąganie przenosi pojedyncze pliki między tą maszyną a jednym serwerem; aby przenosić dane między dwoma serwerami, połącz Publikację przez HTTP lub polecenia terminala (scp/rsync).
