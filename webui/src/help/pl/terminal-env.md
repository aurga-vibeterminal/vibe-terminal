Zmienne środowiskowe pozwalają prowadzić pogrupowane zestawy zmiennych per projekt (`ANDROID_HOME`, klucze API, ustawienia proxy) i aplikować je lub czyścić na dowolnej sesji terminala jednym kliknięciem. Żyją w konfiguracji serwera; wartości są szyfrowane w spoczynku.

## Utrzymanie w Web UI

Sekcja `Environment Variables` organizuje je w drzewo:

- **Grupy** — czyste kontenery, zagnieżdżalne (prawy przycisk → `New Group`), do porządkowania według projektu lub scenariusza;
- **Kolekcje** — liście drzewa i faktyczni posiadacze zmiennych (`New Leaf`); tylko kolekcje można aplikować do terminala;
- Pary klucz/wartość każdej kolekcji edytuje się przez `Edit Variables` (`Add Variable`); wartości są domyślnie ukryte;
- Kolekcje można duplikować, by wywieźć warianty (ten sam zestaw z jedną zmienioną wartością).

![Strona Environment Variables — drzewo grup/kolekcji](images/terminal-env-tree.jpg)

## Aplikowanie do terminala

Prawy przycisk w terminalu → `Envs`:

- Podmenu odzwierciedla drzewo z Web UI; rozwiń kolekcję i wybierz `Apply`, by wstrzyknąć jej zmienne do bieżącej sesji, albo `Clear`, by je usunąć;
- Nowo uruchomione procesy widzą zmienne (już działające programy pierwszoplanowe — nie);
- Kolekcje pojawiają się zwinięte pod swoimi grupami; cała pozycja `Envs` chowa się, póki drzewo nie ma kolekcji.

![Prawy przycisk terminala — zastosuj i wyczyść w podmenu Envs](images/terminal-env-menu.mp4)
