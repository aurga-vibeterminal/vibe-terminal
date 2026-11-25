Dodatki rozszerzają serwer o nowe możliwości bez ruszania rdzenia: dostawcy LLM, backendy rozpoznawania mowy, aktualizatory DNS dynamicznego, dodatkowe powierzchnie narzędzi MCP i wykrywanie aplikacji agentów kodu — wszystko to dodatki. Zarządza nimi sekcja `Addons` w Web UI.

## Kategorie

Każdy dodatek deklaruje typ, a sekcja grupuje karty odpowiednio:

- **Applications** — uczą obserwatora terminala rozpoznawać CLI agenta kodu (wbudowane: Claude Code, Codex, Kimi Code, Pi i OpenCode): wykrywanie procesów, stan zajęty/bezczynny, haki automatyzacji;
- **LLM Providers** — backendy uzupełniania czatu, z których wybierają funkcje AI (wbudowany dostawca zgodny z OpenAI);
- **Voice / ASR** — backendy rozpoznawania mowy (wbudowany dodatek `vibe-voice` wskazuje nadzorowany lokalny serwer ASR);
- **Dynamic DNS** — utrzymują nazwę hosta zgodną z bieżącym adresem (wbudowany Cloudflare);
- **MCP** — dodatkowe powierzchnie narzędzi MCP serwowane na hoście MCP (wbudowany EasyEDA Pro);
- wszystko inne trafia do grupy **Other**.

## Wbudowane i wgrane dodatki

- **Wbudowane** dodatki jadą z programem i są zawsze dostępne. Ich manifest i źródło pozostają nietknięte — nadpisane źródło można w każdej chwili przywrócić;
- **Wgrane** dodatki są twoje: zip z manifestem i źródłem JavaScript, instalowany przyciskiem `Upload Addon`;
- Plakietka `Modified` oznacza dodatek, którego źródło nadpisałeś.

Każda karta pokazuje typ, plakietki i przełącznik włączenia; rozwinięta odsłania opis, autora, pola konfiguracji i kod źródłowy.

![Sekcja Addons — zgrupowane karty z plakietkami i przełącznikami](images/addon-section.jpg)

## Konfiguracja

Manifest dodatku deklaruje jego pola konfiguracji, a rozwinięta karta zamienia je w formularz. Wartości są przechowywane per dodatek i działają tylko, gdy jest włączony; pola hasła pozostają zamaskowane. Wyłączenie dodatku zachowuje jego konfigurację — po ponownym włączeniu kontynuujesz, gdzie skończyłeś.
