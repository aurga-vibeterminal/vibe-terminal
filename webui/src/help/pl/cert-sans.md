Certyfikat liścia jest ważny tylko dla nazw wymienionych w jego **alternatywnych nazwach podmiotu (SAN)**. Gdy klient się łączy, nazwa hosta lub adres IP w jego adresie URL muszą dokładnie odpowiadać jednemu z tych wpisów — symbole wieloznaczne i dopasowywanie po prefiksie nie są obsługiwane. Ta strona opisuje zarządzanie tą listą po stronie serwera; sprawdzanie po stronie klienta znajduje się na ostatniej stronie, Ścisła weryfikacja na klientach.

## Wykrywanie automatyczne

Serwer w sposób ciągły wykrywa, pod jakimi nazwami jest osiągalny — aktualne nazwy hostów i adresy aktywnych interfejsów sieciowych — i wymienia je w sekcji `Wykryte automatycznie`. Te wpisy podążają za maszyną i nie da się ich edytować ręcznie: dodaj interfejs sieciowy lub zmień nazwę hosta, a lista zaktualizuje się przy następnym odnowieniu.

## Dodatkowe domeny i adresy IP

Automatyczne wykrywanie nie wie wszystkiego: nazwy DDNS, wpisy DNS split-horizon albo adres za NAT trzeba wymienić ręcznie. Dodaj je w sekcji `Dodatkowe domeny i adresy IP`:

1. Wpisz nazwę hosta lub adres IP (na przykład `server.home.arpa` lub `10.0.0.8`) i naciśnij `Dodaj`;
2. Powtórz dla każdej nazwy, której będą używać klienci. Usuń wpis przyciskiem kosza;
3. Sprawdź `Podgląd końcowej listy SAN` — scalony widok wpisów wykrytych i ręcznych, który poniesie następny certyfikat;
4. Naciśnij `Zapisz i wystaw ponownie`. Zapisanie powoduje podpisanie nowego 90-dniowego certyfikatu liścia z nową listą SAN i wczytanie go na gorąco — istniejące połączenia pozostają nienaruszone, a nowe uzgadniania od razu używają nowego certyfikatu.

## Ręczne ponowne wystawienie i status

Panel `Certyfikat liścia serwera` pokazuje aktywny certyfikat: plakietkę statusu, podmiot, odcisk SHA-256, okres ważności i wpisy SAN. Plakietka statusu oznacza:

- `Dostępny` — wczytano ważny certyfikat liścia i jest on używany;
- `Aktualizacja nie powiodła się` — ostatnia próba ponownego wystawienia się nie powiodła; poprzedni certyfikat pozostaje aktywny, a błąd jest wyświetlany na stronie;
- `Nie wygenerowano` — nie ma jeszcze ani głównego CA, ani certyfikatu liścia; użyj `Wygeneruj CA i certyfikat`, aby utworzyć oba.

`Wystaw certyfikat liścia ponownie` od razu podpisuje świeży certyfikat liścia, bez czekania na automatyczny wyzwalacz — przydatne tuż po zmianie nazwy hosta lub interfejsów albo aby uwzględnić edycje SAN wykonane poza interfejsem WWW.

## Rozpowszechnianie głównego CA

Klienci, którzy będą korzystać ze ścisłej weryfikacji, potrzebują publicznego certyfikatu głównego CA. Pobierz go z karty `Prywatne główne CA`:

- `Pobierz główne CA` zapisuje plik PEM;
- `Kopiuj główne CA` umieszcza PEM w schowku, gotowy do wklejenia w pole CA klienta.

Główne CA nie zmienia się przy odnowieniach certyfikatu liścia ani edycjach SAN — wystarczy rozpowszechnić je raz. Zanim włączysz ścisłą weryfikację gdziekolwiek, porównaj odcisk SHA-256 certyfikatu pokazany na tej stronie z tym, co otrzymał klient (zobacz ostatnią stronę, Ścisła weryfikacja na klientach).

> Pobrany plik i schowek zawierają wyłącznie publiczny certyfikat CA — nigdy klucz prywatny. Przekazuj go tylko osobom, które mają móc weryfikować Twój serwer.
